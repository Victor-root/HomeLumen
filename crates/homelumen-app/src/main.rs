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
            size: Size::new(1180.0, 820.0),
            min_size: Some(Size::new(940.0, 660.0)),
            icon: icon::window(),
            ..window::Settings::default()
        })
        .centered()
        .run()
}
