//! The window icon, drawn rather than shipped.
//!
//! It is the same house-and-bulb mark as the header (see
//! `crate::widget::mark`), rasterised at start-up, so HomeLumen carries its
//! identity into the taskbar without an image file. On Windows this is only
//! half the story: see `build.rs` for the other half, the icon Explorer and
//! the taskbar's own jump list read straight off the executable rather than
//! off a running window.

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

/// The chimney: top-left corner and size, also shared with the header mark.
const CHIMNEY: (f32, f32, f32, f32) = (0.64, 0.26, 0.10, 0.20);

/// The gradient every part of the mark shares, sampled by how far down the
/// whole mark a point sits rather than by its own shape.
const GOLD_TOP: (f32, f32, f32) = (1.0, 0.910, 0.671);
const GOLD_BOTTOM: (f32, f32, f32) = (0.929, 0.639, 0.180);
const GOLD_SPAN: (f32, f32) = (0.16, 0.92);

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

    let chimney = (
        at(CHIMNEY.0, CHIMNEY.1).0,
        at(CHIMNEY.0, CHIMNEY.1).1,
        CHIMNEY.2 * 100.0 * scale,
        CHIMNEY.3 * 100.0 * scale,
    );
    let chimney_round = chimney.2 * 0.25;

    let glass = at(0.50, 0.58);
    let radius = 19.0 * scale;

    let thread_half_width = radius * 0.55;
    let thread_half_height = 2.0 * scale;
    let thread_gap = 1.2 * scale;
    let thread_round = thread_half_height;
    let tip_radius = 1.4 * scale;

    let mut out = Vec::with_capacity((side * side * 4) as usize);

    for y in 0..side {
        for x in 0..side {
            let px = x as f32 + 0.5;
            let py = y as f32 + 0.5;
            let tone = gold_at(py / side as f32);

            let mut layer = [0.0, 0.0, 0.0, 0.0];

            // The chimney, behind the roofline.
            let chimney_d = rounded_rect_distance(
                px - (chimney.0 + chimney.2 / 2.0),
                py - (chimney.1 + chimney.3 / 2.0),
                chimney.2 / 2.0,
                chimney.3 / 2.0,
                chimney_round,
            );
            layer = over(layer, tone, edge(chimney_d));

            // The roofline, on top of the chimney: the only shape that has
            // to stay a crisp, unbroken outline however much the glass
            // beneath it grows.
            let roof_d = polyline_distance(px, py, &roof) - stroke / 2.0;
            layer = over(layer, tone, edge(roof_d));

            // The bulb: a globe over a threaded base, tucked inside the
            // house.
            let dist = ((px - glass.0).powi(2) + (py - glass.1).powi(2)).sqrt();
            layer = over(layer, tone, edge(dist - radius));

            let base_start = glass.1 + radius + thread_half_height;
            let band_center = |band: i32| {
                base_start
                    + band as f32 * (2.0 * thread_half_height + thread_gap)
            };

            for band in 0..3 {
                let band_d = rounded_rect_distance(
                    px - glass.0,
                    py - band_center(band),
                    thread_half_width,
                    thread_half_height,
                    thread_round,
                );
                layer = over(layer, tone, edge(band_d));
            }

            let tip_y =
                band_center(2) + thread_half_height + thread_gap + tip_radius;
            let tip_d = ((px - glass.0).powi(2) + (py - tip_y).powi(2)).sqrt()
                - tip_radius;
            layer = over(layer, tone, edge(tip_d));

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

/// The mark's shared gradient colour at `y_fraction` down the whole icon.
fn gold_at(y_fraction: f32) -> (f32, f32, f32) {
    let t = ((y_fraction - GOLD_SPAN.0) / (GOLD_SPAN.1 - GOLD_SPAN.0))
        .clamp(0.0, 1.0);
    lerp(GOLD_TOP, GOLD_BOTTOM, t)
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
