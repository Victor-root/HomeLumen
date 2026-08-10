//! The handful of marks the interface needs, drawn rather than fetched.
//!
//! Keeping them as vectors means no icon font, no asset pipeline, and a stroke
//! weight that matches the typography at every size. Every shape below is
//! transcribed from a Tabler Icons outline: each straight run and each arc
//! carries the exact numbers Tabler draws it with, just rebased as a
//! fraction of `side` and an offset from the frame's own centre instead of a
//! 24-unit viewbox, so the glyph stays crisp at whatever size a button asks
//! it to be.

use std::f32::consts::{FRAC_PI_2, PI, TAU};

use iced::mouse;
use iced::time::{Duration, Instant};
use iced::widget::canvas::path::{Arc, Builder};
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
    /// Open the settings menu.
    Settings,
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

/// One edge of a traced outline: a straight run to a point, or an arc around
/// a centre. Every number is a fraction of `side`, an offset from the
/// frame's own centre for points and arc centres alike, so the same table
/// draws the glyph at any size.
#[derive(Clone, Copy)]
enum Seg {
    Line(f32, f32),
    Arc(f32, f32, f32, f32, f32),
}

/// How finely an arc is sampled when it is flattened into line segments.
/// Plenty for the small radii these outlines use, at any size a button asks
/// the glyph to draw itself at.
const ARC_STEPS: u32 = 20;

/// Traces `segs` onto `path`, continuing from wherever it already is.
///
/// Arcs are sampled into line segments rather than drawn with
/// [`Builder::arc`]: that call always starts a fresh sub-path underneath,
/// which is exactly right for a standalone ring but silently breaks a
/// multi-arc outline into disconnected pieces, each one closed on its own
/// rather than carrying on to the next segment.
fn trace(path: &mut Builder, center: Point, side: f32, segs: &[Seg]) {
    for seg in segs {
        match *seg {
            Seg::Line(x, y) => path.line_to(center + Vector::new(x, y) * side),
            Seg::Arc(cx, cy, r, from, to) => {
                arc_line_to(
                    path,
                    center + Vector::new(cx, cy) * side,
                    r * side,
                    from,
                    to,
                );
            }
        }
    }
}

/// Samples an arc as a run of straight lines from wherever the path already
/// is, so it stays part of the current sub-path. See [`trace`] for why.
fn arc_line_to(
    path: &mut Builder,
    arc_center: Point,
    radius: f32,
    from: f32,
    to: f32,
) {
    for step in 1..=ARC_STEPS {
        let t = from + (to - from) * (step as f32 / ARC_STEPS as f32);
        path.line_to(arc_center + Vector::new(t.cos(), t.sin()) * radius);
    }
}

/// Moves to a fractional point, the way every traced outline below starts.
fn start(path: &mut Builder, center: Point, side: f32, x: f32, y: f32) {
    path.move_to(center + Vector::new(x, y) * side);
}

/// The rounded-octagon badge behind the "A" in the auto-brightness mark, and
/// the "A" itself: three subpaths of one fill, the letter's counter cut out
/// by an even-odd rule rather than by winding direction.
const AUTO_BADGE: [Seg; 32] = [
    Seg::Line(0.12146, -0.29167),
    Seg::Line(0.25000, -0.29167),
    Seg::Arc(0.24999, -0.25000, 0.04167, -1.57066, -0.11727),
    Seg::Line(0.29167, -0.25000),
    Seg::Line(0.29167, -0.12146),
    Seg::Line(0.38363, -0.02946),
    Seg::Arc(0.35416, 0.00000, 0.04167, -0.78535, 0.65976),
    Seg::Line(0.38363, 0.02946),
    Seg::Line(0.29167, 0.12142),
    Seg::Line(0.29167, 0.25000),
    Seg::Arc(0.25000, 0.24999, 0.04167, 0.00013, 1.45353),
    Seg::Line(0.25000, 0.29167),
    Seg::Line(0.12142, 0.29167),
    Seg::Line(0.02946, 0.38363),
    Seg::Arc(-0.00000, 0.35416, 0.04167, 0.78545, 2.23056),
    Seg::Line(-0.02946, 0.38363),
    Seg::Line(-0.12146, 0.29167),
    Seg::Line(-0.25000, 0.29167),
    Seg::Arc(-0.24999, 0.25000, 0.04167, 1.57093, 3.02432),
    Seg::Line(-0.29167, 0.25000),
    Seg::Line(-0.29167, 0.12146),
    Seg::Line(-0.38362, 0.02946),
    Seg::Arc(-0.35416, -0.00000, 0.04167, 2.35624, 3.80135),
    Seg::Line(-0.38362, -0.02946),
    Seg::Line(-0.29167, -0.12150),
    Seg::Line(-0.29167, -0.25000),
    Seg::Arc(-0.25000, -0.24999, 0.04167, -3.14146, -1.68806),
    Seg::Line(-0.25000, -0.29167),
    Seg::Line(-0.12150, -0.29167),
    Seg::Line(-0.02946, -0.38363),
    Seg::Arc(0.00000, -0.35416, 0.04167, -2.35604, -0.78555),
    Seg::Line(0.02946, -0.38363),
];

