//! What HomeLumen holds on to, and what it does when something happens.

use std::net::IpAddr;

use homelumen_core::{
    Account, Color as LightColor, Command, DeviceId, DeviceKind,
};
use homelumen_engine::{self as engine, Handle, LightSnapshot, Request};
use iced::{Element, Size, Subscription, Task};

use crate::design::{Lang, LangPreference, Mode, Preference, Skin};
use crate::screen;

/// Which screen is in front.
#[derive(Debug, Clone, PartialEq, Eq)]
enum Route {
    Home,
    Light(DeviceId),
    Settings,
}

/// The whole state of the interface.
pub struct App {
    engine: Option<Handle>,
    lights: Vec<LightSnapshot>,
    route: Route,
    tab: DeviceKind,
    panel: usize,
    preference: Preference,
    system: Mode,
    lang_preference: LangPreference,
    detected_lang: Lang,
    /// The Tuya cloud project as it stands in the settings fields, which is
    /// only what is on file until the user edits it.
    tuya: Account,
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
            route: Route::Home,
            tab: DeviceKind::Light,
            panel: 0,
            preference: Preference::default(),
            // Until the desktop answers, HomeLumen shows the skin it is
            // designed around rather than guessing at the other one.
            system: Mode::Night,
            lang_preference: LangPreference::default(),
            // Unlike the skin, read once and settled here rather than
            // asked for through a `Task`: the system's locale is already
            // sitting there for the reading, nothing to wait on.
            detected_lang: Lang::detect(),
            tuya: homelumen_tuya::account().unwrap_or_default(),
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
    /// Switch between the lights tab and the plugs tab.
    Tab(DeviceKind),
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
    /// Step to the next skin preference.
    CycleSkin,
    /// The desktop says it is set to this skin.
    SystemSkin(Mode),
    /// Look for lights again.
    Sweep,
    /// Open or close the address field.
    AddressToggle,
    /// The address being typed changed.
    AddressTyped(String),
    /// Reach the typed address.
    AddressSubmit,
    /// Open the settings screen.
    OpenSettings,
    /// Set the language preference.
    LangChanged(LangPreference),
    /// The Tuya client id being typed changed.
    TuyaIdTyped(String),
    /// The Tuya secret being typed changed.
    TuyaSecretTyped(String),
    /// Keep the typed Tuya project and go looking with it.
    TuyaSave,
    /// Put the last message away.
    Dismiss,
    /// The window changed size.
    WindowSized(Size),
}

impl App {
    /// Starts the interface, asking the desktop which skin it is set to.
    ///
    /// The answer also arrives on every later change through
    /// [`App::subscription`]; this first read is what keeps the automatic
    /// preference from having to wait for the desktop to change its mind
    /// before it knows anything at all.
    pub fn boot() -> (Self, Task<Message>) {
        (
            Self::default(),
            iced::system::theme()
                .map(|mode| Message::SystemSkin(system_skin(mode))),
        )
    }

    /// The skin currently in use.
    pub fn skin(&self) -> Skin {
        Skin::of(self.preference.resolve(self.system))
    }

    /// The language currently in use.
    pub fn lang(&self) -> Lang {
        self.lang_preference.resolve(self.detected_lang)
    }

    /// The theme iced needs for the few things HomeLumen does not paint itself.
    pub fn theme(&self) -> iced::Theme {
        match self.preference.resolve(self.system) {
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
            Some(light) => format!("{} · Home Lumen", light.descriptor.name),
            None => "Home Lumen".to_owned(),
        };

        let width = self.window_size.width.round() as i32;
        let height = self.window_size.height.round() as i32;
        format!("{name} · {width} × {height}")
    }

    /// The engine runs for as long as the window does; window-size events
    /// keep the title's live readout accurate, and the desktop says so here
    /// whenever it switches between its light and dark hours.
    pub fn subscription(&self) -> Subscription<Message> {
        Subscription::batch([
            Subscription::run(engine::run).map(Message::Engine),
            window_size_events().map(Message::WindowSized),
            iced::system::theme_changes()
                .map(|mode| Message::SystemSkin(system_skin(mode))),
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
                self.route = Route::Light(device);
                self.panel = 0;
            }
            Message::Back => self.route = Route::Home,
            Message::Tab(tab) => self.tab = tab,

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
            Message::CycleSkin => self.preference = self.preference.next(),
            Message::SystemSkin(mode) => self.system = mode,

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

            Message::OpenSettings => self.route = Route::Settings,
            Message::LangChanged(preference) => {
                self.lang_preference = preference
            }

            Message::TuyaIdTyped(typed) => self.tuya.id = typed,
            Message::TuyaSecretTyped(typed) => self.tuya.secret = typed,
            Message::TuyaSave => self.keep_tuya(),

            Message::Dismiss => self.notice = None,
            Message::WindowSized(size) => self.window_size = size,
        }

        Task::none()
    }

    /// Draws whichever screen is in front.
    pub fn view(&self) -> Element<'_, Message> {
        let skin = self.skin();
        let lang = self.lang();

        if self.route == Route::Settings {
            return screen::settings::view(
                skin,
                lang,
                self.lang_preference,
                screen::settings::TuyaDraft {
                    id: &self.tuya.id,
                    secret: &self.tuya.secret,
                },
            );
        }

        match self.open_light() {
            Some(light) => screen::light::view(light, self.panel, skin, lang),
            None => screen::home::view(
                &self.lights,
                skin,
                lang,
                self.preference,
                self.tab,
                self.scanning,
                self.address.as_deref(),
                self.notice.as_deref(),
            ),
        }
    }

    fn open_light(&self) -> Option<&LightSnapshot> {
        match &self.route {
            Route::Light(device) => self.light(device),
            _ => None,
        }
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

    /// Files the typed Tuya project and goes looking straight away, so the
    /// plugs answer for whether the codes were right rather than a message
    /// saying they were saved.
    fn keep_tuya(&mut self) {
        match homelumen_tuya::set_account(&self.tuya.id, &self.tuya.secret) {
            Ok(()) => {
                self.tuya.id = self.tuya.id.trim().to_owned();
                self.tuya.secret = self.tuya.secret.trim().to_owned();
                self.notice = None;
                self.route = Route::Home;
                self.tab = DeviceKind::Plug;

                if let Some(engine) = &self.engine {
                    engine.send(Request::Scan);
                }
            }
            Err(error) => {
                self.notice =
                    Some(self.lang().could_not_save(&error.to_string()));
            }
        }
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
                self.notice = Some(self.lang().not_an_ip_address(typed.trim()));
            }
        }
    }
}

/// The skin the desktop itself is set to, as far as it will say.
///
/// A desktop that expresses no preference at all gets HomeLumen's own
/// reference skin rather than an arbitrary one of the two.
fn system_skin(mode: iced::theme::Mode) -> Mode {
    match mode {
        iced::theme::Mode::Light => Mode::Day,
        iced::theme::Mode::Dark | iced::theme::Mode::None => Mode::Night,
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
