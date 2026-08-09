//! The control that chooses which panel is on screen.
//!
//! A capsule holding a highlight that slides between segments, so switching
//! panels reads as movement rather than as a redraw.

use iced::advanced::layout::{self, Layout};
use iced::advanced::widget::{Widget, tree};
use iced::advanced::{Clipboard, Renderer as _, Shell, mouse, renderer};
use iced::animation::Animation;
use iced::time::Instant;
use iced::touch;
use iced::{
    Background, Element, Event, Length, Point, Rectangle, Renderer, Size,
    Theme, window,
};

use crate::design::{Skin, motion, round, tone, typo};
use crate::paint::{self, Anchor};

/// Height of the control at its most spacious.
pub const HEIGHT: f32 = 46.0;

const SEGMENT: f32 = 132.0;
const INSET: f32 = 4.0;

/// A row of exclusive choices.
pub struct Segmented<'a, Message> {
    labels: Vec<&'a str>,
    active: usize,
    skin: Skin,
    height: f32,
    on_select: Box<dyn Fn(usize) -> Message + 'a>,
}

impl<'a, Message> Segmented<'a, Message> {
    /// Builds the control from its labels and the index currently chosen.
    pub fn new(
        labels: Vec<&'a str>,
        active: usize,
        skin: Skin,
        on_select: impl Fn(usize) -> Message + 'a,
    ) -> Self {
        Self {
            labels,
            active,
            skin,
            height: HEIGHT,
            on_select: Box::new(on_select),
        }
    }

    /// Overrides the control's height, so it can shrink on a tight window.
    pub fn height(mut self, height: f32) -> Self {
        self.height = height;
        self
    }

    fn width(&self) -> f32 {
        self.labels.len() as f32 * SEGMENT + INSET * 2.0
    }

    fn segment_at(&self, bounds: Rectangle, x: f32) -> Option<usize> {
        let local = x - bounds.x - INSET;
        if local < 0.0 {
            return None;
        }

        let index = (local / SEGMENT) as usize;
        (index < self.labels.len()).then_some(index)
    }
}

struct State {
    now: Instant,
    highlight: Animation<f32>,
    hovered: Option<usize>,
}

impl<Message> Widget<Message, Theme, Renderer> for Segmented<'_, Message>
where
    Message: Clone,
{
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<State>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(State {
            now: Instant::now(),
            highlight: motion::page(self.active as f32),
            hovered: None,
        })
    }

    fn size(&self) -> Size<Length> {
        Size::new(Length::Fixed(self.width()), Length::Fixed(self.height))
    }

    fn layout(
        &mut self,
        _tree: &mut tree::Tree,
        _renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        layout::atomic(
            limits,
            Length::Fixed(self.width()),
            Length::Fixed(self.height),
        )
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
        let pointed = cursor
            .position_over(bounds)
            .and_then(|at| self.segment_at(bounds, at.x));
        let state = tree.state.downcast_mut::<State>();

        match event {
            Event::Window(window::Event::RedrawRequested(now)) => {
                state.now = *now;
                state.highlight.go_mut(self.active as f32, *now);
                state.hovered = pointed;

                if state.highlight.is_animating(*now) {
                    shell.request_redraw();
                }
            }
            Event::Mouse(mouse::Event::CursorMoved { .. })
            | Event::Touch(touch::Event::FingerMoved { .. }) => {
                if state.hovered != pointed {
                    shell.request_redraw();
                }
            }
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left))
            | Event::Touch(touch::Event::FingerPressed { .. }) => {
                if let Some(index) = pointed
                    && index != self.active
                {
                    shell.publish((self.on_select)(index));
                    shell.capture_event();
                    shell.request_redraw();
                }
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
        let skin = self.skin;
        let bounds = layout.bounds();
        let position =
            state.highlight.interpolate_with(|value| value, state.now);

        renderer.fill_quad(
            paint::outlined(bounds, round::FULL, skin.edge_soft),
            Background::Color(tone::mix(skin.canvas, skin.surface, 0.55)),
        );

        let highlight = Rectangle {
            x: bounds.x + INSET + position * SEGMENT,
            y: bounds.y + INSET,
            width: SEGMENT,
            height: self.height - INSET * 2.0,
        };

        paint::lift(
            renderer,
            highlight,
            round::FULL,
            skin.shadow,
            skin.depth() * 0.28,
            10.0,
        );

        renderer.fill_quad(
            paint::outlined(highlight, round::FULL, skin.edge),
            Background::Color(skin.surface_lift),
        );

        for (index, label) in self.labels.iter().enumerate() {
            let distance = (index as f32 - position).abs().clamp(0.0, 1.0);
            let hovered = state.hovered == Some(index);

            let ink = tone::mix(
                skin.ink,
                if hovered { skin.ink_soft } else { skin.ink_faint },
                distance,
            );

            paint::write(
                renderer,
                label,
                typo::MEDIUM,
                typo::BODY,
                ink,
                Point::new(
                    bounds.x + INSET + SEGMENT * (index as f32 + 0.5),
                    bounds.center_y(),
                ),
                Anchor::Center,
                SEGMENT,
                bounds,
            );
        }
    }
}

impl<'a, Message> From<Segmented<'a, Message>> for Element<'a, Message>
where
    Message: Clone + 'a,
{
    fn from(segmented: Segmented<'a, Message>) -> Self {
        Element::new(segmented)
    }
}
