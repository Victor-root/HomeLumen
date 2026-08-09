//! The light bulb at the top of a light's screen.
//!
//! It is the light itself, not just a picture of one: its colour, its reach
//! and its slow breathing all come from the state of the device. The shape
//! reads as a bulb rather than a plain sphere, but nothing about it is
//! static artwork; the glow around it is what says a light is on, so the
//! glass itself stays free of decoration.

use iced::mouse;
use iced::time::{Duration, Instant};
use iced::widget::canvas::{
    self, Canvas, Frame, Geometry, Path, Stroke, Style,
};
use iced::{
    Color, Element, Length, Point, Rectangle, Renderer, Size, Theme, Vector,
};

use crate::design::{Skin, tone};

/// Layers the halo is built from.
const HALO: usize = 34;

/// How much wider the halo swells at the top of a breath.
const SWELL: f32 = 0.02;

/// A light, drawn as a bulb.
pub struct Bulb {
    glow: Color,
    intensity: f32,
    skin: Skin,
}

impl Bulb {
    /// Builds a bulb for a light putting out `intensity` of `glow`.
    pub fn new(glow: Color, intensity: f32, skin: Skin) -> Self {
        Self { glow, intensity: intensity.clamp(0.0, 1.0), skin }
    }
}

/// Kept by the runtime between frames, so the bulb can breathe.
#[derive(Default)]
pub struct State {
    origin: Option<Instant>,
    now: Option<Instant>,
}

impl<Message> canvas::Program<Message> for Bulb {
    type State = State;

