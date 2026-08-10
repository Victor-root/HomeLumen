//! The JSON vocabulary of Kasa bulbs, and its translation to and from the
//! generic HomeLumen model.

use homelumen_core::{
    BrightnessRange, Capabilities, Color, ColorTemperatureRange, Command,
    DeviceDescriptor, DeviceId, DeviceKind, Error, LightState, Result,
};
use serde::Deserialize;
use serde_json::{Value, json};

/// Slug used to namespace the device identifiers this driver mints.
pub const DRIVER: &str = "kasa";

/// Manufacturer name, for display.
pub const VENDOR: &str = "TP-Link Kasa";

/// Fade applied to every change, in milliseconds. Short enough to feel
/// immediate, long enough to avoid a harsh step.
const TRANSITION_MS: u32 = 220;

/// Request every device answers, on UDP as well as on TCP.
pub fn sysinfo_request() -> Vec<u8> {
    json!({ "system": { "get_sysinfo": {} } }).to_string().into_bytes()
}

/// Request applying a batch of commands in a single round trip.
pub fn transition_request(commands: &[Command]) -> Result<Vec<u8>> {
    let mut params = json!({
        "ignore_default": 1,
        "transition_period": TRANSITION_MS,
    });
    let fields =
        params.as_object_mut().expect("the literal above is an object");

    for command in commands {
        match command {
            Command::Power(on) => {
                fields.insert("on_off".into(), json!(u8::from(*on)));
            }
            Command::Brightness(level) => {
                fields.insert("brightness".into(), json!(*level));
            }
            Command::Color(Color::White { kelvin }) => {
                fields.insert("color_temp".into(), json!(*kelvin));
                fields.insert("hue".into(), json!(0));
                fields.insert("saturation".into(), json!(0));
            }
            Command::Color(Color::Tint { hue, saturation }) => {
                // A non-zero temperature keeps the bulb in white mode and the
                // hue would be ignored.
                fields.insert("color_temp".into(), json!(0));
                fields.insert("hue".into(), json!(*hue));
                fields.insert("saturation".into(), json!(*saturation));
            }
            Command::Effect(_) => {
                return Err(Error::Unsupported(
                    "ces ampoules n'ont pas d'effets intégrés".into(),
                ));
            }
        }
    }

    Ok(json!({
        "smartlife.iot.smartbulb.lightingservice": {
            "transition_light_state": params,
        }
    })
    .to_string()
    .into_bytes())
}

/// Reads the `get_sysinfo` answer of a device.
pub fn parse_sysinfo(payload: &[u8]) -> Result<SysInfo> {
    let envelope: Value = serde_json::from_slice(payload)
        .map_err(|error| Error::Protocol(error.to_string()))?;

    let body = envelope
        .get("system")
        .and_then(|system| system.get("get_sysinfo"))
        .ok_or_else(|| Error::Protocol("réponse sans get_sysinfo".into()))?;

    let info: SysInfo = serde_json::from_value(body.clone())
        .map_err(|error| Error::Protocol(error.to_string()))?;

    if info.err_code != 0 {
        return Err(Error::Rejected(format!("err_code {}", info.err_code)));
    }

    Ok(info)
}

/// Reads the answer to a [`transition_request`].
pub fn parse_transition(payload: &[u8]) -> Result<LightState> {
    let envelope: Value = serde_json::from_slice(payload)
        .map_err(|error| Error::Protocol(error.to_string()))?;

    let body = envelope
        .get("smartlife.iot.smartbulb.lightingservice")
        .and_then(|service| service.get("transition_light_state"))
        .ok_or_else(|| {
            Error::Protocol("réponse sans transition_light_state".into())
        })?;

    let bulb: BulbState = serde_json::from_value(body.clone())
        .map_err(|error| Error::Protocol(error.to_string()))?;

    if bulb.err_code != 0 {
        return Err(Error::Rejected(format!("err_code {}", bulb.err_code)));
    }

    Ok(bulb.to_light_state())
}

/// The subset of `get_sysinfo` HomeLumen cares about.
#[derive(Debug, Clone, Deserialize)]
pub struct SysInfo {
    #[serde(default)]
    alias: String,
    #[serde(default)]
    model: String,
    #[serde(rename = "deviceId", default)]
    device_id: String,
    #[serde(default)]
    mic_mac: String,
    #[serde(default)]
    is_dimmable: u8,
    #[serde(default)]
    is_color: u8,
    #[serde(default)]
    is_variable_color_temp: u8,
    #[serde(default)]
    light_state: BulbState,
    #[serde(default)]
    err_code: i32,
}

impl SysInfo {
    /// Whether the answer came from a light rather than from a plug or a switch.
    pub fn is_light(&self) -> bool {
        self.is_dimmable == 1
            || self.is_color == 1
            || self.is_variable_color_temp == 1
    }

