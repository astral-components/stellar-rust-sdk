//! Comprehensive SDK error taxonomy for Horizon, RPC, XDR, and cryptography.
//!
//! [`SdkError`] is the only error type returned by public APIs. Horizon JSON
//! error payloads (HTTP 400 with `extras.result_codes`) are parsed into
//! structured variants so callers can branch on transaction and operation codes
//! without scraping error strings.

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Horizon `extras` object attached to error responses.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct HorizonExtras {
    /// Envelope XDR that Horizon rejected, when provided.
    #[serde(default)]
    pub envelope_xdr: Option<String>,
    /// Result XDR for a failed submission.
    #[serde(default)]
    pub result_xdr: Option<String>,
    /// Structured transaction / operation result codes.
    #[serde(default)]
    pub result_codes: Option<HorizonResultCodes>,
    /// Server-side diagnostic string.
    #[serde(default)]
    pub invalid_field: Option<String>,
}

/// `extras.result_codes` from a Horizon error or transaction response.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct HorizonResultCodes {
    /// Transaction-level code (`tx_success`, `tx_bad_seq`, `tx_insufficient_fee`, …).
    #[serde(default)]
    pub transaction: Option<String>,
    /// Per-operation codes (`op_underfunded`, `op_no_destination`, …).
    #[serde(default)]
    pub operations: Option<Vec<String>>,
}

/// Decoded Horizon problem+json error body.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HorizonErrorBody {
    /// RFC 7807 type URI.
    #[serde(rename = "type", default)]
    pub type_url: Option<String>,
    /// Short error title.
    #[serde(default)]
    pub title: Option<String>,
    /// HTTP status echoed by Horizon.
    #[serde(default)]
    pub status: Option<u16>,
    /// Human-readable detail.
    #[serde(default)]
    pub detail: Option<String>,
    /// Instance identifier.
    #[serde(default)]
    pub instance: Option<String>,
    /// Extra diagnostic payload.
    #[serde(default)]
    pub extras: Option<HorizonExtras>,
}

/// Top-level error returned by SDK operations.
#[derive(Debug, Error)]
pub enum SdkError {
    /// TCP / TLS connect or request exceeded the configured timeout.
    #[error("network timeout after {duration_ms}ms: {context}")]
    Timeout {
        /// Timeout budget in milliseconds.
        duration_ms: u64,
        /// Operation that timed out (e.g. `GET /accounts/{id}`).
        context: String,
    },

    /// Non-success HTTP status that is not a structured Horizon problem document.
    #[error("http {status}: {message}")]
    HttpStatus {
        /// HTTP status code.
        status: u16,
        /// Response body (truncated by the client).
        message: String,
    },

    /// Horizon problem+json payload (typically HTTP 400 / 404 / 429).
    #[error("horizon {status} {title}: {detail}")]
    Horizon {
        /// HTTP status.
        status: u16,
        /// Error title (`Transaction Failed`, `Rate Limit Exceeded`, …).
        title: String,
        /// Detail text.
        detail: String,
        /// RFC 7807 type URI.
        type_url: Option<String>,
        /// Horizon extras including result codes.
        extras: Option<HorizonExtras>,
    },

    /// XDR encode or decode failure.
    #[error("xdr error: {0}")]
    Xdr(String),

    /// StrKey (`G`/`S`/`C`/`M`) parse or checksum failure.
    #[error("strkey error: {0}")]
    StrKey(String),

    /// Soroban RPC `simulateTransaction` rejected the envelope.
    #[error("soroban simulation rejected: {error}")]
    SimulationRejected {
        /// RPC error string or diagnostic.
        error: String,
        /// Diagnostic events returned by the simulation, if any.
        events: Vec<String>,
    },

    /// Address validation failed (wrong prefix, length, or checksum).
    #[error("invalid address: {0}")]
    InvalidAddress(String),

    /// Ed25519 signing or key derivation failed.
    #[error("signing error: {0}")]
    Signing(String),