    fn update(
        &self,
        state: &mut Self::State,
        event: &canvas::Event,
        _bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Option<canvas::Action<Message>> {
        let canvas::Event::Window(iced::window::Event::RedrawRequested(now)) =
            event
        else {
            return None;
        };

        state.origin.get_or_insert(*now);
        state.now = Some(*now);

        // A lit bulb breathes. A dark one has no reason to cost a frame.
        (self.intensity > 0.0).then(|| {
            canvas::Action::request_redraw_at(*now + Duration::from_millis(33))
        })
    }

    fn draw(
        &self,
        state: &Self::State,
        renderer: &Renderer,
        _theme: &Theme,
        bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<Geometry> {
        let mut frame = Frame::new(renderer, bounds.size());
        let side = bounds.width.min(bounds.height);

        let phase = match (state.origin, state.now) {
            (Some(origin), Some(now)) => (now - origin).as_secs_f32(),
            _ => 0.0,
        };

        let breath = 1.0 + SWELL * (phase * 0.85).sin() * self.intensity;

        // `frame.center()`, not `bounds.center()`: the frame's own coordinate
        // space starts at (0, 0) regardless of where the canvas sits in the
        // window, so drawing at the widget's window-relative centre would
        // land outside the frame and clip away entirely. The glass sits a
        // touch above that centre, leaving the base its own room below
        // rather than splitting the difference with it.
        let center = frame.center();
        let glass = Point::new(center.x, center.y - side * 0.07);
        let radius = side * 0.30;

        // The frame clips whatever leaves it, and the glass sitting above
        // centre makes the top edge the near one, so that is what the halo
        // has to answer to. Dividing by the swell leaves the breath its room
        // as well: without it the widest moment of every breath would meet
        // the edge and flatten into a hard line across the glow.
        let reach = (glass.y - 1.0) / (1.0 + SWELL) * breath;

        for layer in 0..HALO {
            let step = layer as f32 / (HALO - 1) as f32;
            let halo_radius = reach - (reach - radius) * step;
            let opacity = (0.020 + 0.042 * step.powf(1.8)) * self.intensity;

            frame.fill(
                &Path::circle(glass, halo_radius),
                tone::fade(self.glow, opacity),
            );
        }

        self.draw_base(&mut frame, glass, radius);
        self.draw_glass(&mut frame, glass, radius);

        vec![frame.into_geometry()]
    }
}

impl Bulb {
    /// The socket: drawn first and mostly tucked behind the glass, so only
    /// the sliver below it reads as a base to screw the bulb into.
    fn draw_base(&self, frame: &mut Frame, glass: Point, radius: f32) {
        let width = radius * 1.15;
        let height = radius * 0.85;
        let top = glass.y + radius * 0.62;
        let tones = tones(self.skin, self.glow, self.intensity);

        frame.fill(
            &Path::rounded_rectangle(
                Point::new(glass.x - width / 2.0, top),
                Size::new(width, height),
                (height * 0.32).into(),
            ),
            tones.metal,
        );
    }

    /// The glass itself: the same warm, gradient-lit sphere the orb always
    /// was, just no longer the only shape on screen.
    fn draw_glass(&self, frame: &mut Frame, glass: Point, radius: f32) {
        let tones = tones(self.skin, self.glow, self.intensity);

        frame.fill(
            &Path::circle(glass, radius),
            canvas::gradient::Linear::new(
                Point::new(glass.x, glass.y - radius),
                Point::new(glass.x, glass.y + radius),
            )
            .add_stop(0.0, tones.top)
            .add_stop(1.0, tones.bottom),
        );

        frame.stroke(
            &Path::circle(glass, radius),
            Stroke {
                style: Style::Solid(tones.stroke),
                width: 1.0,
                ..Stroke::default()
            },
        );

        // A single specular highlight is what turns a disc into a sphere.
        frame.fill(
            &Path::circle(
                glass + Vector::new(-radius * 0.33, -radius * 0.38),
                radius * 0.26,
            ),
            tones.highlight,
        );
    }
}

/// The colours a bulb is drawn from, given how lit it is.
///
/// Shared with the home screen's tile, which draws the same silhouette with
/// the plain quad renderer instead of a canvas frame: the geometry differs
/// with the drawing API, but the colour maths, contrast fixes included,
/// stays in exactly one place either way.
pub struct Tones {
    /// The socket screwing the glass into its base.
    pub metal: Color,
    /// Top stop of the glass's gradient.
    pub top: Color,
    /// Bottom stop of the glass's gradient.
    pub bottom: Color,
    /// The hairline around the glass.
    pub stroke: Color,
    /// The specular dot that turns the disc into a sphere.
    pub highlight: Color,
}

/// Computes [`Tones`] for a bulb putting out `intensity` of `glow`.
pub fn tones(skin: Skin, glow: Color, intensity: f32) -> Tones {
    let intensity = intensity.clamp(0.0, 1.0);

    // `edge` toward `ink_faint`, not `surface_lift`/`surface` toward `edge`:
    // in the day skin those pairs are both near white, so a resting tone
    // built from them reads as no bulb at all against the page. Leaning on
    // ink_faint keeps the glass and the socket visible as their own shapes
    // in both skins, the same fix the switch's track needed for the same
    // reason.
    let metal = tone::mix(
        tone::mix(skin.edge, skin.ink_faint, 0.5),
        glow,
        intensity * 0.12,
    );

    let base = tone::mix(skin.edge, skin.ink_faint, 0.3);
    let face = tone::mix(base, glow, intensity);

    Tones {
        metal,
        top: tone::mix(face, Color::WHITE, 0.20 + 0.18 * intensity),
        bottom: tone::mix(face, skin.canvas, 0.18),
        stroke: tone::mix(
            tone::mix(skin.edge, skin.ink_faint, 0.6),
            tone::fade(Color::WHITE, 0.35),
            intensity,
        ),
        highlight: tone::fade(Color::WHITE, 0.10 + 0.14 * intensity),
    }
}

/// Places a bulb of the given side length.
pub fn bulb<'a, Message>(light: Bulb, side: f32) -> Element<'a, Message>
where
    Message: 'a,
{
    Canvas::new(light)
        .width(Length::Fixed(side))
        .height(Length::Fixed(side))
        .into()
}
