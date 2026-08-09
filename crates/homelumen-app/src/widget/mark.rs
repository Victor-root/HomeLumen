//! The HomeLumen mark: a house with its bulb lit, drawn with the same glass
//! sphere every light on screen uses.

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

        let frame_tone = Color::from_rgb(0.580, 0.529, 0.463);

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
                style: Style::Solid(frame_tone),
                width: side * 0.06,
                line_cap: canvas::LineCap::Round,
                line_join: canvas::LineJoin::Round,
                ..Stroke::default()
            },
        );

        // The bulb: the same glass-over-a-socket sphere every light on
        // screen is drawn from, just sized to fill the house rather than
        // sit beside a name.
        let glass = at(0.50, 0.66);
        let radius = side * 0.22;

        let base_size = Size::new(side * 0.18, side * 0.10);
        frame.fill(
            &Path::rounded_rectangle(
                glass + Vector::new(-base_size.width / 2.0, radius * 0.62),
                base_size,
                (base_size.height * 0.32).into(),
            ),
            frame_tone,
        );

        frame.fill(
            &Path::circle(glass, radius),
            canvas::gradient::Linear::new(
                Point::new(glass.x, glass.y - radius),
                Point::new(glass.x, glass.y + radius),
            )
            .add_stop(0.0, Color::from_rgb(1.0, 0.859, 0.624))
            .add_stop(1.0, Color::from_rgb(0.949, 0.651, 0.247)),
        );

        frame.stroke(
            &Path::circle(glass, radius),
            Stroke {
                style: Style::Solid(Color::from_rgb(0.804, 0.502, 0.180)),
                width: side * 0.008,
                ..Stroke::default()
            },
        );

        frame.fill(
            &Path::circle(
                glass + Vector::new(-radius * 0.33, -radius * 0.38),
                radius * 0.26,
            ),
            Color { a: 0.35, ..Color::WHITE },
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
