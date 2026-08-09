use std::fmt;

use crate::capability::Capabilities;

/// Identity of a physical device, stable across every route that reaches it.
///
/// A driver builds it from something the hardware itself owns (a serial
/// number, a MAC address, a manufacturer device id) and never from an IP address
/// or a user-visible name. That is what lets the same bulb, seen once on the
/// LAN and once through a manufacturer account, stay a single light in the
/// interface.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct DeviceId(String);

impl DeviceId {
    /// Builds an identifier namespaced by the driver that minted it.
    pub fn new(driver: &str, hardware_id: &str) -> Self {
        Self(format!("{driver}:{}", hardware_id.trim().to_lowercase()))
    }

    /// The identifier as a string, for storage or logging.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for DeviceId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// Who a device is and what it can do.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeviceDescriptor {
    /// Stable identity of the hardware.
    pub id: DeviceId,
    /// Name chosen by the user, as stored on the device or in the account.
    pub name: String,
    /// Manufacturer, for display only.
    pub vendor: String,
    /// Product reference, for display only. The interface never branches on it.
    pub model: String,
    /// What the device is able to do.
    pub capabilities: Capabilities,
}
