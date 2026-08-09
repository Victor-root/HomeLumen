//! The window icon, drawn rather than shipped.
//!
//! It is the same house-and-bulb mark as the header, rasterised at start-up,
//! so HomeLumen carries its identity into the taskbar without an image file.
//! On Windows this is only half the story: see `build.rs` for the other
//! half, the icon Explorer and the taskbar's own jump list read straight off
//! the executable rather than off a running window.

use iced::window::icon::{self, Icon};

const SIDE: u32 = 128;

/// The roofline, in the same fractional coordinates as the header mark.
const ROOF: [(f32, f32); 6] = [
    (0.12, 0.58),
    (0.50, 0.18),
    (0.88, 0.58),
    (0.88, 0.96),
    (0.12, 0.96),
    (0.12, 0.58),
];

/// Builds the window icon.
pub fn window() -> Option<Icon> {
    icon::from_rgba(pixels(SIDE), SIDE, SIDE).ok()
}

/// The icon's raw RGBA pixels, exposed on its own so it can be rasterised
/// outside of a running window too, for a fixed asset that never drifts from
/// what HomeLumen itself draws.
pub fn pixels(side: u32) -> Vec<u8> {
    let scale = side as f32 / 100.0;
    let at = |x: f32, y: f32| (x * 100.0 * scale, y * 100.0 * scale);

    let roof: Vec<(f32, f32)> = ROOF.iter().map(|(x, y)| at(*x, *y)).collect();
    let stroke = 6.0 * scale;

    let glass = at(0.50, 0.66);
    let radius = 22.0 * scale;

    let base_half = (9.0 * scale, 5.0 * scale);
    let base_center = at(0.50, 0.846);
    let base_round = 3.0 * scale;

    let frame_tone = (0.580, 0.529, 0.463);
    let glass_stroke = (0.804, 0.502, 0.180);
    let glass_top = (1.0, 0.859, 0.624);
    let glass_bottom = (0.949, 0.651, 0.247);

    let mut out = Vec::with_capacity((side * side * 4) as usize);

    for y in 0..side {
        for x in 0..side {
            let px = x as f32 + 0.5;
            let py = y as f32 + 0.5;

            let mut layer = [0.0, 0.0, 0.0, 0.0];

            // The socket, tucked behind the glass.
            let base_d = rounded_rect_distance(
                px - base_center.0,
                py - base_center.1,
                base_half.0,
                base_half.1,
                base_round,
            );
            layer = over(layer, frame_tone, edge(base_d));

            // The glass: a vertical gradient standing in for the sphere the
            // real bulb is lit with, plus the hairline around it.
            let dist = ((px - glass.0).powi(2) + (py - glass.1).powi(2)).sqrt();
            let glass_d = dist - radius;
            let t =
                ((py - (glass.1 - radius)) / (radius * 2.0)).clamp(0.0, 1.0);
            let glass_color = lerp(glass_top, glass_bottom, t);
            layer = over(layer, glass_color, edge(glass_d));
            layer = over(
                layer,
                glass_stroke,
                edge((dist - radius).abs() - stroke * 0.07),
            );

            // The roofline, on top: the only shape that has to stay a crisp,
            // unbroken outline however much the glass beneath it grows.
            let roof_d = polyline_distance(px, py, &roof) - stroke / 2.0;
            layer = over(layer, frame_tone, edge(roof_d));

            out.extend_from_slice(&[
                channel(layer[0]),
                channel(layer[1]),
                channel(layer[2]),
                channel(layer[3]),
            ]);
        }
    }

    out
}

/// One pixel of antialiasing on either side of a signed distance.
fn edge(distance: f32) -> f32 {
    (0.5 - distance).clamp(0.0, 1.0)
}

fn channel(value: f32) -> u8 {
    (value.clamp(0.0, 1.0) * 255.0).round() as u8
}

fn lerp(a: (f32, f32, f32), b: (f32, f32, f32), t: f32) -> (f32, f32, f32) {
    (a.0 + (b.0 - a.0) * t, a.1 + (b.1 - a.1) * t, a.2 + (b.2 - a.2) * t)
}

/// Composites `src` (at `src_a` coverage) over `dst`, both already
/// premultiplied by nothing (straight alpha in, straight alpha out).
fn over(dst: [f32; 4], src_rgb: (f32, f32, f32), src_a: f32) -> [f32; 4] {
    let out_a = src_a + dst[3] * (1.0 - src_a);

    if out_a <= 0.0 {
        return [0.0, 0.0, 0.0, 0.0];
    }

    let mix = |src: f32, behind: f32| {
        (src * src_a + behind * dst[3] * (1.0 - src_a)) / out_a
    };

    [
        mix(src_rgb.0, dst[0]),
        mix(src_rgb.1, dst[1]),
        mix(src_rgb.2, dst[2]),
        out_a,
    ]
}

/// The shortest distance from a point to any segment of an open polyline.
fn polyline_distance(px: f32, py: f32, points: &[(f32, f32)]) -> f32 {
    points
        .windows(2)
        .map(|pair| segment_distance(px, py, pair[0], pair[1]))
        .fold(f32::INFINITY, f32::min)
}

fn segment_distance(px: f32, py: f32, a: (f32, f32), b: (f32, f32)) -> f32 {
    let (abx, aby) = (b.0 - a.0, b.1 - a.1);
    let (apx, apy) = (px - a.0, py - a.1);
    let len_sq = abx * abx + aby * aby;

    let t = if len_sq > 0.0 {
        ((apx * abx + apy * aby) / len_sq).clamp(0.0, 1.0)
    } else {
        0.0
    };

    let (cx, cy) = (a.0 + t * abx, a.1 + t * aby);
    ((px - cx).powi(2) + (py - cy).powi(2)).sqrt()
}

/// Signed distance to a rounded rectangle: negative inside, positive out.
fn rounded_rect_distance(
    dx: f32,
    dy: f32,
    half_width: f32,
    half_height: f32,
    radius: f32,
) -> f32 {
    let qx = dx.abs() - (half_width - radius);
    let qy = dy.abs() - (half_height - radius);
    let outside = (qx.max(0.0).powi(2) + qy.max(0.0).powi(2)).sqrt();
    let inside = qx.max(qy).min(0.0);

    outside + inside - radius
}
