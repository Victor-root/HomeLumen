use iced::Color;

/// Builds a colour from an `RRGGBB` literal.
const fn hex(value: u32) -> Color {
    Color {
        r: ((value >> 16) & 0xFF) as f32 / 255.0,
        g: ((value >> 8) & 0xFF) as f32 / 255.0,
        b: (value & 0xFF) as f32 / 255.0,
        a: 1.0,
    }
}

/// Which of the two skins is on screen.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    /// Deep ink. The skin HomeLumen is designed around.
    Night,
    /// Warm paper.
    Day,
}

impl Mode {
    /// The other one.
    pub fn flipped(self) -> Self {
        match self {
            Mode::Night => Mode::Day,
            Mode::Day => Mode::Night,
        }
    }
}

/// The whole colour vocabulary of the interface.
///
/// Nothing outside this file picks a colour: every widget reads the skin, which
/// is what keeps the two themes honest.
#[derive(Debug, Clone, Copy)]
pub struct Skin {
    /// Which skin this is.
    pub mode: Mode,
    /// The window itself.
    pub canvas: Color,
    /// The wash laid over the canvas, top of the page.
    pub veil: Color,
    /// Cards and panels.
    pub surface: Color,
    /// A surface raised above another one.
    pub surface_lift: Color,
    /// Hairlines separating surfaces.
    pub edge: Color,
    /// The same hairline, one step quieter.
    pub edge_soft: Color,
    /// Titles and values.
    pub ink: Color,
    /// Secondary copy.
    pub ink_soft: Color,
    /// Captions, units, disabled marks.
    pub ink_faint: Color,
    /// Ink laid over a fully lit surface.
    pub ink_over_light: Color,
    /// The warm signature of HomeLumen.
    pub accent: Color,
    /// A deeper accent, for gradients and pressed states.
    pub accent_deep: Color,
    /// Something went wrong.
    pub alarm: Color,
    /// The colour depth is cast in.
    pub shadow: Color,
}

impl Skin {
    /// Deep ink, warm light. The reference skin.
    pub const NIGHT: Self = Self {
        mode: Mode::Night,
        canvas: hex(0x07080B),
        veil: hex(0x0D1017),
        surface: hex(0x101319),
        surface_lift: hex(0x161A23),
        edge: hex(0x232936),
        edge_soft: hex(0x171B24),
        ink: hex(0xEEF1F6),
        ink_soft: hex(0x99A1B2),
        ink_faint: hex(0x5B6373),
        ink_over_light: hex(0x1A1204),
        accent: hex(0xFFC773),
        accent_deep: hex(0xF2A63F),
        alarm: hex(0xFF8B7A),
        shadow: hex(0x000000),
    };

    /// Warm paper.
    pub const DAY: Self = Self {
        mode: Mode::Day,
        canvas: hex(0xECEEF3),
        veil: hex(0xF5F6FA),
        surface: hex(0xFFFFFF),
        surface_lift: hex(0xFFFFFF),
        edge: hex(0xDFE3EC),
        edge_soft: hex(0xEDF0F6),
        ink: hex(0x0F131A),
        ink_soft: hex(0x5A6272),
        ink_faint: hex(0x98A0B0),
        ink_over_light: hex(0x2A1B05),
        accent: hex(0xD9871F),
        accent_deep: hex(0xB76F14),
        alarm: hex(0xC94A38),
        shadow: hex(0x2A3040),
    };

    /// The skin for a mode.
    pub fn of(mode: Mode) -> Self {
        match mode {
            Mode::Night => Self::NIGHT,
            Mode::Day => Self::DAY,
        }
    }

    /// How strongly depth is cast, which differs a lot between the two skins:
    /// a shadow on paper is a whisper, a shadow on ink is a void.
    pub fn depth(&self) -> f32 {
        match self.mode {
            Mode::Night => 0.55,
            Mode::Day => 0.14,
        }
    }
}

/// The rhythm the whole interface is laid out on.
pub mod space {
    /// Between a label and its value.
    pub const TIGHT: f32 = 6.0;
    /// Inside a small control.
    pub const SNUG: f32 = 12.0;
    /// Between related blocks.
    pub const STEP: f32 = 18.0;
    /// Inside a card.
    pub const ROOM: f32 = 26.0;
    /// Between sections.
    pub const GAP: f32 = 34.0;
    /// Around the page.
    pub const MARGIN: f32 = 44.0;
}

/// Corner radii, from the smallest chip to the largest panel.
pub mod round {
    /// Buttons and small controls.
    pub const CONTROL: f32 = 18.0;
    /// Panels.
    pub const PANEL: f32 = 24.0;
    /// Tiles and the large controls.
    pub const TILE: f32 = 28.0;
    /// Anything meant to read as a capsule.
    pub const FULL: f32 = 999.0;
}
