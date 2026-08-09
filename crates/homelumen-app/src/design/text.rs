//! Every word HomeLumen shows, in every language it speaks.
//!
//! Nothing outside this file writes a sentence: a screen asks `Lang` for the
//! one it needs, the same way it asks `Skin` for a colour, so adding a
//! language never means hunting through the screens for what to translate.

/// A language HomeLumen can speak.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Lang {
    En,
    Fr,
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

    pub fn my_lights(self) -> &'static str {
        match self {
            Self::En => "My lights",
            Self::Fr => "Mes lumières",
        }
    }

    pub fn scanning(self) -> &'static str {
        match self {
            Self::En => "Searching…",
            Self::Fr => "Recherche en cours",
        }
    }

    pub fn one_light_on(self) -> &'static str {
        match self {
            Self::En => "One light, on",
            Self::Fr => "Une lumière, allumée",
        }
    }

    pub fn one_light_off(self) -> &'static str {
        match self {
            Self::En => "One light, off",
            Self::Fr => "Une lumière, éteinte",
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

    pub fn searching_for_lights(self) -> &'static str {
        match self {
            Self::En => "Searching for lights",
            Self::Fr => "Recherche des lumières",
        }
    }

    pub fn no_lights_yet(self) -> &'static str {
        match self {
            Self::En => "No lights yet",
            Self::Fr => "Aucune lumière pour l'instant",
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
