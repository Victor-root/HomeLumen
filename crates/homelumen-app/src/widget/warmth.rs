//! The white temperature band.
//!
//! The control *is* the scale: the surface shows the actual colour of every
//! white the light can produce, from its warmest to its coolest, so choosing
//! one is a matter of pointing at it rather than reading a number.

use std::f32::consts::PI;

use iced::advanced::layout::{self, Layout};
use iced::advanced::widget::{Widget, tree};
use iced::advanced::{Clipboard, Renderer as _, Shell, mouse, renderer};
use iced::{
    Background, Border, Color, Element, Event, Length, Radians, Rectangle,
    Renderer, Shadow, Size, Theme, Vector, gradient,
};

use crate::design::{Skin, tone};
use crate::paint;
use crate::widget::track::{self, Gesture, Slide};

/// Height of the band at its most spacious.
pub const HEIGHT: f32 = 116.0;

const KNOB_WIDTH: f32 = 14.0;
const KNOB_INSET: f32 = 9.0;

/// A white temperature control.
pub struct Warmth<'a, Message> {
    value: f32,
    warmest: f32,
    coolest: f32,
    skin: Skin,
    height: f32,
    on_change: Box<dyn Fn(f32) -> Message + 'a>,
}

impl<'a, Message> Warmth<'a, Message> {
    /// Builds a band spanning `warmest..=coolest` kelvin, sitting at `value`
    /// expressed as `0.0..=1.0`.
    pub fn new(
        value: f32,
        warmest: u16,
        coolest: u16,
        skin: Skin,
        on_change: impl Fn(f32) -> Message + 'a,
    ) -> Self {
        Self {
            value: value.clamp(0.0, 1.0),
            warmest: f32::from(warmest),
            coolest: f32::from(coolest),
            skin,
            height: HEIGHT,
            on_change: Box::new(on_change),
        }
    }

    /// Overrides the control's height, so it can shrink on a tight window.
    pub fn height(mut self, height: f32) -> Self {
        self.height = height;
        self
    }

    fn shade(&self, fraction: f32) -> Color {
        tone::kelvin(self.warmest + (self.coolest - self.warmest) * fraction)
    }
}

impl<Message> Widget<Message, Theme, Renderer> for Warmth<'_, Message>
where
    Message: Clone,
{
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<Slide>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(Slide::new(self.value))
    }

    fn size(&self) -> Size<Length> {
        Size::new(Length::Fill, Length::Fixed(self.height))
    }

    fn layout(
        &mut self,
        _tree: &mut tree::Tree,
        _renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        layout::atomic(limits, Length::Fill, Length::Fixed(self.height))
    }

    fn update(
        &mut self,
        tree: &mut tree::Tree,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        _renderer: &Renderer,
        _clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        _viewport: &Rectangle,
    ) {
        let bounds = layout.bounds();
        let slide = tree.state.downcast_mut::<Slide>();
        let cap = KNOB_INSET + KNOB_WIDTH / 2.0;

        match track::track(slide, event, bounds, cursor, self.value, cap) {
            Gesture::Moved(fraction) => {
                shell.publish((self.on_change)(fraction));
                shell.capture_event();
                shell.request_redraw();
            }
            Gesture::Released => shell.request_redraw(),
            Gesture::Restless => shell.request_redraw(),
            Gesture::Idle => {}
        }
    }

    fn draw(
        &self,
        tree: &tree::Tree,
        renderer: &mut Renderer,
        _theme: &Theme,
        _style: &renderer::Style,
        layout: Layout<'_>,
        _cursor: mouse::Cursor,
        _viewport: &Rectangle,
    ) {
        let slide = tree.state.downcast_ref::<Slide>();
        let skin = self.skin;
        let bounds = layout.bounds();
        let radius = self.height / 2.0;
        let attention = slide.attention();
        let shown = slide.shown();

        let mut band = gradient::Linear::new(Radians(PI / 2.0));
        for step in 0..8 {
            let offset = step as f32 / 7.0;
            band = band.add_stop(offset, self.shade(offset));
        }

        paint::lift(
            renderer,
            bounds.shrink(8.0),
            radius,
            skin.shadow,
            skin.depth() * 0.3,
            18.0,
        );

        renderer.fill_quad(
            paint::plate(bounds, radius),
            Background::Gradient(iced::Gradient::Linear(band)),
        );

        // A hairline over the band keeps it from bleeding into the page,
        // whichever skin is on.
        renderer.fill_quad(
            paint::outlined(bounds, radius, tone::fade(skin.shadow, 0.12)),
            Background::Color(Color::TRANSPARENT),
        );

        let cap = KNOB_INSET + KNOB_WIDTH / 2.0;
        let center = bounds.x + cap + (bounds.width - cap * 2.0) * shown;
        let knob = Rectangle {
            x: center - KNOB_WIDTH / 2.0,
            y: bounds.y + KNOB_INSET,
            width: KNOB_WIDTH,
            height: bounds.height - KNOB_INSET * 2.0,
        };

        let chosen = self.shade(shown);

        renderer.fill_quad(
            renderer::Quad {
                bounds: knob.expand(2.0 * attention),
                border: Border {
                    radius: (KNOB_WIDTH / 2.0 + 2.0 * attention).into(),
                    width: 3.0,
                    color: Color::WHITE,
                },
                shadow: Shadow {
                    color: tone::fade(skin.shadow, 0.35),
                    offset: Vector::new(0.0, 3.0),
                    blur_radius: 12.0,
                },
                ..renderer::Quad::default()
            },
            Background::Color(chosen),
        );
    }
}

impl<'a, Message> From<Warmth<'a, Message>> for Element<'a, Message>
where
    Message: Clone + 'a,
{
    fn from(warmth: Warmth<'a, Message>) -> Self {
        Element::new(warmth)
    }
}
