//! Async Horizon HTTP client with timeouts, User-Agent, and retries.

use crate::errors::SdkError;
use crate::network::Network;
use crate::USER_AGENT;
use reqwest::header::{HeaderMap, HeaderValue, USER_AGENT as UA};
use reqwest::{Client, Method, Url};
use serde::de::DeserializeOwned;
use std::time::Duration;

/// Default HTTP timeout applied to each Horizon request.
pub const DEFAULT_TIMEOUT: Duration = Duration::from_secs(30);

/// Default number of retries for transient transport failures.
pub const DEFAULT_MAX_RETRIES: u32 = 3;

/// Configurable builder for [`HorizonClient`].
#[derive(Debug, Clone)]
pub struct HorizonClientBuilder {
    base_url: String,
    timeout: Duration,
    max_retries: u32,
    user_agent: String,
}

impl HorizonClientBuilder {
    /// Start a builder targeting `base_url` (Horizon root, no trailing path).
    pub fn new(base_url: impl Into<String>) -> Self {
        Self {
            base_url: base_url.into(),
            timeout: DEFAULT_TIMEOUT,
            max_retries: DEFAULT_MAX_RETRIES,
            user_agent: USER_AGENT.to_string(),
        }
    }

    /// Per-request timeout.
    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Retry count for timeouts, connect errors, 429, and 5xx.
    pub fn max_retries(mut self, max_retries: u32) -> Self {
        self.max_retries = max_retries;
        self
    }

    /// Override the User-Agent header (defaults to `Astral-Stellar-Rust-SDK/{version}`).
    pub fn user_agent(mut self, user_agent: impl Into<String>) -> Self {
        self.user_agent = user_agent.into();
        self
    }

    /// Build the client.
    pub fn build(self) -> Result<HorizonClient, SdkError> {
        let base_url = self.base_url.trim_end_matches('/').to_string();
        Url::parse(&base_url).map_err(|e| SdkError::InvalidUrl(e.to_string()))?;

        let mut headers = HeaderMap::new();
        headers.insert(
            UA,
            HeaderValue::from_str(&self.user_agent)
                .map_err(|e| SdkError::message(format!("invalid user-agent: {e}")))?,
        );

        let inner = Client::builder()
            .timeout(self.timeout)
            .connect_timeout(self.timeout)
            .default_headers(headers)
            .build()
            .map_err(SdkError::from)?;

        Ok(HorizonClient {
            inner,
            base_url,
            timeout: self.timeout,
            max_retries: self.max_retries,
        })
    }
}

/// Asynchronous Horizon REST client.
#[derive(Debug, Clone)]
pub struct HorizonClient {
    inner: Client,
    base_url: String,
    timeout: Duration,
    max_retries: u32,
}

impl HorizonClient {
    /// Client pointed at a well-known [`Network`] Horizon URL.
    pub fn for_network(network: &Network) -> Result<Self, SdkError> {
        let url = network.horizon_url().ok_or_else(|| {
            SdkError::InvalidUrl("custom networks require HorizonClient::builder".into())
        })?;
        HorizonClientBuilder::new(url).build()
    }

    /// Client pointed at SDF Testnet.
    pub fn testnet() -> Result<Self, SdkError> {
        Self::for_network(&Network::Testnet)
    }

    /// Client pointed at the Public Network.
    pub fn public() -> Result<Self, SdkError> {
        Self::for_network(&Network::Public)
    }

    /// Start a builder against an arbitrary Horizon base URL.
    pub fn builder(base_url: impl Into<String>) -> HorizonClientBuilder {
        HorizonClientBuilder::new(base_url)
    }

    /// Horizon root URL (no trailing slash).
    pub fn base_url(&self) -> &str {
        &self.base_url
    }

    /// Configured timeout.
    pub fn timeout(&self) -> Duration {
        self.timeout
    }

