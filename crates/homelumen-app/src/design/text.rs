//! Every word HomeLumen shows, in every language it speaks.
//!
//! Nothing outside this file writes a sentence: a screen asks `Lang` for the
//! one it needs, the same way it asks `Skin` for a colour, so adding a
//! language never means hunting through the screens for what to translate.

use homelumen_core::DeviceKind;

/// A language HomeLumen can speak.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Lang {
    En,
    Fr,
}

/// Which language the user asked for.
///
/// Not the same question as [`Lang`]: "whatever the system is set to" is a
/// perfectly good answer here, and only becomes one language or the other
/// once the system has actually been read.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum LangPreference {
    /// Follow the system.
    #[default]
    Auto,
    /// English, whatever the system says.
    En,
    /// French, whatever the system says.
    Fr,
}

impl LangPreference {
    /// The language to actually speak, given what the system was read as at
    /// start-up. `detected` is only ever consulted for
    /// [`LangPreference::Auto`].
    pub fn resolve(self, detected: Lang) -> Lang {
        match self {
            Self::Auto => detected,
            Self::En => Lang::En,
            Self::Fr => Lang::Fr,
        }
    }
}

impl Lang {
    /// Reads the system's own locale and answers with the closest language
    /// HomeLumen speaks. English otherwise: a locale HomeLumen does not know
    /// is not evidence for French over English, and neither is failing to
    /// read one at all.
    pub fn detect() -> Self {
        sys_locale::get_locale()
            .and_then(|tag| {
                let primary = tag.split(['-', '_']).next()?.to_lowercase();
                (primary == "fr").then_some(Self::Fr)
            })
            .unwrap_or(Self::En)
    }

