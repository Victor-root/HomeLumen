use crate::light::Color;

/// An intent expressed by the user, in the generic vocabulary.
///
/// Commands are applied as a batch so a driver can collapse them into a single
/// round trip: turning a light on and setting its colour at once must not make
/// it flash through its previous colour.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Command {
    /// Switch the light on or off.
    Power(bool),
    /// Set the dimming level, in percent.
    Brightness(u8),
    /// Set the colour, white or tinted.
    Color(Color),
    /// Start an effect, or stop the running one with `None`.
    Effect(Option<String>),
}
