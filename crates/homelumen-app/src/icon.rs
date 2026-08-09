//! The window icon, drawn rather than shipped.
//!
//! It is the same open halo as the mark in the header, rasterised at start-up,
//! so HomeLumen carries its identity into the taskbar without an image file.

use std::f32::consts::{PI, TAU};

use iced::window::icon::{self, Icon};

use crate::design::tone;

const SIDE: u32 = 128;

/// Builds the window icon.
pub fn window() -> Option<Icon> {
    let side = SIDE as f32;
    let center = side / 2.0;
    let radius = side * 0.34;
    let stroke = side * 0.115;

    let opening = 0.18;
    let start = TAU * 0.10 - PI / 2.0;
    let end = start + TAU * (1.0 - opening);

    let head = (center + radius * start.cos(), center + radius * start.sin());

    let mut pixels = Vec::with_capacity((SIDE * SIDE * 4) as usize);

    for y in 0..SIDE {
        for x in 0..SIDE {
            let px = x as f32 + 0.5;
            let py = y as f32 + 0.5;
            let dx = px - center;
            let dy = py - center;

            let distance = (dx * dx + dy * dy).sqrt();
            let angle = dy.atan2(dx);

            // How far the pixel is from the ideal stroke, and from the round
            // cap sitting at the start of the arc.
            let ring = (distance - radius).abs() - stroke / 2.0;
            let on_arc = within(angle, start, end);
            let cap = ((px - head.0).powi(2) + (py - head.1).powi(2)).sqrt()
                - stroke * 0.62;

            let coverage =
                if on_arc { edge(ring).max(edge(cap)) } else { edge(cap) };

            if coverage <= 0.0 {
                pixels.extend_from_slice(&[0, 0, 0, 0]);
                continue;
            }

            // Warm where the light starts, cool where it fades.
            let along = ((angle - start).rem_euclid(TAU) / TAU).clamp(0.0, 1.0);
            let colour = tone::kelvin(2200.0 + along * 5600.0);

            pixels.extend_from_slice(&[
                channel(colour.r),
                channel(colour.g),
                channel(colour.b),
                channel(coverage),
            ]);
        }
    }

    icon::from_rgba(pixels, SIDE, SIDE).ok()
}

/// One pixel of antialiasing on either side of a signed distance.
fn edge(distance: f32) -> f32 {
    (0.5 - distance).clamp(0.0, 1.0)
}

fn within(angle: f32, start: f32, end: f32) -> bool {
    (angle - start).rem_euclid(TAU) <= (end - start).rem_euclid(TAU)
}

fn channel(value: f32) -> u8 {
    (value.clamp(0.0, 1.0) * 255.0).round() as u8
}
