//! The HomeLumen mark.
//!
//! The artwork itself, `assets/icon/logo.svg`, compiled into the binary and
//! drawn as it was authored rather than rebuilt out of shapes here: the
//! header, the window icon (see `crate::icon`) and the installer all read
//! that one file, so none of them can drift from the others.

use iced::widget::svg::{self, Svg};
use iced::{Element, Length};

const LOGO: &[u8] = include_bytes!("../../../../assets/icon/logo.svg");

/// Places the mark at the given side length.
pub fn mark<'a, Message>(side: f32) -> Element<'a, Message>
where
    Message: 'a,
{
    Svg::new(svg::Handle::from_memory(LOGO))
        .width(Length::Fixed(side))
        .height(Length::Fixed(side))
        .into()
}