    /// Builds the generic description of the device.
    pub fn to_descriptor(&self) -> Result<DeviceDescriptor> {
        let hardware_id = if self.device_id.is_empty() {
            &self.mic_mac
        } else {
            &self.device_id
        };

        if hardware_id.is_empty() {
            return Err(Error::Protocol(
                "appareil sans identifiant matériel".into(),
            ));
        }

        let reference = model_reference(&self.model);
        let name = if self.alias.trim().is_empty() {
            reference.to_owned()
        } else {
            self.alias.trim().to_owned()
        };

        Ok(DeviceDescriptor {
            id: DeviceId::new(DRIVER, hardware_id),
            // This driver only ever keeps what `is_light` accepted; see
            // `LanProvider::discover` and `::probe`.
            kind: DeviceKind::Light,
            name,
            vendor: VENDOR.to_owned(),
            model: reference.to_owned(),
            capabilities: self.capabilities(),
        })
    }

    /// The current state of the light.
    pub fn to_light_state(&self) -> LightState {
        self.light_state.to_light_state()
    }

    fn capabilities(&self) -> Capabilities {
        Capabilities {
            power: true,
            brightness: (self.is_dimmable == 1)
                .then_some(BrightnessRange::PERCENT),
            color_temperature: (self.is_variable_color_temp == 1)
                .then(|| white_range(&self.model)),
            color: self.is_color == 1,
            effects: Vec::new(),
        }
    }
}

/// The `light_state` object, also returned verbatim by `transition_light_state`.
#[derive(Debug, Clone, Default, Deserialize)]
struct BulbState {
    #[serde(default)]
    on_off: u8,
    #[serde(default)]
    hue: u16,
    #[serde(default)]
    saturation: u8,
    #[serde(default)]
    color_temp: u16,
    #[serde(default)]
    brightness: u8,
    /// Values the bulb will return to when switched back on. Present only while
    /// the bulb is off, and the only way to know its colour in that state.
    #[serde(default)]
    dft_on_state: Option<Box<BulbState>>,
    #[serde(default)]
    err_code: i32,
}

impl BulbState {
    fn to_light_state(&self) -> LightState {
        let lit = self.dft_on_state.as_deref().unwrap_or(self);

        LightState {
            power: self.on_off == 1,
            brightness: Some(lit.brightness),
            color: Some(if lit.color_temp > 0 {
                Color::White { kelvin: lit.color_temp }
            } else if lit.saturation > 0 {
                Color::Tint { hue: lit.hue, saturation: lit.saturation }
            } else {
                Color::DEFAULT_WHITE
            }),
            effect: None,
        }
    }
}

/// Strips the regional suffix Kasa appends, turning `LB130(EU)` into `LB130`.
fn model_reference(model: &str) -> &str {
    model.split('(').next().unwrap_or(model).trim()
}

/// The white range of a bulb.
///
/// Kasa firmwares announce *that* a bulb has a tunable white, never its range,
/// so the driver carries the table. This is exactly the kind of knowledge that
/// belongs to a driver and must never reach the interface.
fn white_range(model: &str) -> ColorTemperatureRange {
    const WIDE: ColorTemperatureRange =
        ColorTemperatureRange { min_kelvin: 2500, max_kelvin: 9000 };
    const STANDARD: ColorTemperatureRange =
        ColorTemperatureRange { min_kelvin: 2700, max_kelvin: 6500 };

    match model_reference(model).to_uppercase().as_str() {
        "LB130" | "LB230" | "KL125" | "KL130" | "KL135" | "KL430" => WIDE,
        _ => STANDARD,
    }
}

#[cfg(test)]
mod tests {
    use super::{parse_sysinfo, transition_request, white_range};
    use homelumen_core::{Color, Command};

    const LB130: &[u8] = br#"{"system":{"get_sysinfo":{
        "sw_ver":"1.8.11","hw_ver":"1.0","model":"LB130(EU)",
        "deviceId":"801E2F0B","alias":"Salon","mic_type":"IOT.SMARTBULB",
        "mic_mac":"50C7BF00","is_dimmable":1,"is_color":1,
        "is_variable_color_temp":1,
        "light_state":{"on_off":0,"dft_on_state":{
            "mode":"normal","hue":220,"saturation":80,
            "color_temp":0,"brightness":45}},
        "err_code":0}}}"#;

    #[test]
    fn reads_capabilities_from_the_device() {
        let info = parse_sysinfo(LB130).expect("valid sysinfo");
        let descriptor = info.to_descriptor().expect("descriptor");

        assert_eq!(descriptor.name, "Salon");
        assert_eq!(descriptor.model, "LB130");
        assert!(descriptor.capabilities.color);
        assert_eq!(
            descriptor.capabilities.color_temperature.map(|r| r.max_kelvin),
            Some(9000)
        );
    }

    #[test]
    fn reads_the_colour_of_a_bulb_that_is_off() {
        let state =
            parse_sysinfo(LB130).expect("valid sysinfo").to_light_state();

        assert!(!state.power);
        assert_eq!(state.brightness, Some(45));
        assert_eq!(state.color, Some(Color::Tint { hue: 220, saturation: 80 }));
    }

    #[test]
    fn a_tint_clears_the_white_temperature() {
        let request = transition_request(&[Command::Color(Color::Tint {
            hue: 12,
            saturation: 90,
        })])
        .expect("supported command");
        let request = String::from_utf8(request).expect("utf-8");

        assert!(request.contains("\"color_temp\":0"));
        assert!(request.contains("\"hue\":12"));
    }

    #[test]
    fn narrow_white_range_for_the_lb120() {
        assert_eq!(white_range("LB120(US)").max_kelvin, 6500);
    }
}
