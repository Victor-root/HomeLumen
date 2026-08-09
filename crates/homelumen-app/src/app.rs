//! What HomeLumen holds on to, and what it does when something happens.

use std::net::IpAddr;

use homelumen_core::{Color as LightColor, Command, DeviceId};
use homelumen_engine::{self as engine, Handle, LightSnapshot, Request};
use iced::{Element, Size, Subscription, Task};

use crate::design::{Mode, Skin};
use crate::screen;

/// The whole state of the interface.
pub struct App {
    engine: Option<Handle>,
    lights: Vec<LightSnapshot>,
    focus: Option<DeviceId>,
    panel: usize,
    mode: Mode,
    scanning: bool,
    notice: Option<String>,
    address: Option<String>,
    window_size: Size,
}

impl Default for App {
    fn default() -> Self {
        Self {
            engine: None,
            lights: Vec::new(),
            focus: None,
            panel: 0,
            mode: Mode::Night,
            scanning: false,
            notice: None,
            address: None,
            window_size: Size::ZERO,
        }
    }
}

/// Everything that can happen.
#[derive(Debug, Clone)]
pub enum Message {
    /// The engine has something to say.
    Engine(engine::Event),
    /// Open a light.
    Open(DeviceId),
    /// Go back to the lights.
    Back,
    /// Switch a light.
    Toggle(DeviceId),
    /// Set the brightness of a light, as `0.0..=1.0`.
    Dim(DeviceId, f32),
    /// Set the colour of a light, as a hue in degrees and a saturation.
    Tint(DeviceId, f32, f32),
    /// Set the white of a light, as `0.0..=1.0` of its range.
    White(DeviceId, f32),
    /// Show another panel of the open light.
    Panel(usize),
    /// Swap the skin.
    FlipSkin,
    /// Look for lights again.
    Sweep,
    /// Open or close the address field.
    AddressToggle,
    /// The address being typed changed.
    AddressTyped(String),
    /// Reach the typed address.
    AddressSubmit,
    /// Put the last message away.
    Dismiss,
    /// The window changed size.
    WindowSized(Size),
}

impl App {
    /// The skin currently in use.
    pub fn skin(&self) -> Skin {
        Skin::of(self.mode)
    }

    /// The theme iced needs for the few things HomeLumen does not paint itself.
    pub fn theme(&self) -> iced::Theme {
        match self.mode {
            Mode::Night => iced::Theme::Dark,
            Mode::Day => iced::Theme::Light,
        }
    }

    /// The window title.
    ///
    /// Carries the window's own logical size, in the same unit
    /// `window::Settings.size` expects: read it here after resizing to a
    /// size worth keeping, and that is exactly the value to hardcode back.
    pub fn title(&self) -> String {
        let name = match self.open_light() {
            Some(light) => format!("{} · HomeLumen", light.descriptor.name),
            None => "HomeLumen".to_owned(),
        };

        let width = self.window_size.width.round() as i32;
        let height = self.window_size.height.round() as i32;
        format!("{name} · {width} × {height}")
    }

    /// The engine runs for as long as the window does; window-size events
    /// keep the title's live readout accurate.
    pub fn subscription(&self) -> Subscription<Message> {
        Subscription::batch([
            Subscription::run(engine::run).map(Message::Engine),
            window_size_events().map(Message::WindowSized),
        ])
    }

    /// Takes in one thing that happened.
    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Engine(engine::Event::Ready(handle)) => {
                self.engine = Some(handle);
            }
            Message::Engine(engine::Event::Scanning(running)) => {
                self.scanning = running;
            }
            Message::Engine(engine::Event::Updated(light)) => {
                self.remember(light)
            }
            Message::Engine(engine::Event::Failed { message, .. }) => {
                self.notice = Some(message);
            }

            Message::Open(device) => {
                self.focus = Some(device);
                self.panel = 0;
            }
            Message::Back => self.focus = None,

            Message::Toggle(device) => {
                if let Some(light) = self.light(&device) {
                    let lit = light.state.power;
                    self.apply(&device, vec![Command::Power(!lit)]);
                }
            }

