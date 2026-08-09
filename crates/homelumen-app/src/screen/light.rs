//! One light, in full.
//!
//! Which controls appear is decided by the capabilities the device announced,
//! and by nothing else. A bulb without colour simply has no colour panel.

use homelumen_core::{Capabilities, Color as LightColor};
use homelumen_engine::LightSnapshot;
use iced::widget::{
    Column, button, center, column, container, responsive, row, scrollable,
    space,
};
use iced::{Center, Element, Fill, Length};

use crate::app::Message;
use crate::design::{Skin, space as gap, tone, typo};
use crate::style::{self, label};
use crate::widget::glyph::{Glyph, glyph};
use crate::widget::level::Level;
use crate::widget::orb::{Orb, orb};
use crate::widget::pages::Pages;
use crate::widget::power::Power;
use crate::widget::segmented::Segmented;
use crate::widget::warmth::Warmth;
use crate::widget::wheel::{Wheel, wheel};

/// Side of the hero orb.
const ORB: f32 = 204.0;

/// Width of the column holding the advanced panels, when there is room to
/// sit it next to the controls.
const ASIDE: f32 = 404.0;

/// Side of the colour disc.
const DISC: f32 = 348.0;

/// Height reserved for the advanced panels.
const PANEL: f32 = 408.0;

/// Widest the controls get when a light has no advanced panel at all.
const SOLO: f32 = 560.0;

/// Below this width the advanced panels no longer fit next to the controls,
/// so the two stack instead of sitting side by side. HomeLumen's window can
/// be shrunk well past this point, so the screen has to keep working there.
const BREAKPOINT: f32 = 760.0;

/// Draws the screen of one light.
///
/// The body scrolls: HomeLumen's window can be shrunk down to the point
/// where a light with both a colour wheel and a white band no longer fits
/// in one screenful, and nothing here should become unreachable because of
/// it.
pub fn view(
    light: &LightSnapshot,
    panel: usize,
    skin: Skin,
) -> Element<'_, Message> {
    let page = column![
        top(light, skin),
        responsive(move |available| {
            let narrow = available.width < BREAKPOINT;
            let controls = controls(light, skin, narrow);

            let body: Element<'_, Message> = match advanced(light, panel, skin)
            {
                Some(panels) if !narrow => row![
                    container(controls).width(Fill),
                    container(panels).width(Length::Fixed(ASIDE)),
                ]
                .spacing(gap::GAP)
                .into(),
                Some(panels) => column![controls, panels]
                    .spacing(gap::GAP)
                    .width(Fill)
                    .into(),
                None => container(controls)
                    .max_width(SOLO)
                    .width(Fill)
                    .center_x(Fill)
                    .into(),
            };

            scrollable(
                container(body).padding([gap::GAP, gap::MARGIN]).width(Fill),
            )
            .height(Fill)
            .style(style::scroller(skin))
            .into()
        }),
    ]
    .width(Fill)
    .height(Fill);

    container(page)
        .padding([gap::ROOM, 0.0])
        .width(Fill)
        .height(Fill)
        .style(style::page(skin))
        .into()
}

/// The identity of the light and the two controls every light deserves.
///
/// `narrow` stacks the orb above its name instead of beside it, once the
/// window is too tight for the two to sit on the same line.
fn controls(
    light: &LightSnapshot,
    skin: Skin,
    narrow: bool,
) -> Element<'_, Message> {
    let device = light.descriptor.id.clone();
    let capabilities = &light.descriptor.capabilities;
    let emission = tone::emission(&light.state);

    let identity =
        orb(Orb::new(emission, tone::intensity(&light.state), skin), ORB);
    let name =
        label(&light.descriptor.name, typo::DISPLAY, typo::SEMIBOLD, skin.ink)
            .line_height(typo::SNUG_LEADING);
    let model = label(
        format!("{} · {}", light.descriptor.model, light.descriptor.vendor),
        typo::LABEL,
        typo::REGULAR,
        skin.ink_faint,
    );

    let hero: Element<'_, Message> = if narrow {
        column![
            identity,
            column![
                name.align_x(Center).width(Fill),
                model.align_x(Center).width(Fill)
            ]
            .spacing(gap::TIGHT)
            .width(Fill),
        ]
        .spacing(gap::STEP)
        .align_x(Center)
        .width(Fill)
        .into()
    } else {
        row![identity, column![name, model].spacing(gap::TIGHT).width(Fill)]
            .spacing(gap::STEP)
            .align_y(Center)
            .into()
    };

    let mut stack = Column::new().spacing(gap::GAP).width(Fill);
    stack = stack.push(hero);

    let mut knobs = Column::new().spacing(gap::STEP).width(Fill);

    if capabilities.power {
        knobs = knobs.push(Power::new(
            light.state.power,
            emission,
            skin,
            Message::Toggle(device.clone()),
        ));
    }

    if let Some(range) = capabilities.brightness {
        let level = range.fraction(light.state.brightness.unwrap_or(range.max));

        knobs = knobs.push(
            Level::new(level, emission, skin, move |fraction| {
                Message::Dim(device.clone(), fraction)
            })
            .enabled(light.state.power),
        );
    }

    stack.push(knobs).into()
}

