use std::net::IpAddr;
use std::sync::Arc;

use async_trait::async_trait;
use tokio::sync::mpsc;

use crate::command::Command;
use crate::device::DeviceDescriptor;
use crate::error::{Error, Result};
use crate::light::LightState;
use crate::transport::Transport;

/// One concrete way of talking to one device.
///
/// A device with both a LAN address and a manufacturer account has two
/// endpoints; they answer for the same [`DeviceId`](crate::DeviceId) and the
/// engine picks whichever is preferred and healthy.
#[async_trait]
pub trait Endpoint: Send + Sync {
    /// The kind of route this endpoint represents.
    fn transport(&self) -> Transport;

    /// Where the endpoint points, for display: an IP address, an account name.
    fn address(&self) -> String;

    /// Reads the current state of the device.
    async fn state(&self) -> Result<LightState>;

    /// Applies a batch of commands and returns the resulting state.
    async fn apply(&self, commands: &[Command]) -> Result<LightState>;
}

/// A device found by a [`Provider`], along with the route that found it.
pub struct Discovered {
    /// Who the device is and what it can do.
    pub descriptor: DeviceDescriptor,
    /// Its state at the moment of discovery.
    pub state: LightState,
    /// The route the provider just proved to work.
    pub endpoint: Arc<dyn Endpoint>,
}

/// Where a [`Provider`] pushes what it finds, as it finds it.
///
/// Discovery streams rather than returning a list, so the first bulb appears in
/// the interface without waiting for the slowest one.
pub type DiscoverySink = mpsc::Sender<Discovered>;

/// A source of devices: one manufacturer over one kind of route.
///
/// `Kasa over the LAN` and `Kasa through the manufacturer account` are two
/// providers, and a single bulb reachable both ways is reported by both.
#[async_trait]
pub trait Provider: Send + Sync {
    /// Stable slug, used to namespace the device identifiers it mints.
    fn id(&self) -> &'static str;

    /// Human readable name of the provider.
    fn name(&self) -> &'static str;

    /// Looks for devices and pushes every one it finds into `sink`.
    ///
    /// Returns once the sweep is over. A provider that cannot run right now,
    /// for want of credentials or of a network, returns an error rather than
    /// blocking the others.
    async fn discover(&self, sink: DiscoverySink) -> Result<()>;

    /// Contacts a single device at an address given by the user.
    ///
    /// Broadcast discovery is blocked on plenty of networks, so an address
    /// typed by hand must reach the same devices. Providers that do not work
    /// by address, such as a manufacturer account, keep the default.
    async fn probe(&self, _address: IpAddr) -> Result<Discovered> {
        Err(Error::Unsupported(format!(
            "{} ne sait pas contacter une adresse directement",
            self.name()
        )))
    }
}
