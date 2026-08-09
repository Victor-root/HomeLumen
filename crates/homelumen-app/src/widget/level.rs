//! The brightness capsule.
//!
//! A single wide surface that fills with the light's own colour. The reading
//! sits inside it and flips ink as the fill passes underneath, which is the one
//! flourish this control allows itself.

use std::f32::consts::PI;

use iced::advanced::layout::{self, Layout};
use iced::advanced::widget::{Widget, tree};
use iced::advanced::{Clipboard, Renderer as _, Shell, mouse, renderer};
use iced::{
    Background, Border, Color, Element, Event, Length, Point, Radians,
    Rectangle, Renderer, Shadow, Size, Theme, Vector, gradient,
};

use crate::design::{Skin, tone, typo};
use crate::paint::{self, Anchor};
use crate::widget::track::{self, Gesture, Slide};

/// Height of the capsule at its most spacious. Deliberately generous: this
/// is the control the hand reaches for first.
pub const HEIGHT: f32 = 116.0;

const PAD: f32 = 30.0;

const KNOB_WIDTH: f32 = 14.0;
const KNOB_INSET: f32 = 9.0;

/// A brightness control.
pub struct Level<'a, Message> {
    value: f32,
    reading: u8,
    glow: Color,
    skin: Skin,
    enabled: bool,
    height: f32,
    on_change: Box<dyn Fn(f32) -> Message + 'a>,
}

impl<'a, Message> Level<'a, Message> {
    /// Builds a brightness control filled to `value` (`0.0..=1.0`, the
    /// fraction of the device's own range) and printing `reading` as the
    /// percentage.
    ///
    /// The two are not the same number to compute from one another: the
    /// device's usable range can start above zero, so a fraction of it and
    /// the plain reading a person would recognise (the same one the home
    /// screen already shows) drift apart near the low end. Taking `reading`
    /// as given, rather than deriving it from `value`, is what keeps this
    /// control and the tile agreeing on what the light is actually at.
    pub fn new(
        value: f32,
        reading: u8,
        glow: Color,
        skin: Skin,
        on_change: impl Fn(f32) -> Message + 'a,
    ) -> Self {
        Self {
            value: value.clamp(0.0, 1.0),
            reading,
            glow,
            skin,
            enabled: true,
            height: HEIGHT,
            on_change: Box::new(on_change),
        }
    }

    /// Dims the whole control when the light is off.
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    /// Overrides the control's height, so it can shrink on a tight window.
    pub fn height(mut self, height: f32) -> Self {
        self.height = height;
        self
    }
}

impl<Message> Widget<Message, Theme, Renderer> for Level<'_, Message>
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
        let cap = self.height / 2.0;

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

        let dimmed = if self.enabled { 1.0 } else { 0.30 };
        let glow = self.glow;

        renderer.fill_quad(
            paint::outlined(
                bounds,
                radius,
                tone::mix(skin.edge, glow, 0.12 * attention),
            ),
            Background::Color(skin.surface),
        );

        let head = radius + (bounds.width - radius * 2.0) * shown;
        let fill = Rectangle { width: head + radius, ..bounds };

        paint::glow(
            renderer,
            fill.shrink(6.0),
            radius,
            glow,
            0.22 * dimmed * (0.7 + 0.3 * attention),
            34.0,
        );

        renderer.fill_quad(
            paint::plate(fill, radius),
            Background::Gradient(iced::Gradient::Linear(
                gradient::Linear::new(Radians(PI / 2.0))
                    .add_stop(
                        0.0,
                        tone::fade(tone::mix(glow, skin.canvas, 0.22), dimmed),
                    )
                    .add_stop(0.6, tone::fade(glow, dimmed))
                    .add_stop(
                        1.0,
                        tone::fade(tone::mix(glow, Color::WHITE, 0.18), dimmed),
                    ),
            )),
        );

        // The reading, drawn twice: once in the ink that reads over the fill,
        // once in the ink that reads over the track, each clipped to its side.
        let reading = format!("{} %", self.reading);
        let anchor = Point::new(bounds.x + PAD, bounds.center_y());
        let width = bounds.width - PAD * 2.0;

        let over_fill = if !self.enabled {
            skin.ink_faint
        } else if tone::luminance(glow) > 0.55 {
            skin.ink_over_light
        } else {
            Color::WHITE
        };

        let split = bounds.x + fill.width;

        paint::write(
            renderer,
            &reading,
            typo::SEMIBOLD,
            30.0,
            over_fill,
            anchor,
            Anchor::Left,
            width,
            Rectangle { width: fill.width, ..bounds },
        );

        paint::write(
            renderer,
            &reading,
            typo::SEMIBOLD,
            30.0,
            if self.enabled { skin.ink_soft } else { skin.ink_faint },
            anchor,
            Anchor::Left,
            width,
            Rectangle {
                x: split,
                width: (bounds.x + bounds.width - split).max(0.0),
                ..bounds
            },
        );

        // A handle at the fill's edge, so the capsule reads as something to
        // grab and drag rather than just a reading that happens to change.
        // A slim bar rather than a disc: a disc this tall would sit right on
        // top of the reading at low values instead of beside it.
        let width = KNOB_WIDTH + 2.0 * attention;
        let height = bounds.height - KNOB_INSET * 2.0;
        let center = Point::new(bounds.x + head, bounds.center_y());

        renderer.fill_quad(
            renderer::Quad {
                bounds: Rectangle {
                    x: center.x - width / 2.0,
                    y: center.y - height / 2.0,
                    width,
                    height,
                },
                border: Border {
                    radius: (width / 2.0).into(),
                    width: 3.0,
                    color: Color::WHITE,
                },
                shadow: Shadow {
                    color: tone::fade(skin.shadow, 0.35),
                    offset: Vector::new(0.0, 2.0),
                    blur_radius: 10.0,
                },
                ..renderer::Quad::default()
            },
            Background::Color(tone::mix(glow, Color::WHITE, 0.25)),
        );
    }
}

impl<'a, Message> From<Level<'a, Message>> for Element<'a, Message>
where
    Message: Clone + 'a,
{
    fn from(level: Level<'a, Message>) -> Self {
        Element::new(level)
    }
}
