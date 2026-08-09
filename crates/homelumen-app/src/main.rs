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

fn main() -> iced::Result {
    iced::application(App::default, App::update, App::view)
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
        .window(window::Settings {
            size: COMPACT_SIZE,
            min_size: Some(COMPACT_SIZE),
            icon: icon::window(),
            ..window::Settings::default()
        })
        .centered()
        .run()
}
