//! Tuya plugs, reached through the manufacturer's own cloud.
//!
//! There is no local route here, only [`Transport::Cloud`]: a plug found
//! this way works from anywhere, at the cost of needing Tuya's servers to be
//! reachable at all. Local control is a separate driver, for later.

use std::sync::Arc;

use async_trait::async_trait;
use homelumen_core::{
    Command, Discovered, DiscoverySink, Endpoint, Error, LightState, Provider,
    Result, Transport,
};
use reqwest::Method;

use crate::protocol::{self, DRIVER};
use crate::wire::{DataCenter, Session};

/// HomeLumen's own Tuya Cloud Project credentials, registered once for the
/// whole application: no HomeLumen user ever creates a project or types a
/// secret of their own. Empty until that one-time registration is done, at
/// which point this driver quietly finds nothing rather than failing to
/// build.
const CLIENT_ID: &str = "";
const CLIENT_SECRET: &str = "";

/// Finds Tuya plugs already linked to HomeLumen's own cloud project.
pub struct CloudProvider {
    session: Session,
}

impl Default for CloudProvider {
    fn default() -> Self {
        Self {
            session: Session::new(DataCenter::EUROPE, CLIENT_ID, CLIENT_SECRET),
        }
    }
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
        if !self.session.is_configured() {
            return Err(Error::Unauthorized(
                "identifiants Tuya de l'application non configurés".into(),
            ));
        }

        let result: protocol::DeviceListResult =
            self.session.call_as_app(Method::GET, "/v1.0/devices", b"").await?;

        for device in result.devices {
            if !device.is_switch() {
                continue;
            }

            let found = Discovered {
                descriptor: device.to_descriptor(),
                state: device.to_light_state(),
                endpoint: Arc::new(CloudEndpoint {
                    session: self.session.clone(),
                    device_id: device.id,
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
        let body = protocol::commands_request(commands)?;

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