    /// Transaction failed with a known Stellar result code.
    #[error("transaction failed: {result_code}")]
    TransactionFailed {
        /// Transaction result code (`tx_bad_seq`, `tx_insufficient_fee`, …).
        result_code: String,
        /// Operation result codes when present.
        op_codes: Vec<String>,
    },

    /// JSON (de)serialization failure.
    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),

    /// Underlying HTTP client failure from `reqwest`.
    #[error("http client error: {0}")]
    Http(#[from] reqwest::Error),

    /// URL could not be parsed or joined.
    #[error("invalid url: {0}")]
    InvalidUrl(String),

    /// Catch-all for unexpected internal conditions.
    #[error("{0}")]
    Message(String),
}

impl SdkError {
    /// Construct a contextual [`SdkError::Message`].
    pub fn message(msg: impl Into<String>) -> Self {
        Self::Message(msg.into())
    }

    /// Parse a Horizon HTTP error body into [`SdkError::Horizon`] or [`SdkError::HttpStatus`].
    ///
    /// When the body is problem+json with `extras.result_codes.transaction`, the
    /// error is promoted to [`SdkError::TransactionFailed`].
    pub fn from_horizon_response(status: u16, body: &str) -> Self {
        match serde_json::from_str::<HorizonErrorBody>(body) {
            Ok(parsed) => {
                if let Some(codes) = parsed
                    .extras
                    .as_ref()
                    .and_then(|e| e.result_codes.as_ref())
                {
                    if let Some(tx_code) = codes.transaction.as_deref() {
                        if tx_code != "tx_success" {
                            return Self::TransactionFailed {
                                result_code: tx_code.to_string(),
                                op_codes: codes.operations.clone().unwrap_or_default(),
                            };
                        }
                    }
                }
                Self::Horizon {
                    status: parsed.status.unwrap_or(status),
                    title: parsed
                        .title
                        .unwrap_or_else(|| "Horizon error".to_string()),
                    detail: parsed.detail.unwrap_or_else(|| body.to_string()),
                    type_url: parsed.type_url,
                    extras: parsed.extras,
                }
            }
            Err(_) => Self::HttpStatus {
                status,
                message: truncate(body, 2048),
            },
        }
    }

    /// Returns `true` when the failure is a network timeout.
    pub fn is_timeout(&self) -> bool {
        matches!(self, Self::Timeout { .. })
    }

    /// Returns `true` when the failure originated from Horizon.
    pub fn is_horizon(&self) -> bool {
        matches!(
            self,
            Self::Horizon { .. } | Self::TransactionFailed { .. }
        )
    }

    /// Returns `true` when the failure is an HTTP / transport problem.
    pub fn is_network(&self) -> bool {
        matches!(
            self,
            Self::Timeout { .. }
                | Self::Http(_)
                | Self::HttpStatus { .. }
                | Self::InvalidUrl(_)
        )
    }

    /// Returns `true` when the client may retry (timeouts, 429, 5xx, connect errors).
    pub fn is_retryable(&self) -> bool {
        match self {
            Self::Timeout { .. } => true,
            Self::HttpStatus { status, .. } => *status == 429 || *status >= 500,
            Self::Horizon { status, .. } => *status == 429 || *status >= 500,
            Self::Http(err) => err.is_timeout() || err.is_connect() || err.is_request(),
            _ => false,
        }
    }

    /// Returns `true` for client-side validation / 4xx user errors.
    pub fn is_user_error(&self) -> bool {
        match self {
            Self::StrKey(_)
            | Self::InvalidAddress(_)
            | Self::Xdr(_)
            | Self::Signing(_)
            | Self::InvalidUrl(_) => true,
            Self::HttpStatus { status, .. } | Self::Horizon { status, .. } => {
                (400..500).contains(status) && *status != 429
            }
            Self::TransactionFailed { .. } => true,
            _ => false,
        }
    }

