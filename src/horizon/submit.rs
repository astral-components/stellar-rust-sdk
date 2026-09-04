//! Horizon transaction submission pipeline.

use crate::errors::SdkError;
use crate::horizon::client::HorizonClient;
use serde::{Deserialize, Serialize};

/// `extras.result_codes` on a successful or failed submission.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct SubmitResultCodes {
    /// Transaction-level code.
    #[serde(default)]
    pub transaction: Option<String>,
    /// Per-operation codes.
    #[serde(default)]
    pub operations: Option<Vec<String>>,
}

/// Horizon `POST /transactions` response body.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SubmitResponse {
    /// Transaction hash (hex).
    #[serde(default)]
    pub hash: Option<String>,
    /// Ledger the transaction landed in.
    #[serde(default)]
    pub ledger: Option<u32>,
    /// Result XDR (base64).
    #[serde(default)]
    pub result_xdr: Option<String>,
    /// Envelope XDR (base64).
    #[serde(default)]
    pub envelope_xdr: Option<String>,
    /// Whether Horizon considers the tx successful.
    #[serde(default)]
    pub successful: Option<bool>,
    /// Paging token.
    #[serde(default)]
    pub paging_token: Option<String>,
    /// Result codes (present on some error wrappers).
    #[serde(default)]
    pub result_codes: Option<SubmitResultCodes>,
}

impl SubmitResponse {
    /// Returns `true` when Horizon reported success.
    pub fn is_success(&self) -> bool {
        self.successful.unwrap_or(false)
    }
}

impl HorizonClient {
    /// Submit a base64-encoded transaction envelope XDR.
    ///
    /// Transient transport failures are retried by the client. Horizon
    /// application errors (`tx_bad_seq`, `tx_insufficient_fee`, `op_underfunded`)
    /// are mapped onto [`SdkError`] and are **not** retried.
    pub async fn submit_transaction(&self, envelope_xdr: &str) -> Result<SubmitResponse, SdkError> {
        match self
            .post_form::<SubmitResponse>("transactions", &[("tx", envelope_xdr)])
            .await
        {
            Ok(resp) => {
                if resp.successful == Some(false) {
                    if let Some(codes) = &resp.result_codes {
                        return Err(SdkError::TransactionFailed {
                            result_code: codes
                                .transaction
                                .clone()
                                .unwrap_or_else(|| "tx_failed".into()),
                            op_codes: codes.operations.clone().unwrap_or_default(),
                        });
                    }
                }
                Ok(resp)
            }
            Err(err) => Err(err),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn success_flag() {
        let ok = SubmitResponse {
            hash: Some("ab".into()),
            ledger: Some(1),
            result_xdr: None,
            envelope_xdr: None,
            successful: Some(true),
            paging_token: None,
            result_codes: None,
        };
        assert!(ok.is_success());
    }

    #[test]
    fn known_codes_are_user_errors() {
        let err = SdkError::TransactionFailed {
            result_code: "tx_bad_seq".into(),
            op_codes: vec!["op_underfunded".into()],
        };
        assert!(err.is_bad_seq());
        assert!(err.is_underfunded());
        assert!(!err.is_retryable());
    }
}
