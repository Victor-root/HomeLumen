//! The tile a light takes on the home screen.
//!
//! It answers three questions at a glance (which light, is it on, how bright)
//! and nothing else. A lit tile is warmer, deeper and casts its own colour onto
//! the page, so a room can be read without touching a single word.

use std::f32::consts::PI;

use iced::advanced::layout::{self, Layout};
use iced::advanced::widget::{Widget, tree};
use iced::advanced::{Clipboard, Renderer as _, Shell, mouse, renderer};
use iced::animation::Animation;
use iced::time::Instant;
use iced::touch;
use iced::{
    Background, Border, Color, Element, Event, Length, Point, Radians,
    Rectangle, Renderer, Shadow, Size, Theme, Vector, gradient, window,
};

use crate::design::{Lang, Skin, motion, round, tone, typo};
use crate::paint::{self, Anchor};
use crate::widget::bulb;

/// Height of a tile. Wide enough to breathe, short enough to fit three rows.
pub const HEIGHT: f32 = 152.0;

const PAD: f32 = 24.0;
const LAMP: f32 = 54.0;
const KNOB: f32 = 42.0;
const BAR: f32 = 26.0;

/// The line everything on a tile is arranged around, leaving the brightness bar
/// its own band along the bottom.
fn spine(card: Rectangle) -> f32 {
    card.y + (card.height - BAR) / 2.0
}

/// A light, as a tile.
pub struct Tile<'a, Message> {
    name: &'a str,
    lit: bool,
    level: f32,
    reading: String,
    glow: Color,
    online: bool,
    skin: Skin,
    lang: Lang,
    on_open: Message,
    on_toggle: Option<Message>,
}

impl<'a, Message> Tile<'a, Message> {
    /// Builds a tile for a light.
    pub fn new(
        name: &'a str,
        reading: String,
        skin: Skin,
        lang: Lang,
        on_open: Message,
    ) -> Self {
        Self {
            name,
            lit: false,
            level: 1.0,
            reading,
            glow: skin.accent,
            online: true,
            skin,
            lang,
            on_open,
            on_toggle: None,
        }
    }

    /// Whether the light is on, how bright, and in which colour.
    pub fn light(mut self, lit: bool, level: f32, glow: Color) -> Self {
        self.lit = lit;
        self.level = level.clamp(0.0, 1.0);
        self.glow = glow;
        self
    }

    /// Whether any route still reaches the light.
    pub fn online(mut self, online: bool) -> Self {
        self.online = online;
        self
    }

    /// Makes the corner control switch the light.
    pub fn on_toggle(mut self, message: Message) -> Self {
        self.on_toggle = Some(message);
        self
    }

    fn knob_bounds(&self, bounds: Rectangle) -> Rectangle {
        Rectangle {
            x: bounds.x + bounds.width - PAD - KNOB,
            y: spine(bounds) - KNOB / 2.0,
            width: KNOB,
            height: KNOB,
        }
    }
}

struct State {
    now: Instant,
    hover: Animation<bool>,
    over_knob: Animation<bool>,
    press: Animation<bool>,
    lit: Animation<bool>,
    level: Animation<f32>,
    armed: bool,
}

impl State {
    fn new(lit: bool, level: f32) -> Self {
        Self {
            now: Instant::now(),
            hover: motion::hover(),
            over_knob: motion::hover(),
            press: motion::press(),
            lit: motion::switch(lit),
            level: motion::settle(level),
            armed: false,
        }
    }

    fn busy(&self) -> bool {
        self.hover.is_animating(self.now)
            || self.over_knob.is_animating(self.now)
            || self.press.is_animating(self.now)
            || self.lit.is_animating(self.now)
            || self.level.is_animating(self.now)
    }
}

