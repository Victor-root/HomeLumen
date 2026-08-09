//! The handful of marks the interface needs, drawn rather than fetched.
//!
//! Keeping them as vectors means no icon font, no asset pipeline, and a stroke
//! weight that matches the typography at every size.

use std::f32::consts::{FRAC_PI_2, PI, TAU};

use iced::mouse;
use iced::time::{Duration, Instant};
use iced::widget::canvas::path::Arc;
use iced::widget::canvas::{
    self, Canvas, Fill, Frame, Geometry, Path, Stroke, Style, fill,
};
use iced::{
    Color, Element, Length, Point, Radians, Rectangle, Renderer, Theme, Vector,
};

/// The marks HomeLumen draws.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Glyph {
    /// Add something.
    Plus,
    /// Go back.
    Back,
    /// The dark skin.
    Moon,
    /// The light skin.
    Sun,
    /// Whichever skin the desktop is set to.
    Auto,
    /// Look for lights. Turns while a sweep is running.
    Sweep {
        /// Whether a sweep is running right now.
        busy: bool,
    },
}

struct Drawing {
    glyph: Glyph,
    color: Color,
}

/// Kept by the runtime between frames, so a sweep can turn.
#[derive(Default)]
pub struct State {
    origin: Option<Instant>,
    now: Option<Instant>,
}

impl<Message> canvas::Program<Message> for Drawing {
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

        matches!(self.glyph, Glyph::Sweep { busy: true }).then(|| {
            canvas::Action::request_redraw_at(*now + Duration::from_millis(24))
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
        let side = bounds.width.min(bounds.height);
        let weight = (side * 0.11).max(1.6);

        let line = Stroke {
            style: Style::Solid(self.color),
            width: weight,
            line_cap: canvas::LineCap::Round,
            line_join: canvas::LineJoin::Round,
            ..Stroke::default()
        };

        match self.glyph {
            Glyph::Plus => {
                let arm = side * 0.30;
                frame.stroke(
                    &Path::line(
                        Point::new(center.x - arm, center.y),
                        Point::new(center.x + arm, center.y),
                    ),
                    line,
                );
                frame.stroke(
                    &Path::line(
                        Point::new(center.x, center.y - arm),
                        Point::new(center.x, center.y + arm),
                    ),
                    line,
                );
            }

            Glyph::Back => {
                let reach = side * 0.24;
                frame.stroke(
                    &Path::new(|path| {
                        path.move_to(Point::new(
                            center.x + reach * 0.6,
                            center.y - reach,
                        ));
                        path.line_to(Point::new(
                            center.x - reach * 0.6,
                            center.y,
                        ));
                        path.line_to(Point::new(
                            center.x + reach * 0.6,
                            center.y + reach,
                        ));
                    }),
                    line,
                );
            }

            Glyph::Moon => {
                // Two discs and an even-odd fill: whatever they share is cut
                // out, which leaves a true crescent rather than a masked one.
                let radius = side * 0.34;
                frame.fill(
                    &Path::new(|path| {
                        path.circle(center, radius);
                        path.circle(
                            center + Vector::new(radius * 0.55, -radius * 0.42),
                            radius * 0.92,
                        );
                    }),
                    Fill {
                        style: Style::Solid(self.color),
                        rule: fill::Rule::EvenOdd,
                    },
                );
            }

            Glyph::Sun => {
                frame.fill(&Path::circle(center, side * 0.19), self.color);

                for ray in 0..8 {
                    let angle = ray as f32 / 8.0 * TAU;
                    let direction = Vector::new(angle.cos(), angle.sin());
                    frame.stroke(
                        &Path::line(
                            center + direction * (side * 0.28),
                            center + direction * (side * 0.40),
                        ),
                        line,
                    );
                }
            }

            Glyph::Auto => {
                // A disc half outlined and half filled: the two skins in one
                // mark, which is the shape a desktop uses for exactly this.
                let radius = side * 0.30;

                frame.fill(
                    &Path::new(|path| {
                        path.move_to(Point::new(center.x, center.y - radius));
                        path.arc(Arc {
                            center,
                            radius,
                            start_angle: Radians(-FRAC_PI_2),
                            end_angle: Radians(FRAC_PI_2),
                        });
                        path.close();
                    }),
                    self.color,
                );

                frame.stroke(&Path::circle(center, radius), line);
            }

            Glyph::Sweep { busy } => {
                let radius = side * 0.32;
                let phase = match (state.origin, state.now, busy) {
                    (Some(origin), Some(now), true) => {
                        (now - origin).as_secs_f32() * 2.4
                    }
                    _ => 0.0,
                };

                let from = phase - FRAC_PI_2;
                let sweep = if busy { PI * 1.15 } else { PI * 1.55 };

                frame.stroke(
                    &Path::new(|path| {
                        path.arc(Arc {
                            center,
                            radius,
                            start_angle: Radians(from),
                            end_angle: Radians(from + sweep),
                        });
                    }),
                    line,
                );

                let tip = from + sweep;
                frame.fill(
                    &Path::circle(
                        center + Vector::new(tip.cos(), tip.sin()) * radius,
                        weight * 0.85,
                    ),
                    self.color,
                );
            }
        }

        vec![frame.into_geometry()]
    }
}

/// Places a mark of the given side length.
pub fn glyph<'a, Message>(
    glyph: Glyph,
    color: Color,
    side: f32,
) -> Element<'a, Message>
where
    Message: 'a,
{
    Canvas::new(Drawing { glyph, color })
        .width(Length::Fixed(side))
        .height(Length::Fixed(side))
        .into()
}
