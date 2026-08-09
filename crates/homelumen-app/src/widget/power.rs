//! The main on/off control of a light.
//!
//! Full width, unmissable, and it changes colour rather than changing a label:
//! the state of the light is the state of the control.

use std::f32::consts::PI;

use iced::advanced::layout::{self, Layout};
use iced::advanced::widget::{Widget, tree};
use iced::advanced::{Clipboard, Renderer as _, Shell, mouse, renderer};
use iced::animation::Animation;
use iced::time::Instant;
use iced::touch;
use iced::{
    Background, Border, Color, Element, Event, Length, Point, Radians,
    Rectangle, Renderer, Size, Theme, gradient, window,
};

use crate::design::{Skin, motion, round, tone, typo};
use crate::paint::{self, Anchor};
use crate::widget::tile::power_glyph;

/// Height of the control at its most spacious.
pub const HEIGHT: f32 = 86.0;

const PAD: f32 = 26.0;
const SWITCH_WIDTH: f32 = 58.0;
const SWITCH_HEIGHT: f32 = 32.0;

/// The on/off control.
pub struct Power<Message> {
    lit: bool,
    glow: Color,
    skin: Skin,
    on_toggle: Message,
    height: f32,
}

impl<Message> Power<Message> {
    /// Builds the control for a light that is currently on or off.
    pub fn new(lit: bool, glow: Color, skin: Skin, on_toggle: Message) -> Self {
        Self { lit, glow, skin, on_toggle, height: HEIGHT }
    }

    /// Overrides the control's height, so it can shrink on a tight window.
    pub fn height(mut self, height: f32) -> Self {
        self.height = height;
        self
    }
}

struct State {
    now: Instant,
    hover: Animation<bool>,
    press: Animation<bool>,
    lit: Animation<bool>,
}

impl State {
    fn busy(&self) -> bool {
        self.hover.is_animating(self.now)
            || self.press.is_animating(self.now)
            || self.lit.is_animating(self.now)
    }
}

impl<Message> Widget<Message, Theme, Renderer> for Power<Message>
where
    Message: Clone,
{
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<State>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(State {
            now: Instant::now(),
            hover: motion::hover(),
            press: motion::press(),
            lit: motion::switch(self.lit),
        })
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
        let state = tree.state.downcast_mut::<State>();

        match event {
            Event::Window(window::Event::RedrawRequested(now)) => {
                state.now = *now;
                state.lit.go_mut(self.lit, *now);
                state.hover.go_mut(cursor.is_over(bounds), *now);

                if state.busy() {
                    shell.request_redraw();
                }
            }
            Event::Mouse(mouse::Event::CursorMoved { .. })
            | Event::Touch(touch::Event::FingerMoved { .. }) => {
                if cursor.is_over(bounds) != state.hover.value() {
                    shell.request_redraw();
                }
            }
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left))
            | Event::Touch(touch::Event::FingerPressed { .. }) => {
                if cursor.is_over(bounds) {
                    state.press.go_mut(true, state.now);
                    shell.publish(self.on_toggle.clone());
                    shell.capture_event();
                    shell.request_redraw();
                }
            }
            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left))
            | Event::Touch(touch::Event::FingerLifted { .. })
            | Event::Touch(touch::Event::FingerLost { .. }) => {
                state.press.go_mut(false, state.now);
                shell.request_redraw();
            }
            _ => {}
        }
    }

    fn mouse_interaction(
        &self,
        _tree: &tree::Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        _viewport: &Rectangle,
        _renderer: &Renderer,
    ) -> mouse::Interaction {
        if cursor.is_over(layout.bounds()) {
            mouse::Interaction::Pointer
        } else {
            mouse::Interaction::None
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
        let state = tree.state.downcast_ref::<State>();
        let now = state.now;
        let hover = state.hover.interpolate(0.0, 1.0, now);
        let press = state.press.interpolate(0.0, 1.0, now);
        let lit = state.lit.interpolate(0.0, 1.0, now);

        let skin = self.skin;
        let bounds = layout.bounds().shrink(press * 2.0);
        let radius = round::TILE;

        paint::glow(
            renderer,
            bounds.shrink(8.0),
            radius,
            self.glow,
            0.30 * lit * (1.0 + 0.3 * hover),
            38.0,
        );

        let resting =
            tone::mix(skin.surface, skin.surface_lift, 0.5 + 0.5 * hover);
        let face = tone::mix(resting, self.glow, lit);

        renderer.fill_quad(
            renderer::Quad {
                bounds,
                border: Border {
                    radius: radius.into(),
                    width: 1.0,
                    color: tone::mix(
                        skin.edge,
                        self.glow,
                        lit * 0.6 + hover * 0.1,
                    ),
                },
                ..renderer::Quad::default()
            },
            Background::Gradient(iced::Gradient::Linear(
                gradient::Linear::new(Radians(PI))
                    .add_stop(0.0, tone::mix(face, Color::WHITE, 0.07 * lit))
                    .add_stop(1.0, tone::mix(face, skin.canvas, 0.10)),
            )),
        );

        let ink = tone::mix(
            tone::mix(skin.ink_soft, skin.ink, hover),
            skin.ink_over_light,
            lit,
        );

        power_glyph(
            renderer,
            Point::new(bounds.x + PAD + 11.0, bounds.center_y()),
            22.0,
            ink,
            face,
        );

        paint::write(
            renderer,
            if self.lit { "Allumée" } else { "Éteinte" },
            typo::SEMIBOLD,
            typo::LEAD,
            ink,
            Point::new(bounds.x + PAD + 44.0, bounds.center_y()),
            Anchor::Left,
            bounds.width,
            bounds,
        );

        let switch = Rectangle {
            x: bounds.x + bounds.width - PAD - SWITCH_WIDTH,
            y: bounds.center_y() - SWITCH_HEIGHT / 2.0,
            width: SWITCH_WIDTH,
            height: SWITCH_HEIGHT,
        };

        renderer.fill_quad(
            paint::plate(switch, SWITCH_HEIGHT / 2.0),
            Background::Color(tone::mix(
                tone::mix(skin.canvas, skin.edge, 0.5),
                tone::fade(skin.ink_over_light, 0.22),
                lit,
            )),
        );

        let travel = SWITCH_WIDTH - SWITCH_HEIGHT + 6.0;
        let dot = SWITCH_HEIGHT - 6.0;

        renderer.fill_quad(
            paint::plate(
                Rectangle {
                    x: switch.x + 3.0 + travel * lit - 3.0 * lit,
                    y: switch.y + 3.0,
                    width: dot,
                    height: dot,
                },
                dot / 2.0,
            ),
            Background::Color(tone::mix(
                skin.ink_soft,
                Color::WHITE,
                0.4 + 0.6 * lit,
            )),
        );
    }
}

impl<'a, Message> From<Power<Message>> for Element<'a, Message>
where
    Message: Clone + 'a,
{
    fn from(power: Power<Message>) -> Self {
        Element::new(power)
    }
}
