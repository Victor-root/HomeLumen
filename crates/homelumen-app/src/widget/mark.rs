//! The HomeLumen mark: a house with its bulb lit, one warm gold gradient
//! running through both.
//!
//! Every coordinate here is measured off the vector artwork HomeLumen ships
//! with (fractions of its 1254-unit canvas), not eyeballed: see
//! `crate::icon`, which shares the same numbers for the window icon.

use iced::mouse;
use iced::widget::canvas::{
    self, Canvas, Frame, Geometry, Path, Stroke, Style,
};
use iced::{
    Color, Element, Length, Point, Rectangle, Renderer, Size, Theme, Vector,
};

/// The roofline, stroked as one open path rather than closed along the
/// floor: a wall, the eave tip reaching past it, the peak, and back down
/// the mirrored side. Each wall ends where its stroke does; the flare
/// beneath it is a separate shape, see `FOOT_LEFT`/`FOOT_RIGHT`.
const ROOF: [(f32, f32); 7] = [
    (0.2583, 0.6802),
    (0.2583, 0.4705),
    (0.2104, 0.4322),
    (0.4959, 0.1974),
    (0.7814, 0.4322),
    (0.7336, 0.4705),
    (0.7336, 0.6802),
];

/// Width of the roofline stroke.
const ROOF_STROKE: f32 = 0.0477;

/// The chimney, breaking the right roof slope: top-left corner, size.
const CHIMNEY: (f32, f32, f32, f32) = (0.6475, 0.2360, 0.0772, 0.1140);

/// Each wall flares into a foot wider than the wall itself, rather than
/// the stroke just rounding off at its end: seven points tracing the
/// flare's own edge (it fills a shape, so unlike `ROOF` these are the
/// boundary, not a centreline the stroke would widen out from).
const FOOT_LEFT: [(f32, f32); 7] = [
    (0.2345, 0.6802),
    (0.3086, 0.7520),
    (0.3947, 0.7520),
    (0.4075, 0.7376),
    (0.3684, 0.6986),
    (0.3397, 0.6986),
    (0.2823, 0.6427),
];
const FOOT_RIGHT: [(f32, f32); 7] = [
    (0.7574, 0.6802),
    (0.6833, 0.7520),
    (0.5971, 0.7520),
    (0.5844, 0.7376),
    (0.6234, 0.6986),
    (0.6522, 0.6986),
    (0.7096, 0.6427),
];

/// The bulb's globe: centre, radius.
const GLOBE: (f32, f32, f32) = (0.496, 0.489, 0.126);

/// The threaded base below the globe: two bands, narrower than the last,
/// then a domed tip. Each as (half width, half height), stacked by their
/// own measured gaps rather than an even split.
const BAND_1: (f32, f32) = (0.061, 0.0115);
const BAND_2: (f32, f32) = (0.0575, 0.0115);
const TIP: (f32, f32) = (0.0375, 0.0205);
const GLOBE_TO_BAND_1: f32 = 0.016;
const BAND_GAP: f32 = 0.012;
const BAND_TO_TIP: f32 = 0.013;

/// The brand mark.
pub struct Mark;

impl<Message> canvas::Program<Message> for Mark {
    type State = ();

    fn draw(
        &self,
        _state: &Self::State,
        renderer: &Renderer,
        _theme: &Theme,
        bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<Geometry> {
        let mut frame = Frame::new(renderer, bounds.size());
        let side = bounds.width.min(bounds.height);
        let origin = frame.center() - Vector::new(side / 2.0, side / 2.0);
        let at = |x: f32, y: f32| origin + Vector::new(x * side, y * side);

        let gold = || {
            canvas::gradient::Linear::new(at(0.5, 0.16), at(0.5, 0.92))
                .add_stop(0.0, Color::from_rgb(1.0, 0.910, 0.671))
                .add_stop(1.0, Color::from_rgb(0.929, 0.639, 0.180))
        };

        let polygon = |points: &[(f32, f32)]| {
            Path::new(|path| {
                path.move_to(at(points[0].0, points[0].1));
                for (x, y) in &points[1..] {
                    path.line_to(at(*x, *y));
                }
                path.close();
            })
        };

        let chimney = Path::rounded_rectangle(
            at(CHIMNEY.0, CHIMNEY.1),
            Size::new(CHIMNEY.2 * side, CHIMNEY.3 * side),
            (CHIMNEY.2 * side * 0.134).into(),
        );
        frame.fill(&chimney, gold());

        let house = Path::new(|path| {
            path.move_to(at(ROOF[0].0, ROOF[0].1));
            for (x, y) in &ROOF[1..] {
                path.line_to(at(*x, *y));
            }
        });

        frame.stroke(
            &house,
            Stroke {
                style: Style::Gradient(gold().into()),
                width: ROOF_STROKE * side,
                line_cap: canvas::LineCap::Round,
                line_join: canvas::LineJoin::Round,
                ..Stroke::default()
            },
        );

        frame.fill(&polygon(&FOOT_LEFT), gold());
        frame.fill(&polygon(&FOOT_RIGHT), gold());

        // The bulb: a globe over a threaded base, the same gradient as the
        // house rather than a glass tone of its own.
        let glass = at(GLOBE.0, GLOBE.1);
        let radius = GLOBE.2 * side;

        frame.fill(&Path::circle(glass, radius), gold());

        let band = |half: (f32, f32), center_y: f32| {
            Path::rounded_rectangle(
                Point::new(
                    glass.x - half.0 * side,
                    at(0.0, center_y).y - half.1 * side,
                ),
                Size::new(half.0 * 2.0 * side, half.1 * 2.0 * side),
                (half.1 * side).into(),
            )
        };

        let band_1_y = GLOBE.1 + GLOBE.2 + GLOBE_TO_BAND_1 + BAND_1.1;
        frame.fill(&band(BAND_1, band_1_y), gold());

        let band_2_y = band_1_y + BAND_1.1 + BAND_GAP + BAND_2.1;
        frame.fill(&band(BAND_2, band_2_y), gold());

        let tip_y = band_2_y + BAND_2.1 + BAND_TO_TIP + TIP.1;
        frame.fill(&band(TIP, tip_y), gold());

        vec![frame.into_geometry()]
    }
}

/// Places the mark at the given side length.
pub fn mark<'a, Message>(side: f32) -> Element<'a, Message>
where
    Message: 'a,
{
    Canvas::new(Mark)
        .width(Length::Fixed(side))
        .height(Length::Fixed(side))
        .into()
}
