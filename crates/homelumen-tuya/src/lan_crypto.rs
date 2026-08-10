//! AES-128-ECB, the cipher Tuya's local protocol uses throughout: for the
//! fixed key that lightly obscures a discovery beacon, and for a paired
//! device's own key once known.

use aes::Aes128;
use ecb::cipher::block_padding::Pkcs7;
use ecb::cipher::{BlockDecryptMut, BlockEncryptMut, KeyInit};
use homelumen_core::{Error, Result};
use md5::{Digest, Md5};

type Encryptor = ecb::Encryptor<Aes128>;
type Decryptor = ecb::Decryptor<Aes128>;

/// The key that lightly obscures a discovery beacon in transit: the MD5
/// digest of a fixed, publicly known string. Not a secret and identical for
/// every Tuya device that exists, reverse-engineered by the `tuya-convert`
/// project and reused unchanged by every open local-Tuya client since.
pub fn discovery_key() -> [u8; 16] {
    Md5::digest(b"yGAdlopoPVldABfn").into()
}

/// Block size of AES, in bytes: how far PKCS7 pads out to.
const BLOCK: usize = 16;

/// Encrypts `plain` with `key`, padding it to a whole number of blocks.
pub fn encrypt(key: &[u8; 16], plain: &[u8]) -> Vec<u8> {
    // PKCS7 always adds between one and `BLOCK` bytes, even to input that is
    // already block-aligned, so the buffer needs room for a whole block past
    // whatever `plain` rounds down to.
    let mut buffer = vec![0_u8; (plain.len() / BLOCK + 1) * BLOCK];
    buffer[..plain.len()].copy_from_slice(plain);

    let ciphertext = Encryptor::new_from_slice(key)
        .expect("a 16-byte key is always valid for AES-128")
        .encrypt_padded_mut::<Pkcs7>(&mut buffer, plain.len())
        .expect("the buffer was sized with room for the padding");

    ciphertext.to_vec()
}

/// Decrypts `cipher` with `key` and strips its padding.
///
/// Fails when `cipher` was not actually encrypted with `key`: the padding
/// that comes out will not, in general, be valid PKCS7.
pub fn decrypt(key: &[u8; 16], cipher: &[u8]) -> Result<Vec<u8>> {
    let mut buffer = cipher.to_vec();

    let plain = Decryptor::new_from_slice(key)
        .expect("a 16-byte key is always valid for AES-128")
        .decrypt_padded_mut::<Pkcs7>(&mut buffer)
        .map_err(|_| Error::Protocol("déchiffrement local invalide".into()))?;

    Ok(plain.to_vec())
}

#[cfg(test)]
mod tests {
    use super::{decrypt, discovery_key, encrypt};

    fn hex_bytes(hex: &str) -> Vec<u8> {
        hex::decode(hex).expect("valid hex")
    }

    /// The worked example from FIPS-197 Appendix B, exercised against the
    /// raw block cipher underneath [`encrypt`]/[`decrypt`] so a broken key
    /// schedule or a swapped byte order is caught here rather than against a
    /// real plug.
    #[test]
    fn matches_the_documented_aes128_test_vector() {
        use aes::Aes128;
        use ecb::cipher::{BlockEncrypt, KeyInit};

        let key = hex_bytes("2b7e151628aed2a6abf7158809cf4f3c");
        let plaintext = hex_bytes("3243f6a8885a308d313198a2e0370734");
        let expected = hex_bytes("3925841d02dc09fbdc118597196a0b32");

        let cipher = Aes128::new_from_slice(&key).expect("16-byte key");
        let mut block = aes::Block::clone_from_slice(&plaintext);
        cipher.encrypt_block(&mut block);

        assert_eq!(block.as_slice(), expected.as_slice());
    }

    #[test]
    fn round_trip_restores_the_payload() {
        let key = [7_u8; 16];
        let payload = br#"{"devId":"abc","dps":{"1":true}}"#;

        let cipher = encrypt(&key, payload);
        assert_eq!(decrypt(&key, &cipher).expect("valid"), payload);
    }

    #[test]
    fn the_wrong_key_does_not_silently_succeed() {
        let payload = b"whatever a device might say back";
        let cipher = encrypt(&[1_u8; 16], payload);

        assert!(decrypt(&[2_u8; 16], &cipher).is_err());
    }

    #[test]
    fn the_discovery_key_is_the_hash_not_the_phrase() {
        // A key that happens to be the right length is not the key: this
        // guards against ever "simplifying" `discovery_key` into the raw
        // ASCII bytes, which are 16 bytes long too but wrong.
        assert_ne!(discovery_key(), *b"yGAdlopoPVldABfn");
    }
}
