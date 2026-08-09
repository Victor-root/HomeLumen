//! The luminous sphere at the top of a light's screen.
//!
//! It is the light itself, not a picture of a bulb: its colour, its reach and
//! its slow breathing all come from the state of the device.

use iced::mouse;
use iced::time::{Duration, Instant};
use iced::widget::canvas::{
    self, Canvas, Frame, Geometry, Path, Stroke, Style,
};
use iced::{Color, Element, Length, Point, Rectangle, Renderer, Theme, Vector};

use crate::design::{Skin, tone};

/// Layers the halo is built from.
const HALO: usize = 34;

/// A light, drawn as light.
pub struct Orb {
    glow: Color,
    intensity: f32,
    skin: Skin,
}

impl Orb {
    /// Builds an orb for a light putting out `intensity` of `glow`.
    pub fn new(glow: Color, intensity: f32, skin: Skin) -> Self {
        Self { glow, intensity: intensity.clamp(0.0, 1.0), skin }
    }
}

/// Kept by the runtime between frames, so the orb can breathe.
#[derive(Default)]
pub struct State {
    origin: Option<Instant>,
    now: Option<Instant>,
}

impl<Message> canvas::Program<Message> for Orb {
    type State = State;

    fn update(
        &self,
        state: &mut Self::State,
        event: &canvas::Event,
        _bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Option<canvas::Action<Message>> {
        let canvas::Event::Window(iced::window::Event::RedrawRequested(now)) =
            event
        else {
            return None;
        };

        state.origin.get_or_insert(*now);
        state.now = Some(*now);

        // A lit orb breathes. A dark one has no reason to cost a frame.
        (self.intensity > 0.0).then(|| {
            canvas::Action::request_redraw_at(*now + Duration::from_millis(33))
        })
    }

    fn draw(
        &self,
        state: &Self::State,
        renderer: &Renderer,
        _theme: &Theme,
        bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<Geometry> {
        let mut frame = Frame::new(renderer, bounds.size());
        let center = frame.center();

        let phase = match (state.origin, state.now) {
            (Some(origin), Some(now)) => (now - origin).as_secs_f32(),
            _ => 0.0,
        };

        let breath = 1.0 + 0.02 * (phase * 0.85).sin() * self.intensity;
        let reach = (bounds.width.min(bounds.height) / 2.0 - 2.0) * breath;
        let core = reach * 0.47;

        for layer in 0..HALO {
            let step = layer as f32 / (HALO - 1) as f32;
            let radius = reach - (reach - core) * step;
            let opacity = (0.020 + 0.042 * step.powf(1.8)) * self.intensity;

            frame.fill(
                &Path::circle(center, radius),
                tone::fade(self.glow, opacity),
            );
        }

        let glass = tone::mix(self.skin.surface_lift, self.skin.edge, 0.35);
        let face = tone::mix(glass, self.glow, self.intensity);

        frame.fill(
            &Path::circle(center, core),
            canvas::gradient::Linear::new(
                Point::new(center.x, center.y - core),
                Point::new(center.x, center.y + core),
            )
            .add_stop(
                0.0,
                tone::mix(face, Color::WHITE, 0.20 + 0.18 * self.intensity),
            )
            .add_stop(1.0, tone::mix(face, self.skin.canvas, 0.18)),
        );

        frame.stroke(
            &Path::circle(center, core),
            Stroke {
                style: Style::Solid(tone::mix(
                    tone::fade(self.skin.edge, 0.9),
                    tone::fade(Color::WHITE, 0.35),
                    self.intensity,
                )),
                width: 1.0,
                ..Stroke::default()
            },
        );

        // A single specular highlight is what turns a disc into a sphere.
        frame.fill(
            &Path::circle(
                center + Vector::new(-core * 0.33, -core * 0.38),
                core * 0.26,
            ),
            tone::fade(Color::WHITE, 0.10 + 0.14 * self.intensity),
        );

        vec![frame.into_geometry()]
    }
}

/// Places an orb of the given side length.
pub fn orb<'a, Message>(light: Orb, side: f32) -> Element<'a, Message>
where
    Message: 'a,
{
    Canvas::new(light)
        .width(Length::Fixed(side))
        .height(Length::Fixed(side))
        .into()
}
