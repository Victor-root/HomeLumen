use std::net::IpAddr;
use std::sync::Arc;
use std::time::Duration;

use futures::Stream;
use homelumen_core::{
    Command, DeviceId, Discovered, Error, LightState, Provider, Result,
    Transport,
};
use tokio::sync::mpsc;

use crate::registry::Registry;
use crate::snapshot::LightSnapshot;

/// How often known lights are read back, so a change made elsewhere (a wall
/// switch, another app) shows up here too.
const REFRESH_EVERY: Duration = Duration::from_secs(5);

/// How often the network is swept again, to pick up lights that just powered on
/// and to give a route that failed another chance.
const RESCAN_EVERY: Duration = Duration::from_secs(45);

/// What the interface asks the engine to do.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Request {
    /// Look for lights on every driver.
    Scan,
    /// Apply a batch of intents to one light.
    Apply {
        /// Which light.
        device: DeviceId,
        /// What to change, applied in one round trip.
        commands: Vec<Command>,
    },
    /// Try to reach a light at an address given by the user.
    AddByAddress(IpAddr),
}

/// What the engine tells the interface.
#[derive(Debug, Clone)]
pub enum Event {
    /// The engine is up; this is how to talk to it.
    Ready(Handle),
    /// A sweep started or ended.
    Scanning(bool),
    /// A light appeared, or something about it changed.
    Updated(LightSnapshot),
    /// Something went wrong, worth telling the user about.
    Failed {
        /// The light concerned, when the failure is about one in particular.
        device: Option<DeviceId>,
        /// A message written for a human.
        message: String,
    },
}

/// The interface's way of talking to the engine.
#[derive(Debug, Clone)]
pub struct Handle {
    signals: mpsc::UnboundedSender<Signal>,
}

impl Handle {
    /// Queues a request. Dropping the engine simply makes this a no-op.
    pub fn send(&self, request: Request) {
        let _ = self.signals.send(Signal::Request(request));
    }
}

/// Starts the engine and streams everything it has to say.
///
/// The first item is always [`Event::Ready`], carrying the [`Handle`].
pub fn run() -> impl Stream<Item = Event> {
    futures::stream::unfold(Pump::Boot, |pump| async move {
        match pump {
            Pump::Boot => {
                let (signals, inbox) = mpsc::unbounded_channel();
                let (events, outbox) = mpsc::unbounded_channel();

                tokio::spawn(drive(inbox, signals.clone(), events));
                tokio::spawn(heartbeat(signals.clone()));

                Some((
                    Event::Ready(Handle { signals: signals.clone() }),
                    Pump::Running(outbox),
                ))
            }
            Pump::Running(mut outbox) => {
                let event = outbox.recv().await?;
                Some((event, Pump::Running(outbox)))
            }
        }
    })
}

enum Pump {
    Boot,
    Running(mpsc::UnboundedReceiver<Event>),
}

/// Everything that can reach the engine loop, from the interface or from its
/// own background tasks.
enum Signal {
    Request(Request),
    Found(Box<Discovered>),
    /// A driver could not sweep at all, for want of credentials or of a
    /// network. Reported only while `visible`, so a person who just asked
    /// to look learns why nothing showed up, but the heartbeat's own silent
    /// retries never nag about a driver that stays unconfigured.
    SweepFailed {
        provider: &'static str,
        error: Error,
    },
    SweepFinished,
    /// A round trip came back.
    Outcome {
        device: DeviceId,
        transport: Transport,
        /// The intents to replay on another route should this one be dead.
        /// `None` for a plain state read, which is not worth retrying.
        replay: Option<Vec<Command>>,
        result: Result<LightState>,
    },
    /// Time to read the known lights back.
    Poll,
    /// Time to sweep again on its own, unasked. Distinct from
    /// `Request(Request::Scan)` so the heartbeat's own sweeps never turn on
    /// the "scanning" indicator: a light appearing on its own is routine,
    /// worth showing only when a person actually asked to look.
    Rescan,
}

/// Every driver HomeLumen ships with. Adding a manufacturer happens here and
/// nowhere else.
fn drivers() -> Vec<Arc<dyn Provider>> {
    vec![
        Arc::new(homelumen_kasa::LanProvider),
        Arc::new(homelumen_tuya::CloudProvider::default()),
        Arc::new(homelumen_tuya::LanProvider::default()),
    ]
}

