//! Soroban JSON-RPC client (Stellar RPC).

mod client;
mod types;

pub use client::RpcClient;
pub use types::{
    GetHealthResponse, GetLatestLedgerResponse, GetNetworkResponse, RpcError, RpcRequest,
    RpcResponse,
};

/// Official Soroban RPC URL for SDF Testnet.
pub const TESTNET_URL: &str = "https://soroban-testnet.stellar.org";

/// Official Soroban RPC URL for the Stellar Public Network.
pub const PUBLIC_URL: &str = "https://mainnet.sorobanrpc.com";

/// Official Soroban RPC URL for SDF Futurenet.
pub const FUTURENET_URL: &str = "https://rpc-futurenet.stellar.org";

/// JSON-RPC 2.0 protocol version advertised by Stellar RPC nodes.
pub const JSON_RPC_VERSION: &str = "2.0";

/// Returns the Soroban RPC base URL for a well-known network label.
pub fn url_for(network: &str) -> Option<&'static str> {
    match network {
        "public" | "mainnet" => Some(PUBLIC_URL),
        "testnet" => Some(TESTNET_URL),
        "futurenet" => Some(FUTURENET_URL),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn json_rpc_version_is_2() {
        assert_eq!(JSON_RPC_VERSION, "2.0");
    }

    #[test]
    fn well_known_rpc_urls() {
        assert_eq!(url_for("testnet"), Some(TESTNET_URL));
        assert!(url_for("unknown").is_none());
    }
}
