//! JSON-RPC 2.0 request/response types for Stellar RPC.

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::rpc::JSON_RPC_VERSION;

/// JSON-RPC request envelope.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RpcRequest<T> {
    /// Protocol version (`"2.0"`).
    pub jsonrpc: String,
    /// Request id.
    pub id: u64,
    /// Method name.
    pub method: String,
    /// Parameters (object or array).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<T>,
}

impl<T> RpcRequest<T> {
    /// Construct a JSON-RPC 2.0 request.
    pub fn new(id: u64, method: impl Into<String>, params: Option<T>) -> Self {
        Self {
            jsonrpc: JSON_RPC_VERSION.to_string(),
            id,
            method: method.into(),
            params,
        }
    }
}

/// JSON-RPC error object.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RpcError {
    /// Numeric code.
    pub code: i64,
    /// Message.
    pub message: String,
    /// Optional data payload.
    #[serde(default)]
    pub data: Option<Value>,
}

/// JSON-RPC response envelope.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(bound(deserialize = "T: Deserialize<'de>"))]
pub struct RpcResponse<T> {
    /// Protocol version.
    #[serde(default)]
    pub jsonrpc: String,
    /// Request id.
    #[serde(default)]
    pub id: Option<Value>,
    /// Success result.
    #[serde(default)]
    pub result: Option<T>,
    /// Error object.
    #[serde(default)]
    pub error: Option<RpcError>,
}

/// `getHealth` result.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GetHealthResponse {
    /// `"healthy"` when the node is serving traffic.
    pub status: String,
}

impl GetHealthResponse {
    /// Returns `true` when `status == "healthy"`.
    pub fn is_healthy(&self) -> bool {
        self.status.eq_ignore_ascii_case("healthy")
    }
}

/// `getNetwork` result.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GetNetworkResponse {
    /// Friendbot URL when the network is a test network.
    #[serde(default)]
    pub friendbot_url: Option<String>,
    /// Network passphrase.
    pub passphrase: String,
    /// Protocol version.
    #[serde(default)]
    pub protocol_version: Option<u32>,
}

/// `getLatestLedger` result.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GetLatestLedgerResponse {
    /// Latest ledger id (hex).
    #[serde(default)]
    pub id: Option<String>,
    /// Protocol version.
    #[serde(default)]
    pub protocol_version: Option<u32>,
    /// Ledger sequence.
    pub sequence: u32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn request_serializes_jsonrpc_2() {
        let req = RpcRequest::<()>::new(1, "getHealth", None);
        let v = serde_json::to_value(&req).unwrap();
        assert_eq!(v["jsonrpc"], "2.0");
        assert_eq!(v["method"], "getHealth");
        assert!(v.get("params").is_none());
    }

    #[test]
    fn health_helper() {
        assert!(GetHealthResponse {
            status: "healthy".into()
        }
        .is_healthy());
    }
}