async fn drive(
    mut inbox: mpsc::UnboundedReceiver<Signal>,
    signals: mpsc::UnboundedSender<Signal>,
    events: mpsc::UnboundedSender<Event>,
) {
    let providers = drivers();
    let mut registry = Registry::default();
    let mut sweeping = false;
    // Whether the sweep in flight, if any, is one the "scanning" indicator
    // should own. Kept apart from `sweeping` because a request to scan can
    // land while the heartbeat's own silent sweep is already running: it
    // rides along on that same sweep rather than starting a second one.
    let mut visible = false;

    let _ = signals.send(Signal::Request(Request::Scan));

    while let Some(signal) = inbox.recv().await {
        match signal {
            Signal::Request(Request::Scan) => {
                if !visible {
                    visible = true;
                    let _ = events.send(Event::Scanning(true));
                }

                if !sweeping {
                    sweeping = true;
                    tokio::spawn(sweep(providers.clone(), signals.clone()));
                }
            }

            Signal::Request(Request::Apply { device, commands }) => {
                registry.anticipate(&device, &commands);
                announce(&registry, &events, &device);

                if registry.is_busy(&device) {
                    registry.defer(&device, commands);
                } else if !dispatch(
                    &mut registry,
                    &signals,
                    &device,
                    Some(commands),
                ) {
                    let _ = events.send(Event::Failed {
                        device: Some(device),
                        message: "plus aucune route vers cette lumière".into(),
                    });
                }
            }

            Signal::Request(Request::AddByAddress(address)) => {
                tokio::spawn(reach(
                    providers.clone(),
                    address,
                    signals.clone(),
                    events.clone(),
                ));
            }

            Signal::Found(found) => {
                let device = registry.absorb(*found);
                announce(&registry, &events, &device);
            }

            Signal::SweepFailed { provider, error } => {
                if visible {
                    let _ = events.send(Event::Failed {
                        device: None,
                        message: format!("{provider}: {error}"),
                    });
                } else {
                    log::warn!("{provider}: {error}");
                }
            }

            Signal::SweepFinished => {
                sweeping = false;

                if visible {
                    visible = false;
                    let _ = events.send(Event::Scanning(false));
                }
            }

            Signal::Outcome { device, transport, replay, result } => {
                registry.set_busy(&device, false);

                match result {
                    Ok(state) => registry.settle(&device, state),
                    Err(error) => {
                        registry.demote(&device, transport);

                        // A read that failed is left to the next poll; an
                        // intent the user expressed gets one more shot, on
                        // whichever route is still believed to work.
                        if let Some(commands) = replay {
                            let retried = registry.has_healthy_route(&device)
                                && dispatch(
                                    &mut registry,
                                    &signals,
                                    &device,
                                    Some(commands),
                                );

                            if !retried {
                                let _ = events.send(Event::Failed {
                                    device: Some(device.clone()),
                                    message: error.to_string(),
                                });
                            }
                        }
                    }
                }

                announce(&registry, &events, &device);

                if !registry.is_busy(&device)
                    && let Some(held) = registry.take_deferred(&device)
                {
                    let _ =
                        dispatch(&mut registry, &signals, &device, Some(held));
                }
            }

            Signal::Poll => {
                for device in registry.ids() {
                    if !registry.is_busy(&device) {
                        let _ =
                            dispatch(&mut registry, &signals, &device, None);
                    }
                }
            }

            Signal::Rescan => {
                if sweeping {
                    continue;
                }
                sweeping = true;
                tokio::spawn(sweep(providers.clone(), signals.clone()));
            }
        }
    }
}

/// Sends a light's current picture to the interface.
fn announce(
    registry: &Registry,
    events: &mpsc::UnboundedSender<Event>,
    device: &DeviceId,
) {
    if let Some(snapshot) = registry.snapshot(device) {
        let _ = events.send(Event::Updated(snapshot));
    }
}

/// Starts a round trip on the best available route.
///
/// `commands` set means applying intents; `None` means reading the state back.
/// Returns whether a route was found at all.
fn dispatch(
    registry: &mut Registry,
    signals: &mpsc::UnboundedSender<Signal>,
    device: &DeviceId,
    commands: Option<Vec<Command>>,
) -> bool {
    let Some((transport, endpoint)) = registry.best_route(device) else {
        return false;
    };

    registry.set_busy(device, true);

    let signals = signals.clone();
    let device = device.clone();

    tokio::spawn(async move {
        let result = match &commands {
            Some(commands) => endpoint.apply(commands).await,
            None => endpoint.state().await,
        };

        let _ = signals.send(Signal::Outcome {
            device,
            transport,
            replay: commands,
            result,
        });
    });

    true
}

async fn sweep(
    providers: Vec<Arc<dyn Provider>>,
    signals: mpsc::UnboundedSender<Signal>,
) {
    let (sink, mut found) = mpsc::channel::<Discovered>(32);
    let mut running = Vec::with_capacity(providers.len());

    for provider in providers {
        let sink = sink.clone();
        let signals = signals.clone();
        running.push(tokio::spawn(async move {
            if let Err(error) = provider.discover(sink).await {
                let _ = signals.send(Signal::SweepFailed {
                    provider: provider.name(),
                    error,
                });
            }
        }));
    }
    drop(sink);

    while let Some(device) = found.recv().await {
        if signals.send(Signal::Found(Box::new(device))).is_err() {
            break;
        }
    }

    for task in running {
        let _ = task.await;
    }

    let _ = signals.send(Signal::SweepFinished);
}

async fn reach(
    providers: Vec<Arc<dyn Provider>>,
    address: IpAddr,
    signals: mpsc::UnboundedSender<Signal>,
    events: mpsc::UnboundedSender<Event>,
) {
    let mut refusal = None;

    for provider in providers {
        match provider.probe(address).await {
            Ok(found) => {
                let _ = signals.send(Signal::Found(Box::new(found)));
                return;
            }
            Err(Error::Unsupported(_)) => {}
            Err(error) => refusal = Some(error),
        }
    }

    let message = match refusal {
        Some(error) => format!("{address}: {error}"),
        None => format!("aucun pilote ne sait contacter {address}"),
    };

    let _ = events.send(Event::Failed { device: None, message });
}

async fn heartbeat(signals: mpsc::UnboundedSender<Signal>) {
    let mut refresh = tokio::time::interval(REFRESH_EVERY);
    let mut rescan = tokio::time::interval(RESCAN_EVERY);

    // The first tick of an interval fires immediately; the engine already scans
    // on start-up, so both are consumed here.
    refresh.tick().await;
    rescan.tick().await;

    loop {
        let sent = tokio::select! {
            _ = refresh.tick() => signals.send(Signal::Poll),
            _ = rescan.tick() => signals.send(Signal::Rescan),
        };

        if sent.is_err() {
            break;
        }
    }
}
