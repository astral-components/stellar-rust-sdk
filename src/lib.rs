//! # Stellar Rust SDK
//!
//! Production-grade client library for the [Stellar](https://stellar.org) network.
//! The crate covers Horizon REST access, Soroban JSON-RPC, Ed25519 key management,
//! StrKey address codecs, and typed transaction construction.
//!
//! ## Crate layout
//!
//! - [`errors`] — unified [`SdkError`] taxonomy for HTTP, XDR, StrKey, and RPC failures
//! - [`address`] — StrKey wrappers for `G…`, `S…`, `C…`, and `M…` encodings
//! - [`keypair`] — Ed25519 key generation, seed import, and payload signing
//! - [`network`] — public / testnet / futurenet passphrases and network IDs
//! - [`horizon`] — asynchronous Horizon HTTP client and endpoint builders
//!
//! Enable the `contract-types` feature to re-export the `soroban-sdk` crate.

#![forbid(unsafe_code)]
#![deny(missing_docs)]
#![warn(clippy::all)]

pub mod address;
pub mod errors;
pub mod horizon;
pub mod keypair;
pub mod network;

pub use errors::SdkError;

/// Soroban RPC URL constants (JSON-RPC client lands in a later commit).
pub mod rpc {
    /// Official Soroban RPC URL for SDF Testnet.
    pub const TESTNET_URL: &str = "https://soroban-testnet.stellar.org";
    /// Official Soroban RPC URL for the Stellar Public Network.
    pub const PUBLIC_URL: &str = "https://mainnet.sorobanrpc.com";
    /// Official Soroban RPC URL for SDF Futurenet.
    pub const FUTURENET_URL: &str = "https://rpc-futurenet.stellar.org";
}

/// Re-export of `soroban-sdk` (feature `contract-types`).
#[cfg(feature = "contract-types")]
pub use soroban_sdk;

/// Semantic version of this SDK crate, sourced from `Cargo.toml`.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Canonical HTTP / RPC User-Agent advertised by all network clients.
pub const USER_AGENT: &str = concat!("Astral-Stellar-Rust-SDK/", env!("CARGO_PKG_VERSION"));

/// Official Stellar testnet Horizon URL.
pub const TESTNET_HORIZON_URL: &str = "https://horizon-testnet.stellar.org";

/// Official Stellar public Horizon URL.
pub const PUBLIC_HORIZON_URL: &str = "https://horizon.stellar.org";

/// Official Soroban testnet RPC URL.
pub const TESTNET_RPC_URL: &str = "https://soroban-testnet.stellar.org";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn version_is_semver() {
        let parts: Vec<_> = VERSION.split('.').collect();
        assert_eq!(parts.len(), 3, "VERSION must be major.minor.patch");
        assert!(parts.iter().all(|p| p.parse::<u32>().is_ok()));
    }

    #[test]
    fn user_agent_identifies_astral_sdk() {
        assert!(USER_AGENT.starts_with("Astral-Stellar-Rust-SDK/"));
        assert!(USER_AGENT.contains(VERSION));
    }

    #[test]
    fn serde_json_dependency_is_wired() {
        let value = serde_json::json!({ "horizon": TESTNET_HORIZON_URL });
        assert_eq!(value["horizon"], TESTNET_HORIZON_URL);
    }
}
