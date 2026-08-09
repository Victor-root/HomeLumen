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

/// The footprint of the Windows Calculator app: HomeLumen opens at this size
/// and refuses to shrink past it, while still resizing larger or maximizing
/// freely through the title bar.
const COMPACT_SIZE: Size = Size::new(344.0, 704.0);

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
