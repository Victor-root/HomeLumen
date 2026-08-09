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

                device.state = found.state;
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
    pub fn settle(&mut self, id: &DeviceId, state: LightState) {
        if let Some(device) = self.devices.get_mut(id) {
            device.state = state;
            device.online = true;
        }
    }

    /// Applies a change to the known state right away, before the device has
    /// confirmed it, so the interface never lags behind the pointer.
    pub fn anticipate(&mut self, id: &DeviceId, commands: &[Command]) {
        let Some(device) = self.devices.get_mut(id) else {
            return;
        };

        for command in commands {
            match command {
                Command::Power(on) => device.state.power = *on,
                Command::Brightness(level) => {
                    device.state.brightness = Some(*level)
                }
                Command::Color(color) => device.state.color = Some(*color),
                Command::Effect(effect) => device.state.effect = effect.clone(),
            }
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
