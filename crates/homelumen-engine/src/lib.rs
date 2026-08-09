//! The part of HomeLumen that keeps track of the lights.
//!
//! It runs every driver, merges what they find into a single list of devices,
//! picks the most local route that still answers, and turns the interface's
//! intents into commands the devices actually receive.

mod engine;
mod registry;
mod snapshot;

pub use engine::{Event, Handle, Request, run};
pub use snapshot::LightSnapshot;
