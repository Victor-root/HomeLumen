//! The screen HomeLumen opens on: the lights, and nothing else.

use homelumen_engine::LightSnapshot;
use iced::widget::{
    Column, Grid, button, center, column, container, responsive, row,
    scrollable, space, text_input,
};
use iced::{Center, Element, Fill, Length};

use crate::app::Message;
use crate::design::{Mode, Skin, space as gap, tone, typo};
use crate::style::{self, label};
use crate::widget::glyph::{Glyph, glyph};
use crate::widget::mark::mark;
use crate::widget::tile::Tile;

/// Widest a tile is allowed to grow before the grid adds a column.
const COLUMN: f32 = 372.0;

/// Draws the home screen.
pub fn view<'a>(
    lights: &'a [LightSnapshot],
    skin: Skin,
    scanning: bool,
    address: Option<&'a str>,
    notice: Option<&'a str>,
) -> Element<'a, Message> {
    let body = if lights.is_empty() {
        empty(skin, scanning)
    } else {
        column![heading(lights, skin, scanning), grid(lights, skin)]
            .spacing(gap::GAP)
            .into()
    };

    let mut page = Column::new().spacing(gap::STEP);
    page = page.push(header(skin, scanning));

    if let Some(typed) = address {
        page = page.push(inset(address_panel(typed, skin)));
    }

    if let Some(message) = notice {
        page = page.push(inset(banner(message, skin)));
    }

    page = page.push(
        scrollable(inset(body).padding([gap::SNUG, gap::MARGIN]).width(Fill))
            .height(Fill)
            .style(style::scroller(skin)),
    );

    container(page)
        .padding([gap::ROOM, 0.0])
        .width(Fill)
        .height(Fill)
        .style(style::page(skin))
        .into()
}

/// Keeps every band of the page on the same left and right margin.
fn inset<'a>(
    content: impl Into<Element<'a, Message>>,
) -> container::Container<'a, Message> {
    container(content).padding([0.0, gap::MARGIN])
}

fn header_controls<'a>(skin: Skin, scanning: bool) -> Element<'a, Message> {
    row![
        round(Glyph::Sweep { busy: scanning }, Message::Sweep, skin),
        round(Glyph::Plus, Message::AddressToggle, skin),
        round(
            match skin.mode {
                Mode::Night => Glyph::Sun,
                Mode::Day => Glyph::Moon,
            },
            Message::FlipSkin,
            skin,
        ),
    ]
    .spacing(8)
    .into()
}

/// Below this width the "HomeLumen" wordmark no longer fits next to the
/// header's controls, so only the mark itself is shown.
const HEADER_BREAKPOINT: f32 = 300.0;

fn header<'a>(skin: Skin, scanning: bool) -> Element<'a, Message> {
    inset(
        responsive(move |available| {
            let wordmark: Element<'_, Message> = if available.width
                < HEADER_BREAKPOINT
            {
                mark(27.0)
            } else {
                row![
                    mark(27.0),
                    label("HomeLumen", typo::BODY, typo::MEDIUM, skin.ink_soft),
                ]
                .spacing(11)
                .align_y(Center)
                .into()
            };

            row![wordmark, space::horizontal(), header_controls(skin, scanning)]
                .align_y(Center)
                .width(Fill)
                .into()
        })
        .height(Length::Shrink),
    )
    .into()
}

fn heading<'a>(
    lights: &'a [LightSnapshot],
    skin: Skin,
    scanning: bool,
) -> Element<'a, Message> {
    let lit = lights.iter().filter(|light| light.state.power).count();

    let summary = match (scanning, lit, lights.len()) {
        (true, _, _) => "Recherche en cours".to_owned(),
        (_, 1, 1) => "Une lumière, allumée".to_owned(),
        (_, 0, 1) => "Une lumière, éteinte".to_owned(),
        (_, 0, _) => "Tout est éteint".to_owned(),
        (_, lit, total) if lit == total => "Tout est allumé".to_owned(),
        (_, 1, total) => format!("1 allumée sur {total}"),
        (_, lit, total) => format!("{lit} allumées sur {total}"),
    };

    column![
        label("Mes lumières", typo::DISPLAY, typo::SEMIBOLD, skin.ink)
            .line_height(typo::SNUG_LEADING),
        label(summary, typo::BODY, typo::REGULAR, skin.ink_soft),
    ]
    .spacing(gap::TIGHT)
    .into()
}

fn grid<'a>(lights: &'a [LightSnapshot], skin: Skin) -> Element<'a, Message> {
    Grid::with_children(lights.iter().map(|light| {
        let device = light.descriptor.id.clone();
        let level = f32::from(light.state.brightness.unwrap_or(100)) / 100.0;

        let mut card = Tile::new(
            &light.descriptor.name,
            reading(light),
            skin,
            Message::Open(device.clone()),
        )
        .light(light.state.power, level, tone::emission(&light.state))
        .online(light.online);

        if light.descriptor.capabilities.power {
            card = card.on_toggle(Message::Toggle(device));
        }

        Element::from(card)
    }))
    .fluid(COLUMN)
    .spacing(20.0)
    .height(Length::Shrink)
    .into()
}

