//! The window icon: the logo artwork, rasterised at start-up.
//!
//! It is the same file the header draws (see `crate::widget::mark`), so
//! HomeLumen carries its identity into the taskbar without a second copy of
//! the mark to keep in step. On Windows this is only half the story: see
//! `build.rs` for the other half, the icon Explorer and the taskbar's own
//! jump list read straight off the executable rather than off a running
//! window.

use iced::window::icon::{self, Icon};
use resvg::{tiny_skia, usvg};

const LOGO: &[u8] = include_bytes!("../../../assets/icon/logo.svg");

const SIDE: u32 = 256;

/// Builds the window icon.
pub fn window() -> Option<Icon> {
    icon::from_rgba(pixels(SIDE)?, SIDE, SIDE).ok()
}

/// The logo rasterised square at `side` pixels, as straight RGBA.
fn pixels(side: u32) -> Option<Vec<u8>> {
    let tree = usvg::Tree::from_data(LOGO, &usvg::Options::default()).ok()?;
    let scale = side as f32 / tree.size().width();

    let mut pixmap = tiny_skia::Pixmap::new(side, side)?;
    resvg::render(
        &tree,
        tiny_skia::Transform::from_scale(scale, scale),
        &mut pixmap.as_mut(),
    );

    let mut out = Vec::with_capacity((side * side * 4) as usize);

    for pixel in pixmap.pixels() {
        let colour = pixel.demultiply();
        out.extend_from_slice(&[
            colour.red(),
            colour.green(),
            colour.blue(),
            colour.alpha(),
        ]);
    }

    Some(out)
}
