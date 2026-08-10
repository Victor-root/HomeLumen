//! Signed HTTP calls against Tuya's Cloud API.

use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use homelumen_core::{Account, Error, Result};
use reqwest::{Method, RequestBuilder};
use serde::Deserialize;
use serde::de::DeserializeOwned;
use tokio::sync::Mutex;
use uuid::Uuid;

use crate::protocol::TokenResult;
use crate::signing::{self, Signable};

/// One region's Tuya Cloud host.
#[derive(Debug, Clone, Copy)]
pub struct DataCenter(&'static str);

impl DataCenter {
    /// Central Europe: where a Smart Life account created in France lives.
    pub const EUROPE: Self = Self("https://openapi.tuyaeu.com");
}

/// How long an access token is trusted before it is fetched again, kept a
/// margin short of the two hours Tuya actually grants it for.
const TOKEN_MARGIN: Duration = Duration::from_secs(300);

struct CachedToken {
    access_token: String,
    good_until: Instant,
}

/// The credentials and endpoint every signed call needs, and the one access
/// token every plug shares.
///
/// Cheap to clone: the token cache lives behind an [`Arc`], so every clone
/// still refreshes the same token instead of each fetching its own.
#[derive(Clone)]
pub struct Session {
    http: reqwest::Client,
    center: DataCenter,
    account: Account,
    token: Arc<Mutex<Option<CachedToken>>>,
}

impl Session {
    pub fn new(center: DataCenter, account: Account) -> Self {
        Self {
            http: reqwest::Client::new(),
            center,
            account,
            token: Arc::new(Mutex::new(None)),
        }
    }

    /// A call made as HomeLumen's own application, fetching and caching the
    /// access token this needs behind the scenes.
    pub async fn call_as_app<T: DeserializeOwned>(
        &self,
        method: Method,
        path_and_query: &str,
        body: &[u8],
    ) -> Result<T> {
        let token = self.app_token().await?;
        self.call(method, path_and_query, &token, body).await
    }

    async fn app_token(&self) -> Result<String> {
        let mut cached = self.token.lock().await;

        if let Some(token) = cached.as_ref()
            && token.good_until > Instant::now()
        {
            return Ok(token.access_token.clone());
        }

        let result: TokenResult = self
            .call_unauthenticated(Method::GET, "/v1.0/token?grant_type=1")
            .await?;

        let good_until = Instant::now()
            + Duration::from_secs(result.expire_time)
                .saturating_sub(TOKEN_MARGIN);
        let TokenResult { access_token, .. } = result;

        *cached = Some(CachedToken {
            access_token: access_token.clone(),
            good_until,
        });

        Ok(access_token)
    }

    /// A call signed with the app's own secret alone, before any user token
    /// exists: only the token endpoint is ever called this way.
    async fn call_unauthenticated<T: DeserializeOwned>(
        &self,
        method: Method,
        path_and_query: &str,
    ) -> Result<T> {
        let (t, nonce) = freshness();
        let signable = Signable {
            method: method.as_str(),
            url: path_and_query,
            body: b"",
            headers: &[],
        };
        let sign = signing::sign_token_request(
            &self.account.id,
            &self.account.secret,
            t,
            &nonce,
            &signable,
        );

        let request =
            self.authorize(method, path_and_query, t, &nonce, &sign, None);
        send(request).await
    }

    /// A call signed with an access token, on top of the app's own secret.
    async fn call<T: DeserializeOwned>(
        &self,
        method: Method,
        path_and_query: &str,
        access_token: &str,
        body: &[u8],
    ) -> Result<T> {
        let (t, nonce) = freshness();
        let signable = Signable {
            method: method.as_str(),
            url: path_and_query,
            body,
            headers: &[],
        };
        let sign = signing::sign_request(
            &self.account.id,
            &self.account.secret,
            access_token,
            t,
            &nonce,
            &signable,
        );

        let mut request = self.authorize(
            method,
            path_and_query,
            t,
            &nonce,
            &sign,
            Some(access_token),
        );

        if !body.is_empty() {
            request = request
                .header("Content-Type", "application/json")
                .body(body.to_vec());
        }

        send(request).await
    }

    fn authorize(
        &self,
        method: Method,
        path_and_query: &str,
        t: u64,
        nonce: &str,
        sign: &str,
        access_token: Option<&str>,
    ) -> RequestBuilder {
        let url = format!("{}{path_and_query}", self.center.0);

        let mut request = self
            .http
            .request(method, url)
            .header("client_id", &self.account.id)
            .header("sign", sign)
            .header("sign_method", "HMAC-SHA256")
            .header("t", t.to_string())
            .header("nonce", nonce);

        if let Some(access_token) = access_token {
            request = request.header("access_token", access_token);
        }

        request
    }
}

async fn send<T: DeserializeOwned>(request: RequestBuilder) -> Result<T> {
    let response = request
        .send()
        .await
        .map_err(|error| Error::Unreachable(error.to_string()))?;

    let envelope: Envelope<T> = response
        .json()
        .await
        .map_err(|error| Error::Protocol(error.to_string()))?;

    if !envelope.success {
        let reason = envelope
            .msg
            .unwrap_or_else(|| format!("code {}", envelope.code.unwrap_or(0)));
        return Err(Error::Rejected(reason));
    }

    envelope
        .result
        .ok_or_else(|| Error::Protocol("réponse sans résultat".into()))
}

/// A timestamp and nonce, minted fresh for one request: Tuya rejects a
/// signature computed from a stale or reused pair.
fn freshness() -> (u64, String) {
    let t = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("the system clock reads after 1970")
        .as_millis() as u64;

    (t, Uuid::new_v4().simple().to_string())
}

#[derive(Deserialize)]
struct Envelope<T> {
    success: bool,
    result: Option<T>,
    msg: Option<String>,
    code: Option<i64>,
}