/// The one line of state a tile shows.
fn reading(light: &LightSnapshot) -> String {
    if !light.state.power {
        return "Éteinte".to_owned();
    }

    match light.state.brightness {
        Some(level) => format!("{level} %"),
        None => "Allumée".to_owned(),
    }
}

fn empty<'a>(skin: Skin, scanning: bool) -> Element<'a, Message> {
    let headline = if scanning {
        "Recherche des lumières"
    } else {
        "Aucune lumière pour l'instant"
    };

    center(
        column![
            mark(68.0),
            column![
                label(headline, typo::TITLE, typo::SEMIBOLD, skin.ink),
                label(
                    "HomeLumen interroge votre réseau local. Si la diffusion est bloquée, ajoutez une adresse à la main.",
                    typo::BODY,
                    typo::REGULAR,
                    skin.ink_faint,
                )
                .center()
                .width(Length::Fixed(430.0)),
            ]
            .spacing(gap::SNUG)
            .align_x(Center),
            button(label(
                "Ajouter une adresse",
                typo::BODY,
                typo::MEDIUM,
                skin.ink_over_light,
            ))
            .padding([14.0, 24.0])
            .style(style::solid(skin))
            .on_press(Message::AddressToggle),
        ]
        .spacing(gap::GAP)
        .align_x(Center),
    )
    .height(Length::Fixed(470.0))
    .into()
}

/// Below this width the label, field and button no longer fit on one line,
/// so the panel stacks them instead. HomeLumen's window can be shrunk well
/// past this point, so the panel has to keep working there.
const ADDRESS_BREAKPOINT: f32 = 560.0;

fn address_copy<'a>(skin: Skin) -> Element<'a, Message> {
    column![
        label("Ajouter par adresse", typo::LEAD, typo::SEMIBOLD, skin.ink),
        label(
            "L'adresse locale de la lumière sur votre réseau.",
            typo::LABEL,
            typo::REGULAR,
            skin.ink_faint,
        ),
    ]
    .spacing(gap::TIGHT)
    .width(Fill)
    .into()
}

fn address_field<'a>(typed: &'a str, skin: Skin) -> Element<'a, Message> {
    text_input("192.168.1.42", typed)
        .on_input(Message::AddressTyped)
        .on_submit(Message::AddressSubmit)
        .padding([13.0, 16.0])
        .size(typo::BODY)
        .font(typo::REGULAR)
        .width(Fill)
        .style(style::field(skin))
        .into()
}

fn address_submit<'a>(skin: Skin, fill: bool) -> Element<'a, Message> {
    let content =
        label("Ajouter", typo::BODY, typo::MEDIUM, skin.ink_over_light)
            .width(Fill)
            .align_x(Center);

    let mut submit = button(content)
        .padding([13.0, 22.0])
        .style(style::solid(skin))
        .on_press(Message::AddressSubmit);

    if fill {
        submit = submit.width(Fill);
    }

    submit.into()
}

fn address_panel<'a>(typed: &'a str, skin: Skin) -> Element<'a, Message> {
    container(responsive(move |available| {
        if available.width >= ADDRESS_BREAKPOINT {
            row![
                address_copy(skin),
                container(address_field(typed, skin))
                    .width(Length::Fixed(220.0)),
                address_submit(skin, false),
            ]
            .spacing(gap::STEP)
            .align_y(Center)
            .into()
        } else {
            column![
                address_copy(skin),
                address_field(typed, skin),
                address_submit(skin, true),
            ]
            .spacing(gap::STEP)
            .into()
        }
    }))
    .padding(gap::ROOM)
    .width(Fill)
    .style(style::card(skin))
    .into()
}

fn banner<'a>(message: &'a str, skin: Skin) -> Element<'a, Message> {
    container(
        row![
            label(message, typo::BODY, typo::REGULAR, skin.alarm).width(Fill),
            button(label("Fermer", typo::LABEL, typo::MEDIUM, skin.ink_soft))
                .padding([8.0, 14.0])
                .style(style::quiet(skin))
                .on_press(Message::Dismiss),
        ]
        .spacing(gap::SNUG)
        .align_y(Center),
    )
    .padding([gap::SNUG, gap::STEP])
    .width(Fill)
    .style(style::card(skin))
    .into()
}

fn round<'a>(
    kind: Glyph,
    message: Message,
    skin: Skin,
) -> Element<'a, Message> {
    button(glyph(kind, skin.ink_soft, 20.0))
        .width(Length::Fixed(42.0))
        .height(Length::Fixed(42.0))
        .padding(11.0)
        .style(style::quiet(skin))
        .on_press(message)
        .into()
}
