use std::collections::BTreeMap;
use std::sync::Arc;

use homelumen_core::{
    Command, DeviceDescriptor, DeviceId, Discovered, Endpoint, LightState,
    Transport,
};

use crate::snapshot::LightSnapshot;

/// Everything HomeLumen currently knows about the lights around it.
#[derive(Default)]
pub struct Registry {
    devices: BTreeMap<DeviceId, Device>,
}

impl Registry {
    /// Files a discovery under the identity of its device.
    ///
    /// A light already known through another route keeps its place: only its
    /// route list and its capabilities grow. That is what keeps a bulb seen
    /// both on the LAN and through an account a single light on screen.
    pub fn absorb(&mut self, found: Discovered) -> DeviceId {
        let id = found.descriptor.id.clone();
        let transport = found.endpoint.transport();
        let address = found.endpoint.address();

        match self.devices.get_mut(&id) {
            Some(device) => {
                device
                    .descriptor
                    .capabilities
                    .absorb(&found.descriptor.capabilities);

                if !found.descriptor.name.trim().is_empty() {
                    device.descriptor.name = found.descriptor.name;
                }

                // A route with no better information of its own, such as a
                // local broadcast that carries no product name, must not
                // blank out a nicer one another route already supplied.
                if !found.descriptor.model.trim().is_empty() {
                    device.descriptor.model = found.descriptor.model;
                }

                // Not `device.state = found.state`: a sweep's answer can be
                // in flight for up to `SWEEP_WINDOW`, long enough for a
                // command issued mid-sweep to anticipate and settle before
                // the sweep's own, now-stale answer lands. State stays the
                // job of `anticipate` and `settle`, which are ordered
                // against the user's intent; discovery only owns routing.
                device.online = true;
                device.attach(Route {
                    endpoint: found.endpoint,
                    transport,
                    address,
                    healthy: true,
                });
            }
            None => {
                let mut device = Device {
                    descriptor: found.descriptor,
                    state: found.state,
                    routes: Vec::new(),
                    online: true,
                    inflight: false,
                    pending: None,
                };
                device.attach(Route {
                    endpoint: found.endpoint,
                    transport,
                    address,
                    healthy: true,
                });
                self.devices.insert(id.clone(), device);
            }
        }

        id
    }

    /// The best route to a device: the most local one that still answers.
    ///
    /// When every route has failed they are all given another chance rather
    /// than leaving the light unreachable forever; a bulb that came back on the
    /// network must be usable again without restarting anything.
    pub fn best_route(
        &mut self,
        id: &DeviceId,
    ) -> Option<(Transport, Arc<dyn Endpoint>)> {
        let device = self.devices.get_mut(id)?;

        if device.routes.iter().all(|route| !route.healthy) {
            for route in &mut device.routes {
                route.healthy = true;
            }
        }

        device
            .routes
            .iter()
            .find(|route| route.healthy)
            .map(|route| (route.transport, Arc::clone(&route.endpoint)))
    }

    /// Whether a route is still believed to work, without reviving any.
    pub fn has_healthy_route(&self, id: &DeviceId) -> bool {
        self.devices.get(id).is_some_and(|device| {
            device.routes.iter().any(|route| route.healthy)
        })
    }

    /// Records that a route just failed, so the next attempt falls through to
    /// the one below it.
    pub fn demote(&mut self, id: &DeviceId, transport: Transport) {
        let Some(device) = self.devices.get_mut(id) else {
            return;
        };

        for route in &mut device.routes {
            if route.transport == transport {
                route.healthy = false;
            }
        }

        device.online = device.routes.iter().any(|route| route.healthy);
    }

    /// Replaces the known state of a device and marks it reachable.
    ///
    /// Whatever is still held back for this light is folded back in on top.
    /// The answer that just landed describes the command before those, so
    /// letting it stand on its own would walk a value the user has already
    /// moved past back to where it was, once per round trip, for as long as
    /// a drag keeps producing intents.
    pub fn settle(&mut self, id: &DeviceId, state: LightState) {
        let Some(device) = self.devices.get_mut(id) else {
            return;
        };

        device.state = state;
        device.online = true;

        if let Some(held) = device.pending.take() {
            for command in &held {
                device.expect(command);
            }
            device.pending = Some(held);
        }
    }

    /// Applies a change to the known state right away, before the device has
    /// confirmed it, so the interface never lags behind the pointer.
    pub fn anticipate(&mut self, id: &DeviceId, commands: &[Command]) {
        let Some(device) = self.devices.get_mut(id) else {
            return;
        };

        for command in commands {
            device.expect(command);
        }
    }

    /// Whether a request is already on the wire for this device.
    pub fn is_busy(&self, id: &DeviceId) -> bool {
        self.devices.get(id).is_some_and(|device| device.inflight)
    }

    /// Marks a device as having a request on the wire, or not.
    pub fn set_busy(&mut self, id: &DeviceId, busy: bool) {
        if let Some(device) = self.devices.get_mut(id) {
            device.inflight = busy;
        }
    }

    /// Holds back a command batch until the current one comes back.
    ///
    /// Dragging a slider produces far more intents than a bulb can swallow, so
    /// only the latest value of each kind is kept; the light ends up exactly
    /// where the pointer was released, without a queue of stale steps.
    pub fn defer(&mut self, id: &DeviceId, commands: Vec<Command>) {
        let Some(device) = self.devices.get_mut(id) else {
            return;
        };

        let mut merged = device.pending.take().unwrap_or_default();
        for command in commands {
            merged.retain(|held| discriminant(held) != discriminant(&command));
            merged.push(command);
        }
        device.pending = Some(merged);
    }

