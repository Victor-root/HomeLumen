//! Tuya / Smart Life driver.
//!
//! Covers plugs reached through Tuya's own cloud, using the Smart Life
//! account already linked to HomeLumen's cloud project. Everything about
//! signing requests and shaping Tuya's JSON stays here; the rest of
//! HomeLumen only ever sees the generic capabilities this driver reports.

mod cloud;
mod protocol;
mod signing;
mod wire;

pub use cloud::{CloudProvider, account, set_account};
