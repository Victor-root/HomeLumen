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

/// The DPS code a single-socket smart plug is switched through, per Tuya's
/// standard instruction set for the switch category. A device that never
/// reports this code is not something this driver knows how to drive yet.
const SWITCH_CODE: &str = "switch_1";

/// Answer to `GET /v1.0/token`, for either `grant_type`.
#[derive(Debug, Deserialize)]
pub struct TokenResult {
    pub access_token: String,
    /// Seconds until the token goes stale.
    pub expire_time: u64,
}

/// Answer to `GET /v1.0/devices`.
#[derive(Debug, Deserialize)]
pub struct DeviceListResult {
    pub devices: Vec<TuyaDevice>,
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
    /// Whether this record is a switch this driver knows how to drive: it
    /// must report the one DPS code the driver reads and writes.
    pub fn is_switch(&self) -> bool {
        self.status.iter().any(|status| status.code == SWITCH_CODE)
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
            .status
            .iter()
            .find(|status| status.code == SWITCH_CODE)
            .and_then(|status| status.value.as_bool())
            .unwrap_or(false);

        LightState { power, ..LightState::default() }
    }
}

/// Body for `POST /v1.0/devices/{id}/commands`.
pub fn commands_request(commands: &[Command]) -> Result<Vec<u8>> {
    let mut dps = Vec::with_capacity(commands.len());

    for command in commands {
        match command {
            Command::Power(on) => {
                dps.push(json!({ "code": SWITCH_CODE, "value": on }));
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
    use super::{DeviceListResult, commands_request};
    use homelumen_core::Command;

    // A trimmed version of the example response from Tuya's own device
    // management documentation, kept close to the original field order and
    // values so a change in what the driver reads is easy to see against it.
    const DEVICE_LIST: &str = r#"{
        "devices": [
            {
                "id": "747b2165d9449964eebb",
                "name": "Prise cuisine",
                "product_name": "Smart Plug",
                "online": true,
                "status": [
                    { "code": "switch_1", "value": true }
                ]
            }
        ],
        "total": 1
    }"#;

    #[test]
    fn reads_a_switched_on_plug_from_the_device_list() {
        let result: DeviceListResult =
            serde_json::from_str(DEVICE_LIST).expect("valid device list");
        let device = &result.devices[0];

        assert!(device.is_switch());

        let descriptor = device.to_descriptor();
        assert_eq!(descriptor.name, "Prise cuisine");
        assert!(descriptor.capabilities.power);

        assert!(device.to_light_state().power);
    }

    #[test]
    fn a_brightness_command_is_rejected() {
        let request = commands_request(&[Command::Brightness(50)]);
        assert!(request.is_err());
    }

    #[test]
    fn a_power_command_becomes_a_switch_dps() {
        let request = commands_request(&[Command::Power(true)])
            .expect("power is supported");
        let request = String::from_utf8(request).expect("utf-8");

        assert!(request.contains("\"code\":\"switch_1\""));
        assert!(request.contains("\"value\":true"));
    }
}
