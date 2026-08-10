//! Framing and transport for Tuya's local protocol, versions 3.2 and 3.3.
//!
//! Discovery is a passive listen on two UDP ports for devices' own
//! unprompted beacons; control is a TCP connection on port 6668 carrying
//! AES-encrypted, CRC32-checked frames. Both share the same envelope: a
//! 16-byte header (`prefix`, `seqno`, `cmd`, `length`, each a big-endian
//! `u32`), the body, then a trailing CRC32 and a 4-byte suffix; an answer's
//! body additionally starts with a 4-byte return code the request has no
//! room for.

use std::net::{IpAddr, SocketAddr};
use std::time::Duration;

use homelumen_core::{Error, Result};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpStream, UdpSocket};
use tokio::time::Instant;

use crate::lan_crypto;

/// Passive discovery, unencrypted: pre-3.2 devices only.
pub const DISCOVERY_PORT_PLAIN: u16 = 6666;
/// Passive discovery, obscured with the fixed [`lan_crypto::discovery_key`].
pub const DISCOVERY_PORT_ENCRYPTED: u16 = 6667;
/// Where a device takes a TCP connection for control.
pub const CONTROL_PORT: u16 = 6668;

/// Reads every `dps` a device currently reports.
pub const CMD_DP_QUERY: u32 = 0x0A;
/// Sets one or more `dps`.
pub const CMD_CONTROL: u32 = 0x07;

const PREFIX: u32 = 0x0000_55AA;
const SUFFIX: u32 = 0x0000_AA55;

const HEADER_LEN: usize = 16;
const RETCODE_LEN: usize = 4;
const CRC_LEN: usize = 4;
const SUFFIX_LEN: usize = 4;
const RESPONSE_OVERHEAD: usize = RETCODE_LEN + CRC_LEN + SUFFIX_LEN;

/// Refuse absurd frames rather than allocating whatever the peer announces.
const MAX_FRAME: u32 = 64 * 1024;

/// Largest datagram a beacon is expected to fit in.
const DATAGRAM: usize = 4096;

/// A discovery datagram answered by one device, its envelope already
/// stripped and, for the encrypted port, already decrypted; still the raw
/// JSON bytes, not yet parsed.
pub struct Beacon {
    pub address: IpAddr,
    pub payload: Vec<u8>,
}

/// Listens on both discovery ports for `window` and collects every beacon
/// heard, decrypting the ones that need it.
pub async fn listen(window: Duration) -> Result<Vec<Beacon>> {
    let plain = UdpSocket::bind(("0.0.0.0", DISCOVERY_PORT_PLAIN)).await;
    let encrypted =
        UdpSocket::bind(("0.0.0.0", DISCOVERY_PORT_ENCRYPTED)).await;

    if plain.is_err() && encrypted.is_err() {
        return Err(Error::Unreachable(format!(
            "impossible d'écouter la découverte locale Tuya: {} / {}",
            plain.expect_err("checked above"),
            encrypted.expect_err("checked above"),
        )));
    }

    let (mut from_plain, from_encrypted) = tokio::join!(
        collect(plain.ok(), window, None),
        collect(encrypted.ok(), window, Some(lan_crypto::discovery_key())),
    );

    from_plain.extend(from_encrypted);
    Ok(from_plain)
}

async fn collect(
    socket: Option<UdpSocket>,
    window: Duration,
    key: Option<[u8; 16]>,
) -> Vec<Beacon> {
    let Some(socket) = socket else {
        return Vec::new();
    };

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
            Ok(Ok((length, from))) => {
                match decode_beacon(&buffer[..length], key.as_ref()) {
                    Ok(payload) => {
                        beacons.push(Beacon { address: from.ip(), payload })
                    }
                    Err(error) => {
                        log::debug!("{from} a annoncé n'importe quoi: {error}")
                    }
                }
            }
            Ok(Err(error)) => {
                log::debug!("écoute locale interrompue: {error}");
                break;
            }
            Err(_) => break,
        }
    }

    beacons
}

fn decode_beacon(raw: &[u8], key: Option<&[u8; 16]>) -> Result<Vec<u8>> {
    if raw.len() < HEADER_LEN {
        return Err(Error::Protocol("balise tronquée".into()));
    }

    let (header, rest) = raw.split_at(HEADER_LEN);
    let payload = unwrap_frame(header, rest)?;

    match key {
        Some(key) => lan_crypto::decrypt(key, &payload),
        None => Ok(payload),
    }
}

