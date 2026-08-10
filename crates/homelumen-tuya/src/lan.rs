//! Tuya plugs, reached directly on the local network.
//!
//! [`Transport::Lan`]: once a device's own key is known, this needs no
//! round trip through Tuya's servers at all, and keeps working even while
//! [`crate::cloud::CloudProvider`] cannot reach them, expired trial or not.
//! That key is never typed and Tuya keeps no page to read it back from: the
//! only legitimate way to learn it is the same cloud account, asked once and
//! then cached under the device's own identity, see
//! [`homelumen_core::vault::device_secret`].

use std::collections::HashSet;
use std::net::IpAddr;
use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use homelumen_core::{
    Command, DeviceId, Discovered, DiscoverySink, Endpoint, Error, LightState,
    Provider, Result, Transport, vault,
};

use crate::wire::SessionCache;
use crate::{DRIVER, cloud, lan_protocol, lan_wire};

/// How long a discovery sweep listens for beacons. Tuya devices broadcast at
/// no documented interval, unlike Kasa's; generous enough to give a quiet
/// one a real chance without holding up a visible scan too long.
const SWEEP_WINDOW: Duration = Duration::from_secs(5);

/// How long a device is given to answer a direct request.
const CALL_TIMEOUT: Duration = Duration::from_millis(2500);

/// Finds Tuya plugs by listening for their own local broadcasts.
#[derive(Default)]
pub struct LanProvider {
    sessions: SessionCache,
}

impl LanProvider {
    /// Asks the cloud account for every linked device's local key and caches
    /// what it gets back, so future sweeps need it less and less.
    async fn refresh_keys(&self) -> Result<()> {
        let session = self.sessions.get().await?;

        for device in cloud::all_devices(&session).await? {
            if device.local_key.trim().is_empty() {
                continue;
            }

            let id = DeviceId::new(DRIVER, &device.id);
            if let Err(error) = vault::set_device_secret(&id, &device.local_key)
            {
                log::warn!("{id}: clé locale non enregistrée: {error}");
            }
        }

        Ok(())
    }
}

#[async_trait]
impl Provider for LanProvider {
    fn id(&self) -> &'static str {
        DRIVER
    }

    fn name(&self) -> &'static str {
        "Tuya, réseau local"
    }

    async fn discover(&self, sink: DiscoverySink) -> Result<()> {
        let mut seen = Vec::new();
        let mut known = HashSet::new();

        for beacon in lan_wire::listen(SWEEP_WINDOW).await? {
            let report = match lan_protocol::parse_beacon(&beacon.payload) {
                Ok(report) => report,
                Err(error) => {
                    log::debug!(
                        "{}: balise Tuya illisible: {error}",
                        beacon.address
                    );
                    continue;
                }
            };

            if !report.is_supported() {
                log::debug!(
                    "{}: version locale {:?} non prise en charge",
                    beacon.address,
                    report.version
                );
                continue;
            }

            // A device broadcasting more than once inside the listen window
            // must not earn it two round trips.
            if known.insert(report.gw_id.clone()) {
                seen.push((beacon.address, report.gw_id));
            }
        }

        if seen.is_empty() {
            return Ok(());
        }

        let missing_a_key = seen.iter().any(|(_, gw_id)| {
            vault::device_secret(&DeviceId::new(DRIVER, gw_id)).is_none()
        });

        if missing_a_key {
            if let Err(error) = self.refresh_keys().await {
                log::debug!("clés locales Tuya non rafraîchies: {error}");
            }
        }

        for (address, gw_id) in seen {
            let id = DeviceId::new(DRIVER, &gw_id);

            let Some(secret) = vault::device_secret(&id) else {
                continue;
            };
            let Ok(key) = parse_key(&secret) else {
                log::warn!("{id}: clé locale mal formée, oubliée");
                let _ = vault::forget_device_secret(&id);
                continue;
            };

            match query(address, &gw_id, &key).await {
                Ok(report) => {
                    let Some(dp) = report.switch_dp() else {
                        log::debug!(
                            "{id}: aucun interrupteur reconnu localement"
                        );
                        continue;
                    };

                    let found = Discovered {
                        descriptor: lan_protocol::descriptor(&gw_id),
                        state: report.to_light_state(),
                        endpoint: Arc::new(LanEndpoint {
                            address,
                            device_id: gw_id,
                            key,
                            switch_dp: dp.to_owned(),
                        }),
                    };

                    if sink.send(found).await.is_err() {
                        return Ok(());
                    }
                }
                Err(Error::Protocol(_)) => {
                    log::debug!("{id}: clé locale rejetée, oubliée");
                    let _ = vault::forget_device_secret(&id);
                }
                Err(error) => {
                    log::debug!("{id}: injoignable en local: {error}");
                }
            }
        }

        Ok(())
    }

    // A LAN address alone is not enough to know which device answers there
    // or which key to speak to it with: `probe` keeps the default, which
    // refuses cleanly rather than pretending to try.
}

async fn query(
    address: IpAddr,
    device_id: &str,
    key: &[u8; 16],
) -> Result<lan_protocol::StateReport> {
    let request = lan_protocol::query_request(device_id);
    let answer = lan_wire::call(
        address,
        lan_wire::CMD_DP_QUERY,
        key,
        &request,
        CALL_TIMEOUT,
    )
    .await?;

    lan_protocol::parse_state(&answer)
}

/// A cached local key is stored as the string Tuya hands out; on the wire it
/// is its own raw bytes used directly as the AES-128 key, not hex-decoded.
fn parse_key(secret: &str) -> Result<[u8; 16]> {
    <[u8; 16]>::try_from(secret.as_bytes()).map_err(|_| {
        Error::Protocol(format!(
            "clé locale de longueur inattendue: {} octets",
            secret.len()
        ))
    })
}

/// A plug reachable directly on the local network.
struct LanEndpoint {
    address: IpAddr,
    device_id: String,
    key: [u8; 16],
    /// The dp this device was found to switch through: see
    /// [`lan_protocol::StateReport::switch_dp`].
    switch_dp: String,
}

#[async_trait]
impl Endpoint for LanEndpoint {
    fn transport(&self) -> Transport {
        Transport::Lan
    }

    fn address(&self) -> String {
        self.address.to_string()
    }

    async fn state(&self) -> Result<LightState> {
        let report = query(self.address, &self.device_id, &self.key).await?;
        Ok(report.to_light_state())
    }

    async fn apply(&self, commands: &[Command]) -> Result<LightState> {
        let mut on = None;
        for command in commands {
            match command {
                Command::Power(power) => on = Some(*power),
                Command::Brightness(_)
                | Command::Color(_)
                | Command::Effect(_) => {
                    return Err(Error::Unsupported(
                        "cette prise ne sait faire qu'allumer et éteindre"
                            .into(),
                    ));
                }
            }
        }

        let Some(on) = on else {
            return self.state().await;
        };

        let request =
            lan_protocol::control_request(&self.device_id, &self.switch_dp, on);
        lan_wire::call(
            self.address,
            lan_wire::CMD_CONTROL,
            &self.key,
            &request,
            CALL_TIMEOUT,
        )
        .await?;

        self.state().await
    }
}
