//! A strip of full-width panels that slides sideways.
//!
//! The advanced controls of a light live on their own page each. Only one is
//! ever on screen, and moving between them is a real travel rather than a
//! swap, so it stays obvious that nothing was lost on the way.

use iced::advanced::layout::{self, Layout};
use iced::advanced::widget::{Operation, Widget, tree};
use iced::advanced::{
    Clipboard, Renderer as _, Shell, mouse, overlay, renderer,
};
use iced::animation::Animation;
use iced::time::Instant;
use iced::{
    Element, Event, Length, Point, Rectangle, Renderer, Size, Theme, Vector,
    window,
};

use crate::design::motion;

/// Space left between two panels, so the one arriving is clearly a second one.
const GUTTER: f32 = 48.0;

/// A horizontal strip of panels.
pub struct Pages<'a, Message> {
    panels: Vec<Element<'a, Message>>,
    active: usize,
    height: f32,
}

impl<'a, Message> Pages<'a, Message> {
    /// Builds the strip from its panels and the index currently shown.
    pub fn new(
        panels: Vec<Element<'a, Message>>,
        active: usize,
        height: f32,
    ) -> Self {
        Self { panels, active, height }
    }
}

struct State {
    now: Instant,
    offset: Animation<f32>,
}

impl<Message> Widget<Message, Theme, Renderer> for Pages<'_, Message> {
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<State>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(State {
            now: Instant::now(),
            offset: motion::page(self.active as f32),
        })
    }

    fn children(&self) -> Vec<tree::Tree> {
        self.panels.iter().map(tree::Tree::new).collect()
    }

    fn diff(&self, tree: &mut tree::Tree) {
        tree.diff_children(&self.panels);
    }

    fn size(&self) -> Size<Length> {
        Size::new(Length::Fill, Length::Fixed(self.height))
    }

    fn layout(
        &mut self,
        tree: &mut tree::Tree,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        let size = limits.resolve(
            Length::Fill,
            Length::Fixed(self.height),
            Size::new(0.0, self.height),
        );

        let offset = {
            let state = tree.state.downcast_ref::<State>();
            state.offset.interpolate_with(|value| value, state.now)
        };

        let stride = size.width + GUTTER;
        let panel = layout::Limits::new(Size::ZERO, size);

        let children = self
            .panels
            .iter_mut()
            .zip(tree.children.iter_mut())
            .enumerate()
            .map(|(index, (panel_element, panel_tree))| {
                let node = panel_element
                    .as_widget_mut()
                    .layout(panel_tree, renderer, &panel);

                node.move_to(Point::new((index as f32 - offset) * stride, 0.0))
            })
            .collect();

        layout::Node::with_children(size, children)
    }

    fn update(
        &mut self,
        tree: &mut tree::Tree,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        viewport: &Rectangle,
    ) {
        if let Event::Window(window::Event::RedrawRequested(now)) = event {
            let state = tree.state.downcast_mut::<State>();
            state.now = *now;
            state.offset.go_mut(self.active as f32, *now);

            if state.offset.is_animating(*now) {
                shell.request_redraw();
                shell.invalidate_layout();
            }
        }

        for ((panel, panel_tree), panel_layout) in self
            .panels
            .iter_mut()
            .zip(tree.children.iter_mut())
            .zip(layout.children())
        {
            panel.as_widget_mut().update(
                panel_tree,
                event,
                panel_layout,
                cursor,
                renderer,
                clipboard,
                shell,
                viewport,
            );
        }
    }

    fn operate(
        &mut self,
        tree: &mut tree::Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
        operation: &mut dyn Operation,
    ) {
        for ((panel, panel_tree), panel_layout) in self
            .panels
            .iter_mut()
            .zip(tree.children.iter_mut())
            .zip(layout.children())
        {
            panel.as_widget_mut().operate(
                panel_tree,
                panel_layout,
                renderer,
                operation,
            );
        }
    }

    fn mouse_interaction(
        &self,
        tree: &tree::Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &Renderer,
    ) -> mouse::Interaction {
        let bounds = layout.bounds();

        self.panels
            .iter()
            .zip(tree.children.iter())
            .zip(layout.children())
            .filter(|(_, panel_layout)| {
                panel_layout.bounds().intersects(&bounds)
            })
            .map(|((panel, panel_tree), panel_layout)| {
                panel.as_widget().mouse_interaction(
                    panel_tree,
                    panel_layout,
                    cursor,
                    viewport,
                    renderer,
                )
            })
            .max()
            .unwrap_or_default()
    }

    fn draw(
        &self,
        tree: &tree::Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        let bounds = layout.bounds();

        renderer.with_layer(bounds, |renderer| {
            for ((panel, panel_tree), panel_layout) in self
                .panels
                .iter()
                .zip(tree.children.iter())
                .zip(layout.children())
            {
                if !panel_layout.bounds().intersects(&bounds) {
                    continue;
                }

                panel.as_widget().draw(
                    panel_tree,
                    renderer,
                    theme,
                    style,
                    panel_layout,
                    cursor,
                    viewport,
                );
            }
        });
    }

    fn overlay<'a>(
        &'a mut self,
        tree: &'a mut tree::Tree,
        layout: Layout<'a>,
        renderer: &Renderer,
        viewport: &Rectangle,
        translation: Vector,
    ) -> Option<overlay::Element<'a, Message, Theme, Renderer>> {
        overlay::from_children(
            &mut self.panels,
            tree,
            layout,
            renderer,
            viewport,
            translation,
        )
    }
}

impl<'a, Message> From<Pages<'a, Message>> for Element<'a, Message>
where
    Message: 'a,
{
    fn from(pages: Pages<'a, Message>) -> Self {
        Element::new(pages)
    }
}
