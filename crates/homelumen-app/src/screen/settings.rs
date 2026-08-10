//! Settings: the language, and the accounts HomeLumen speaks through.

use iced::widget::{button, column, container, row, scrollable, text_input};
use iced::{Center, Element, Fill, Right};

use crate::app::Message;
use crate::design::{Lang, LangPreference, Skin, space as gap, typo};
use crate::screen::home::round;
use crate::style::{self, label};
use crate::widget::glyph::Glyph;
use crate::widget::segmented::Segmented;

/// Width of one choice in the language picker: narrow enough that all three
/// still fit inside HomeLumen's compact window width.
const LANG_SEGMENT: f32 = 78.0;

/// The account section never grows past a comfortable reading width, however
/// wide the window gets: a text field stretched across a maximised window is
/// no easier to fill in and no easier to read back.
const WIDEST: f32 = 520.0;

/// What the user typed into the Tuya section, which is not the same thing as
/// what is on file: it only becomes an account once saved.
pub struct TuyaDraft<'a> {
    pub id: &'a str,
    pub secret: &'a str,
}

/// Draws the settings screen.
pub fn view<'a>(
    skin: Skin,
    lang: Lang,
    lang_preference: LangPreference,
    tuya: TuyaDraft<'a>,
) -> Element<'a, Message> {
    let header = row![
        round(Glyph::Back, Message::Back, skin),
        label(lang.settings(), typo::TITLE, typo::SEMIBOLD, skin.ink),
    ]
    .spacing(gap::STEP)
    .align_y(Center);

    let body = column![
        language(skin, lang, lang_preference),
        smart_life(skin, lang, tuya)
    ]
    .spacing(gap::GAP)
    .width(Fill);

    container(
        column![
            header,
            scrollable(body).height(Fill).style(style::scroller(skin))
        ]
        .spacing(gap::GAP),
    )
    .padding([gap::ROOM, gap::MARGIN])
    .width(Fill)
    .height(Fill)
    .style(style::page(skin))
    .into()
}

fn language<'a>(
    skin: Skin,
    lang: Lang,
    lang_preference: LangPreference,
) -> Element<'a, Message> {
    let active = match lang_preference {
        LangPreference::Auto => 0,
        LangPreference::En => 1,
        LangPreference::Fr => 2,
    };

    column![
        label(lang.language(), typo::LABEL, typo::REGULAR, skin.ink_faint),
        Segmented::new(
            vec!["Auto", "English", "Français"],
            active,
            skin,
            |index| Message::LangChanged(match index {
                1 => LangPreference::En,
                2 => LangPreference::Fr,
                _ => LangPreference::Auto,
            }),
        )
        .segment_width(LANG_SEGMENT)
        .height(38.0),
    ]
    .spacing(gap::TIGHT)
    .into()
}

/// The Tuya cloud project, which is what turns the plugs tab from an empty
/// promise into the account's own plugs.
fn smart_life<'a>(
    skin: Skin,
    lang: Lang,
    tuya: TuyaDraft<'a>,
) -> Element<'a, Message> {
    let field = |placeholder: &'a str, value: &'a str, secret: bool| {
        text_input(placeholder, value)
            .padding([13.0, 16.0])
            .size(typo::BODY)
            .font(typo::REGULAR)
            .width(Fill)
            .secure(secret)
            .style(style::field(skin))
    };

    column![
        label(lang.smart_life(), typo::LEAD, typo::SEMIBOLD, skin.ink),
        label(
            lang.smart_life_hint(),
            typo::LABEL,
            typo::REGULAR,
            skin.ink_faint
        ),
        field("Client ID", tuya.id, false).on_input(Message::TuyaIdTyped),
        field("Client Secret", tuya.secret, true)
            .on_input(Message::TuyaSecretTyped)
            .on_submit(Message::TuyaSave),
        container(
            button(label(
                lang.save(),
                typo::BODY,
                typo::MEDIUM,
                skin.ink_over_light,
            ))
            .padding([13.0, 26.0])
            .style(style::solid(skin))
            .on_press(Message::TuyaSave)
        )
        .width(Fill)
        .align_x(Right),
    ]
    .spacing(gap::SNUG)
    .max_width(WIDEST)
    .into()
}