            Message::Dim(device, fraction) => {
                if let Some(range) = self
                    .light(&device)
                    .and_then(|light| light.descriptor.capabilities.brightness)
                {
                    let level = range.from_fraction(fraction);
                    self.apply(&device, vec![Command::Brightness(level)]);
                }
            }

            Message::Tint(device, hue, saturation) => {
                self.apply(
                    &device,
                    vec![Command::Color(LightColor::Tint {
                        hue: hue.round().rem_euclid(360.0) as u16,
                        saturation: (saturation * 100.0)
                            .round()
                            .clamp(0.0, 100.0)
                            as u8,
                    })],
                );
            }

            Message::White(device, fraction) => {
                if let Some(range) = self.light(&device).and_then(|light| {
                    light.descriptor.capabilities.color_temperature
                }) {
                    let kelvin = range.from_fraction(fraction);
                    self.apply(
                        &device,
                        vec![Command::Color(LightColor::White { kelvin })],
                    );
                }
            }

            Message::Panel(index) => self.panel = index,
            Message::FlipSkin => self.mode = self.mode.flipped(),

            Message::Sweep => {
                if let Some(engine) = &self.engine {
                    engine.send(Request::Scan);
                }
            }

            Message::AddressToggle => {
                self.address = match self.address {
                    Some(_) => None,
                    None => Some(String::new()),
                };
            }
            Message::AddressTyped(typed) => self.address = Some(typed),
            Message::AddressSubmit => self.reach(),

            Message::Dismiss => self.notice = None,
            Message::WindowSized(size) => self.window_size = size,
        }

        Task::none()
    }

    /// Draws whichever screen is in front.
    pub fn view(&self) -> Element<'_, Message> {
        let skin = self.skin();

        match self.open_light() {
            Some(light) => screen::light::view(light, self.panel, skin),
            None => screen::home::view(
                &self.lights,
                skin,
                self.scanning,
                self.address.as_deref(),
                self.notice.as_deref(),
            ),
        }
    }

    fn open_light(&self) -> Option<&LightSnapshot> {
        self.focus.as_ref().and_then(|device| self.light(device))
    }

    fn light(&self, device: &DeviceId) -> Option<&LightSnapshot> {
        self.lights.iter().find(|light| &light.descriptor.id == device)
    }

    fn remember(&mut self, light: LightSnapshot) {
        match self
            .lights
            .iter_mut()
            .find(|known| known.descriptor.id == light.descriptor.id)
        {
            Some(known) => *known = light,
            None => {
                self.lights.push(light);
                self.lights.sort_by(|a, b| {
                    a.descriptor
                        .name
                        .to_lowercase()
                        .cmp(&b.descriptor.name.to_lowercase())
                        .then_with(|| a.descriptor.id.cmp(&b.descriptor.id))
                });
            }
        }
    }

    /// Sends intents to a light, switching it on first when the intent only
    /// makes sense on a lit bulb.
    fn apply(&self, device: &DeviceId, mut commands: Vec<Command>) {
        let Some(engine) = &self.engine else {
            return;
        };

        let dark = self.light(device).is_some_and(|light| !light.state.power);

        let shapes_the_light = commands
            .iter()
            .any(|command| !matches!(command, Command::Power(_)));

        if dark && shapes_the_light {
            commands.insert(0, Command::Power(true));
        }

        engine.send(Request::Apply { device: device.clone(), commands });
    }

    fn reach(&mut self) {
        let Some(typed) = self.address.as_deref() else {
            return;
        };

        match typed.trim().parse::<IpAddr>() {
            Ok(address) => {
                if let Some(engine) = &self.engine {
                    engine.send(Request::AddByAddress(address));
                }
                self.address = None;
                self.notice = None;
            }
            Err(_) => {
                self.notice = Some(format!(
                    "« {} » n'est pas une adresse IP",
                    typed.trim()
                ));
            }
        }
    }
}

/// The window's logical size, whenever it opens or gets resized.
fn window_size_events() -> Subscription<Size> {
    iced::event::listen_with(|event, _status, _window| match event {
        iced::Event::Window(iced::window::Event::Opened { size, .. }) => {
            Some(size)
        }
        iced::Event::Window(iced::window::Event::Resized(size)) => Some(size),
        _ => None,
    })
}
