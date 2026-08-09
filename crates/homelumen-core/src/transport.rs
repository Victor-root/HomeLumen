use std::fmt;

/// How a device is being reached.
///
/// HomeLumen is local first: the order below is the order in which routes are
/// tried, and a route is only skipped when it is known to be unreachable.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Transport {
    /// Straight to the device on the local network. No account, no round trip
    /// through the internet, lowest latency.
    Lan,
    /// Through the manufacturer's own service.
    Cloud,
    /// Through a HomeLumen relay reaching another site.
    Gateway,
}

impl Transport {
    /// Every transport, from most to least preferred.
    pub const ALL: [Transport; 3] =
        [Transport::Lan, Transport::Cloud, Transport::Gateway];

    /// Lower is better. Used to order the routes of a single device.
    pub fn preference(self) -> u8 {
        match self {
            Transport::Lan => 0,
            Transport::Cloud => 1,
            Transport::Gateway => 2,
        }
    }
}

impl fmt::Display for Transport {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Transport::Lan => "Local",
            Transport::Cloud => "Cloud",
            Transport::Gateway => "Passerelle",
        })
    }
}
