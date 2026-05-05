#![allow(clippy::doc_markdown)] // "NetBox" is a product name, not a code symbol
#![recursion_limit = "256"]
//! NetBox REST API client.
//!
//! # Example
//!
//! ```no_run
//! # async fn example() -> netbox_client::Result<()> {
//! let client = netbox_client::NetboxClient::new("https://netbox.example.com", "abc123token")?;
//! let user = client.authentication_check().await?;
//! println!("{user:?}");
//! # Ok(()) }
//! ```

// ── JSON backend mutual exclusion ──────────────────────────────────────────

#[cfg(all(feature = "serde_json", feature = "sonic-rs"))]
compile_error!("features `serde_json` and `sonic-rs` are mutually exclusive; enable only one");
#[cfg(all(feature = "serde_json", feature = "simd-json"))]
compile_error!("features `serde_json` and `simd-json` are mutually exclusive; enable only one");
#[cfg(all(feature = "sonic-rs", feature = "simd-json"))]
compile_error!("features `sonic-rs` and `simd-json` are mutually exclusive; enable only one");
#[cfg(not(any(feature = "serde_json", feature = "sonic-rs", feature = "simd-json")))]
compile_error!(
    "at least one of the `serde_json`, `sonic-rs`, or `simd-json` features must be enabled"
);

// ── JSON backend shim ──────────────────────────────────────────────────────

#[cfg(feature = "serde_json")]
pub(crate) mod json {
    pub use serde_json::{from_slice, to_vec, Error, Value};
}

#[cfg(feature = "sonic-rs")]
pub(crate) mod json {
    pub use sonic_rs::{from_slice, to_vec, Error, Value};
}

#[cfg(feature = "simd-json")]
pub(crate) mod json {
    pub use simd_json::{to_vec, Error, OwnedValue as Value};
    pub fn from_slice<T>(input: &[u8]) -> Result<T, Error>
    where
        T: for<'de> serde::Deserialize<'de>,
    {
        let mut bytes = input.to_vec();
        simd_json::serde::from_slice(&mut bytes)
    }
}

/// The JSON value type from the active backend.
pub use json::Value as JsonValue;
/// The JSON error type from the active backend.
pub use json::Error as JsonError;

// ── Imports ────────────────────────────────────────────────────────────────

use log::{debug, log_enabled, Level};

pub mod models;
pub use models::*;

pub mod circuits;
pub mod dcim;
pub mod ipam;

pub type Result<T> = std::result::Result<T, Error>;

// ── Error ──────────────────────────────────────────────────────────────────

/// Crate-level error type.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// An HTTP transport error.
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),

    /// A non-2xx response from the NetBox API.
    #[error("API error {status}: {body}")]
    Api {
        status: reqwest::StatusCode,
        body: ApiError,
    },

    /// A JSON encode/decode error from the active backend.
    #[error("JSON error: {0}")]
    Json(#[from] json::Error),
}

/// Error body returned by NetBox on non-2xx responses.
#[derive(Debug, Clone, serde::Deserialize, Default)]
pub struct ApiError {
    /// Human-readable error detail.
    pub detail: Option<String>,
}

impl std::fmt::Display for ApiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.detail {
            Some(d) => f.write_str(d),
            None => f.write_str("(no detail)"),
        }
    }
}

// ── Debug-logged HTTP helpers ──────────────────────────────────────────────

fn log_request(req: &reqwest::Request) {
    debug!("--> {} {}", req.method(), req.url());
    for (name, value) in req.headers() {
        debug!("    {}: {}", name, value.to_str().unwrap_or("<binary>"));
    }
    if let Some(bytes) = req.body().and_then(reqwest::Body::as_bytes) {
        if !bytes.is_empty() {
            debug!("    {}", String::from_utf8_lossy(bytes));
        }
    }
}

fn log_response_head(resp: &reqwest::Response) {
    debug!("<-- {}", resp.status());
    for (name, value) in resp.headers() {
        debug!("    {}: {}", name, value.to_str().unwrap_or("<binary>"));
    }
}

pub(crate) trait RequestBuilderExt {
    /// Serialize `body` via the active JSON backend and attach it as the
    /// request body with `Content-Type: application/json`.
    fn body_json<B: serde::Serialize>(self, body: &B) -> Result<Self>
    where
        Self: Sized;

    /// Send the request, check for non-2xx, and deserialize the body via the
    /// active JSON backend. Logs the full request/response at DEBUG when enabled.
    async fn send_json<T>(self) -> Result<T>
    where
        T: for<'de> serde::Deserialize<'de>;
}

impl RequestBuilderExt for reqwest::RequestBuilder {
    fn body_json<B: serde::Serialize>(self, body: &B) -> Result<Self> {
        let bytes = json::to_vec(body).map_err(Error::Json)?;
        Ok(self.header("Content-Type", "application/json").body(bytes))
    }

    async fn send_json<T>(self) -> Result<T>
    where
        T: for<'de> serde::Deserialize<'de>,
    {
        let do_log = log_enabled!(Level::Debug);

        if do_log {
            if let Some(snapshot) = self.try_clone() {
                if let Ok(req) = snapshot.build() {
                    log_request(&req);
                }
            }
        }

        let resp = self.send().await?;

        if do_log {
            log_response_head(&resp);
        }

        let status = resp.status();
        let bytes = resp.bytes().await?;

        if do_log && !bytes.is_empty() {
            debug!("    {}", String::from_utf8_lossy(&bytes));
        }

        if !status.is_success() {
            let body = json::from_slice::<ApiError>(&bytes).unwrap_or_default();
            return Err(Error::Api { status, body });
        }

        json::from_slice::<T>(&bytes).map_err(Error::Json)
    }
}

