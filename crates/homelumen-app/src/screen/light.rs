//! One light, in full.
//!
//! Which controls appear is decided by the capabilities the device announced,
//! and by nothing else. A bulb without colour simply has no colour panel.
//!
//! Everything on this screen also answers to how much room the window
//! actually gives it. HomeLumen's window can be shrunk a long way, and a
//! light with both a colour field and a white band has a lot to show; rather
//! than let the screen overflow into a scrollbar, the orb, the field, the
//! panels and the spacing between them all shrink together, continuously,
//! down to a floor that still reads and still works. The scrollable wrapped
//! around it all is a safety net for a capability combination this budget
//! did not anticipate, not the everyday way of reaching a control.

use homelumen_core::{Capabilities, Color as LightColor};
use homelumen_engine::LightSnapshot;
use iced::widget::{
    Column, button, center, column, container, responsive, row, scrollable,
    space,
};
use iced::{Center, Element, Fill, Length, Size};

use crate::app::Message;
use crate::design::{Skin, tone, typo};
use crate::style::{self, label};
use crate::widget::field::{Field, field};
use crate::widget::glyph::{Glyph, glyph};
use crate::widget::level::Level;
use crate::widget::orb::{Orb, orb};
use crate::widget::pages::Pages;
use crate::widget::power::Power;
use crate::widget::segmented::Segmented;
use crate::widget::warmth::Warmth;

/// Below this width the advanced panels no longer fit next to the controls,
/// so the two stack instead of sitting side by side.
const BREAKPOINT: f32 = 760.0;

/// Widest the controls get when a light has no advanced panel at all.
const SOLO: f32 = 560.0;

/// Height of the back button and its row. Small and fixed: it never needs
/// to give up room, so it never competes for any.
const TOP_BAR: f32 = 42.0;

/// Draws the screen of one light.
pub fn view(
    light: &LightSnapshot,
    panel: usize,
    skin: Skin,
) -> Element<'_, Message> {
    let page = column![
        top(light, skin),
        responsive(move |available| {
            let narrow = available.width < BREAKPOINT;
            let capabilities = &light.descriptor.capabilities;
            let density = Density::solve(available, narrow, capabilities);

            let controls = controls(light, skin, &density);

            let body: Element<'_, Message> =
                match advanced(light, panel, skin, &density) {
                    Some(panels) if !narrow => row![
                        container(controls).width(Fill),
                        container(panels).width(Length::Fixed(density.aside)),
                    ]
                    .spacing(density.gap)
                    .into(),
                    Some(panels) => column![controls, panels]
                        .spacing(density.gap)
                        .width(Fill)
                        .into(),
                    None => container(controls)
                        .max_width(SOLO)
                        .width(Fill)
                        .center_x(Fill)
                        .into(),
                };

            scrollable(
                container(body)
                    .padding([density.gap, density.margin])
                    .width(Fill),
            )
            .height(Fill)
            .style(style::scroller(skin))
            .into()
        }),
    ]
    .width(Fill)
    .height(Fill);

    container(page)
        .padding([gap::ROOM_REF, 0.0])
        .width(Fill)
        .height(Fill)
        .style(style::page(skin))
        .into()
}

/// The identity of the light and the two controls every light deserves.
fn controls<'a>(
    light: &'a LightSnapshot,
    skin: Skin,
    density: &Density,
) -> Element<'a, Message> {
    let device = light.descriptor.id.clone();
    let capabilities = &light.descriptor.capabilities;
    let emission = tone::emission(&light.state);

    let identity = orb(
        Orb::new(emission, tone::intensity(&light.state), skin),
        density.orb,
    );

    let name = label(
        &light.descriptor.name,
        density.name_text,
        typo::SEMIBOLD,
        skin.ink,
    )
    .line_height(typo::SNUG_LEADING);

    let model = label(
        format!("{} · {}", light.descriptor.model, light.descriptor.vendor),
        density.model_text,
        typo::REGULAR,
        skin.ink_faint,
    );

    let hero = row![
        identity,
        column![name, model].spacing(density.tight).width(Fill),
    ]
    .spacing(density.step)
    .align_y(Center);

    let mut stack = Column::new().spacing(density.gap).width(Fill);
    stack = stack.push(hero);

    let mut knobs = Column::new().spacing(density.step).width(Fill);

    if capabilities.power {
        knobs = knobs.push(
            Power::new(
                light.state.power,
                emission,
                skin,
                Message::Toggle(device.clone()),
            )
            .height(density.power),
        );
    }

    if let Some(range) = capabilities.brightness {
        let level = range.fraction(light.state.brightness.unwrap_or(range.max));

        knobs = knobs.push(
            Level::new(level, emission, skin, move |fraction| {
                Message::Dim(device.clone(), fraction)
            })
            .enabled(light.state.power)
            .height(density.level),
        );
    }

    stack.push(knobs).into()
}