    /// Join `path` onto the Horizon base URL.
    pub fn url(&self, path: &str) -> Result<Url, SdkError> {
        let base = Url::parse(&format!("{}/", self.base_url))
            .map_err(|e| SdkError::InvalidUrl(e.to_string()))?;
        base.join(path.trim_start_matches('/'))
            .map_err(|e| SdkError::InvalidUrl(e.to_string()))
    }

    /// GET JSON, inspecting HTTP status and mapping Horizon problem+json.
    pub async fn get_json<T: DeserializeOwned>(&self, path: &str) -> Result<T, SdkError> {
        let url = self.url(path)?;
        self.request_json(Method::GET, url, None).await
    }

    /// GET JSON from a fully constructed URL (query builders use this).
    pub async fn get_url_json<T: DeserializeOwned>(&self, url: Url) -> Result<T, SdkError> {
        self.request_json(Method::GET, url, None).await
    }

    /// POST `application/x-www-form-urlencoded` and parse JSON.
    pub async fn post_form<T: DeserializeOwned>(
        &self,
        path: &str,
        form: &[(&str, &str)],
    ) -> Result<T, SdkError> {
        let url = self.url(path)?;
        self.request_json(Method::POST, url, Some(form)).await
    }

    async fn request_json<T: DeserializeOwned>(
        &self,
        method: Method,
        url: Url,
        form: Option<&[(&str, &str)]>,
    ) -> Result<T, SdkError> {
        let mut attempt = 0;
        loop {
            match self.send_once(method.clone(), url.clone(), form).await {
                Ok(bytes) => {
                    return serde_json::from_slice(&bytes).map_err(SdkError::from);
                }
                Err(err) if err.is_retryable() && attempt < self.max_retries => {
                    attempt += 1;
                    let backoff = Duration::from_millis(150 * 2u64.pow(attempt));
                    tokio::time::sleep(backoff).await;
                }
                Err(err) => return Err(err),
            }
        }
    }

    async fn send_once(
        &self,
        method: Method,
        url: Url,
        form: Option<&[(&str, &str)]>,
    ) -> Result<Vec<u8>, SdkError> {
        let mut req = self.inner.request(method, url);
        if let Some(form) = form {
            req = req.form(&form);
        }
        let response = req.send().await.map_err(|e| map_reqwest(e, self.timeout))?;
        let status = response.status();
        let bytes = response.bytes().await.map_err(SdkError::from)?;
        if status.is_success() {
            return Ok(bytes.to_vec());
        }
        let body = String::from_utf8_lossy(&bytes);
        Err(SdkError::from_horizon_response(status.as_u16(), &body))
    }
}

fn map_reqwest(err: reqwest::Error, timeout: Duration) -> SdkError {
    if err.is_timeout() {
        SdkError::Timeout {
            duration_ms: timeout.as_millis() as u64,
            context: err.to_string(),
        }
    } else {
        SdkError::from(err)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builder_rejects_invalid_url() {
        let err = HorizonClientBuilder::new("not a url").build().unwrap_err();
        assert!(matches!(err, SdkError::InvalidUrl(_)));
    }

    #[test]
    fn builder_sets_timeout_and_parses_base() {
        let client = HorizonClientBuilder::new("https://horizon-testnet.stellar.org/")
            .timeout(Duration::from_secs(5))
            .max_retries(1)
            .user_agent("Astral-Stellar-Rust-SDK/test")
            .build()
            .unwrap();
        assert_eq!(client.base_url(), "https://horizon-testnet.stellar.org");
        assert_eq!(client.timeout(), Duration::from_secs(5));
        let url = client.url("/accounts/GTEST").unwrap();
        assert!(url.as_str().ends_with("/accounts/GTEST"));
    }

    #[test]
    fn user_agent_contains_astral_brand() {
        assert!(USER_AGENT.starts_with("Astral-Stellar-Rust-SDK/"));
    }

    #[test]
    fn retryable_status_codes() {
        let err = SdkError::HttpStatus {
            status: 429,
            message: "slow".into(),
        };
        assert!(err.is_retryable());
    }
}
