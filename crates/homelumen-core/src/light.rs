/// What a light is currently doing.
///
/// Every field beyond [`power`](LightState::power) is optional: a device only
/// reports what it actually has.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct LightState {
    /// Whether the light is emitting.
    pub power: bool,
    /// Current dimming level, in percent.
    pub brightness: Option<u8>,
    /// Current colour, whether white or saturated.
    pub color: Option<Color>,
    /// Identifier of the running effect, if any.
    pub effect: Option<String>,
}

/// The colour of a light, expressed the way the light itself thinks about it.
///
/// Lights are either in a white mode driven by a colour temperature, or in a
/// tinted mode driven by a hue. Keeping the two apart avoids the lossy round
/// trip through RGB that would otherwise wash out warm whites.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Color {
    /// White light at the given temperature, in kelvin.
    White { kelvin: u16 },
    /// Tinted light: `hue` in `0..=360`, `saturation` in `0..=100`.
    Tint { hue: u16, saturation: u8 },
}

impl Color {
    /// Neutral warm white, used whenever a device reports no colour at all.
    pub const DEFAULT_WHITE: Self = Self::White { kelvin: 2700 };
}
