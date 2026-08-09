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
use iced::{Center, Element, Fill, Length, Padding, Size};

use crate::app::Message;
use crate::design::{Skin, tone, typo};
use crate::style::{self, label};
use crate::widget::bulb::{Bulb, bulb};
use crate::widget::field::{Field, field};
use crate::widget::glyph::{Glyph, glyph};
use crate::widget::level::Level;
use crate::widget::pages::Pages;
use crate::widget::power::Power;
use crate::widget::segmented::Segmented;
use crate::widget::warmth::Warmth;

/// Below this width the advanced panels no longer fit next to the controls,
/// so the two stack instead of sitting side by side.
const BREAKPOINT: f32 = 760.0;

/// Widest the controls get when a light has no advanced panel at all.
const SOLO: f32 = 560.0;

/// Side of the back button. Small and fixed, and its own row above the
/// hero rather than sharing it: what the arrow needs to stay tappable, not
/// what the bulb and the name would need to give up to make room for it.
/// Deliberately slight, so it reads as a way out rather than a control
/// competing with the bulb for attention.
const BACK: f32 = 26.0;

/// Draws the screen of one light.
pub fn view(
    light: &LightSnapshot,
    panel: usize,
    skin: Skin,
) -> Element<'_, Message> {
    let page = responsive(move |available| {
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

        // One vertical padding, not two: the body's own is already what
        // keeps the page off the top and bottom of the window, and a second
        // one wrapped around the scrollable only ever added to it. Top and
        // bottom differ on purpose: the back button above the hero is
        // small and wants to sit close to the edge, while the bottom still
        // answers to `gap`, the same room every other section gets.
        scrollable(
            container(body)
                .padding(Padding {
                    top: density.crown,
                    right: density.margin,
                    bottom: density.gap,
                    left: density.margin,
                })
                .width(Fill),
        )
        .height(Fill)
        .style(style::scroller(skin))
        .into()
    });

    container(page).width(Fill).height(Fill).style(style::page(skin)).into()
}

/// The identity of the light and the two controls every light deserves.
///
/// The back button gets a slim row entirely to itself, above the hero,
/// rather than riding along inside it: small and out of the way, so the
/// hero row underneath answers only to the bulb and the name instead of
/// splitting its width three ways.
fn controls<'a>(
    light: &'a LightSnapshot,
    skin: Skin,
    density: &Density,
) -> Element<'a, Message> {
    let device = light.descriptor.id.clone();
    let capabilities = &light.descriptor.capabilities;
    let emission = tone::emission(&light.state);

    let back = button(glyph(Glyph::Back, skin.ink_soft, 13.0))
        .width(Length::Fixed(BACK))
        .height(Length::Fixed(BACK))
        .padding(6.5)
        .style(style::ghost(skin))
        .on_press(Message::Back);

    let identity = bulb(
        Bulb::new(emission, tone::intensity(&light.state), skin),
        density.orb,
    );

    let name = label(
        &light.descriptor.name,
        density.name_text,
        typo::SEMIBOLD,
        skin.ink,
    )
    .line_height(typo::SNUG_LEADING);

    // Where the model line ends: the address, or why there isn't one. No
    // separate chip for it any more, so a light's whole identity is one
    // read instead of two things to glance between.
    let status = if light.online {
        light.address.clone()
    } else {
        "Hors ligne".to_owned()
    };

    let model = label(
        format!(
            "{} · {} · {status}",
            light.descriptor.model, light.descriptor.vendor
        ),
        density.model_text,
        typo::REGULAR,
        if light.online { skin.ink_faint } else { skin.alarm },
    );

    let hero = row![
        identity,
        column![name, model].spacing(density.tight).width(Fill),
    ]
    .spacing(density.step)
    .align_y(Center);

    let mut stack = Column::new().spacing(density.gap).width(Fill);
    stack = stack.push(column![back, hero].spacing(density.tight).width(Fill));

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
        let reading = light.state.brightness.unwrap_or(range.max);
        let level = range.fraction(reading);

        knobs = knobs.push(
            Level::new(level, reading, emission, skin, move |fraction| {
                Message::Dim(device.clone(), fraction)
            })
            .enabled(light.state.power)
            .height(density.level),
        );
    }

    stack.push(knobs).into()
}