// ── ClientBuilder ──────────────────────────────────────────────────────────

/// Builder for constructing a [`NetboxClient`] with advanced HTTP settings.
///
/// Obtain via [`NetboxClient::builder`].
#[derive(Debug, Default)]
pub struct ClientBuilder {
    accept_invalid_certs: bool,
}

impl ClientBuilder {
    /// Creates a new builder with default settings (TLS verification enabled).
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Disables TLS certificate verification.
    ///
    /// **Security warning**: only use on trusted private networks with
    /// self-signed certificates.
    #[must_use]
    pub fn danger_accept_invalid_certs(mut self, accept: bool) -> Self {
        self.accept_invalid_certs = accept;
        self
    }

    /// Builds the [`NetboxClient`].
    ///
    /// # Errors
    ///
    /// Returns an error if the underlying HTTP client cannot be constructed.
    pub fn build(
        self,
        base_url: impl Into<String>,
        token: impl Into<String>,
    ) -> Result<NetboxClient> {
        let http = reqwest::Client::builder()
            .danger_accept_invalid_certs(self.accept_invalid_certs)
            .build()
            .map_err(Error::Http)?;
        Ok(NetboxClient {
            base_url: base_url.into().trim_end_matches('/').to_owned(),
            token: token.into(),
            http,
        })
    }
}

// ── NetboxClient ───────────────────────────────────────────────────────────

/// Async client for the NetBox REST API.
#[derive(Debug)]
pub struct NetboxClient {
    base_url: String,
    token: String,
    http: reqwest::Client,
}

impl NetboxClient {
    /// Creates a new client with default settings.
    ///
    /// For advanced configuration (e.g. disabling TLS verification) use
    /// [`NetboxClient::builder`] instead.
    ///
    /// # Errors
    ///
    /// Returns an error if the underlying HTTP client cannot be constructed.
    pub fn new(base_url: impl Into<String>, token: impl Into<String>) -> Result<Self> {
        Self::builder().build(base_url, token)
    }

    /// Returns a [`ClientBuilder`] for advanced HTTP configuration.
    #[must_use]
    pub fn builder() -> ClientBuilder {
        ClientBuilder::new()
    }

    // ── Authentication ──────────────────────────────────────────────────────

    /// Returns the authenticated user if the token is valid.
    ///
    /// Maps to `GET /api/authentication-check/`
    /// (`operationId`: `authentication_check_retrieve`).
    ///
    /// # Errors
    ///
    /// Returns an error if the HTTP request fails or the token is rejected
    /// (e.g. 403 with `{"detail": "Invalid token."}`).
    pub async fn authentication_check(&self) -> Result<JsonValue> {
        let url = format!("{}/api/authentication-check/", self.base_url);
        self.http
            .get(&url)
            .header("Authorization", format!("Token {}", self.token))
            .send_json::<JsonValue>()
            .await
    }
}

// ── Shared helpers ─────────────────────────────────────────────────────────

/// Shared helper for DELETE endpoints that return 204 No Content.
pub(crate) async fn delete_no_content(
    http: &reqwest::Client,
    url: &str,
    token: &str,
) -> Result<()> {
    let resp = http
        .delete(url)
        .header("Authorization", format!("Token {token}"))
        .send()
        .await
        .map_err(Error::Http)?;
    if resp.status().is_success() {
        Ok(())
    } else {
        let status = resp.status();
        let bytes = resp.bytes().await.map_err(Error::Http)?;
        let body = json::from_slice::<ApiError>(&bytes).unwrap_or_default();
        Err(Error::Api { status, body })
    }
}

// ── Tests ──────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use wiremock::{
        matchers::{header, method, path},
        Mock, MockServer, ResponseTemplate,
    };

    #[tokio::test]
    async fn authentication_check_returns_user_on_valid_token() {
        let server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/api/authentication-check/"))
            .and(header("Authorization", "Token secret123"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "id": 1,
                "url": "https://netbox.example.com/api/users/users/1/",
                "display": "admin",
                "username": "admin"
            })))
            .mount(&server)
            .await;

        let client = NetboxClient::new(server.uri(), "secret123").unwrap();
        let result = client.authentication_check().await.unwrap();
        assert_eq!(result["username"], "admin");
    }

    #[tokio::test]
    async fn authentication_check_returns_api_error_on_invalid_token() {
        let server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/api/authentication-check/"))
            .respond_with(ResponseTemplate::new(403).set_body_json(serde_json::json!({
                "detail": "Invalid token."
            })))
            .mount(&server)
            .await;

        let client = NetboxClient::new(server.uri(), "badtoken").unwrap();
        let err = client.authentication_check().await.unwrap_err();
        match err {
            Error::Api { status, body } => {
                assert_eq!(status, reqwest::StatusCode::FORBIDDEN);
                assert_eq!(body.detail.as_deref(), Some("Invalid token."));
            }
            other => panic!("expected Error::Api, got {other:?}"),
        }
    }
}