fn top<'a>(light: &LightSnapshot, skin: Skin) -> Element<'a, Message> {
    let back = button(glyph(Glyph::Back, skin.ink_soft, 20.0))
        .width(Length::Fixed(42.0))
        .height(Length::Fixed(42.0))
        .padding(11.0)
        .style(style::quiet(skin))
        .on_press(Message::Back);

    let route = match (&light.transport, light.online) {
        (Some(transport), true) => format!("{transport} · {}", light.address),
        _ => "Hors ligne".to_owned(),
    };

    let ink = if light.online { skin.ink_soft } else { skin.alarm };

    container(
        row![
            back,
            space::horizontal(),
            container(label(route, typo::MICRO, typo::MEDIUM, ink))
                .padding([7.0, 14.0])
                .style(style::chip(skin)),
        ]
        .align_y(Center)
        .width(Fill),
    )
    .padding([0.0, gap::MARGIN])
    .into()
}

/// Builds the horizontal panels, if the light has anything to put in them.
fn advanced(
    light: &LightSnapshot,
    panel: usize,
    skin: Skin,
) -> Option<Element<'_, Message>> {
    let Capabilities { color, color_temperature, .. } =
        light.descriptor.capabilities;

    let device = light.descriptor.id.clone();
    let mut labels = Vec::new();
    let mut panels: Vec<Element<'_, Message>> = Vec::new();

    if color {
        let (hue, saturation) = match light.state.color {
            Some(LightColor::Tint { hue, saturation }) => {
                (f32::from(hue), f32::from(saturation) / 100.0)
            }
            _ => (38.0, 0.0),
        };

        let device = device.clone();

        labels.push("Couleur");
        panels.push(
            center(wheel(
                Wheel::new(hue, saturation, skin, move |hue, saturation| {
                    Message::Tint(device.clone(), hue, saturation)
                }),
                DISC,
            ))
            .into(),
        );
    }

    if let Some(range) = color_temperature {
        let kelvin = match light.state.color {
            Some(LightColor::White { kelvin }) => range.clamp(kelvin),
            _ => range.clamp(2700),
        };

        labels.push("Blanc");
        panels.push(
            column![
                label(
                    format!("{kelvin} K"),
                    typo::DISPLAY,
                    typo::SEMIBOLD,
                    skin.ink
                ),
                Warmth::new(
                    range.fraction(kelvin),
                    range.min_kelvin,
                    range.max_kelvin,
                    skin,
                    move |fraction| Message::White(device.clone(), fraction),
                ),
                row![
                    label("Chaud", typo::MICRO, typo::MEDIUM, skin.ink_faint),
                    space::horizontal(),
                    label("Froid", typo::MICRO, typo::MEDIUM, skin.ink_faint),
                ]
                .width(Fill),
            ]
            .spacing(gap::ROOM)
            .align_x(Center)
            .width(Fill)
            .into(),
        );
    }

    if panels.is_empty() {
        return None;
    }

    // Each panel is centred inside the strip, so a short one does not leave the
    // taller one's height as a hole underneath it.
    let panels = panels.into_iter().map(|panel| center(panel).into()).collect();

    let active = panel.min(labels.len() - 1);
    let strip = Pages::new(panels, active, PANEL);

    if labels.len() == 1 {
        return Some(strip.into());
    }

    Some(
        column![Segmented::new(labels, active, skin, Message::Panel), strip]
            .spacing(gap::ROOM)
            .align_x(Center)
            .width(Fill)
            .into(),
    )
}