    /// Transaction result code when this error carries one.
    pub fn result_code(&self) -> Option<&str> {
        match self {
            Self::TransactionFailed { result_code, .. } => Some(result_code),
            Self::Horizon { extras, .. } => extras
                .as_ref()
                .and_then(|e| e.result_codes.as_ref())
                .and_then(|c| c.transaction.as_deref()),
            _ => None,
        }
    }

    /// Operation result codes when this error carries them.
    pub fn op_codes(&self) -> &[String] {
        match self {
            Self::TransactionFailed { op_codes, .. } => op_codes,
            Self::Horizon { extras, .. } => extras
                .as_ref()
                .and_then(|e| e.result_codes.as_ref())
                .and_then(|c| c.operations.as_deref())
                .unwrap_or(&[]),
            _ => &[],
        }
    }

    /// Returns `true` when Horizon reported `tx_bad_seq`.
    pub fn is_bad_seq(&self) -> bool {
        self.result_code() == Some("tx_bad_seq")
    }

    /// Returns `true` when Horizon reported `tx_insufficient_fee`.
    pub fn is_insufficient_fee(&self) -> bool {
        self.result_code() == Some("tx_insufficient_fee")
    }

    /// Returns `true` when any operation code is `op_underfunded`.
    pub fn is_underfunded(&self) -> bool {
        self.op_codes().iter().any(|c| c == "op_underfunded")
    }
}

fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        format!("{}…", &s[..max])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const FAILED_TX: &str = r#"{
        "type": "https://stellar.org/horizon-errors/transaction_failed",
        "title": "Transaction Failed",
        "status": 400,
        "detail": "The transaction failed when submitted to the stellar network.",
        "extras": {
            "result_codes": {
                "transaction": "tx_bad_seq",
                "operations": ["op_underfunded"]
            }
        }
    }"#;

    #[test]
    fn parses_horizon_result_codes_into_transaction_failed() {
        let err = SdkError::from_horizon_response(400, FAILED_TX);
        assert!(err.is_horizon());
        assert!(err.is_user_error());
        assert!(err.is_bad_seq());
        assert!(err.is_underfunded());
        assert_eq!(err.result_code(), Some("tx_bad_seq"));
        assert_eq!(err.op_codes(), &["op_underfunded".to_string()]);
    }

    #[test]
    fn insufficient_fee_helper() {
        let body = r#"{
            "title": "Transaction Failed",
            "status": 400,
            "detail": "fee too low",
            "extras": { "result_codes": { "transaction": "tx_insufficient_fee" } }
        }"#;
        let err = SdkError::from_horizon_response(400, body);
        assert!(err.is_insufficient_fee());
        assert!(!err.is_retryable());
    }

    #[test]
    fn raw_body_falls_back_to_http_status() {
        let err = SdkError::from_horizon_response(502, "bad gateway");
        assert!(err.is_network());
        assert!(err.is_retryable());
        match err {
            SdkError::HttpStatus { status, message } => {
                assert_eq!(status, 502);
                assert_eq!(message, "bad gateway");
            }
            other => panic!("unexpected {other:?}"),
        }
    }

    #[test]
    fn timeout_is_retryable_network_error() {
        let err = SdkError::Timeout {
            duration_ms: 10_000,
            context: "GET /accounts".into(),
        };
        assert!(err.is_timeout());
        assert!(err.is_network());
        assert!(err.is_retryable());
        assert!(!err.is_user_error());
    }

    #[test]
    fn strkey_errors_are_user_errors() {
        let err = SdkError::StrKey("bad checksum".into());
        assert!(err.is_user_error());
        assert!(!err.is_retryable());
        assert!(!err.is_horizon());
    }

    #[test]
    fn rate_limit_is_retryable_horizon_error() {
        let body = r#"{"title":"Rate Limit Exceeded","status":429,"detail":"slow down"}"#;
        let err = SdkError::from_horizon_response(429, body);
        assert!(err.is_horizon());
        assert!(err.is_retryable());
        assert!(!err.is_user_error());
    }
}
