//! The JSON vocabulary of Tuya's Cloud API, and its translation to and from
//! the generic HomeLumen model.

use homelumen_core::{
    Capabilities, Command, DeviceDescriptor, DeviceId, DeviceKind, Error,
    LightState, Result,
};
use serde::Deserialize;
use serde_json::json;

/// Slug used to namespace the device identifiers this driver mints.
pub const DRIVER: &str = "tuya";

/// Manufacturer name, for display.
pub const VENDOR: &str = "Tuya";

/// The DPS codes a single-socket smart plug is switched through. Tuya's own
/// examples disagree with each other: a plain switch category device answers
/// to `switch`, the standard instruction set for a single-gang switch to
/// `switch_1`. Both are accepted rather than guessing which one a given
/// plug uses.
const SWITCH_CODES: [&str; 2] = ["switch", "switch_1"];

/// Answer to `GET /v1.0/token`, for either `grant_type`.
#[derive(Debug, Deserialize)]
pub struct TokenResult {
    pub access_token: String,
    /// Seconds until the token goes stale.
    pub expire_time: u64,
}

/// One page of `GET /v1.0/iot-01/associated-users/devices`: every device
/// tied to any account this project has been linked to, since a project set
/// up through "Link Tuya App Account" is not itself a specific account.
#[derive(Debug, Default, Deserialize)]
pub struct AssociatedDevicesPage {
    // Tuya's own examples disagree on which of these carries the page: both
    // are accepted, and whichever came back non-empty is used.
    #[serde(default)]
    devices: Vec<TuyaDevice>,
    #[serde(default)]
    list: Vec<TuyaDevice>,
    #[serde(default)]
    pub has_more: bool,
    #[serde(default)]
    pub last_row_key: String,
}

impl AssociatedDevicesPage {
    /// This page's devices.
    pub fn into_devices(self) -> Vec<TuyaDevice> {
        if self.devices.is_empty() { self.list } else { self.devices }
    }
}

/// The subset of a device's cloud record HomeLumen cares about.
#[derive(Debug, Deserialize)]
pub struct TuyaDevice {
    pub id: String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub product_name: String,
    #[serde(default)]
    pub status: Vec<DeviceStatus>,
}

/// One `{code, value}` pair of a device's reported state. `value`'s shape
/// depends on `code` and is not the same JSON type from one device to the
/// next, even for what should be a plain switch, so it stays generic here
/// and is only interpreted where a specific code is read.
#[derive(Debug, Deserialize)]
pub struct DeviceStatus {
    pub code: String,
    pub value: serde_json::Value,
}

impl TuyaDevice {
    /// The DPS code this particular device switches through, if it reports
    /// any of the ones a plug is known to use.
    pub fn switch_code(&self) -> Option<&str> {
        self.status
            .iter()
            .map(|status| status.code.as_str())
            .find(|code| SWITCH_CODES.contains(code))
    }

    /// Builds the generic description of the device.
    pub fn to_descriptor(&self) -> DeviceDescriptor {
        let name = if self.name.trim().is_empty() {
            self.product_name.clone()
        } else {
            self.name.clone()
        };

        DeviceDescriptor {
            id: DeviceId::new(DRIVER, &self.id),
            kind: DeviceKind::Plug,
            name,
            vendor: VENDOR.to_owned(),
            model: self.product_name.clone(),
            capabilities: Capabilities {
                power: true,
                ..Capabilities::default()
            },
        }
    }

    /// The current state of the plug.
    pub fn to_light_state(&self) -> LightState {
        let power = self
            .switch_code()
            .and_then(|code| {
                self.status.iter().find(|status| status.code == code)
            })
            .and_then(|status| status.value.as_bool())
            .unwrap_or(false);

        LightState { power, ..LightState::default() }
    }
}

/// Body for `POST /v1.0/devices/{id}/commands`, switching through
/// `switch_code`: the one this specific device was found to answer to.
pub fn commands_request(
    commands: &[Command],
    switch_code: &str,
) -> Result<Vec<u8>> {
    let mut dps = Vec::with_capacity(commands.len());

    for command in commands {
        match command {
            Command::Power(on) => {
                dps.push(json!({ "code": switch_code, "value": on }));
            }
            Command::Brightness(_) | Command::Color(_) | Command::Effect(_) => {
                return Err(Error::Unsupported(
                    "cette prise ne sait faire qu'allumer et éteindre".into(),
                ));
            }
        }
    }

    Ok(json!({ "commands": dps }).to_string().into_bytes())
}

#[cfg(test)]
mod tests {
    use super::{AssociatedDevicesPage, TuyaDevice, commands_request};
    use homelumen_core::Command;

    // Tuya's own two examples disagree on the DPS code a socket answers to:
    // a metering socket ("cz" category) uses `switch`, the standard
    // single-gang instruction set uses `switch_1`. Both are exercised here.
    const NAMED_SWITCH: &str = r#"{
        "id": "27511006b4e62d4b",
        "name": "Prise cuisine",
        "product_name": "Wi-Fi Smart Metering Socket",
        "online": true,
        "status": [
            { "code": "cur_power", "value": 0 },
            { "code": "switch", "value": true }
        ]
    }"#;

    const NUMBERED_SWITCH: &str = r#"{
        "id": "747b2165d9449964eebb",
        "name": "Prise salon",
        "product_name": "Smart Plug",
        "online": true,
        "status": [
            { "code": "switch_1", "value": false }
        ]
    }"#;

    #[test]
    fn reads_a_metering_socket_by_its_plain_switch_code() {
        let device: TuyaDevice =
            serde_json::from_str(NAMED_SWITCH).expect("valid device");

        assert_eq!(device.switch_code(), Some("switch"));

        let descriptor = device.to_descriptor();
        assert_eq!(descriptor.name, "Prise cuisine");
        assert!(descriptor.capabilities.power);
        assert!(device.to_light_state().power);
    }

    #[test]
    fn reads_a_single_gang_plug_by_its_numbered_switch_code() {
        let device: TuyaDevice =
            serde_json::from_str(NUMBERED_SWITCH).expect("valid device");

        assert_eq!(device.switch_code(), Some("switch_1"));
        assert!(!device.to_light_state().power);
    }

    #[test]
    fn a_brightness_command_is_rejected() {
        let request = commands_request(&[Command::Brightness(50)], "switch");
        assert!(request.is_err());
    }

    #[test]
    fn a_power_command_uses_the_device_s_own_switch_code() {
        let request = commands_request(&[Command::Power(true)], "switch_1")
            .expect("power is supported");
        let request = String::from_utf8(request).expect("utf-8");

        assert!(request.contains("\"code\":\"switch_1\""));
        assert!(request.contains("\"value\":true"));
    }

    #[test]
    fn an_associated_devices_page_reads_the_devices_key() {
        let page: AssociatedDevicesPage = serde_json::from_str(&format!(
            r#"{{"devices": [{NAMED_SWITCH}], "has_more": true, "last_row_key": "abc"}}"#
        ))
        .expect("valid page");

        assert!(page.has_more);
        assert_eq!(page.last_row_key, "abc");
        assert_eq!(page.into_devices().len(), 1);
    }

    #[test]
    fn an_associated_devices_page_falls_back_to_the_list_key() {
        let page: AssociatedDevicesPage = serde_json::from_str(&format!(
            r#"{{"list": [{NAMED_SWITCH}, {NUMBERED_SWITCH}], "has_more": false, "last_row_key": ""}}"#
        ))
        .expect("valid page");

        assert!(!page.has_more);
        assert_eq!(page.into_devices().len(), 2);
    }
}