impl<Message> Widget<Message, Theme, Renderer> for Tile<'_, Message>
where
    Message: Clone,
{
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<State>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(State::new(self.lit, self.level))
    }

    fn size(&self) -> Size<Length> {
        Size::new(Length::Fill, Length::Fixed(HEIGHT))
    }

    fn layout(
        &mut self,
        _tree: &mut tree::Tree,
        _renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        layout::atomic(limits, Length::Fill, Length::Fixed(HEIGHT))
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
        let knob = self.knob_bounds(bounds).expand(4.0);
        let state = tree.state.downcast_mut::<State>();

        match event {
            Event::Window(window::Event::RedrawRequested(now)) => {
                state.now = *now;
                state.lit.go_mut(self.lit, *now);
                state.level.go_mut(self.level, *now);
                state.hover.go_mut(cursor.is_over(bounds), *now);
                state.over_knob.go_mut(cursor.is_over(knob), *now);

                if state.busy() {
                    shell.request_redraw();
                }
            }
            Event::Mouse(mouse::Event::CursorMoved { .. })
            | Event::Touch(touch::Event::FingerMoved { .. }) => {
                let hovering = cursor.is_over(bounds);
                if hovering != state.hover.value()
                    || cursor.is_over(knob) != state.over_knob.value()
                {
                    shell.request_redraw();
                }
            }
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left))
            | Event::Touch(touch::Event::FingerPressed { .. }) => {
                if cursor.is_over(knob) {
                    if let Some(toggle) = &self.on_toggle {
                        shell.publish(toggle.clone());
                        shell.capture_event();
                    }
                } else if cursor.is_over(bounds) {
                    state.armed = true;
                    state.press.go_mut(true, state.now);
                    shell.capture_event();
                    shell.request_redraw();
                }
            }
            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left))
            | Event::Touch(touch::Event::FingerLifted { .. })
            | Event::Touch(touch::Event::FingerLost { .. }) => {
                let armed = std::mem::take(&mut state.armed);
                state.press.go_mut(false, state.now);

                if armed && cursor.is_over(bounds) && !cursor.is_over(knob) {
                    shell.publish(self.on_open.clone());
                }
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
        let level = state.level.interpolate_with(|value| value, now);
        let knob_hover = state.over_knob.interpolate(0.0, 1.0, now);

        let skin = self.skin;
        let emission = self.glow;
        let presence = lit * (0.45 + 0.55 * level);

        let bounds = layout.bounds();
        let card = Rectangle { y: bounds.y - 3.0 * hover, ..bounds }
            .shrink(press * 2.5);

        // The light the tile puts into the page.
        paint::glow(
            renderer,
            card.shrink(10.0),
            round::TILE,
            emission,
            0.30 * presence * (1.0 + 0.35 * hover),
            46.0 + 14.0 * hover,
        );

        // Depth, so a card sits on the page rather than being printed on it.
        paint::lift(
            renderer,
            card,
            round::TILE,
            skin.shadow,
            skin.depth() * (0.30 + 0.24 * hover),
            16.0 + 14.0 * hover,
        );

        // The card itself: a vertical wash, warmed by the light it carries.
        let tint = 0.05 * presence + 0.02 * hover;
        let top = tone::mix(skin.surface_lift, emission, tint);
        let bottom = tone::mix(skin.surface, emission, tint * 0.45);

        renderer.fill_quad(
            renderer::Quad {
                bounds: card,
                border: Border {
                    radius: round::TILE.into(),
                    width: 1.0,
                    color: tone::mix(
                        skin.edge,
                        emission,
                        0.22 * presence + 0.10 * hover,
                    ),
                },
                ..renderer::Quad::default()
            },
            Background::Gradient(iced::Gradient::Linear(
                gradient::Linear::new(Radians(PI))
                    .add_stop(0.0, top)
                    .add_stop(1.0, bottom),
            )),
        );

        self.draw_lamp(renderer, card, emission, presence, level);
        self.draw_knob(renderer, card, emission, lit, knob_hover);
        self.draw_copy(renderer, card, presence);
        self.draw_level(renderer, card, emission, presence, level);
    }
}

