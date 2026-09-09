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

/// The roofline, in the same fractional coordinates as the header mark:
/// stroked as one open path rather than closed along the floor, a wall,
/// the eave tip reaching past it, the peak, and back down the mirrored
/// side. Each wall ends where its stroke does; the flare beneath it is a
/// separate shape, see `FOOT_LEFT`/`FOOT_RIGHT`.
const ROOF: [(f32, f32); 7] = [
    (0.2583, 0.6802),
    (0.2583, 0.4705),
    (0.2104, 0.4322),
    (0.4959, 0.1974),
    (0.7814, 0.4322),
    (0.7336, 0.4705),
    (0.7336, 0.6802),
];

/// Width of the roofline stroke.
const ROOF_STROKE: f32 = 0.0477;

/// The chimney: top-left corner and size, also shared with the header mark.
const CHIMNEY: (f32, f32, f32, f32) = (0.6475, 0.2360, 0.0772, 0.1140);

/// Each wall flares into a foot wider than the wall itself, rather than
/// the stroke just rounding off at its end: seven points tracing the
/// flare's own edge, shared with the header mark.
const FOOT_LEFT: [(f32, f32); 7] = [
    (0.2345, 0.6802),
    (0.3086, 0.7520),
    (0.3947, 0.7520),
    (0.4075, 0.7376),
    (0.3684, 0.6986),
    (0.3397, 0.6986),
    (0.2823, 0.6427),
];
const FOOT_RIGHT: [(f32, f32); 7] = [
    (0.7574, 0.6802),
    (0.6833, 0.7520),
    (0.5971, 0.7520),
    (0.5844, 0.7376),
    (0.6234, 0.6986),
    (0.6522, 0.6986),
    (0.7096, 0.6427),
];

/// The bulb's globe: centre, radius. Shared with the header mark.
const GLOBE: (f32, f32, f32) = (0.496, 0.489, 0.126);

/// The threaded base below the globe: two bands, narrower than the last,
/// then a domed tip. Each as (half width, half height), stacked by their
/// own measured gaps rather than an even split. Shared with the header mark.
const BAND_1: (f32, f32) = (0.061, 0.0115);
const BAND_2: (f32, f32) = (0.0575, 0.0115);
const TIP: (f32, f32) = (0.0375, 0.0205);
const GLOBE_TO_BAND_1: f32 = 0.016;
const BAND_GAP: f32 = 0.012;
const BAND_TO_TIP: f32 = 0.013;

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
    let stroke = ROOF_STROKE * 100.0 * scale;

    let chimney = (
        at(CHIMNEY.0, CHIMNEY.1).0,
        at(CHIMNEY.0, CHIMNEY.1).1,
        CHIMNEY.2 * 100.0 * scale,
        CHIMNEY.3 * 100.0 * scale,
    );
    let chimney_round = chimney.2 * 0.134;

    let foot_left: Vec<(f32, f32)> =
        FOOT_LEFT.iter().map(|(x, y)| at(*x, *y)).collect();
    let foot_right: Vec<(f32, f32)> =
        FOOT_RIGHT.iter().map(|(x, y)| at(*x, *y)).collect();

    let glass = at(GLOBE.0, GLOBE.1);
    let radius = GLOBE.2 * 100.0 * scale;

    let band_1_half = (BAND_1.0 * 100.0 * scale, BAND_1.1 * 100.0 * scale);
    let band_2_half = (BAND_2.0 * 100.0 * scale, BAND_2.1 * 100.0 * scale);
    let tip_half = (TIP.0 * 100.0 * scale, TIP.1 * 100.0 * scale);

    let band_1_y =
        glass.1 + radius + GLOBE_TO_BAND_1 * 100.0 * scale + band_1_half.1;
    let band_2_y =
        band_1_y + band_1_half.1 + BAND_GAP * 100.0 * scale + band_2_half.1;
    let tip_y =
        band_2_y + band_2_half.1 + BAND_TO_TIP * 100.0 * scale + tip_half.1;

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

            // The roofline: an open path, each wall capped by its own
            // flared foot rather than closed along the floor.
            let roof_d = polyline_distance(px, py, &roof) - stroke / 2.0;
            layer = over(layer, tone, edge(roof_d));

            // Each foot, wider than the wall it flares from, covering the
            // wall's own rounded end.
            for foot in [&foot_left, &foot_right] {
                layer = over(layer, tone, edge(polygon_sdf(px, py, foot)));
            }

            // The bulb: a globe over a threaded base, tucked inside the
            // house.
            let dist = ((px - glass.0).powi(2) + (py - glass.1).powi(2)).sqrt();
            layer = over(layer, tone, edge(dist - radius));

            for (half, center_y) in [
                (band_1_half, band_1_y),
                (band_2_half, band_2_y),
                (tip_half, tip_y),
            ] {
                let band_d = rounded_rect_distance(
                    px - glass.0,
                    py - center_y,
                    half.0,
                    half.1,
                    half.1,
                );
                layer = over(layer, tone, edge(band_d));
            }

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

/// Signed distance to a filled closed polygon: negative inside, positive
/// out. Used for the feet, whose flare is not a shape `rounded_rect_distance`
/// can express.
fn polygon_sdf(px: f32, py: f32, points: &[(f32, f32)]) -> f32 {
    let n = points.len();
    let distance = (0..n)
        .map(|i| segment_distance(px, py, points[i], points[(i + 1) % n]))
        .fold(f32::INFINITY, f32::min);

    if point_in_polygon(px, py, points) { -distance } else { distance }
}

/// Whether a point lies inside a closed polygon (even-odd rule).
fn point_in_polygon(px: f32, py: f32, points: &[(f32, f32)]) -> bool {
    let n = points.len();
    let mut inside = false;

    for i in 0..n {
        let (ax, ay) = points[i];
        let (bx, by) = points[(i + 1) % n];

        if (ay > py) != (by > py) {
            let x_crossing = ax + (py - ay) / (by - ay) * (bx - ax);
            if px < x_crossing {
                inside = !inside;
            }
        }
    }

    inside
}