fn top<'a>(light: &LightSnapshot, skin: Skin) -> Element<'a, Message> {
    let back = button(glyph(Glyph::Back, skin.ink_soft, 20.0))
        .width(Length::Fixed(TOP_BAR))
        .height(Length::Fixed(TOP_BAR))
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
    .padding([0.0, gap::MARGIN_REF])
    .into()
}

/// Builds the horizontal panels, if the light has anything to put in them.
fn advanced<'a>(
    light: &'a LightSnapshot,
    panel: usize,
    skin: Skin,
    density: &Density,
) -> Option<Element<'a, Message>> {
    let Capabilities { color, color_temperature, .. } =
        light.descriptor.capabilities;

    let device = light.descriptor.id.clone();
    let mut labels = Vec::new();
    let mut panels: Vec<Element<'a, Message>> = Vec::new();

    if color {
        let (hue, saturation) = match light.state.color {
            Some(LightColor::Tint { hue, saturation }) => {
                (f32::from(hue), f32::from(saturation) / 100.0)
            }
            _ => (38.0, 0.0),
        };

        let device = device.clone();

        labels.push("Couleur");
        panels.push(field(
            Field::new(hue, saturation, skin, move |hue, saturation| {
                Message::Tint(device.clone(), hue, saturation)
            }),
            density.swatch,
        ));
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
                    density.name_text,
                    typo::SEMIBOLD,
                    skin.ink
                ),
                Warmth::new(
                    range.fraction(kelvin),
                    range.min_kelvin,
                    range.max_kelvin,
                    skin,
                    move |fraction| Message::White(device.clone(), fraction),
                )
                .height(density.warmth),
                row![
                    label("Chaud", typo::MICRO, typo::MEDIUM, skin.ink_faint),
                    space::horizontal(),
                    label("Froid", typo::MICRO, typo::MEDIUM, skin.ink_faint),
                ]
                .width(Fill),
            ]
            .spacing(density.room)
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
    let strip = Pages::new(panels, active, density.panel);

    if labels.len() == 1 {
        return Some(strip.into());
    }

    Some(
        column![
            Segmented::new(labels, active, skin, Message::Panel)
                .height(density.segment),
            strip,
        ]
        .spacing(density.room)
        .align_x(Center)
        .width(Fill)
        .into(),
    )
}

/// The reference (most spacious) and floor (most compact) size of everything
/// on this screen that is allowed to shrink.
mod gap {
    pub const ORB_REF: f32 = 204.0;
    pub const ORB_FLOOR: f32 = 60.0;

    /// Height of the colour field: it fills whatever width it is given, so
    /// only its height answers to density.
    pub const SWATCH_REF: f32 = 220.0;
    pub const SWATCH_FLOOR: f32 = 120.0;

    /// Width of the advanced panels' column in the wide layout.
    pub const ASIDE_REF: f32 = 420.0;
    pub const ASIDE_FLOOR: f32 = 220.0;

    /// The colour field cannot usefully get narrower than this; below it,
    /// the narrow layout stacks instead of squeezing the field further.
    pub const FIELD_MIN_WIDTH: f32 = 200.0;

    pub const PANEL_REF: f32 = 280.0;
    pub const PANEL_FLOOR: f32 = 140.0;

    pub const POWER_REF: f32 = crate::widget::power::HEIGHT;
    pub const POWER_FLOOR: f32 = 52.0;

    pub const LEVEL_REF: f32 = crate::widget::level::HEIGHT;
    pub const LEVEL_FLOOR: f32 = 56.0;

    pub const WARMTH_REF: f32 = crate::widget::warmth::HEIGHT;
    pub const WARMTH_FLOOR: f32 = 56.0;

    pub const SEGMENT_REF: f32 = crate::widget::segmented::HEIGHT;
    pub const SEGMENT_FLOOR: f32 = 36.0;

    pub const GAP_REF: f32 = crate::design::space::GAP;
    pub const GAP_FLOOR: f32 = 10.0;

    pub const ROOM_REF: f32 = crate::design::space::ROOM;
    pub const ROOM_FLOOR: f32 = 10.0;

    pub const STEP_REF: f32 = crate::design::space::STEP;
    pub const STEP_FLOOR: f32 = 6.0;

    pub const TIGHT_REF: f32 = crate::design::space::TIGHT;
    pub const TIGHT_FLOOR: f32 = 4.0;

    pub const MARGIN_REF: f32 = crate::design::space::MARGIN;
    pub const MARGIN_FLOOR: f32 = 16.0;

    pub const NAME_REF: f32 = crate::design::typo::DISPLAY;
    pub const NAME_FLOOR: f32 = 19.0;

    pub const MODEL_REF: f32 = crate::design::typo::LABEL;
    pub const MODEL_FLOOR: f32 = 11.0;
}

