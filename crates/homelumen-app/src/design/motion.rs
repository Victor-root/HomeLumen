//! How things move.
//!
//! Short, decelerating, never bouncy. Motion here exists to explain a change,
//! not to be noticed.

use iced::animation::{Animation, Easing};
use iced::time::Duration;

/// A pointer entering or leaving something.
pub fn hover() -> Animation<bool> {
    Animation::new(false)
        .easing(Easing::EaseOut)
        .duration(Duration::from_millis(150))
}

/// A press, which has to feel instant.
pub fn press() -> Animation<bool> {
    Animation::new(false)
        .easing(Easing::EaseOut)
        .duration(Duration::from_millis(90))
}

/// A light coming on or going off: the one animation allowed to take its time.
pub fn switch(lit: bool) -> Animation<bool> {
    Animation::new(lit)
        .easing(Easing::EaseInOut)
        .duration(Duration::from_millis(420))
}

/// A value settling after the pointer let go of it.
pub fn settle(start: f32) -> Animation<f32> {
    Animation::new(start)
        .easing(Easing::EaseOut)
        .duration(Duration::from_millis(260))
}

/// A panel sliding into view.
pub fn page(start: f32) -> Animation<f32> {
    Animation::new(start)
        .easing(Easing::EaseInOut)
        .duration(Duration::from_millis(340))
}
