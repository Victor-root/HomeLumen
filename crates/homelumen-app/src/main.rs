//! HomeLumen: one screen for your lights, one screen for a light.

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;
mod design;
mod icon;
mod paint;
mod screen;
mod style;
mod widget;

use iced::{Size, window};

use app::App;
use design::typo;

/// The compact footprint HomeLumen opens at, measured directly off the
/// window itself: HomeLumen refuses to shrink past this size, while still
/// resizing larger or maximizing freely through the title bar.
const COMPACT_SIZE: Size = Size::new(385.0, 573.0);

/// The window HomeLumen opens.
fn window() -> window::Settings {
    window::Settings {
        size: COMPACT_SIZE,
        min_size: Some(COMPACT_SIZE),
        icon: icon::window(),
        platform_specific: platform_specific(),
        ..window::Settings::default()
    }
}

/// On most Linux desktops, GNOME chief among them, the taskbar or dock
/// entry for a running window is matched to an installed `.desktop` file by
/// this id, not by the icon the window sets on itself at start-up; see
/// `packaging/linux` for the file that has to agree with it. No other
/// platform HomeLumen ships for reads anything from here.
#[cfg(target_os = "linux")]
fn platform_specific() -> window::settings::PlatformSpecific {
    window::settings::PlatformSpecific {
        application_id: "homelumen".into(),
        ..Default::default()
    }
}

#[cfg(not(target_os = "linux"))]
fn platform_specific() -> window::settings::PlatformSpecific {
    window::settings::PlatformSpecific::default()
}

fn main() -> iced::Result {
    iced::application(App::boot, App::update, App::view)
        .title(App::title)
        .subscription(App::subscription)
        .theme(App::theme)
        .style(|app: &App, _theme| iced::theme::Style {
            background_color: app.skin().canvas,
            text_color: app.skin().ink,
        })
        .font(typo::EMBEDDED)
        .default_font(typo::REGULAR)
        .antialiasing(true)
        .window(window())
        .centered()
        .run()
}
