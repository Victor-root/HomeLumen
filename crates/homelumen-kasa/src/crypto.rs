//! The obfuscation TP-Link puts on the wire.
//!
//! Every byte is XORed with the previous *ciphertext* byte, the first one with
//! a fixed seed. It is not encryption and TP-Link never claimed it was; it is
//! simply the format the devices expect.

const SEED: u8 = 0xAB;

/// Obfuscates a request payload.
pub fn encrypt(plain: &[u8]) -> Vec<u8> {
    let mut key = SEED;
    plain
        .iter()
        .map(|byte| {
            key ^= byte;
            key
        })
        .collect()
}

/// Restores a response payload.
pub fn decrypt(cipher: &[u8]) -> Vec<u8> {
    let mut key = SEED;
    cipher
        .iter()
        .map(|byte| {
            let plain = key ^ byte;
            key = *byte;
            plain
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::{decrypt, encrypt};

    #[test]
    fn round_trip_restores_the_payload() {
        let payload = br#"{"system":{"get_sysinfo":{}}}"#;
        assert_eq!(decrypt(&encrypt(payload)), payload);
    }

    #[test]
    fn matches_the_wire_format() {
        // First byte of the canonical `{"system":...}` request, as captured
        // from a Kasa bulb: 0xAB ^ b'{'.
        assert_eq!(encrypt(b"{")[0], 0xAB ^ b'{');
    }
}
