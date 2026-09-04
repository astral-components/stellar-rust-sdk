//! # Stellar Rust SDK
//!
//! Production-grade client library for the [Stellar](https://stellar.org) network.
//! This crate is the native Rust SDK for Horizon REST, Soroban JSON-RPC, Ed25519
//! key management, StrKey codecs, and typed transaction construction.
//!
//! Subsequent commits introduce dedicated modules for errors, addresses, keypairs,
//! networks, Horizon, RPC, transactions, and Soroban invocation. This commit
//! establishes the workspace crate, shared constants, and the root error surface.

#![forbid(unsafe_code)]
#![deny(missing_docs)]
#![warn(clippy::all)]

/// Placeholder error module; replaced by the full taxonomy in the next commit.
pub mod errors {
    //! Root SDK error type shared by all public APIs.

    use thiserror::Error;

    /// Top-level error returned by SDK operations.
    #[derive(Debug, Error)]
    pub enum SdkError {
        /// TCP / TLS connect or request exceeded the configured timeout.
        #[error("network timeout after {duration_ms}ms: {context}")]
        Timeout {
            /// Timeout budget in milliseconds.
            duration_ms: u64,
            /// Operation that timed out.
            context: String,
        },

        /// Non-success HTTP status.
        #[error("http {status}: {message}")]
        HttpStatus {
            /// HTTP status code.
            status: u16,
            /// Response body excerpt.
            message: String,
        },

        /// Catch-all for unexpected internal conditions.
        #[error("{0}")]
        Message(String),
    }

    impl SdkError {
        /// Construct a contextual [`SdkError::Message`].
        pub fn message(msg: impl Into<String>) -> Self {
            Self::Message(msg.into())
        }

        /// Returns `true` when the failure is a network timeout.
        pub fn is_timeout(&self) -> bool {
            matches!(self, Self::Timeout { .. })
        }
    }
}

pub use errors::SdkError;

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

    #[test]
    fn stub_error_message_roundtrip() {
        let err = SdkError::message("bootstrap");
        assert_eq!(err.to_string(), "bootstrap");
        assert!(!err.is_timeout());
    }
}
