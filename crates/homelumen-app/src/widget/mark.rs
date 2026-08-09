//! The HomeLumen mark: an open halo, warm where the light starts and cool where
//! it fades, with the point of emission left as a solid dot.

use std::f32::consts::{PI, TAU};

use iced::mouse;
use iced::widget::canvas::path::Arc;
use iced::widget::canvas::{
    self, Canvas, Frame, Geometry, Path, Stroke, Style,
};
use iced::{
    Color, Element, Length, Point, Radians, Rectangle, Renderer, Theme, Vector,
};

/// The gap in the halo, as a fraction of a full turn.
const OPENING: f32 = 0.18;

/// Where the halo starts, measured from the top and turning clockwise.
const START: f32 = 0.10;

/// The brand mark.
///
/// Its colours come from the light itself rather than from the skin, so the
/// halo reads the same on ink and on paper.
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
        let center = frame.center();
        let side = bounds.width.min(bounds.height);
        let radius = side / 2.0 - side * 0.14;
        let stroke = side * 0.13;

        let from = TAU * START - PI / 2.0;
        let to = from + TAU * (1.0 - OPENING);

        let halo = Path::new(|path| {
            path.arc(Arc {
                center,
                radius,
                start_angle: Radians(from),
                end_angle: Radians(to),
            });
        });

        let warm = Color::from_rgb(1.0, 0.541, 0.169);
        let cool = Color::from_rgb(0.561, 0.714, 1.0);

        frame.stroke(
            &halo,
            Stroke {
                style: Style::Gradient(
                    canvas::gradient::Linear::new(
                        center + Vector::new(-radius, radius),
                        center + Vector::new(radius, -radius),
                    )
                    .add_stop(0.0, warm)
                    .add_stop(0.52, Color::from_rgb(1.0, 0.769, 0.420))
                    .add_stop(1.0, cool)
                    .into(),
                ),
                width: stroke,
                line_cap: canvas::LineCap::Round,
                ..Stroke::default()
            },
        );

        let head = Point::new(
            center.x + radius * from.cos(),
            center.y + radius * from.sin(),
        );

        frame.fill(&Path::circle(head, stroke * 0.62), warm);

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
