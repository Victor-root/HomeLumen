//! Tuya / Smart Life driver.
//!
//! Two routes to the same plugs: [`CloudProvider`] through the Smart Life
//! account already linked to HomeLumen's cloud project, [`LanProvider`]
//! straight to the device on the local network once that account has taught
//! it the device's own key. Everything about signing requests and shaping
//! Tuya's JSON stays here; the rest of HomeLumen only ever sees the generic
//! capabilities these drivers report.

mod cloud;
mod lan;
mod lan_crypto;
mod lan_protocol;
mod lan_wire;
mod protocol;
mod signing;
mod wire;

/// Slug used to namespace the device identifiers both drivers mint.
const DRIVER: &str = "tuya";

/// Manufacturer name, for display.
const VENDOR: &str = "Tuya";

pub use cloud::{CloudProvider, account, set_account};
pub use lan::LanProvider;
