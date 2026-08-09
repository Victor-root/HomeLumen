//! Small drawing helpers shared by the bespoke controls.

use iced::advanced::Renderer as _;
use iced::advanced::renderer::Quad;
use iced::advanced::text::Renderer as TextRenderer;
use iced::advanced::text::{
    Alignment as TextAlignment, Shaping, Text, Wrapping,
};
use iced::alignment::Vertical;
use iced::widget::text::LineHeight;
use iced::{
    Background, Border, Color, Font, Pixels, Point, Rectangle, Renderer,
    Shadow, Size, Vector,
};

use crate::design::tone;

/// A rounded rectangle, the shape almost everything in HomeLumen is made of.
pub fn plate(bounds: Rectangle, radius: f32) -> Quad {
    Quad {
        bounds,
        border: Border { radius: radius.into(), ..Border::default() },
        ..Quad::default()
    }
}

/// The same, with a hairline around it.
pub fn outlined(bounds: Rectangle, radius: f32, color: Color) -> Quad {
    Quad {
        bounds,
        border: Border { radius: radius.into(), width: 1.0, color },
        ..Quad::default()
    }
}

/// Casts light around a shape without drawing the shape itself.
///
/// A lit bulb has to feel like it is putting something into the room; this is
/// how a tile earns a presence an outline never could.
pub fn glow(
    renderer: &mut Renderer,
    bounds: Rectangle,
    radius: f32,
    color: Color,
    strength: f32,
    blur: f32,
) {
    if strength <= 0.001 {
        return;
    }

    renderer.fill_quad(
        Quad {
            bounds,
            border: Border { radius: radius.into(), ..Border::default() },
            shadow: Shadow {
                color: tone::fade(color, strength),
                offset: Vector::new(0.0, blur * 0.18),
                blur_radius: blur,
            },
            ..Quad::default()
        },
        Background::Color(Color::TRANSPARENT),
    );
}

/// Casts depth under a surface.
pub fn lift(
    renderer: &mut Renderer,
    bounds: Rectangle,
    radius: f32,
    color: Color,
    strength: f32,
    height: f32,
) {
    if strength <= 0.001 {
        return;
    }

    renderer.fill_quad(
        Quad {
            bounds,
            border: Border { radius: radius.into(), ..Border::default() },
            shadow: Shadow {
                color: tone::fade(color, strength),
                offset: Vector::new(0.0, height * 0.5),
                blur_radius: height * 1.6,
            },
            ..Quad::default()
        },
        Background::Color(Color::TRANSPARENT),
    );
}

/// Where a run of text hangs from.
#[derive(Debug, Clone, Copy)]
pub enum Anchor {
    /// The point is the left edge of the run.
    Left,
    /// The point is the middle of the run.
    Center,
}

impl Anchor {
    fn alignment(self) -> TextAlignment {
        match self {
            Anchor::Left => TextAlignment::Left,
            Anchor::Center => TextAlignment::Center,
        }
    }
}

/// Draws a single line of text hanging from `at`, vertically centred on it.
#[allow(clippy::too_many_arguments)]
pub fn write(
    renderer: &mut Renderer,
    content: &str,
    font: Font,
    size: f32,
    color: Color,
    at: Point,
    anchor: Anchor,
    width: f32,
    clip: Rectangle,
) {
    renderer.fill_text(
        Text {
            content: content.to_owned(),
            bounds: Size::new(width, size * 2.0),
            size: Pixels(size),
            line_height: LineHeight::Relative(1.2),
            font,
            align_x: anchor.alignment(),
            align_y: Vertical::Center,
            shaping: Shaping::Advanced,
            wrapping: Wrapping::None,
        },
        at,
        color,
        clip,
    );
}