/// How large everything on this screen gets to be, all at once.
///
/// Every field here is the same fraction of the way from its floor to its
/// reference size, so the whole screen breathes together rather than some
/// controls staying generous while others go cramped.
struct Density {
    orb: f32,
    swatch: f32,
    aside: f32,
    panel: f32,
    power: f32,
    level: f32,
    warmth: f32,
    segment: f32,
    gap: f32,
    room: f32,
    step: f32,
    tight: f32,
    margin: f32,
    name_text: f32,
    model_text: f32,
}

impl Density {
    fn at(t: f32) -> Self {
        let t = t.clamp(0.0, 1.0);
        let lerp = |floor: f32, reference: f32| floor + (reference - floor) * t;

        Self {
            orb: lerp(gap::ORB_FLOOR, gap::ORB_REF),
            swatch: lerp(gap::SWATCH_FLOOR, gap::SWATCH_REF),
            aside: lerp(gap::ASIDE_FLOOR, gap::ASIDE_REF),
            panel: lerp(gap::PANEL_FLOOR, gap::PANEL_REF),
            power: lerp(gap::POWER_FLOOR, gap::POWER_REF),
            level: lerp(gap::LEVEL_FLOOR, gap::LEVEL_REF),
            warmth: lerp(gap::WARMTH_FLOOR, gap::WARMTH_REF),
            segment: lerp(gap::SEGMENT_FLOOR, gap::SEGMENT_REF),
            gap: lerp(gap::GAP_FLOOR, gap::GAP_REF),
            room: lerp(gap::ROOM_FLOOR, gap::ROOM_REF),
            step: lerp(gap::STEP_FLOOR, gap::STEP_REF),
            tight: lerp(gap::TIGHT_FLOOR, gap::TIGHT_REF),
            margin: lerp(gap::MARGIN_FLOOR, gap::MARGIN_REF),
            name_text: lerp(gap::NAME_FLOOR, gap::NAME_REF),
            model_text: lerp(gap::MODEL_FLOOR, gap::MODEL_REF),
        }
    }

    /// Solves for the largest density that still lets everything this light
    /// needs fit inside `available`, laid out the way `narrow` says.
    ///
    /// Every scalable size is the same fraction `t` between its floor and
    /// its reference, so the total space the screen needs is an affine
    /// function of `t`: evaluating it at the two ends and inverting is
    /// exact, not an approximation.
    fn solve(
        available: Size,
        narrow: bool,
        capabilities: &Capabilities,
    ) -> Self {
        let floor = Self::at(0.0);
        let spacious = Self::at(1.0);

        let height_t = solve_axis(
            available.height,
            floor.needed_height(narrow, capabilities),
            spacious.needed_height(narrow, capabilities),
        );

        let width_t = if narrow {
            solve_axis(
                available.width,
                floor.needed_width(capabilities),
                spacious.needed_width(capabilities),
            )
        } else {
            1.0
        };

        Self::at(height_t.min(width_t))
    }

    fn hero_height(&self) -> f32 {
        // 1.15 mirrors `typo::SNUG_LEADING` on the name; 1.3 is iced's
        // default relative line height, left as-is on the model line.
        let text = self.name_text * 1.15 + self.tight + self.model_text * 1.3;

        self.orb.max(text)
    }

    fn controls_height(&self, capabilities: &Capabilities) -> f32 {
        let mut height = self.hero_height();

        if capabilities.power {
            height += self.gap + self.power;
        }

        if capabilities.brightness.is_some() {
            height += self.step + self.level;
        }

        height
    }

    fn panels_height(&self, capabilities: &Capabilities) -> f32 {
        match (capabilities.color, capabilities.color_temperature.is_some()) {
            (true, true) => self.segment + self.room + self.panel,
            (true, false) | (false, true) => self.panel,
            (false, false) => 0.0,
        }
    }

    /// Total height the body would need inside the space `responsive` hands
    /// it, laid out the way `narrow` says. `top()` and the page's own
    /// padding sit outside that space already, so only the padding the body
    /// adds for itself counts here.
    fn needed_height(&self, narrow: bool, capabilities: &Capabilities) -> f32 {
        let controls = self.controls_height(capabilities);
        let panels = self.panels_height(capabilities);

        let content = if panels <= 0.0 {
            controls
        } else if narrow {
            controls + self.gap + panels
        } else {
            controls.max(panels)
        };

        2.0 * self.gap + content
    }

    /// Total width the narrow (stacked) layout would need: only the colour
    /// field forces a minimum here, everything else is happy to fill
    /// whatever is left.
    fn needed_width(&self, capabilities: &Capabilities) -> f32 {
        let field = if capabilities.color { gap::FIELD_MIN_WIDTH } else { 0.0 };
        2.0 * self.margin + field
    }
}

/// The largest `t` in `0.0..=1.0` such that `lerp(floor, reference, t) <=
/// available`, where `floor` and `reference` are `f(0.0)` and `f(1.0)` of
/// some affine `f`.
fn solve_axis(available: f32, floor: f32, reference: f32) -> f32 {
    if reference <= floor {
        return 1.0;
    }

    ((available - floor) / (reference - floor)).clamp(0.0, 1.0)
}
