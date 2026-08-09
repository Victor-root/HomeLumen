//! The gesture shared by every horizontal control.
//!
//! Brightness and white temperature are painted very differently but are
//! grabbed exactly the same way: press anywhere on the capsule, drag, release.

use iced::advanced::mouse;
use iced::animation::{Animation, Easing};
use iced::time::{Duration, Instant};
use iced::touch;
use iced::{Event, Rectangle, window};

use crate::design::motion;

/// The live state of a draggable capsule.
pub struct Slide {
    /// Timestamp of the frame being drawn.
    pub now: Instant,
    /// Whether the pointer is over the control.
    pub hover: Animation<bool>,
    /// Whether the pointer is holding it.
    pub holding: bool,
    shown: Animation<f32>,
}

impl Slide {
    /// A control resting at `value`, expressed as `0.0..=1.0`.
    pub fn new(value: f32) -> Self {
        Self {
            now: Instant::now(),
            hover: motion::hover(),
            holding: false,
            // Just enough smoothing to take the stutter out of a drag without
            // letting the fill trail behind the pointer.
            shown: Animation::new(value)
                .easing(Easing::Linear)
                .duration(Duration::from_millis(80)),
        }
    }

    /// The value to paint right now.
    pub fn shown(&self) -> f32 {
        self.shown.interpolate_with(|value| value, self.now)
    }

    /// How strongly the control is being pointed at.
    pub fn attention(&self) -> f32 {
        let hover = self.hover.interpolate(0.0, 1.0, self.now);
        if self.holding { 1.0 } else { hover }
    }

    /// Whether anything still needs another frame.
    pub fn busy(&self) -> bool {
        self.hover.is_animating(self.now) || self.shown.is_animating(self.now)
    }
}

/// What the pointer just did to a capsule.
pub enum Gesture {
    /// A new value was chosen, and the pointer is still down.
    Moved(f32),
    /// The pointer let go.
    Released,
    /// Nothing changed but the control needs repainting.
    Restless,
    /// Nothing to do.
    Idle,
}

/// Feeds an event to a capsule and reports what came of it.
///
/// `cap` is the distance from either end that the value cannot go past, so the
/// rounded head of a fill lands exactly under the pointer.
pub fn track(
    slide: &mut Slide,
    event: &Event,
    bounds: Rectangle,
    cursor: mouse::Cursor,
    value: f32,
    cap: f32,
) -> Gesture {
    match event {
        Event::Window(window::Event::RedrawRequested(now)) => {
            slide.now = *now;
            slide.shown.go_mut(value, *now);
            slide.hover.go_mut(cursor.is_over(bounds), *now);

            if slide.busy() { Gesture::Restless } else { Gesture::Idle }
        }

        Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left))
        | Event::Touch(touch::Event::FingerPressed { .. }) => {
            match cursor.position_over(bounds) {
                Some(point) => {
                    slide.holding = true;
                    Gesture::Moved(fraction(bounds, point.x, cap))
                }
                None => Gesture::Idle,
            }
        }

        Event::Mouse(mouse::Event::CursorMoved { .. })
        | Event::Touch(touch::Event::FingerMoved { .. }) => {
            if slide.holding {
                match cursor.position() {
                    Some(point) => {
                        Gesture::Moved(fraction(bounds, point.x, cap))
                    }
                    None => Gesture::Idle,
                }
            } else if cursor.is_over(bounds) != slide.hover.value() {
                Gesture::Restless
            } else {
                Gesture::Idle
            }
        }

        Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left))
        | Event::Touch(touch::Event::FingerLifted { .. })
        | Event::Touch(touch::Event::FingerLost { .. }) => {
            if std::mem::take(&mut slide.holding) {
                Gesture::Released
            } else {
                Gesture::Idle
            }
        }

        _ => Gesture::Idle,
    }
}

fn fraction(bounds: Rectangle, x: f32, cap: f32) -> f32 {
    let span = bounds.width - cap * 2.0;
    if span <= 0.0 {
        return 0.0;
    }
    ((x - bounds.x - cap) / span).clamp(0.0, 1.0)
}
