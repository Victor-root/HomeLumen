//! The typographic scale.
//!
//! One family, four weights, six sizes. Everything on screen picks from here so
//! the hierarchy stays readable without a single decorative flourish.

use iced::Font;
use iced::font::{Family, Weight};
use iced::widget::text::LineHeight;

/// The bundled variable family. Shipping it keeps the interface identical on
/// Windows and on Linux, whatever the system has installed.
pub const FAMILY: &str = "Inter Variable";

/// The raw bytes, registered at start-up.
pub const EMBEDDED: &[u8] =
    include_bytes!("../../../../assets/fonts/InterVariable.ttf");

const fn at(weight: Weight) -> Font {
    Font {
        family: Family::Name(FAMILY),
        weight,
        stretch: iced::font::Stretch::Normal,
        style: iced::font::Style::Normal,
    }
}

/// Running copy.
pub const REGULAR: Font = at(Weight::Normal);
/// Labels and captions that need a touch more presence.
pub const MEDIUM: Font = at(Weight::Medium);
/// Titles, names, values.
pub const SEMIBOLD: Font = at(Weight::Semibold);

/// The big number or name a screen is about.
pub const DISPLAY: f32 = 33.0;
/// A section heading.
pub const TITLE: f32 = 21.0;
/// The name on a tile, the value on a control.
pub const LEAD: f32 = 17.0;
/// Running copy.
pub const BODY: f32 = 14.5;
/// Captions under a title.
pub const LABEL: f32 = 13.0;
/// Units and the quietest marks.
pub const MICRO: f32 = 11.5;

/// Tight leading, for large type that should not feel airy.
pub const SNUG_LEADING: LineHeight = LineHeight::Relative(1.15);
