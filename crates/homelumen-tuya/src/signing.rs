//! Tuya's request-signing algorithm.
//!
//! Every call, including the token fetch itself, is signed the same way: a
//! string built from the method, the body's hash and the URL, turned into an
//! HMAC-SHA256 keyed on the app's own secret. See
//! <https://developer.tuya.com/en/docs/iot/new-singnature>.

use hmac::{Hmac, Mac};
use sha2::{Digest, Sha256};

type HmacSha256 = Hmac<Sha256>;

/// Everything a signature is computed over, besides who is asking and when.
pub struct Signable<'a> {
    pub method: &'a str,
    /// Path and sorted query string, e.g. `/v1.0/token?grant_type=1`.
    pub url: &'a str,
    pub body: &'a [u8],
    /// Extra headers folded into the signature, in the order they must
    /// appear. Empty for every call HomeLumen makes today; kept so the
    /// algorithm matches Tuya's own documentation in full, not just the
    /// slice of it this driver happens to use.
    pub headers: &'a [(&'a str, &'a str)],
}

impl Signable<'_> {
    fn string_to_sign(&self) -> String {
        let mut signed_headers = String::new();
        for (key, value) in self.headers {
            signed_headers.push_str(key);
            signed_headers.push(':');
            signed_headers.push_str(value);
            signed_headers.push('\n');
        }

        format!(
            "{}\n{}\n{signed_headers}\n{}",
            self.method,
            content_sha256(self.body),
            self.url,
        )
    }
}

fn content_sha256(body: &[u8]) -> String {
    hex::encode(Sha256::digest(body))
}

/// Signs a token request: nothing yet proves who is asking, so the secret
/// alone stands for the app.
pub fn sign_token_request(
    client_id: &str,
    secret: &str,
    t: u64,
    nonce: &str,
    request: &Signable,
) -> String {
    let message = format!("{client_id}{t}{nonce}{}", request.string_to_sign());
    hmac_hex(secret, &message)
}

/// Signs a request already carrying a token.
pub fn sign_request(
    client_id: &str,
    secret: &str,
    access_token: &str,
    t: u64,
    nonce: &str,
    request: &Signable,
) -> String {
    let message = format!(
        "{client_id}{access_token}{t}{nonce}{}",
        request.string_to_sign()
    );
    hmac_hex(secret, &message)
}

fn hmac_hex(secret: &str, message: &str) -> String {
    let mut mac = HmacSha256::new_from_slice(secret.as_bytes())
        .expect("HMAC-SHA256 accepts a key of any length");
    mac.update(message.as_bytes());
    hex::encode_upper(mac.finalize().into_bytes())
}

#[cfg(test)]
mod tests {
    use super::{Signable, sign_request, sign_token_request};

    // The worked example from Tuya's own signing documentation, reproduced
    // exactly so a broken signature is caught here rather than on a real
    // account.
    const CLIENT_ID: &str = "1KAD46OrT9HafiKdsXeg";
    const SECRET: &str = "4OHBOnWOqaEC1mWXOpVL3yV50s0qGSRC";
    const T: u64 = 1588925778000;
    const NONCE: &str = "5138cc3a9033d69856923fd07b491173";
    const HEADERS: &[(&str, &str)] = &[
        ("area_id", "29a33e8796834b1efa6"),
        ("call_id", "8afdb70ab2ed11eb85290242ac130003"),
    ];

    #[test]
    fn matches_the_documented_token_request_example() {
        let request = Signable {
            method: "GET",
            url: "/v1.0/token?grant_type=1",
            body: b"",
            headers: HEADERS,
        };

        let sign = sign_token_request(CLIENT_ID, SECRET, T, NONCE, &request);

        assert_eq!(
            sign,
            "9E48A3E93B302EEECC803C7241985D0A34EB944F40FB573C7B5C2A82158AF13E"
        );
    }

    #[test]
    fn matches_the_documented_general_request_example() {
        let request = Signable {
            method: "GET",
            url: "/v2.0/apps/schema/users?page_no=1&page_size=50",
            body: b"",
            headers: HEADERS,
        };

        let sign = sign_request(
            CLIENT_ID,
            SECRET,
            "3f4eda2bdec17232f67c0b188af3eec1",
            T,
            NONCE,
            &request,
        );

        assert_eq!(
            sign,
            "AE4481C692AA80B25F3A7E12C3A5FD9BBF6251539DD78E565A1A72A508A88784"
        );
    }
}