    pub fn my_devices(self, kind: DeviceKind) -> &'static str {
        match (self, kind) {
            (Self::En, DeviceKind::Light) => "My lights",
            (Self::En, DeviceKind::Plug) => "My plugs",
            (Self::Fr, DeviceKind::Light) => "Mes lumières",
            (Self::Fr, DeviceKind::Plug) => "Mes prises",
        }
    }

    pub fn scanning(self) -> &'static str {
        match self {
            Self::En => "Searching…",
            Self::Fr => "Recherche en cours",
        }
    }

    pub fn one_device_on(self, kind: DeviceKind) -> &'static str {
        match (self, kind) {
            (Self::En, DeviceKind::Light) => "One light, on",
            (Self::En, DeviceKind::Plug) => "One plug, on",
            (Self::Fr, DeviceKind::Light) => "Une lumière, allumée",
            (Self::Fr, DeviceKind::Plug) => "Une prise, allumée",
        }
    }

    pub fn one_device_off(self, kind: DeviceKind) -> &'static str {
        match (self, kind) {
            (Self::En, DeviceKind::Light) => "One light, off",
            (Self::En, DeviceKind::Plug) => "One plug, off",
            (Self::Fr, DeviceKind::Light) => "Une lumière, éteinte",
            (Self::Fr, DeviceKind::Plug) => "Une prise, éteinte",
        }
    }

    pub fn all_off(self) -> &'static str {
        match self {
            Self::En => "Everything's off",
            Self::Fr => "Tout est éteint",
        }
    }

    pub fn all_on(self) -> &'static str {
        match self {
            Self::En => "Everything's on",
            Self::Fr => "Tout est allumé",
        }
    }

    pub fn one_of(self, total: usize) -> String {
        match self {
            Self::En => format!("1 of {total} on"),
            Self::Fr => format!("1 allumée sur {total}"),
        }
    }

    pub fn some_of(self, lit: usize, total: usize) -> String {
        match self {
            Self::En => format!("{lit} of {total} on"),
            Self::Fr => format!("{lit} allumées sur {total}"),
        }
    }

    /// The tile's own reading, and the power switch's own label: on or off,
    /// nothing else, so the same two words answer for both.
    pub fn on(self) -> &'static str {
        match self {
            Self::En => "On",
            Self::Fr => "Allumée",
        }
    }

    pub fn off(self) -> &'static str {
        match self {
            Self::En => "Off",
            Self::Fr => "Éteinte",
        }
    }

    pub fn offline(self) -> &'static str {
        match self {
            Self::En => "Offline",
            Self::Fr => "Hors ligne",
        }
    }

    pub fn searching_for(self, kind: DeviceKind) -> &'static str {
        match (self, kind) {
            (Self::En, DeviceKind::Light) => "Searching for lights",
            (Self::En, DeviceKind::Plug) => "Searching for plugs",
            (Self::Fr, DeviceKind::Light) => "Recherche des lumières",
            (Self::Fr, DeviceKind::Plug) => "Recherche des prises",
        }
    }

    pub fn no_devices_yet(self, kind: DeviceKind) -> &'static str {
        match (self, kind) {
            (Self::En, DeviceKind::Light) => "No lights yet",
            (Self::En, DeviceKind::Plug) => "No plugs yet",
            (Self::Fr, DeviceKind::Light) => "Aucune lumière pour l'instant",
            (Self::Fr, DeviceKind::Plug) => "Aucune prise pour l'instant",
        }
    }

    pub fn empty_hint(self) -> &'static str {
        match self {
            Self::En => {
                "Home Lumen is scanning your local network. If broadcast is blocked, add an address by hand."
            }
            Self::Fr => {
                "Home Lumen interroge votre réseau local. Si la diffusion est bloquée, ajoutez une adresse à la main."
            }
        }
    }

    pub fn tuya_hint(self) -> &'static str {
        match self {
            Self::En => {
                "Add your Tuya codes in the settings, and the plugs of your Smart Life account appear here."
            }
            Self::Fr => {
                "Ajoutez vos codes Tuya dans les réglages, et les prises de votre compte Smart Life apparaissent ici."
            }
        }
    }

    pub fn open_settings(self) -> &'static str {
        match self {
            Self::En => "Open the settings",
            Self::Fr => "Ouvrir les réglages",
        }
    }

    /// Title of the settings section holding the Tuya cloud project. Not
    /// translated: it is the name of the app the user already has on their
    /// phone, and renaming it would only make it harder to recognise.
    pub fn smart_life(self) -> &'static str {
        "Smart Life"
    }

    pub fn smart_life_hint(self) -> &'static str {
        match self {
            Self::En => {
                "Tuya gives these two codes out per account. Create a free cloud project on iot.tuya.com, link your Smart Life account to it, then paste its codes here."
            }
            Self::Fr => {
                "Tuya distribue ces deux codes par compte. Créez un projet cloud gratuit sur iot.tuya.com, reliez-y votre compte Smart Life, puis collez ses codes ici."
            }
        }
    }

    pub fn save(self) -> &'static str {
        match self {
            Self::En => "Save",
            Self::Fr => "Enregistrer",
        }
    }

    pub fn could_not_save(self, reason: &str) -> String {
        match self {
            Self::En => format!("Could not save: {reason}"),
            Self::Fr => format!("Impossible d'enregistrer : {reason}"),
        }
    }

    pub fn lights_tab(self) -> &'static str {
        match self {
            Self::En => "Lights",
            Self::Fr => "Lumières",
        }
    }

    pub fn plugs_tab(self) -> &'static str {
        match self {
            Self::En => "Plugs",
            Self::Fr => "Prises",
        }
    }

    pub fn add_an_address(self) -> &'static str {
        match self {
            Self::En => "Add an address",
            Self::Fr => "Ajouter une adresse",
        }
    }

    pub fn add_by_address(self) -> &'static str {
        match self {
            Self::En => "Add by address",
            Self::Fr => "Ajouter par adresse",
        }
    }

    pub fn address_hint(self) -> &'static str {
        match self {
            Self::En => "The light's local address on your network.",
            Self::Fr => "L'adresse locale de la lumière sur votre réseau.",
        }
    }

    pub fn add(self) -> &'static str {
        match self {
            Self::En => "Add",
            Self::Fr => "Ajouter",
        }
    }

    pub fn dismiss(self) -> &'static str {
        match self {
            Self::En => "Dismiss",
            Self::Fr => "Fermer",
        }
    }

    pub fn settings(self) -> &'static str {
        match self {
            Self::En => "Settings",
            Self::Fr => "Paramètres",
        }
    }

    pub fn language(self) -> &'static str {
        match self {
            Self::En => "Language",
            Self::Fr => "Langue",
        }
    }

    pub fn color_tab(self) -> &'static str {
        match self {
            Self::En => "Color",
            Self::Fr => "Couleur",
        }
    }

    pub fn white_tab(self) -> &'static str {
        match self {
            Self::En => "White",
            Self::Fr => "Blanc",
        }
    }

    pub fn warm(self) -> &'static str {
        match self {
            Self::En => "Warm",
            Self::Fr => "Chaud",
        }
    }

    pub fn cool(self) -> &'static str {
        match self {
            Self::En => "Cool",
            Self::Fr => "Froid",
        }
    }

    pub fn not_an_ip_address(self, typed: &str) -> String {
        match self {
            Self::En => format!("\"{typed}\" is not an IP address"),
            Self::Fr => format!("« {typed} » n'est pas une adresse IP"),
        }
    }
}
