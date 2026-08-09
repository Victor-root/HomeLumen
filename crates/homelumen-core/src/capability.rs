/// Everything a light is able to do.
///
/// The interface is built from this and from nothing else: an absent capability
/// simply means the matching control is never rendered.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Capabilities {
    /// The light can be switched on and off. Practically always true, but a
    /// permanently powered fixture driven only by scenes would not have it.
    pub power: bool,
    /// The light can be dimmed within the given range.
    pub brightness: Option<BrightnessRange>,
    /// The light can shift between warm and cool white within the given range.
    pub color_temperature: Option<ColorTemperatureRange>,
    /// The light can render arbitrary hues.
    pub color: bool,
    /// Built-in animations the device plays on its own.
    pub effects: Vec<Effect>,
}

impl Capabilities {
    /// Widens `self` so that it covers everything `other` can do.
    ///
    /// A device reachable through several routes may expose slightly different
    /// abilities depending on the route; the union is what the user can
    /// actually reach.
    pub fn absorb(&mut self, other: &Capabilities) {
        self.power |= other.power;
        self.color |= other.color;

        self.brightness = widest(self.brightness, other.brightness, |a, b| {
            BrightnessRange { min: a.min.min(b.min), max: a.max.max(b.max) }
        });

        self.color_temperature =
            widest(self.color_temperature, other.color_temperature, |a, b| {
                ColorTemperatureRange {
                    min_kelvin: a.min_kelvin.min(b.min_kelvin),
                    max_kelvin: a.max_kelvin.max(b.max_kelvin),
                }
            });

        for effect in &other.effects {
            if !self.effects.iter().any(|known| known.id == effect.id) {
                self.effects.push(effect.clone());
            }
        }
    }
}

fn widest<T: Copy>(
    a: Option<T>,
    b: Option<T>,
    merge: fn(T, T) -> T,
) -> Option<T> {
    match (a, b) {
        (Some(a), Some(b)) => Some(merge(a, b)),
        (only, None) | (None, only) => only,
    }
}

/// The dimming range of a light, in percent.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BrightnessRange {
    /// Dimmest level that still emits light.
    pub min: u8,
    /// Brightest level.
    pub max: u8,
}

impl BrightnessRange {
    /// The usual `1..=100` range of a dimmable bulb.
    pub const PERCENT: Self = Self { min: 1, max: 100 };

    /// Keeps `value` inside the range.
    pub fn clamp(&self, value: u8) -> u8 {
        value.clamp(self.min, self.max)
    }

    /// Maps `fraction` (`0.0..=1.0`) onto the range.
    pub fn from_fraction(&self, fraction: f32) -> u8 {
        let span = f32::from(self.max - self.min);
        self.min + (fraction.clamp(0.0, 1.0) * span).round() as u8
    }

    /// Position of `value` inside the range, as `0.0..=1.0`.
    pub fn fraction(&self, value: u8) -> f32 {
        let span = f32::from(self.max - self.min);
        if span <= 0.0 {
            return 1.0;
        }
        f32::from(self.clamp(value) - self.min) / span
    }
}

/// The white range of a light, in kelvin.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ColorTemperatureRange {
    /// Warmest white the light can produce.
    pub min_kelvin: u16,
    /// Coolest white the light can produce.
    pub max_kelvin: u16,
}

impl ColorTemperatureRange {
    /// Keeps `kelvin` inside the range.
    pub fn clamp(&self, kelvin: u16) -> u16 {
        kelvin.clamp(self.min_kelvin, self.max_kelvin)
    }

    /// Maps `fraction` (`0.0` warmest, `1.0` coolest) onto the range.
    pub fn from_fraction(&self, fraction: f32) -> u16 {
        let span = f32::from(self.max_kelvin - self.min_kelvin);
        self.min_kelvin + (fraction.clamp(0.0, 1.0) * span).round() as u16
    }

    /// Position of `kelvin` inside the range, as `0.0..=1.0`.
    pub fn fraction(&self, kelvin: u16) -> f32 {
        let span = f32::from(self.max_kelvin - self.min_kelvin);
        if span <= 0.0 {
            return 0.0;
        }
        f32::from(self.clamp(kelvin) - self.min_kelvin) / span
    }
}

/// A built-in animation exposed by a device.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Effect {
    /// Driver-scoped identifier, sent back verbatim in [`crate::Command::Effect`].
    pub id: String,
    /// Human readable name, shown as-is by the interface.
    pub name: String,
}
