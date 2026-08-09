//! Colour arithmetic: what a light looks like, and how to lay ink over it.

use homelumen_core::{Color as LightColor, LightState};
use iced::Color;

/// Blends `from` into `to`, `amount` running `0.0..=1.0`.
pub fn mix(from: Color, to: Color, amount: f32) -> Color {
    let t = amount.clamp(0.0, 1.0);
    Color {
        r: from.r + (to.r - from.r) * t,
        g: from.g + (to.g - from.g) * t,
        b: from.b + (to.b - from.b) * t,
        a: from.a + (to.a - from.a) * t,
    }
}

/// The same colour at a different opacity.
pub fn fade(color: Color, opacity: f32) -> Color {
    Color { a: opacity.clamp(0.0, 1.0), ..color }
}

/// Perceived lightness, used to decide whether ink should be dark or light.
pub fn luminance(color: Color) -> f32 {
    0.2126 * color.r + 0.7152 * color.g + 0.0722 * color.b
}

/// The colour of a black body at `kelvin`, normalised so the brightest channel
/// reaches 1.
///
/// This is what makes the white control show real warmth rather than a grey
/// slider with a number next to it.
pub fn kelvin(kelvin: f32) -> Color {
    let t = kelvin.clamp(1000.0, 40000.0) / 100.0;

    let red = if t <= 66.0 {
        255.0
    } else {
        329.698_73 * (t - 60.0).powf(-0.133_204_76)
    };

    let green = if t <= 66.0 {
        99.470_8 * t.ln() - 161.119_57
    } else {
        288.122_16 * (t - 60.0).powf(-0.075_514_846)
    };

    let blue = if t >= 66.0 {
        255.0
    } else if t <= 19.0 {
        0.0
    } else {
        138.517_73 * (t - 10.0).ln() - 305.044_8
    };

    Color::from_rgb(
        (red / 255.0).clamp(0.0, 1.0),
        (green / 255.0).clamp(0.0, 1.0),
        (blue / 255.0).clamp(0.0, 1.0),
    )
}

/// A colour from hue in degrees, saturation and value in `0.0..=1.0`.
pub fn hsv(hue: f32, saturation: f32, value: f32) -> Color {
    let hue = hue.rem_euclid(360.0);
    let saturation = saturation.clamp(0.0, 1.0);
    let value = value.clamp(0.0, 1.0);

    let sector = hue / 60.0;
    let chroma = value * saturation;
    let second = chroma * (1.0 - (sector % 2.0 - 1.0).abs());
    let base = value - chroma;

    let (r, g, b) = match sector as u32 {
        0 => (chroma, second, 0.0),
        1 => (second, chroma, 0.0),
        2 => (0.0, chroma, second),
        3 => (0.0, second, chroma),
        4 => (second, 0.0, chroma),
        _ => (chroma, 0.0, second),
    };

    Color::from_rgb(r + base, g + base, b + base)
}

/// The colour a light is putting out, ignoring how bright it is.
pub fn emission(state: &LightState) -> Color {
    match state.color.unwrap_or(LightColor::DEFAULT_WHITE) {
        LightColor::White { kelvin: k } => kelvin(f32::from(k)),
        LightColor::Tint { hue, saturation } => {
            hsv(f32::from(hue), f32::from(saturation) / 100.0, 1.0)
        }
    }
}

/// How much of itself a light is putting out, `0.0..=1.0`.
///
/// A dimmed light does not simply get darker on screen, it gets quieter: the
/// curve keeps a low percentage visible instead of collapsing to nothing.
pub fn intensity(state: &LightState) -> f32 {
    if !state.power {
        return 0.0;
    }

    let level = f32::from(state.brightness.unwrap_or(100)) / 100.0;
    0.35 + 0.65 * level.clamp(0.0, 1.0).powf(0.65)
}
