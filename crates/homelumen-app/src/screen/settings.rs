//! Settings: for now, the language.

use iced::widget::{column, container, row};
use iced::{Center, Element, Fill};

use crate::app::Message;
use crate::design::{Lang, LangPreference, Skin, space as gap, typo};
use crate::screen::home::round;
use crate::style::{self, label};
use crate::widget::glyph::Glyph;
use crate::widget::segmented::Segmented;

/// Width of one choice in the language picker: narrow enough that all three
/// still fit inside HomeLumen's compact window width.
const LANG_SEGMENT: f32 = 78.0;

/// Draws the settings screen.
pub fn view<'a>(
    skin: Skin,
    lang: Lang,
    lang_preference: LangPreference,
) -> Element<'a, Message> {
    let header = row![
        round(Glyph::Back, Message::Back, skin),
        label(lang.settings(), typo::TITLE, typo::SEMIBOLD, skin.ink),
    ]
    .spacing(gap::STEP)
    .align_y(Center);

    let active = match lang_preference {
        LangPreference::Auto => 0,
        LangPreference::En => 1,
        LangPreference::Fr => 2,
    };

    let language = column![
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
    .spacing(gap::TIGHT);

    container(column![header, language].spacing(gap::GAP))
        .padding([gap::ROOM, gap::MARGIN])
        .width(Fill)
        .height(Fill)
        .style(style::page(skin))
        .into()
}
