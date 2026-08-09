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

// The switch, to the Material Design 3 metrics exactly: a 52x32 track that
// is outlined when off and filled when on, and a handle that is 16 across
// when off and 24 when on, so it grows as it crosses rather than being a
// fixed dot that only slides.
const SWITCH_WIDTH: f32 = 52.0;
const SWITCH_HEIGHT: f32 = 32.0;
const TRACK_OUTLINE: f32 = 2.0;
const THUMB_OFF: f32 = 16.0;
const THUMB_ON: f32 = 24.0;

/// Room left around the selected handle. The unselected one keeps the very
/// same centre and simply shrinks inside it, which is what puts the small
/// handle a wider margin from the track's edge than the large one.
const THUMB_INSET: f32 = (SWITCH_HEIGHT - THUMB_ON) / 2.0;

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

        // The row behind the switch is already tinted with the light's own
        // colour, so filling the track with that same colour would make it
        // vanish into its own background on anything but a weak glow. A
        // translucent scrim reads as a distinct shape against any colour
        // instead: dark on a light glow, light on a dark one. `self.glow`
        // rather than `face` picks the side once, so the choice does not
        // flip mid-animation as `lit` moves `face` across the threshold.
        let scrim = if tone::luminance(self.glow) > 0.5 {
            Color::BLACK
        } else {
            Color::WHITE
        };

        // `skin.edge` alone, not a blend toward `skin.canvas`: in the day
        // skin the row's own resting background is already close to white,
        // and a blend that also leans toward canvas nearly matches it,
        // which is the same washed-out-track problem as the coloured one
        // above, just for the off state. The outline leans further still,
        // toward `ink_faint`, so the track reads as outlined and not just
        // filled a shade off from its surroundings.
        let track_off = skin.edge;
        let outline_off = tone::mix(skin.edge, skin.ink_faint, 0.6);

        // Material Design 3 outlines the track only while the switch is off:
        // once it fills, the fill is the whole shape and a ring around it
        // would read as a second, competing edge. So the outline is not
        // recoloured on the way across, it is retired. Its width has to go
        // with its colour, not just the colour: a border keeps its share of
        // the quad whether or not anything is painted in it, so leaving the
        // width behind would hollow the lit pill out by two pixels a side.
        let track_fill = tone::mix(track_off, tone::fade(scrim, 0.32), lit);
        let retreat = 1.0 - lit;

        renderer.fill_quad(
            renderer::Quad {
                bounds: switch,
                border: Border {
                    radius: (SWITCH_HEIGHT / 2.0).into(),
                    width: TRACK_OUTLINE * retreat,
                    color: tone::fade(outline_off, retreat),
                },
                ..renderer::Quad::default()
            },
            Background::Color(track_fill),
        );

        // Off sits small and to the left; on grows and moves to the right,
        // rather than a fixed dot that only slides. Both ends are the same
        // distance from their own edge of the track, so the travel is what
        // Material Design 3 calls it: the width less a handle's worth of
        // inset at each end.
        let thumb = THUMB_OFF + (THUMB_ON - THUMB_OFF) * lit;
        let rest = THUMB_INSET + THUMB_ON / 2.0;
        let travel = SWITCH_WIDTH - 2.0 * rest;
        let center =
            Point::new(switch.x + rest + travel * lit, switch.center_y());

        renderer.fill_quad(
            renderer::Quad {
                bounds: Rectangle {
                    x: center.x - thumb / 2.0,
                    y: center.y - thumb / 2.0,
                    width: thumb,
                    height: thumb,
                },
                border: Border {
                    radius: (thumb / 2.0).into(),
                    ..Border::default()
                },
                ..renderer::Quad::default()
            },
            Background::Color(tone::mix(skin.ink_faint, Color::WHITE, lit)),
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
