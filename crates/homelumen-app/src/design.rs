//! The visual language of HomeLumen: its colours, its typography, its motion.

pub mod motion;
pub mod skin;
pub mod text;
pub mod tone;
pub mod typo;

pub use skin::{Mode, Preference, Skin, round, space};
pub use text::Lang;
