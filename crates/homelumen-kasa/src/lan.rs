//! Reaching Kasa bulbs directly on the local network.

use std::net::{IpAddr, SocketAddr};
use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use homelumen_core::{
    Command, Discovered, DiscoverySink, Endpoint, Error, LightState, Provider,
    Result, Transport,
};

use crate::protocol::{self, DRIVER};
use crate::wire;

/// How long a device is given to answer a direct request.
const CALL_TIMEOUT: Duration = Duration::from_millis(2500);

/// How long a discovery sweep listens for answers.
const SWEEP_WINDOW: Duration = Duration::from_millis(2200);

/// Finds Kasa bulbs by broadcasting on the local networks.
#[derive(Debug, Default, Clone, Copy)]
pub struct LanProvider;

#[async_trait]
impl Provider for LanProvider {
    fn id(&self) -> &'static str {
        DRIVER
    }

    fn name(&self) -> &'static str {
        "TP-Link Kasa, réseau local"
    }

    async fn discover(&self, sink: DiscoverySink) -> Result<()> {
        let beacons =
            wire::broadcast(&protocol::sysinfo_request(), SWEEP_WINDOW).await?;

        for beacon in beacons {
            let info = match protocol::parse_sysinfo(&beacon.payload) {
                Ok(info) => info,
                Err(error) => {
                    log::debug!(
                        "{} a répondu n'importe quoi: {error}",
                        beacon.address
                    );
                    continue;
                }
            };

            if !info.is_light() {
                continue;
            }

            let descriptor = match info.to_descriptor() {
                Ok(descriptor) => descriptor,
                Err(error) => {
                    log::debug!("{} inutilisable: {error}", beacon.address);
                    continue;
                }
            };

            let found = Discovered {
                descriptor,
                state: info.to_light_state(),
                endpoint: Arc::new(LanEndpoint::new(beacon.address.ip())),
            };

            if sink.send(found).await.is_err() {
                break;
            }
        }

        Ok(())
    }

    async fn probe(&self, address: IpAddr) -> Result<Discovered> {
        let endpoint = LanEndpoint::new(address);
        let answer = wire::call(
            endpoint.socket,
            &protocol::sysinfo_request(),
            CALL_TIMEOUT,
        )
        .await?;
        let info = protocol::parse_sysinfo(&answer)?;

        if !info.is_light() {
            return Err(Error::Unsupported(
                "cet appareil Kasa n'est pas une lumière".into(),
            ));
        }

        Ok(Discovered {
            descriptor: info.to_descriptor()?,
            state: info.to_light_state(),
            endpoint: Arc::new(endpoint),
        })
    }
}

/// A bulb reachable at a known address on the local network.
#[derive(Debug, Clone, Copy)]
struct LanEndpoint {
    socket: SocketAddr,
}

impl LanEndpoint {
    fn new(address: IpAddr) -> Self {
        Self { socket: SocketAddr::new(address, wire::PORT) }
    }
}

#[async_trait]
impl Endpoint for LanEndpoint {
    fn transport(&self) -> Transport {
        Transport::Lan
    }

    fn address(&self) -> String {
        self.socket.ip().to_string()
    }

    async fn state(&self) -> Result<LightState> {
        let answer =
            wire::call(self.socket, &protocol::sysinfo_request(), CALL_TIMEOUT)
                .await?;
        Ok(protocol::parse_sysinfo(&answer)?.to_light_state())
    }

    async fn apply(&self, commands: &[Command]) -> Result<LightState> {
        let request = protocol::transition_request(commands)?;
        let answer = wire::call(self.socket, &request, CALL_TIMEOUT).await?;
        protocol::parse_transition(&answer)
    }
}
