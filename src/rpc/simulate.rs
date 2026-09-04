//! `simulateTransaction` RPC method.

use crate::errors::SdkError;
use crate::rpc::client::RpcClient;
use serde::{Deserialize, Serialize};

/// Resource usage reported by simulation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct SimulateTransactionResult {
    /// CPU instructions consumed.
    #[serde(default)]
    pub cpu_insns: Option<String>,
    /// Memory bytes consumed.
    #[serde(default)]
    pub mem_bytes: Option<String>,
    /// Minimum resource fee in stroops (string).
    #[serde(default)]
    pub min_resource_fee: Option<String>,
}

/// Restore preamble when archived entries must be restored first.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RestorePreamble {
    /// Suggested transaction data XDR.
    #[serde(default)]
    pub transaction_data: Option<String>,
    /// Minimum resource fee for the restore.
    #[serde(default)]
    pub min_resource_fee: Option<String>,
}

/// `simulateTransaction` result.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SimulateResponse {
    /// Latest ledger observed.
    #[serde(default)]
    pub latest_ledger: Option<u32>,
    /// Error string when the simulation failed.
    #[serde(default)]
    pub error: Option<String>,
    /// Transaction data XDR (footprint + resources) to embed.
    #[serde(default)]
    pub transaction_data: Option<String>,
    /// Minimum resource fee in stroops.
    #[serde(default)]
    pub min_resource_fee: Option<String>,
    /// Results per host-function invocation.
    #[serde(default)]
    pub results: Option<Vec<serde_json::Value>>,
    /// Diagnostic events (base64 XDR or JSON depending on the node).
    #[serde(default)]
    pub events: Option<Vec<String>>,
    /// Restore preamble.
    #[serde(default)]
    pub restore_preamble: Option<RestorePreamble>,
    /// State changes / footprint summary when provided as `result` or `cost`.
    #[serde(default, alias = "cost")]
    pub result: Option<SimulateTransactionResult>,
}

impl SimulateResponse {
    /// Returns `true` when the simulation did not report an error.
    pub fn is_success(&self) -> bool {
        self.error.is_none()
    }

    /// Parsed CPU instruction count.
    pub fn cpu_instructions(&self) -> u64 {
        self.result
            .as_ref()
            .and_then(|r| r.cpu_insns.as_deref())
            .and_then(|s| s.parse().ok())
            .unwrap_or(0)
    }

    /// Parsed memory-bytes count.
    pub fn memory_bytes(&self) -> u64 {
        self.result
            .as_ref()
            .and_then(|r| r.mem_bytes.as_deref())
            .and_then(|s| s.parse().ok())
            .unwrap_or(0)
    }

    /// Minimum resource fee in stroops, preferring the top-level field.
    pub fn min_resource_fee_stroops(&self) -> u64 {
        self.min_resource_fee
            .as_deref()
            .or(self
                .result
                .as_ref()
                .and_then(|r| r.min_resource_fee.as_deref()))
            .and_then(|s| s.parse().ok())
            .unwrap_or(0)
    }

    /// Authorization / footprint XDR that must be inserted into the envelope.
    pub fn footprint_xdr(&self) -> Option<&str> {
        self.transaction_data.as_deref()
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct SimulateParams<'a> {
    transaction: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    resource_config: Option<ResourceConfig>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ResourceConfig {
    instruction_leeway: u64,
}

impl RpcClient {
    /// Simulate a base64 transaction envelope XDR.
    pub async fn simulate_transaction(
        &self,
        transaction_xdr: &str,
    ) -> Result<SimulateResponse, SdkError> {
        self.simulate_transaction_with_leeway(transaction_xdr, None)
            .await
    }

    /// Simulate with an optional CPU instruction leeway.
    pub async fn simulate_transaction_with_leeway(
        &self,
        transaction_xdr: &str,
        instruction_leeway: Option<u64>,
    ) -> Result<SimulateResponse, SdkError> {
        let params = SimulateParams {
            transaction: transaction_xdr,
            resource_config: instruction_leeway.map(|instruction_leeway| ResourceConfig {
                instruction_leeway,
            }),
        };
        let resp: SimulateResponse = self.call("simulateTransaction", Some(params)).await?;
        if let Some(error) = &resp.error {
            return Err(SdkError::SimulationRejected {
                error: error.clone(),
                events: resp.events.clone().unwrap_or_default(),
            });
        }
        Ok(resp)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fee_and_resource_helpers() {
        let resp = SimulateResponse {
            latest_ledger: Some(1),
            error: None,
            transaction_data: Some("AAAA".into()),
            min_resource_fee: Some("12345".into()),
            results: None,
            events: None,
            restore_preamble: None,
            result: Some(SimulateTransactionResult {
                cpu_insns: Some("1000".into()),
                mem_bytes: Some("2000".into()),
                min_resource_fee: Some("1".into()),
            }),
        };
        assert!(resp.is_success());
        assert_eq!(resp.cpu_instructions(), 1000);
        assert_eq!(resp.memory_bytes(), 2000);
        assert_eq!(resp.min_resource_fee_stroops(), 12345);
        assert_eq!(resp.footprint_xdr(), Some("AAAA"));
    }

    #[test]
    fn simulate_params_use_rpc_camel_case() {
        let params = SimulateParams {
            transaction: "AAAA",
            resource_config: Some(ResourceConfig {
                instruction_leeway: 1_000,
            }),
        };
        let v = serde_json::to_value(&params).unwrap();
        assert_eq!(v["transaction"], "AAAA");
        assert_eq!(v["resourceConfig"]["instructionLeeway"], 1_000);
        assert!(v.get("resource_config").is_none());
    }

    #[test]
    fn simulate_response_reads_rpc_camel_case() {
        let json = r#"{
            "latestLedger": 42,
            "minResourceFee": "99",
            "transactionData": "AABB"
        }"#;
        let resp: SimulateResponse = serde_json::from_str(json).unwrap();
        assert_eq!(resp.latest_ledger, Some(42));
        assert_eq!(resp.min_resource_fee.as_deref(), Some("99"));
        assert_eq!(resp.footprint_xdr(), Some("AABB"));
    }

    #[test]
    fn simulate_response_reads_cost_alias() {
        let json = r#"{
            "cost": { "cpuInsns": "1000", "memBytes": "2000" }
        }"#;
        let resp: SimulateResponse = serde_json::from_str(json).unwrap();
        assert_eq!(resp.cpu_instructions(), 1000);
        assert_eq!(resp.memory_bytes(), 2000);
    }

    #[test]
    fn error_maps_to_simulation_rejected() {
        let err = SdkError::SimulationRejected {
            error: "UnreachableCodeReached".into(),
            events: vec!["evt".into()],
        };
        match err {
            SdkError::SimulationRejected { error, events } => {
                assert!(error.contains("Unreachable"));
                assert_eq!(events.len(), 1);
            }
            _ => panic!("wrong variant"),
        }
    }
}
