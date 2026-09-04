//! Async JSON-RPC client for Soroban RPC nodes.

use crate::errors::SdkError;
use crate::network::Network;
use crate::rpc::types::{
    GetHealthResponse, GetLatestLedgerResponse, GetNetworkResponse, RpcRequest, RpcResponse,
};
use crate::USER_AGENT;
use reqwest::header::{HeaderMap, HeaderValue, CONTENT_TYPE, USER_AGENT as UA};
use reqwest::Client;
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

/// Async Stellar RPC (JSON-RPC 2.0) client.
#[derive(Debug)]
pub struct RpcClient {
    inner: Client,
    endpoint: String,
    next_id: AtomicU64,
}

impl Clone for RpcClient {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
            endpoint: self.endpoint.clone(),
            next_id: AtomicU64::new(self.next_id.load(Ordering::Relaxed)),
        }
    }
}

impl RpcClient {
    /// Connect to `endpoint` (e.g. `https://soroban-testnet.stellar.org`).
    pub fn new(endpoint: impl Into<String>) -> Result<Self, SdkError> {
        let endpoint = endpoint.into().trim_end_matches('/').to_string();
        let mut headers = HeaderMap::new();
        headers.insert(UA, HeaderValue::from_static(USER_AGENT));
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
        let inner = Client::builder()
            .timeout(Duration::from_secs(30))
            .default_headers(headers)
            .build()?;
        Ok(Self {
            inner,
            endpoint,
            next_id: AtomicU64::new(1),
        })
    }

    /// Client for a well-known [`Network`].
    pub fn for_network(network: &Network) -> Result<Self, SdkError> {
        let url = network
            .rpc_url()
            .ok_or_else(|| SdkError::InvalidUrl("custom networks require RpcClient::new".into()))?;
        Self::new(url)
    }

    /// SDF Testnet RPC.
    pub fn testnet() -> Result<Self, SdkError> {
        Self::for_network(&Network::Testnet)
    }

    /// RPC endpoint URL.
    pub fn endpoint(&self) -> &str {
        &self.endpoint
    }

    /// `getHealth`
    pub async fn get_health(&self) -> Result<GetHealthResponse, SdkError> {
        self.call("getHealth", Option::<()>::None).await
    }

    /// `getNetwork`
    pub async fn get_network(&self) -> Result<GetNetworkResponse, SdkError> {
        self.call("getNetwork", Option::<()>::None).await
    }

    /// `getLatestLedger`
    pub async fn get_latest_ledger(&self) -> Result<GetLatestLedgerResponse, SdkError> {
        self.call("getLatestLedger", Option::<()>::None).await
    }

    /// Perform a JSON-RPC method call.
    pub async fn call<P, R>(&self, method: &str, params: Option<P>) -> Result<R, SdkError>
    where
        P: Serialize,
        R: DeserializeOwned,
    {
        let id = self.next_id.fetch_add(1, Ordering::Relaxed);
        let request = RpcRequest::new(id, method, params);
        let response = self
            .inner
            .post(&self.endpoint)
            .json(&request)
            .send()
            .await
            .map_err(SdkError::from)?;
        let status = response.status();
        let bytes = response.bytes().await.map_err(SdkError::from)?;
        if !status.is_success() {
            let body = String::from_utf8_lossy(&bytes);
            return Err(SdkError::HttpStatus {
                status: status.as_u16(),
                message: body.into_owned(),
            });
        }
        let parsed: RpcResponse<R> = serde_json::from_slice(&bytes)?;
        if let Some(err) = parsed.error {
            return Err(SdkError::message(format!(
                "rpc {} ({}): {}",
                method, err.code, err.message
            )));
        }
        parsed
            .result
            .ok_or_else(|| SdkError::message(format!("rpc {method} returned no result")))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn constructs_testnet_client() {
        let client = RpcClient::testnet().unwrap();
        assert!(client.endpoint().contains("soroban-testnet"));
    }
}
