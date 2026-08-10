//! The vendor-neutral heart of HomeLumen.
//!
//! Nothing in this crate knows about a manufacturer, a protocol or a product
//! reference. A light is described by what it *can do* (its [`Capabilities`])
//! and by what it is *currently doing* (its [`LightState`]). Drivers translate
//! that vocabulary into whatever their hardware speaks, and the interface only
//! ever reads capabilities.

mod capability;
mod command;
mod device;
mod error;
mod light;
mod provider;
mod transport;
pub mod vault;

pub use capability::{
    BrightnessRange, Capabilities, ColorTemperatureRange, Effect,
};
pub use command::Command;
pub use device::{DeviceDescriptor, DeviceId, DeviceKind};
pub use error::{Error, Result};
pub use light::{Color, LightState};
pub use provider::{Discovered, DiscoverySink, Endpoint, Provider};
pub use transport::Transport;
pub use vault::Account;