    /// Takes the batch held back for a device, if any.
    pub fn take_deferred(&mut self, id: &DeviceId) -> Option<Vec<Command>> {
        self.devices.get_mut(id).and_then(|device| device.pending.take())
    }

    /// One known light, ready for display.
    pub fn snapshot(&self, id: &DeviceId) -> Option<LightSnapshot> {
        self.devices.get(id).map(Device::snapshot)
    }

    /// The identity of every known light.
    pub fn ids(&self) -> Vec<DeviceId> {
        self.devices.keys().cloned().collect()
    }
}

fn discriminant(command: &Command) -> u8 {
    match command {
        Command::Power(_) => 0,
        Command::Brightness(_) => 1,
        Command::Color(_) => 2,
        Command::Effect(_) => 3,
    }
}

struct Device {
    descriptor: DeviceDescriptor,
    state: LightState,
    routes: Vec<Route>,
    online: bool,
    inflight: bool,
    pending: Option<Vec<Command>>,
}

impl Device {
    fn attach(&mut self, route: Route) {
        self.routes.retain(|known| known.transport != route.transport);
        self.routes.push(route);
        self.routes.sort_by_key(|route| route.transport.preference());
    }

    /// Folds one intent into the known state, as if the light had obeyed.
    fn expect(&mut self, command: &Command) {
        match command {
            Command::Power(on) => self.state.power = *on,
            Command::Brightness(level) => self.state.brightness = Some(*level),
            Command::Color(color) => self.state.color = Some(*color),
            Command::Effect(effect) => self.state.effect = effect.clone(),
        }
    }

    fn snapshot(&self) -> LightSnapshot {
        let active = self
            .routes
            .iter()
            .find(|route| route.healthy)
            .or_else(|| self.routes.first());

        LightSnapshot {
            descriptor: self.descriptor.clone(),
            state: self.state.clone(),
            transport: active.map(|route| route.transport),
            address: active
                .map(|route| route.address.clone())
                .unwrap_or_default(),
            online: self.online,
        }
    }
}

struct Route {
    endpoint: Arc<dyn Endpoint>,
    transport: Transport,
    address: String,
    healthy: bool,
}

#[cfg(test)]
mod tests {
    use async_trait::async_trait;
    use homelumen_core::{
        BrightnessRange, Capabilities, DeviceKind, Result, Transport as Kind,
    };

    use super::*;

    /// A route the registry can file but never has to travel: nothing here
    /// reads a device back or sends it anything.
    struct Nowhere;

    #[async_trait]
    impl Endpoint for Nowhere {
        fn transport(&self) -> Kind {
            Kind::Lan
        }

        fn address(&self) -> String {
            "127.0.0.1".to_owned()
        }

        async fn state(&self) -> Result<LightState> {
            unreachable!("the registry never talks to a device")
        }

        async fn apply(&self, _commands: &[Command]) -> Result<LightState> {
            unreachable!("the registry never talks to a device")
        }
    }

    fn lit(brightness: u8) -> LightState {
        LightState {
            power: true,
            brightness: Some(brightness),
            color: None,
            effect: None,
        }
    }

    fn one_light() -> (Registry, DeviceId) {
        let mut registry = Registry::default();

        let id = registry.absorb(Discovered {
            descriptor: DeviceDescriptor {
                id: DeviceId::new("test", "salon"),
                kind: DeviceKind::Light,
                name: "Salon".to_owned(),
                vendor: "Essai".to_owned(),
                model: "LB130".to_owned(),
                capabilities: Capabilities {
                    power: true,
                    brightness: Some(BrightnessRange::PERCENT),
                    ..Capabilities::default()
                },
            },
            state: lit(10),
            endpoint: Arc::new(Nowhere),
        });

        (registry, id)
    }

    fn brightness(registry: &Registry, id: &DeviceId) -> Option<u8> {
        registry.snapshot(id).unwrap().state.brightness
    }

    #[test]
    fn a_late_answer_does_not_undo_a_newer_intent() {
        let (mut registry, id) = one_light();

        // The pointer keeps moving while the first command is still out on
        // the wire, so the second one is held back behind it.
        registry.anticipate(&id, &[Command::Brightness(62)]);
        registry.anticipate(&id, &[Command::Brightness(70)]);
        registry.defer(&id, vec![Command::Brightness(70)]);

        // The light answers the only command it was actually given.
        registry.settle(&id, lit(62));

        assert_eq!(
            brightness(&registry, &id),
            Some(70),
            "an answer about 62 must not walk 70 back while 70 is queued"
        );
    }

    #[test]
    fn an_answer_stands_once_nothing_is_held_back() {
        let (mut registry, id) = one_light();

        registry.anticipate(&id, &[Command::Brightness(62)]);
        registry.settle(&id, lit(41));

        assert_eq!(
            brightness(&registry, &id),
            Some(41),
            "with no intent outstanding the light has the last word"
        );
    }

    #[test]
    fn a_held_intent_only_shields_what_it_speaks_for() {
        let (mut registry, id) = one_light();

        registry.anticipate(&id, &[Command::Brightness(70)]);
        registry.defer(&id, vec![Command::Brightness(70)]);

        // Someone flicked the wall switch: brightness is spoken for, power
        // is not, so only power takes the light's word for it.
        registry.settle(&id, LightState { power: false, ..lit(62) });

        let state = registry.snapshot(&id).unwrap().state;
        assert!(!state.power);
        assert_eq!(state.brightness, Some(70));
    }
}