const AUTO_LETTER: [Seg; 11] = [
    Seg::Arc(0.00000, -0.04167, 0.12500, -FRAC_PI_2, -PI),
    Seg::Line(-0.12500, 0.10417),
    Seg::Arc(-0.08333, 0.10417, 0.04167, PI, 0.00000),
    Seg::Line(-0.04167, 0.08333),
    Seg::Line(0.04167, 0.08333),
    Seg::Line(0.04167, 0.10417),
    Seg::Arc(0.08333, 0.10416, 0.04167, 3.14146, 1.68806),
    Seg::Line(0.08333, 0.14583),
    Seg::Arc(0.08333, 0.10417, 0.04167, FRAC_PI_2, 0.00000),
    Seg::Line(0.12500, -0.04167),
    Seg::Arc(0.00000, -0.04167, 0.12500, 0.00000, -FRAC_PI_2),
];

const AUTO_LETTER_HOLE: [Seg; 5] = [
    Seg::Arc(0.00000, -0.04167, 0.04167, -FRAC_PI_2, 0.00000),
    Seg::Line(0.04167, 0.00000),
    Seg::Line(-0.04167, 0.00000),
    Seg::Line(-0.04167, -0.04167),
    Seg::Arc(-0.00000, -0.04166, 0.04167, -3.14146, -1.68806),
];

/// The settings gear's rounded-hexagon body, an outline rather than a fill:
/// every corner here is its own small arc, not the stroke's own line join.
const SETTINGS_BODY: [Seg; 10] = [
    Seg::Arc(0.28229, -0.15816, 0.09271, -1.05370, 0.00625),
    Seg::Line(0.37500, 0.14592),
    Seg::Line(0.04550, 0.40500),
    Seg::Arc(-0.00000, 0.32213, 0.09454, 1.06869, 2.07290),
    Seg::Line(-0.32675, 0.22708),
    Seg::Arc(-0.28229, 0.14573, 0.09271, 2.07094, 3.13958),
    Seg::Line(-0.37500, -0.15763),
    Seg::Line(-0.04550, -0.40458),
    Seg::Arc(0.00137, -0.31957, 0.09708, -2.07468, -1.06691),
    Seg::Line(0.32950, -0.23875),
];

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
                let arm = side * 0.29167;
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
                // One outline: a wide arc for the disc's edge, a short curve
                // and a narrower arc biting back into it, closed by the
                // sliver of the two horns. A true crescent, not two circles
                // masking each other.
                frame.fill(
                    &Path::new(|path| {
                        start(path, center, side, 0.00000, -0.41700);
                        arc_line_to(
                            path,
                            center + Vector::new(0.00008, -0.00033) * side,
                            0.41667 * side,
                            -1.57098,
                            -5.88928,
                        );
                        path.bezier_curve_to(
                            center + Vector::new(0.39904, 0.12542) * side,
                            center + Vector::new(0.36500, 0.09108) * side,
                            center + Vector::new(0.33075, 0.10500) * side,
                        );
                        arc_line_to(
                            path,
                            center + Vector::new(0.22907, -0.14602) * side,
                            0.27083 * side,
                            1.18593,
                            3.96386,
                        );
                        path.line_to(
                            center + Vector::new(0.04796, -0.34779) * side,
                        );
                        path.bezier_curve_to(
                            center + Vector::new(0.07092, -0.37404) * side,
                            center + Vector::new(0.05267, -0.41667) * side,
                            center + Vector::new(0.01638, -0.41667) * side,
                        );
                        path.close();
                    }),
                    self.color,
                );
            }

            Glyph::Sun => {
                frame.stroke(&Path::circle(center, side * 0.12500), line);

                for ray in 0..8 {
                    let angle = ray as f32 / 8.0 * TAU;
                    let direction = Vector::new(angle.cos(), angle.sin());
                    frame.stroke(
                        &Path::line(
                            center + direction * (side * 0.29167),
                            center + direction * (side * 0.37500),
                        ),
                        line,
                    );
                }
            }

            Glyph::Auto => {
                frame.fill(
                    &Path::new(|path| {
                        start(path, center, side, 0.02946, -0.38363);
                        trace(path, center, side, &AUTO_BADGE);
                        path.close();

                        start(path, center, side, 0.00000, -0.16667);
                        trace(path, center, side, &AUTO_LETTER);
                        path.close();

                        start(path, center, side, 0.00000, -0.08333);
                        trace(path, center, side, &AUTO_LETTER_HOLE);
                        path.close();
                    }),
                    Fill {
                        style: Style::Solid(self.color),
                        rule: fill::Rule::EvenOdd,
                    },
                );
            }

            Glyph::Sweep { busy } => {
                let phase = match (state.origin, state.now, busy) {
                    (Some(origin), Some(now), true) => {
                        (now - origin).as_secs_f32() * 2.4
                    }
                    _ => 0.0,
                };

                let head = |r: f32, a: f32| {
                    center
                        + Vector::new((a + phase).cos(), (a + phase).sin())
                            * (side * r)
                };

                frame.stroke(
                    &Path::new(|path| {
                        path.arc(Arc {
                            center,
                            radius: side * 0.33333,
                            start_angle: Radians(-3.01126 + phase),
                            end_angle: Radians(2.76255 + phase),
                        });

                        path.move_to(head(0.46993, 2.35306));
                        path.line_to(head(0.35405, 2.78076));
                        path.line_to(head(0.17531, 2.34779));
                    }),
                    line,
                );
            }

            Glyph::Settings => {
                frame.stroke(
                    &Path::new(|path| {
                        start(path, center, side, 0.32812, -0.23875);
                        trace(path, center, side, &SETTINGS_BODY);
                        path.close();
                    }),
                    line,
                );

                frame.stroke(&Path::circle(center, side * 0.12500), line);
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
