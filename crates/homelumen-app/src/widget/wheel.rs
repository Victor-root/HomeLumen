//! The colour disc.
//!
//! Hue runs around it starting at the top, saturation runs from the white
//! centre out to the rim. Large, grabbable anywhere, and it never asks for a
//! number.

use std::f32::consts::{FRAC_PI_2, TAU};

use iced::widget::canvas::{
    self, Cache, Canvas, Frame, Geometry, Path, Stroke, Style,
};
use iced::{Color, Element, Length, Point, Rectangle, Renderer, Theme, Vector};
use iced::{mouse, touch};

use crate::design::{Skin, tone};

/// How many wedges the disc is built from. Past this the seams are invisible.
const WEDGES: usize = 240;

/// Margin left around the disc so its rim is never clipped.
const MARGIN: f32 = 8.0;

const PUCK: f32 = 15.0;

/// A hue and saturation picker.
pub struct Wheel<'a, Message> {
    hue: f32,
    saturation: f32,
    skin: Skin,
    on_change: Box<dyn Fn(f32, f32) -> Message + 'a>,
}

impl<'a, Message> Wheel<'a, Message> {
    /// Builds a disc showing `hue` in degrees and `saturation` in `0.0..=1.0`.
    pub fn new(
        hue: f32,
        saturation: f32,
        skin: Skin,
        on_change: impl Fn(f32, f32) -> Message + 'a,
    ) -> Self {
        Self {
            hue,
            saturation: saturation.clamp(0.0, 1.0),
            skin,
            on_change: Box::new(on_change),
        }
    }

    fn geometry(bounds: Rectangle) -> (Point, f32) {
        let center = Point::new(bounds.width / 2.0, bounds.height / 2.0);
        let radius = bounds.width.min(bounds.height) / 2.0 - MARGIN;
        (center, radius)
    }

    /// Turns a point inside the disc into a hue and a saturation.
    fn read(bounds: Rectangle, at: Point) -> (f32, f32) {
        let (center, radius) = Self::geometry(bounds);
        let offset =
            Vector::new(at.x - bounds.x - center.x, at.y - bounds.y - center.y);

        let hue = (offset.y.atan2(offset.x) + FRAC_PI_2)
            .to_degrees()
            .rem_euclid(360.0);
        let distance = (offset.x * offset.x + offset.y * offset.y).sqrt();

        (hue, (distance / radius).clamp(0.0, 1.0))
    }
}

/// Where the puck sits for a given hue and saturation.
fn puck_at(center: Point, radius: f32, hue: f32, saturation: f32) -> Point {
    let angle = hue.to_radians() - FRAC_PI_2;
    center + Vector::new(angle.cos(), angle.sin()) * (radius * saturation)
}

/// Kept by the runtime between frames: the grab, and the disc itself.
#[derive(Default)]
pub struct State {
    holding: bool,
    disc: Cache,
}

impl<Message> canvas::Program<Message> for Wheel<'_, Message>
where
    Message: Clone,
{
    type State = State;

    fn update(
        &self,
        state: &mut Self::State,
        event: &canvas::Event,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> Option<canvas::Action<Message>> {
        match event {
            canvas::Event::Mouse(mouse::Event::ButtonPressed(
                mouse::Button::Left,
            ))
            | canvas::Event::Touch(touch::Event::FingerPressed { .. }) => {
                let at = cursor.position_over(bounds)?;
                let (_, radius) = Self::geometry(bounds);
                let (hue, saturation) = Self::read(bounds, at);

                // Only a press on the disc itself grabs it; the corners of the
                // square belong to whatever is behind.
                if saturation * radius > radius + PUCK {
                    return None;
                }

                state.holding = true;
                Some(
                    canvas::Action::publish((self.on_change)(hue, saturation))
                        .and_capture(),
                )
            }

            canvas::Event::Mouse(mouse::Event::CursorMoved { .. })
            | canvas::Event::Touch(touch::Event::FingerMoved { .. })
                if state.holding =>
            {
                let at = cursor.position()?;
                let (hue, saturation) = Self::read(bounds, at);
                Some(
                    canvas::Action::publish((self.on_change)(hue, saturation))
                        .and_capture(),
                )
            }

            canvas::Event::Mouse(mouse::Event::ButtonReleased(
                mouse::Button::Left,
            ))
            | canvas::Event::Touch(touch::Event::FingerLifted { .. })
            | canvas::Event::Touch(touch::Event::FingerLost { .. })
                if state.holding =>
            {
                state.holding = false;
                Some(canvas::Action::request_redraw())
            }

            _ => None,
        }
    }

    fn draw(
        &self,
        state: &Self::State,
        renderer: &Renderer,
        _theme: &Theme,
        bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<Geometry> {
        let (center, radius) = Self::geometry(bounds);

        // The disc never changes, so it is built once and kept.
        let disc = state.disc.draw(renderer, bounds.size(), |frame| {
            for wedge in 0..WEDGES {
                let from = wedge as f32 / WEDGES as f32 * TAU;
                let to = (wedge + 1) as f32 / WEDGES as f32 * TAU;
                let middle = (from + to) / 2.0;

                let rim = |angle: f32| {
                    let screen = angle - FRAC_PI_2;
                    center + Vector::new(screen.cos(), screen.sin()) * radius
                };

                let slice = Path::new(|path| {
                    path.move_to(center);
                    path.line_to(rim(from));
                    path.line_to(rim(to));
                    path.close();
                });

                frame.fill(
                    &slice,
                    canvas::gradient::Linear::new(center, rim(middle))
                        .add_stop(0.0, Color::WHITE)
                        .add_stop(
                            1.0,
                            tone::hsv(middle.to_degrees(), 1.0, 1.0),
                        ),
                );
            }
        });

        let mut overlay = Frame::new(renderer, bounds.size());

        overlay.stroke(
            &Path::circle(center, radius),
            Stroke {
                style: Style::Solid(tone::fade(self.skin.shadow, 0.16)),
                width: 1.0,
                ..Stroke::default()
            },
        );

        let puck = puck_at(center, radius, self.hue, self.saturation);

        for step in 0..4 {
            let spread = PUCK + 2.0 + step as f32 * 2.5;
            overlay.fill(
                &Path::circle(puck, spread),
                tone::fade(self.skin.shadow, 0.05),
            );
        }

        overlay.fill(&Path::circle(puck, PUCK), Color::WHITE);
        overlay.fill(
            &Path::circle(puck, PUCK - 4.0),
            tone::hsv(self.hue, self.saturation, 1.0),
        );

        vec![disc, overlay.into_geometry()]
    }

    fn mouse_interaction(
        &self,
        _state: &Self::State,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> mouse::Interaction {
        match cursor.position_over(bounds) {
            Some(at) => {
                let (_, saturation) = Self::read(bounds, at);
                if saturation < 1.0 {
                    mouse::Interaction::Pointer
                } else {
                    mouse::Interaction::None
                }
            }
            None => mouse::Interaction::None,
        }
    }
}

/// Places a colour disc of the given side length.
pub fn wheel<'a, Message>(
    disc: Wheel<'a, Message>,
    side: f32,
) -> Element<'a, Message>
where
    Message: Clone + 'a,
{
    Canvas::new(disc)
        .width(Length::Fixed(side))
        .height(Length::Fixed(side))
        .into()
}
