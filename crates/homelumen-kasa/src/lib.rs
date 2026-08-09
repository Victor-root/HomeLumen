//! TP-Link Kasa driver.
//!
//! Covers the smart bulbs speaking the historic local protocol: the LB and KL
//! families, including the LB120 and LB130. Everything model specific stays
//! here; the rest of HomeLumen only ever sees the generic capabilities this
//! driver reports.

mod crypto;
mod lan;
mod protocol;
mod wire;

pub use lan::LanProvider;
