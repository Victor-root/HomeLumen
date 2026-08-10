//! The JSON vocabulary of Tuya's local protocol, and its translation to and
//! from the generic HomeLumen model.
//!
//! Scoped to protocol versions 3.2 and 3.3, which speak identically for
//! everything this driver does: 3.1 answers in the clear and is old enough
//! not to be worth the separate code path, 3.4 and 3.5 need a session-key
//! handshake this driver does not implement. A device on another version is
//! simply never offered a local route; it keeps working through the cloud
//! one.

use std::collections::BTreeMap;
use std::time::{SystemTime, UNIX_EPOCH};

use homelumen_core::{
    Capabilities, DeviceDescriptor, DeviceId, DeviceKind, Error, LightState,
    Result,
};
use serde::Deserialize;
use serde_json::{Value, json};

use crate::{DRIVER, VENDOR};

/// What a device announces about itself, unprompted, on the local network.
#[derive(Debug, Deserialize)]
pub struct DiscoveryBeacon {
    #[serde(rename = "gwId")]
    pub gw_id: String,
    #[serde(default)]
    pub version: String,
}

impl DiscoveryBeacon {
    /// Whether this is a version [`crate::lan`] knows how to encrypt frames
    /// for. See the module doc for why 3.1, 3.4 and 3.5 are left alone.
    pub fn is_supported(&self) -> bool {
        matches!(self.version.as_str(), "3.2" | "3.3")
    }
}

/// Reads a [`DiscoveryBeacon`] out of a decrypted datagram.
pub fn parse_beacon(payload: &[u8]) -> Result<DiscoveryBeacon> {
    serde_json::from_slice(payload)
        .map_err(|error| Error::Protocol(error.to_string()))
}

/// A device's own report of its `dps`: the numeric-id state map local
/// control speaks in, as opposed to the named codes the cloud API uses.
#[derive(Debug, Default, Deserialize)]
pub struct StateReport {
    #[serde(default)]
    dps: BTreeMap<String, Value>,
}

impl StateReport {
    /// The dp this device switches through: the lowest-numbered dp carrying
    /// a plain on/off value.
    ///
    /// Local control has no named codes, only numbered dps, and learning
    /// their meaning for certain needs a schema call this driver does not
    /// make; `"1"` is, by convention, the primary switch on every
    /// single-relay plug this was tested against, and every real Tuya
    /// client falls back to the same rule. A device with more than one
    /// boolean dp (a multi-gang switch) only ever offers its first gang
    /// locally; every gang keeps working through the cloud route.
    pub fn switch_dp(&self) -> Option<&str> {
        self.dps
            .iter()
            .filter(|(_, value)| value.is_boolean())
            .min_by_key(|(id, _)| id.parse::<u32>().unwrap_or(u32::MAX))
            .map(|(id, _)| id.as_str())
    }

    /// The current state of the plug.
    pub fn to_light_state(&self) -> LightState {
        let power = self
            .switch_dp()
            .and_then(|dp| self.dps.get(dp))
            .and_then(Value::as_bool)
            .unwrap_or(false);

        LightState { power, ..LightState::default() }
    }
}

/// Reads a [`StateReport`] out of a decrypted answer to [`query_request`].
pub fn parse_state(payload: &[u8]) -> Result<StateReport> {
    serde_json::from_slice(payload)
        .map_err(|error| Error::Protocol(error.to_string()))
}

/// The generic description of a plug found on the local network. Carries no
/// name or model: the beacon that found it has neither, unlike the cloud
/// account, which usually already knows both by the time this is seen.
pub fn descriptor(device_id: &str) -> DeviceDescriptor {
    DeviceDescriptor {
        id: DeviceId::new(DRIVER, device_id),
        kind: DeviceKind::Plug,
        name: String::new(),
        vendor: VENDOR.to_owned(),
        model: String::new(),
        capabilities: Capabilities { power: true, ..Capabilities::default() },
    }
}

/// Body of a `DP_QUERY`: asks a device for every dp it currently reports.
pub fn query_request(device_id: &str) -> Vec<u8> {
    json!({
        "gwId": device_id,
        "devId": device_id,
        "uid": device_id,
        "t": timestamp(),
    })
    .to_string()
    .into_bytes()
}

/// Body of a `CONTROL`: sets a single dp.
pub fn control_request(device_id: &str, dp: &str, on: bool) -> Vec<u8> {
    json!({
        "devId": device_id,
        "uid": device_id,
        "t": timestamp(),
        "dps": { dp: on },
    })
    .to_string()
    .into_bytes()
}

/// Seconds since the epoch, as Tuya's local protocol wants it: a string, not
/// a JSON number.
fn timestamp() -> String {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("the system clock reads after 1970")
        .as_secs()
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::{DiscoveryBeacon, control_request, parse_beacon, parse_state};

    const BEACON: &[u8] = br#"{
        "ip": "192.168.1.42",
        "gwId": "eb0123456789abcdefghij",
        "active": 2,
        "ablilty": 0,
        "encrypt": true,
        "productKey": "keydeadbeef12345",
        "version": "3.3"
    }"#;

    #[test]
    fn reads_a_beacon_including_its_misspelled_field() {
        let beacon = parse_beacon(BEACON).expect("valid beacon");

        assert_eq!(beacon.gw_id, "eb0123456789abcdefghij");
        assert!(beacon.is_supported());
    }

    #[test]
    fn a_3_4_beacon_is_not_supported() {
        let beacon = DiscoveryBeacon {
            gw_id: "eb0123456789abcdefghij".to_owned(),
            version: "3.4".to_owned(),
        };

        assert!(!beacon.is_supported());
    }

    #[test]
    fn the_lowest_boolean_dp_is_the_switch() {
        let report = parse_state(br#"{"dps":{"9":0,"1":true,"20":false}}"#)
            .expect("valid report");

        assert_eq!(report.switch_dp(), Some("1"));
        assert!(report.to_light_state().power);
    }

    #[test]
    fn dp_ids_sort_numerically_not_lexically() {
        // A lexical sort would put "10" before "2".
        let report = parse_state(br#"{"dps":{"10":false,"2":true}}"#)
            .expect("valid report");

        assert_eq!(report.switch_dp(), Some("2"));
    }

    #[test]
    fn a_device_reporting_nothing_boolean_has_no_switch() {
        let report = parse_state(br#"{"dps":{"9":0}}"#).expect("valid report");

        assert_eq!(report.switch_dp(), None);
    }

    #[test]
    fn a_control_request_names_its_dp() {
        let request = control_request("eb0123", "1", true);
        let request = String::from_utf8(request).expect("utf-8");

        assert!(request.contains("\"dps\":{\"1\":true}"));
        assert!(request.contains("\"devId\":\"eb0123\""));
    }
}
