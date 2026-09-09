//! The HomeLumen mark: a house with its bulb lit, one warm gold gradient
//! running through both.

use iced::mouse;
use iced::widget::canvas::{
    self, Canvas, Frame, Geometry, Path, Stroke, Style,
};
use iced::{
    Color, Element, Length, Point, Rectangle, Renderer, Size, Theme, Vector,
};

/// The roofline: five points tracing the house from eave to eave, closed
/// along the floor. Fractions of the mark's own side, not the skin: the
/// mark reads the same on ink and on paper, so its colours never come from
/// there either.
const ROOF: [(f32, f32); 5] =
    [(0.12, 0.58), (0.50, 0.18), (0.88, 0.58), (0.88, 0.96), (0.12, 0.96)];

/// The chimney, breaking the right roof slope: top-left corner, size.
const CHIMNEY: (f32, f32, f32, f32) = (0.64, 0.26, 0.10, 0.20);

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

        let chimney = Path::rounded_rectangle(
            at(CHIMNEY.0, CHIMNEY.1),
            Size::new(CHIMNEY.2 * side, CHIMNEY.3 * side),
            (CHIMNEY.2 * side * 0.25).into(),
        );
        frame.fill(&chimney, gold());

        let house = Path::new(|path| {
            path.move_to(at(ROOF[0].0, ROOF[0].1));
            for (x, y) in &ROOF[1..] {
                path.line_to(at(*x, *y));
            }
            path.close();
        });

        frame.stroke(
            &house,
            Stroke {
                style: Style::Gradient(gold().into()),
                width: side * 0.06,
                line_cap: canvas::LineCap::Round,
                line_join: canvas::LineJoin::Round,
                ..Stroke::default()
            },
        );

        // The bulb: a globe over a threaded base, the same gradient as the
        // house rather than a glass tone of its own.
        let glass = at(0.50, 0.58);
        let radius = side * 0.19;

        frame.fill(&Path::circle(glass, radius), gold());

        let thread_size = Size::new(radius * 1.1, side * 0.04);
        let thread_gap = side * 0.012;
        let base_start = glass.y + radius + thread_size.height / 2.0;
        let band_top = |band: i32| {
            base_start - thread_size.height / 2.0
                + (band as f32) * (thread_size.height + thread_gap)
        };

        for band in 0..3 {
            frame.fill(
                &Path::rounded_rectangle(
                    Point::new(
                        glass.x - thread_size.width / 2.0,
                        band_top(band),
                    ),
                    thread_size,
                    (thread_size.height / 2.0).into(),
                ),
                gold(),
            );
        }

        let tip_radius = side * 0.014;
        let tip_y = band_top(2) + thread_size.height + thread_gap + tip_radius;
        frame.fill(
            &Path::circle(Point::new(glass.x, tip_y), tip_radius),
            gold(),
        );

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
