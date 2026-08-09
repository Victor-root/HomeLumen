//! The colour field.
//!
//! Hue runs left to right, saturation runs bottom to top. A rectangle rather
//! than a disc, so it can be as wide as the panel that holds it instead of
//! being capped by whichever side of a square runs out of room first.

use iced::widget::canvas::{
    self, Cache, Canvas, Frame, Geometry, Path, Stroke, Style,
};
use iced::{Color, Element, Length, Point, Rectangle, Renderer, Size, Theme};
use iced::{mouse, touch};

use crate::design::{Skin, tone};

/// How many columns the field is built from. Past this the hue seams are
/// invisible.
const COLUMNS: usize = 240;

const PUCK: f32 = 15.0;

/// Margin kept between the puck's travel and the field's edge, so its glow
/// never runs past the canvas at a corner value like pure white or a hue of
/// exactly zero.
const MARGIN: f32 = 20.0;

/// A hue and saturation picker.
pub struct Field<'a, Message> {
    hue: f32,
    saturation: f32,
    skin: Skin,
    on_change: Box<dyn Fn(f32, f32) -> Message + 'a>,
}

impl<'a, Message> Field<'a, Message> {
    /// Builds a field showing `hue` in degrees and `saturation` in `0.0..=1.0`.
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

    /// Turns a point inside the field into a hue and a saturation.
    fn read(bounds: Rectangle, at: Point) -> (f32, f32) {
        let area = bounds.shrink(MARGIN);
        let hue = (at.x - area.x) / area.width * 360.0;
        let saturation = 1.0 - (at.y - area.y) / area.height;

        (hue.rem_euclid(360.0), saturation.clamp(0.0, 1.0))
    }
}

/// Where the puck sits for a given hue and saturation, in the field's own
/// local coordinates.
fn puck_at(size: Size, hue: f32, saturation: f32) -> Point {
    let area = Rectangle::with_size(size).shrink(MARGIN);

    Point::new(
        area.x + area.width * (hue.rem_euclid(360.0) / 360.0),
        area.y + area.height * (1.0 - saturation),
    )
}

/// Kept by the runtime between frames: the grab, the live drag position while
/// holding, and the field itself.
pub struct State {
    holding: bool,
    shown: (f32, f32),
    field: Cache,
}

impl Default for State {
    fn default() -> Self {
        Self { holding: false, shown: (0.0, 0.0), field: Cache::default() }
    }
}

impl<Message> canvas::Program<Message> for Field<'_, Message>
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
                let (hue, saturation) = Self::read(bounds, at);

                state.holding = true;
                state.shown = (hue, saturation);
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

                state.shown = (hue, saturation);
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
        // The field never changes, so it is built once and kept.
        let field = state.field.draw(renderer, bounds.size(), |frame| {
            for column in 0..COLUMNS {
                let from = column as f32 / COLUMNS as f32 * bounds.width;
                let to = (column + 1) as f32 / COLUMNS as f32 * bounds.width;
                let hue = (column as f32 + 0.5) / COLUMNS as f32 * 360.0;

                let strip = Path::rectangle(
                    Point::new(from, 0.0),
                    Size::new(to - from, bounds.height),
                );

                frame.fill(
                    &strip,
                    canvas::gradient::Linear::new(
                        Point::new(from, 0.0),
                        Point::new(from, bounds.height),
                    )
                    .add_stop(0.0, tone::hsv(hue, 1.0, 1.0))
                    .add_stop(1.0, Color::WHITE),
                );
            }
        });

        let mut overlay = Frame::new(renderer, bounds.size());

        overlay.stroke(
            &Path::rectangle(Point::ORIGIN, bounds.size()),
            Stroke {
                style: Style::Solid(tone::fade(self.skin.shadow, 0.16)),
                width: 1.0,
                ..Stroke::default()
            },
        );

        let (hue, saturation) = if state.holding {
            state.shown
        } else {
            (self.hue, self.saturation)
        };
        let puck = puck_at(bounds.size(), hue, saturation);

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
            tone::hsv(hue, saturation, 1.0),
        );

        vec![field, overlay.into_geometry()]
    }

    fn mouse_interaction(
        &self,
        _state: &Self::State,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> mouse::Interaction {
        if cursor.is_over(bounds) {
            mouse::Interaction::Pointer
        } else {
            mouse::Interaction::None
        }
    }
}

/// Places a colour field of the given height, filling whatever width it is
/// given.
pub fn field<'a, Message>(
    program: Field<'a, Message>,
    height: f32,
) -> Element<'a, Message>
where
    Message: Clone + 'a,
{
    Canvas::new(program)
        .width(Length::Fill)
        .height(Length::Fixed(height))
        .into()
}