/// Sends `json` to `address` as a `cmd` command, encrypted with `key`, and
/// returns the decrypted JSON bytes of the answer.
pub async fn call(
    address: IpAddr,
    cmd: u32,
    key: &[u8; 16],
    json: &[u8],
    timeout: Duration,
) -> Result<Vec<u8>> {
    let address = SocketAddr::new(address, CONTROL_PORT);

    tokio::time::timeout(timeout, exchange(address, cmd, key, json))
        .await
        .map_err(|_| Error::Unreachable(format!("{address} n'a pas répondu")))?
}

async fn exchange(
    address: SocketAddr,
    cmd: u32,
    key: &[u8; 16],
    json: &[u8],
) -> Result<Vec<u8>> {
    let mut stream = TcpStream::connect(address)
        .await
        .map_err(|error| Error::Unreachable(format!("{address}: {error}")))?;
    stream
        .set_nodelay(true)
        .map_err(|error| Error::Unreachable(format!("{address}: {error}")))?;

    let frame = build_request(cmd, key, json);
    stream
        .write_all(&frame)
        .await
        .map_err(|error| Error::Unreachable(format!("{address}: {error}")))?;
    stream
        .flush()
        .await
        .map_err(|error| Error::Unreachable(format!("{address}: {error}")))?;

    let mut header = [0_u8; HEADER_LEN];
    stream
        .read_exact(&mut header)
        .await
        .map_err(|error| Error::Unreachable(format!("{address}: {error}")))?;

    let length = u32::from_be_bytes(header[12..16].try_into().unwrap());
    if !(RESPONSE_OVERHEAD as u32..=MAX_FRAME).contains(&length) {
        return Err(Error::Protocol(format!(
            "taille de trame invalide: {length}"
        )));
    }

    let mut rest = vec![0_u8; length as usize];
    stream
        .read_exact(&mut rest)
        .await
        .map_err(|error| Error::Unreachable(format!("{address}: {error}")))?;

    let payload = unwrap_frame(&header, &rest)?;
    let payload = strip_clear_header(&payload);

    if payload.is_empty() {
        return Ok(Vec::new());
    }

    lan_crypto::decrypt(key, payload)
}

/// Builds a full 55AA frame: header, the version marker `CONTROL` needs in
/// the clear ahead of its ciphertext (see the module doc of
/// [`crate::lan_protocol`] for why only `CONTROL` carries it), the
/// encrypted body, a CRC32 over everything before it, and the suffix.
fn build_request(cmd: u32, key: &[u8; 16], json: &[u8]) -> Vec<u8> {
    const SEQNO: u32 = 1;

    let ciphertext = lan_crypto::encrypt(key, json);

    let mut body = Vec::with_capacity(ciphertext.len() + 15);
    if cmd == CMD_CONTROL {
        body.extend_from_slice(b"3.3");
        body.extend_from_slice(&[0_u8; 12]);
    }
    body.extend_from_slice(&ciphertext);

    let length = (body.len() + CRC_LEN + SUFFIX_LEN) as u32;

    let mut frame =
        Vec::with_capacity(HEADER_LEN + body.len() + CRC_LEN + SUFFIX_LEN);
    frame.extend_from_slice(&PREFIX.to_be_bytes());
    frame.extend_from_slice(&SEQNO.to_be_bytes());
    frame.extend_from_slice(&cmd.to_be_bytes());
    frame.extend_from_slice(&length.to_be_bytes());
    frame.extend_from_slice(&body);

    let crc = crc32fast::hash(&frame);
    frame.extend_from_slice(&crc.to_be_bytes());
    frame.extend_from_slice(&SUFFIX.to_be_bytes());

    frame
}

