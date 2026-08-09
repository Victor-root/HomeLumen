use homelumen_core::{DeviceDescriptor, LightState, Transport};

/// A light as the interface sees it: plain data, no traits, no I/O.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LightSnapshot {
    /// Who the light is and what it can do.
    pub descriptor: DeviceDescriptor,
    /// What it is currently doing.
    pub state: LightState,
    /// The route currently carrying its commands.
    pub transport: Option<Transport>,
    /// Where that route points, for display.
    pub address: String,
    /// Whether at least one route still answers.
    pub online: bool,
}