/// Builds the horizontal panels, if the light has anything to put in them.
fn advanced<'a>(
    light: &'a LightSnapshot,
    panel: usize,
    skin: Skin,
    density: &Density,
) -> Option<Element<'a, Message>> {
    let capabilities = &light.descriptor.capabilities;
    let Capabilities { color, color_temperature, .. } = *capabilities;

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
    let strip = Pages::new(panels, active, density.panel_height(capabilities));

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
    pub const ORB_REF: f32 = 160.0;
    pub const ORB_FLOOR: f32 = 56.0;

    /// Height of the colour field: it fills whatever width it is given, so
    /// only its height answers to density.
    pub const SWATCH_REF: f32 = 380.0;
    pub const SWATCH_FLOOR: f32 = 120.0;

    /// Width of the advanced panels' column in the wide layout.
    pub const ASIDE_REF: f32 = 420.0;
    pub const ASIDE_FLOOR: f32 = 220.0;

    /// The colour field cannot usefully get narrower than this; below it,
    /// the narrow layout stacks instead of squeezing the field further.
    pub const FIELD_MIN_WIDTH: f32 = 200.0;

    /// Least width the name and model column can read in without wrapping,
    /// in the hero row. Not density-scaled: it is a floor on the text
    /// itself, not on the room around it. Sized for the model line, the
    /// widest thing there since it carries the address too.
    pub const HERO_TEXT_MIN: f32 = 230.0;

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

    /// Top padding of the whole screen, apart from `GAP` (the bottom, and
    /// the space between sections): the top only has to clear a small,
    /// discreet back button, not a block of controls, so it earns a much
    /// shorter reach before the header starts.
    pub const CROWN_REF: f32 = 14.0;
    pub const CROWN_FLOOR: f32 = 6.0;

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
    power: f32,
    level: f32,
    warmth: f32,
    segment: f32,
    gap: f32,
    crown: f32,
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
            power: lerp(gap::POWER_FLOOR, gap::POWER_REF),
            level: lerp(gap::LEVEL_FLOOR, gap::LEVEL_REF),
            warmth: lerp(gap::WARMTH_FLOOR, gap::WARMTH_REF),
            segment: lerp(gap::SEGMENT_FLOOR, gap::SEGMENT_REF),
            gap: lerp(gap::GAP_FLOOR, gap::GAP_REF),
            crown: lerp(gap::CROWN_FLOOR, gap::CROWN_REF),
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
    /// exact, not an approximation. The one exception is the tallest-panel
    /// term, which is a maximum of affine pieces; a maximum bends only
    /// upward, so reading it off the line between its ends can ask for a
    /// little more room than it truly needs, and never for less.
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

        let width_t = solve_axis(
            available.width,
            floor.needed_width(narrow, capabilities),
            spacious.needed_width(narrow, capabilities),
        );

        Self::at(height_t.min(width_t))
    }

    fn hero_height(&self) -> f32 {
        // 1.15 mirrors `typo::SNUG_LEADING` on the name; 1.3 is iced's
        // default relative line height, left as-is on the model line.
        let text = self.name_text * 1.15 + self.tight + self.model_text * 1.3;

        self.orb.max(text)
    }

    fn controls_height(&self, capabilities: &Capabilities) -> f32 {
        // The back button's own slim row, then the hero: `BACK` is fixed
        // rather than density-scaled, matching the button itself.
        let mut height = BACK + self.tight + self.hero_height();

        if capabilities.power {
            height += self.gap + self.power;
        }

        if capabilities.brightness.is_some() {
            height += self.step + self.level;
        }

        height
    }

    /// Height of the tallest advanced panel, which is exactly what the strip
    /// holding them all has to be. A strip any taller than its tallest page
    /// is a band of dead space above and below whatever is on screen, and it
    /// stays there however hard the rest of the layout tightens.
    fn panel_height(&self, capabilities: &Capabilities) -> f32 {
        let field = capabilities.color.then_some(self.swatch);

        // The white panel is a reading, the band itself and the two end
        // labels, a room apart. 1.3 is iced's default relative line height,
        // which neither of those labels overrides.
        let white = capabilities.color_temperature.is_some().then_some(
            self.name_text * 1.3
                + self.room
                + self.warmth
                + self.room
                + typo::MICRO * 1.3,
        );

        field.into_iter().chain(white).fold(0.0, f32::max)
    }

    fn panels_height(&self, capabilities: &Capabilities) -> f32 {
        let panel = self.panel_height(capabilities);

        if panel <= 0.0 {
            return 0.0;
        }

        if capabilities.color && capabilities.color_temperature.is_some() {
            self.segment + self.room + panel
        } else {
            panel
        }
    }

    /// Total height `responsive` needs to have been given for the page to
    /// fit, laid out the way `narrow` says. The page's own top and bottom
    /// breathing room is counted in and answers to density like everything
    /// else: a compact window earns that room back rather than holding a
    /// fixed margin no matter how little space is left. Top and bottom are
    /// not the same amount: see `crown`.
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

        self.crown + self.gap + content
    }

    /// Total width the layout would need, stacked or side by side.
    ///
    /// The back button sits above the hero now, not inside it, so only the
    /// bulb and the name's own floor answer for the hero's width. Narrow:
    /// that competes with the colour field's minimum, whichever asks for
    /// more decides, since the two are stacked rather than side by side.
    /// Wide: the aside column's width is counted against the hero too, or a
    /// window just past the breakpoint can ask for more than it has.
    fn needed_width(&self, narrow: bool, capabilities: &Capabilities) -> f32 {
        let hero = self.orb + self.step + gap::HERO_TEXT_MIN;

        if narrow {
            let field =
                if capabilities.color { gap::FIELD_MIN_WIDTH } else { 0.0 };
            return 2.0 * self.margin + field.max(hero);
        }

        let has_aside =
            capabilities.color || capabilities.color_temperature.is_some();
        let aside = if has_aside { self.gap + self.aside } else { 0.0 };

        2.0 * self.margin + hero + aside
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
