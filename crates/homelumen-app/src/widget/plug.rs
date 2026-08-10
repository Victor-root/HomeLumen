//! The outlet at the top of a plug's screen.
//!
//! Drawn from the same glossy material as [`crate::widget::bulb`], so a plug
//! reads as a sibling of a light rather than a different product, but shaped
//! as the socket it is rather than a lamp: a plug switches power to whatever
//! is drawn into it, it does not itself give off light, so unlike the bulb it
//! never breathes.

use iced::mouse;
use iced::widget::canvas::{
    self, Canvas, Frame, Geometry, Path, Stroke, Style,
};
use iced::{
    Color, Element, Length, Point, Rectangle, Renderer, Size, Theme, Vector,
};

use crate::design::Skin;
use crate::widget::bulb;

/// A plug, drawn as the outlet it switches.
pub struct Plug {
    glow: Color,
    intensity: f32,
    skin: Skin,
}

impl Plug {
    /// Builds a plug putting out `intensity` of `glow` onto the page around it.
    pub fn new(glow: Color, intensity: f32, skin: Skin) -> Self {
        Self { glow, intensity: intensity.clamp(0.0, 1.0), skin }
    }
}

impl<Message> canvas::Program<Message> for Plug {
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
        let center = frame.center();
        let radius = side * 0.30;

        draw_body(
            &mut frame,
            center,
            radius,
            self.skin,
            self.glow,
            self.intensity,
        );

        vec![frame.into_geometry()]
    }
}

/// The socket body: a rounded face in the same gradient glass every light
/// uses, marked with the two slots that read as an outlet anywhere in the
/// world.
fn draw_body(
    frame: &mut Frame,
    center: Point,
    radius: f32,
    skin: Skin,
    glow: Color,
    intensity: f32,
) {
    let tones = bulb::tones(skin, glow, intensity);
    let side = radius * 1.8;
    let corner = side * 0.22;
    let face = Rectangle {
        x: center.x - side / 2.0,
        y: center.y - side / 2.0,
        width: side,
        height: side,
    };

    frame.fill(
        &Path::rounded_rectangle(
            Point::new(face.x, face.y),
            Size::new(face.width, face.height),
            corner.into(),
        ),
        canvas::gradient::Linear::new(
            Point::new(center.x, face.y),
            Point::new(center.x, face.y + face.height),
        )
        .add_stop(0.0, tones.top)
        .add_stop(1.0, tones.bottom),
    );

    frame.stroke(
        &Path::rounded_rectangle(
            Point::new(face.x, face.y),
            Size::new(face.width, face.height),
            corner.into(),
        ),
        Stroke {
            style: Style::Solid(tones.stroke),
            width: 1.0,
            ..Stroke::default()
        },
    );

    for slot in slots(center, side) {
        frame.fill(
            &Path::rounded_rectangle(
                Point::new(slot.x, slot.y),
                Size::new(slot.width, slot.height),
                (slot.width / 2.0).into(),
            ),
            tones.metal,
        );
    }

    // A single specular highlight, same relative placement as the bulb's
    // glass, is what keeps the two reading as the same material.
    frame.fill(
        &Path::circle(
            center + Vector::new(-radius * 0.33, -radius * 0.38),
            radius * 0.26,
        ),
        tones.highlight,
    );
}

/// The two outlet slots, centred a touch above the middle of the face.
fn slots(center: Point, side: f32) -> [Rectangle; 2] {
    let width = side * 0.10;
    let height = side * 0.30;
    let gap = side * 0.18;
    let top = center.y - height / 2.0 - side * 0.06;

    [
        Rectangle { x: center.x - gap / 2.0 - width, y: top, width, height },
        Rectangle { x: center.x + gap / 2.0, y: top, width, height },
    ]
}

/// Places a plug of the given side length.
pub fn plug<'a, Message>(outlet: Plug, side: f32) -> Element<'a, Message>
where
    Message: 'a,
{
    Canvas::new(outlet)
        .width(Length::Fixed(side))
        .height(Length::Fixed(side))
        .into()
}