/// Checks a frame's prefix, trailing CRC32 and suffix, turns a non-zero
/// return code into [`Error::Rejected`], and returns whatever sits between
/// the (optional) return code and the CRC: still encrypted, for the caller
/// to decrypt with whichever key applies.
fn unwrap_frame(header: &[u8], rest: &[u8]) -> Result<Vec<u8>> {
    let prefix = u32::from_be_bytes(header[0..4].try_into().unwrap());
    if prefix != PREFIX {
        return Err(Error::Protocol("préfixe de trame inattendu".into()));
    }

    if rest.len() < RESPONSE_OVERHEAD {
        return Err(Error::Protocol("trame tronquée".into()));
    }

    let suffix_at = rest.len() - SUFFIX_LEN;
    let crc_at = suffix_at - CRC_LEN;

    let suffix = u32::from_be_bytes(rest[suffix_at..].try_into().unwrap());
    if suffix != SUFFIX {
        return Err(Error::Protocol("suffixe de trame inattendu".into()));
    }

    let mut hasher = crc32fast::Hasher::new();
    hasher.update(header);
    hasher.update(&rest[..crc_at]);
    let expected =
        u32::from_be_bytes(rest[crc_at..suffix_at].try_into().unwrap());
    if hasher.finalize() != expected {
        return Err(Error::Protocol("somme de contrôle invalide".into()));
    }

    let retcode = u32::from_be_bytes(rest[..RETCODE_LEN].try_into().unwrap());
    if retcode != 0 {
        return Err(Error::Rejected(format!("code {retcode}")));
    }

    Ok(rest[RETCODE_LEN..crc_at].to_vec())
}

/// Strips the clear-text version marker a `CONTROL` request carries, if the
/// device echoed one back; `DP_QUERY` answers never have one, so this is a
/// no-op for them.
fn strip_clear_header(payload: &[u8]) -> &[u8] {
    const MARK_LEN: usize = 15;

    let is_marked = payload.len() >= MARK_LEN
        && &payload[0..3] == b"3.3"
        && payload[3..MARK_LEN].iter().all(|&byte| byte == 0);

    if is_marked { &payload[MARK_LEN..] } else { payload }
}

#[cfg(test)]
mod tests {
    use homelumen_core::Error;

    use super::{
        CMD_CONTROL, CMD_DP_QUERY, PREFIX, SUFFIX, build_request, unwrap_frame,
    };

    /// Hand-builds a device's own answer shape (header, then retcode, body,
    /// CRC32 and suffix), the mirror image of `build_request`, so
    /// `unwrap_frame` can be exercised against bytes nothing in this crate
    /// produced.
    fn response_frame(retcode: u32, body: &[u8]) -> (Vec<u8>, Vec<u8>) {
        let length = (4 + body.len() + 4 + 4) as u32;

        let mut header = Vec::with_capacity(16);
        header.extend_from_slice(&PREFIX.to_be_bytes());
        header.extend_from_slice(&1_u32.to_be_bytes());
        header.extend_from_slice(&CMD_DP_QUERY.to_be_bytes());
        header.extend_from_slice(&length.to_be_bytes());

        let mut rest = Vec::new();
        rest.extend_from_slice(&retcode.to_be_bytes());
        rest.extend_from_slice(body);

        let mut hasher = crc32fast::Hasher::new();
        hasher.update(&header);
        hasher.update(&rest);
        rest.extend_from_slice(&hasher.finalize().to_be_bytes());
        rest.extend_from_slice(&SUFFIX.to_be_bytes());

        (header, rest)
    }

    #[test]
    fn a_well_formed_response_unwraps_to_its_body() {
        let (header, rest) = response_frame(0, b"hello");

        assert_eq!(
            unwrap_frame(&header, &rest).expect("well-formed"),
            b"hello"
        );
    }

    #[test]
    fn a_flipped_byte_fails_the_checksum() {
        let (header, mut rest) = response_frame(0, b"hello");
        let middle = rest.len() / 2;
        rest[middle] ^= 0xFF;

        assert!(unwrap_frame(&header, &rest).is_err());
    }

    #[test]
    fn a_non_zero_retcode_is_rejected() {
        let (header, rest) = response_frame(7, b"");

        assert!(matches!(
            unwrap_frame(&header, &rest),
            Err(Error::Rejected(_))
        ));
    }

    #[test]
    fn a_dp_query_request_carries_recoverable_ciphertext() {
        let key = [3_u8; 16];
        let json = br#"{"gwId":"abc","t":"1"}"#;

        let frame = build_request(CMD_DP_QUERY, &key, json);
        let body_end = frame.len() - 8;
        let ciphertext = &frame[16..body_end];

        assert_eq!(
            crate::lan_crypto::decrypt(&key, ciphertext).expect("valid"),
            json
        );
    }

    #[test]
    fn a_control_request_carries_a_clear_version_marker() {
        let key = [3_u8; 16];
        let frame = build_request(CMD_CONTROL, &key, b"{}");

        assert_eq!(&frame[16..19], b"3.3");
    }
}
