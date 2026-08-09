//! Framing and transport for the Kasa local protocol.
//!
//! Discovery is a single obfuscated datagram broadcast on UDP/9999; control is
//! the same payload over TCP/9999, prefixed by its length.

use std::io;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::time::Duration;

use homelumen_core::{Error, Result};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpStream, UdpSocket};
use tokio::time::Instant;

use crate::crypto;

/// The single port every Kasa device listens on.
pub const PORT: u16 = 9999;

/// Refuse absurd frames rather than allocating whatever the peer announces.
const MAX_FRAME: u32 = 256 * 1024;

/// Largest answer a device sends to a discovery datagram.
const DATAGRAM: usize = 4096;

/// A datagram answered by one device during a discovery sweep.
pub struct Beacon {
    /// Where the answer came from.
    pub address: SocketAddr,
    /// The decoded JSON body.
    pub payload: Vec<u8>,
}

/// Sends `payload` to a device over TCP and returns its answer.
pub async fn call(
    address: SocketAddr,
    payload: &[u8],
    timeout: Duration,
) -> Result<Vec<u8>> {
    tokio::time::timeout(timeout, exchange(address, payload))
        .await
        .map_err(|_| Error::Unreachable(format!("{address} n'a pas répondu")))?
        .map_err(|error| Error::Unreachable(format!("{address}: {error}")))
}

async fn exchange(address: SocketAddr, payload: &[u8]) -> io::Result<Vec<u8>> {
    let mut stream = TcpStream::connect(address).await?;
    stream.set_nodelay(true)?;

    let body = crypto::encrypt(payload);
    let mut frame = Vec::with_capacity(4 + body.len());
    frame.extend_from_slice(&(body.len() as u32).to_be_bytes());
    frame.extend_from_slice(&body);
    stream.write_all(&frame).await?;
    stream.flush().await?;

    let length = stream.read_u32().await?;
    if length == 0 || length > MAX_FRAME {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("taille de trame invalide: {length}"),
        ));
    }

    let mut answer = vec![0_u8; length as usize];
    stream.read_exact(&mut answer).await?;
    Ok(crypto::decrypt(&answer))
}

/// Broadcasts `payload` on every local network and collects the answers until
/// `window` has elapsed.
///
/// One socket is bound per interface: a Windows 11 machine typically carries a
/// Wi-Fi adapter next to several virtual switches, and a single socket bound to
/// `0.0.0.0` would only reach whichever one owns the default route.
pub async fn broadcast(
    payload: &[u8],
    window: Duration,
) -> Result<Vec<Beacon>> {
    let body = crypto::encrypt(payload);
    let target = SocketAddr::new(IpAddr::V4(Ipv4Addr::BROADCAST), PORT);
    let mut listeners = Vec::new();

    for local in broadcast_interfaces() {
        let socket = match bind_broadcaster(local).await {
            Ok(socket) => socket,
            Err(error) => {
                log::debug!("interface {local} inutilisable: {error}");
                continue;
            }
        };

        if let Err(error) = socket.send_to(&body, target).await {
            log::debug!("diffusion impossible depuis {local}: {error}");
            continue;
        }

        listeners.push(tokio::spawn(collect(socket, window)));
    }

    if listeners.is_empty() {
        return Err(Error::Unreachable(
            "aucune interface réseau ne permet la diffusion".into(),
        ));
    }

    let mut beacons = Vec::new();
    for listener in listeners {
        match listener.await {
            Ok(found) => beacons.extend(found),
            Err(error) => log::debug!("écoute interrompue: {error}"),
        }
    }

    Ok(beacons)
}

async fn collect(socket: UdpSocket, window: Duration) -> Vec<Beacon> {
    let deadline = Instant::now() + window;
    let mut buffer = vec![0_u8; DATAGRAM];
    let mut beacons = Vec::new();

    loop {
        let remaining = deadline.saturating_duration_since(Instant::now());
        if remaining.is_zero() {
            break;
        }

        match tokio::time::timeout(remaining, socket.recv_from(&mut buffer))
            .await
        {
            Ok(Ok((length, address))) => beacons.push(Beacon {
                address,
                payload: crypto::decrypt(&buffer[..length]),
            }),
            Ok(Err(error)) => {
                log::debug!("réception interrompue: {error}");
                break;
            }
            Err(_) => break,
        }
    }

    beacons
}

async fn bind_broadcaster(local: Ipv4Addr) -> io::Result<UdpSocket> {
    let socket = UdpSocket::bind(SocketAddr::new(IpAddr::V4(local), 0)).await?;
    socket.set_broadcast(true)?;
    Ok(socket)
}

fn broadcast_interfaces() -> Vec<Ipv4Addr> {
    let Ok(interfaces) = if_addrs::get_if_addrs() else {
        return vec![Ipv4Addr::UNSPECIFIED];
    };

    let mut addresses: Vec<Ipv4Addr> = interfaces
        .into_iter()
        .filter(|interface| !interface.is_loopback())
        .filter_map(|interface| match interface.ip() {
            IpAddr::V4(address) if !address.is_link_local() => Some(address),
            _ => None,
        })
        .collect();

    addresses.sort();
    addresses.dedup();

    if addresses.is_empty() {
        addresses.push(Ipv4Addr::UNSPECIFIED);
    }

    addresses
}