impl<Message> Tile<'_, Message> {
    /// Drawn from the same colour maths as the bulb on the light's own
    /// screen (see [`bulb::tones`]), so a room reads as the same lights
    /// whichever screen is showing them. `radius` is picked so the dome's
    /// diameter fills the same square `LAMP` always reserved for it, the
    /// footprint every other measurement on the tile already assumes.
    fn draw_lamp(
        &self,
        renderer: &mut Renderer,
        card: Rectangle,
        emission: Color,
        presence: f32,
        level: f32,
    ) {
        let radius = LAMP / 2.0;
        let side = radius / 0.30;
        let center = Point::new(card.x + PAD + radius, spine(card));
        let glass = Point::new(center.x, center.y - side * 0.07);

        let disc = Rectangle {
            x: glass.x - radius,
            y: glass.y - radius,
            width: radius * 2.0,
            height: radius * 2.0,
        };

        paint::glow(
            renderer,
            disc,
            radius,
            emission,
            0.55 * presence,
            18.0 + 22.0 * level * presence,
        );

        let tones = bulb::tones(self.skin, emission, presence);

        // The socket, drawn first and mostly tucked behind the glass, so
        // only the sliver below it reads as a base to screw the bulb into.
        let width = radius * 1.15;
        let height = radius * 0.85;
        let top = glass.y + radius * 0.62;

        renderer.fill_quad(
            renderer::Quad {
                bounds: Rectangle {
                    x: glass.x - width / 2.0,
                    y: top,
                    width,
                    height,
                },
                border: Border {
                    radius: (height * 0.32).into(),
                    ..Border::default()
                },
                ..renderer::Quad::default()
            },
            Background::Color(tones.metal),
        );

        renderer.fill_quad(
            renderer::Quad {
                bounds: disc,
                border: Border {
                    radius: radius.into(),
                    width: 1.0,
                    color: tones.stroke,
                },
                ..renderer::Quad::default()
            },
            Background::Gradient(iced::Gradient::Linear(
                gradient::Linear::new(Radians(PI))
                    .add_stop(0.0, tones.top)
                    .add_stop(1.0, tones.bottom),
            )),
        );

        // A single specular highlight is what turns a disc into a sphere.
        let dot = radius * 0.26;
        renderer.fill_quad(
            renderer::Quad {
                bounds: Rectangle {
                    x: glass.x - radius * 0.33 - dot,
                    y: glass.y - radius * 0.38 - dot,
                    width: dot * 2.0,
                    height: dot * 2.0,
                },
                border: Border { radius: dot.into(), ..Border::default() },
                ..renderer::Quad::default()
            },
            Background::Color(tones.highlight),
        );
    }

    fn draw_knob(
        &self,
        renderer: &mut Renderer,
        card: Rectangle,
        emission: Color,
        lit: f32,
        hover: f32,
    ) {
        if self.on_toggle.is_none() {
            return;
        }

        let skin = self.skin;
        let disc = self.knob_bounds(card);
        let radius = KNOB / 2.0;

        let resting =
            tone::mix(skin.surface_lift, skin.edge, 0.35 + 0.25 * hover);
        let face = tone::mix(resting, emission, lit);

        paint::glow(renderer, disc, radius, emission, 0.35 * lit, 20.0);

        renderer.fill_quad(
            renderer::Quad {
                bounds: disc,
                border: Border {
                    radius: radius.into(),
                    width: 1.0,
                    color: tone::mix(skin.edge, emission, lit * 0.7),
                },
                ..renderer::Quad::default()
            },
            Background::Color(face),
        );

        let ink = if lit > 0.5 {
            tone::mix(skin.ink_over_light, skin.ink, 1.0 - lit)
        } else {
            tone::mix(skin.ink_soft, skin.ink, hover)
        };

        power_glyph(renderer, disc.center(), 15.0, ink, face);
    }

    fn draw_copy(
        &self,
        renderer: &mut Renderer,
        card: Rectangle,
        presence: f32,
    ) {
        let skin = self.skin;
        let left = card.x + PAD + LAMP + 20.0;
        let width = (card.x + card.width - PAD - KNOB - 16.0 - left).max(40.0);
        let mid = spine(card);

        paint::write(
            renderer,
            self.name,
            typo::SEMIBOLD,
            typo::LEAD,
            tone::mix(skin.ink_soft, skin.ink, 0.35 + 0.65 * presence),
            Point::new(left, mid - 12.0),
            Anchor::Left,
            width,
            card,
        );

        let (reading, colour) = if self.online {
            (
                self.reading.as_str(),
                tone::mix(skin.ink_faint, skin.ink_soft, presence),
            )
        } else {
            (self.lang.offline(), skin.alarm)
        };

        paint::write(
            renderer,
            reading,
            typo::MEDIUM,
            typo::LABEL,
            colour,
            Point::new(left, mid + 13.0),
            Anchor::Left,
            width,
            card,
        );
    }

    fn draw_level(
        &self,
        renderer: &mut Renderer,
        card: Rectangle,
        emission: Color,
        presence: f32,
        level: f32,
    ) {
        let skin = self.skin;
        let track = Rectangle {
            x: card.x + PAD,
            y: card.y + card.height - BAR + 2.0,
            width: card.width - PAD * 2.0,
            height: 4.0,
        };

        renderer.fill_quad(
            paint::plate(track, 2.0),
            Background::Color(tone::mix(skin.edge_soft, skin.edge, 0.6)),
        );

        if presence <= 0.01 {
            return;
        }

        let filled =
            Rectangle { width: (track.width * level).max(4.0), ..track };

        renderer.fill_quad(
            renderer::Quad {
                bounds: filled,
                border: Border { radius: 2.0.into(), ..Border::default() },
                shadow: Shadow {
                    color: tone::fade(emission, 0.45 * presence),
                    offset: Vector::ZERO,
                    blur_radius: 12.0,
                },
                ..renderer::Quad::default()
            },
            Background::Color(tone::fade(emission, 0.35 + 0.65 * presence)),
        );
    }
}

/// The IEC power mark, built from a ring, a mask and a stem.
pub fn power_glyph(
    renderer: &mut Renderer,
    center: Point,
    size: f32,
    color: Color,
    behind: Color,
) {
    let stroke = (size * 0.13).max(1.6);
    let radius = size / 2.0;

    renderer.fill_quad(
        renderer::Quad {
            bounds: Rectangle {
                x: center.x - radius,
                y: center.y - radius,
                width: size,
                height: size,
            },
            border: Border { radius: radius.into(), width: stroke, color },
            ..renderer::Quad::default()
        },
        Background::Color(Color::TRANSPARENT),
    );

    renderer.fill_quad(
        paint::plate(
            Rectangle {
                x: center.x - stroke * 1.7,
                y: center.y - radius - 1.0,
                width: stroke * 3.4,
                height: size * 0.42,
            },
            0.0,
        ),
        Background::Color(behind),
    );

    renderer.fill_quad(
        paint::plate(
            Rectangle {
                x: center.x - stroke / 2.0,
                y: center.y - size * 0.58,
                width: stroke,
                height: size * 0.52,
            },
            stroke / 2.0,
        ),
        Background::Color(color),
    );
}

impl<'a, Message> From<Tile<'a, Message>> for Element<'a, Message>
where
    Message: Clone + 'a,
{
    fn from(tile: Tile<'a, Message>) -> Self {
        Element::new(tile)
    }
}
