//! Tuya plugs, reached through the manufacturer's own cloud.
//!
//! [`Transport::Cloud`]: a plug found this way works from anywhere, at the
//! cost of needing Tuya's servers to be reachable. [`crate::lan::LanProvider`]
//! is the other route, straight to the device on the local network.

use std::io;
use std::sync::Arc;

use async_trait::async_trait;
use homelumen_core::{
    Account, Command, Discovered, DiscoverySink, Endpoint, LightState,
    Provider, Result, Transport, vault,
};
use reqwest::Method;

use crate::wire::{Session, SessionCache};
use crate::{DRIVER, protocol};

/// The Tuya cloud project HomeLumen has been given, if any.
///
/// Tuya hands these out per project rather than per application, so they
/// cannot be shipped inside HomeLumen: each installation is given its own,
/// once, and this is where they are read back from.
pub fn account() -> Option<Account> {
    vault::account(DRIVER)
}

/// Remembers a Tuya cloud project, for this run and every one after it.
///
/// Surrounding blanks are dropped: these are pasted from a web page, and a
/// stray space would only earn a rejected signature and a puzzling error.
pub fn set_account(id: &str, secret: &str) -> io::Result<()> {
    vault::set_account(
        DRIVER,
        Account { id: id.trim().to_owned(), secret: secret.trim().to_owned() },
    )
}

/// Every device linked to this project's Smart Life account, across as many
/// pages as it takes. Shared with [`crate::lan`], which needs the same
/// listing to learn each device's local key.
pub(crate) async fn all_devices(
    session: &Session,
) -> Result<Vec<protocol::TuyaDevice>> {
    let mut devices = Vec::new();
    let mut last_row_key: Option<String> = None;

    loop {
        let query = match &last_row_key {
            Some(key) => format!(
                "/v1.0/iot-01/associated-users/devices?size=50&last_row_key={key}"
            ),
            None => "/v1.0/iot-01/associated-users/devices?size=50".to_owned(),
        };

        let page: protocol::AssociatedDevicesPage =
            session.call_as_app(Method::GET, &query, b"").await?;

        let has_more = page.has_more;
        let next_row_key = page.last_row_key.clone();
        devices.extend(page.into_devices());

        if !has_more || next_row_key.is_empty() {
            break;
        }
        last_row_key = Some(next_row_key);
    }

    Ok(devices)
}

/// Finds the Tuya plugs the user's Smart Life account has been linked to.
#[derive(Default)]
pub struct CloudProvider {
    sessions: SessionCache,
}

#[async_trait]
impl Provider for CloudProvider {
    fn id(&self) -> &'static str {
        DRIVER
    }

    fn name(&self) -> &'static str {
        "Tuya, via le compte Smart Life"
    }

    async fn discover(&self, sink: DiscoverySink) -> Result<()> {
        let session = self.sessions.get().await?;

        for device in all_devices(&session).await? {
            let Some(switch_code) = device.switch_code() else {
                continue;
            };
            let switch_code = switch_code.to_owned();

            let found = Discovered {
                descriptor: device.to_descriptor(),
                state: device.to_light_state(),
                endpoint: Arc::new(CloudEndpoint {
                    session: session.clone(),
                    device_id: device.id,
                    switch_code,
                }),
            };

            if sink.send(found).await.is_err() {
                break;
            }
        }

        Ok(())
    }

    // Cloud devices are not reached by address: `probe` keeps the default,
    // which refuses cleanly rather than pretending to try.
}

/// A plug reachable through Tuya's cloud.
struct CloudEndpoint {
    session: Session,
    device_id: String,
    /// The DPS code this device was found to switch through: Tuya does not
    /// use the same one for every product, see [`protocol::TuyaDevice::switch_code`].
    switch_code: String,
}

#[async_trait]
impl Endpoint for CloudEndpoint {
    fn transport(&self) -> Transport {
        Transport::Cloud
    }

    fn address(&self) -> String {
        "Smart Life".to_owned()
    }

    async fn state(&self) -> Result<LightState> {
        let device: protocol::TuyaDevice = self
            .session
            .call_as_app(
                Method::GET,
                &format!("/v1.0/devices/{}", self.device_id),
                b"",
            )
            .await?;

        Ok(device.to_light_state())
    }

    async fn apply(&self, commands: &[Command]) -> Result<LightState> {
        let body = protocol::commands_request(commands, &self.switch_code)?;

        self.session
            .call_as_app::<bool>(
                Method::POST,
                &format!("/v1.0/devices/{}/commands", self.device_id),
                &body,
            )
            .await?;

        self.state().await
    }
}
