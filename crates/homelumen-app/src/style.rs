//! Styles for the stock widgets, so nothing on screen keeps its default look.

use iced::widget::text::IntoFragment;
use iced::widget::{Text, button, container, scrollable, text, text_input};
use iced::{Background, Border, Color, Font, Shadow, Theme, Vector};

use crate::design::{Skin, round, tone};

/// A run of text in the HomeLumen scale.
///
/// Every string on screen goes through here, so nothing ever falls back to a
/// default size or a theme colour.
pub fn label<'a>(
    content: impl IntoFragment<'a>,
    size: f32,
    font: Font,
    color: Color,
) -> Text<'a> {
    text(content).size(size).font(font).color(color)
}

/// The page itself: a quiet wash that lifts the top of the window.
pub fn page(skin: Skin) -> impl Fn(&Theme) -> container::Style {
    move |_| container::Style {
        background: Some(Background::Color(skin.canvas)),
        text_color: Some(skin.ink),
        ..container::Style::default()
    }
}

/// A raised surface holding a group of controls.
pub fn card(skin: Skin) -> impl Fn(&Theme) -> container::Style {
    move |_| container::Style {
        background: Some(Background::Color(skin.surface)),
        border: Border {
            radius: round::PANEL.into(),
            width: 1.0,
            color: skin.edge_soft,
        },
        shadow: Shadow {
            color: tone::fade(skin.shadow, skin.depth() * 0.22),
            offset: Vector::new(0.0, 6.0),
            blur_radius: 22.0,
        },
        ..container::Style::default()
    }
}

/// A control that only shows itself when the pointer is near.
pub fn quiet(skin: Skin) -> impl Fn(&Theme, button::Status) -> button::Style {
    move |_, status| {
        let awake =
            matches!(status, button::Status::Hovered | button::Status::Pressed);

        button::Style {
            background: Some(Background::Color(if awake {
                skin.surface_lift
            } else {
                tone::fade(skin.surface, 0.0)
            })),
            text_color: if awake { skin.ink } else { skin.ink_soft },
            border: Border {
                radius: round::FULL.into(),
                width: 1.0,
                color: if awake {
                    tone::mix(skin.edge, skin.ink_faint, 0.35)
                } else {
                    skin.edge
                },
            },
            ..button::Style::default()
        }
    }
}

/// The one button on a screen that carries the action.
pub fn solid(skin: Skin) -> impl Fn(&Theme, button::Status) -> button::Style {
    move |_, status| {
        let base = match status {
            button::Status::Hovered => {
                tone::mix(skin.accent, Color::WHITE, 0.10)
            }
            button::Status::Pressed => skin.accent_deep,
            _ => skin.accent,
        };

        button::Style {
            background: Some(Background::Color(base)),
            text_color: skin.ink_over_light,
            border: Border {
                radius: round::CONTROL.into(),
                ..Border::default()
            },
            shadow: Shadow {
                color: tone::fade(skin.accent, 0.28),
                offset: Vector::new(0.0, 5.0),
                blur_radius: 18.0,
            },
            ..button::Style::default()
        }
    }
}

/// Where an address is typed.
pub fn field(
    skin: Skin,
) -> impl Fn(&Theme, text_input::Status) -> text_input::Style {
    move |_, status| {
        let focused = matches!(status, text_input::Status::Focused { .. });

        text_input::Style {
            background: Background::Color(tone::mix(
                skin.canvas,
                skin.surface,
                0.5,
            )),
            border: Border {
                radius: round::CONTROL.into(),
                width: 1.0,
                color: if focused { skin.accent } else { skin.edge },
            },
            icon: skin.ink_faint,
            placeholder: skin.ink_faint,
            value: skin.ink,
            selection: tone::fade(skin.accent, 0.35),
        }
    }
}

/// A scrollbar that stays out of the way.
pub fn scroller(
    skin: Skin,
) -> impl Fn(&Theme, scrollable::Status) -> scrollable::Style {
    move |_, status| {
        let awake = !matches!(status, scrollable::Status::Active { .. });

        let rail = scrollable::Rail {
            background: None,
            border: Border::default(),
            scroller: scrollable::Scroller {
                background: Background::Color(tone::fade(
                    skin.ink_faint,
                    if awake { 0.55 } else { 0.25 },
                )),
                border: Border {
                    radius: round::FULL.into(),
                    ..Border::default()
                },
            },
        };

        scrollable::Style {
            container: container::Style::default(),
            vertical_rail: rail,
            horizontal_rail: rail,
            gap: None,
            auto_scroll: scrollable::AutoScroll {
                background: Background::Color(skin.surface_lift),
                border: Border {
                    radius: round::FULL.into(),
                    width: 1.0,
                    color: skin.edge,
                },
                shadow: Shadow {
                    color: tone::fade(skin.shadow, skin.depth() * 0.4),
                    offset: Vector::new(0.0, 4.0),
                    blur_radius: 14.0,
                },
                icon: skin.ink_soft,
            },
        }
    }
}
