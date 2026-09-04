//! `getLedgerEntries` — contract state and WASM bytecode.

use crate::errors::SdkError;
use crate::rpc::client::RpcClient;
use base64::{engine::general_purpose::STANDARD, Engine as _};
use serde::{Deserialize, Serialize};

/// Ledger key supplied as base64 XDR (as required by Stellar RPC).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LedgerKey {
    /// Base64-encoded `LedgerKey` XDR.
    pub xdr: String,
}

impl LedgerKey {
    /// Wrap an already-encoded base64 XDR key.
    pub fn from_base64(xdr: impl Into<String>) -> Result<Self, SdkError> {
        let xdr = xdr.into();
        STANDARD
            .decode(&xdr)
            .map_err(|e| SdkError::Xdr(format!("invalid ledger key base64: {e}")))?;
        Ok(Self { xdr })
    }

    /// Encode raw XDR bytes as a ledger key.
    pub fn from_xdr_bytes(bytes: &[u8]) -> Self {
        Self {
            xdr: STANDARD.encode(bytes),
        }
    }
}

/// Ledger entry returned by `getLedgerEntries`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LedgerEntry {
    /// Key XDR (base64).
    #[serde(default)]
    pub key: Option<String>,
    /// Entry XDR (base64).
    #[serde(default)]
    pub xdr: Option<String>,
    /// Last modified ledger.
    #[serde(default)]
    pub last_modified_ledger_seq: Option<u32>,
    /// Live-until ledger for TTL'ed entries.
    #[serde(default)]
    pub live_until_ledger_seq: Option<u32>,
}

impl LedgerEntry {
    /// Decode the entry XDR from base64.
    pub fn entry_bytes(&self) -> Result<Vec<u8>, SdkError> {
        let xdr = self
            .xdr
            .as_deref()
            .ok_or_else(|| SdkError::Xdr("ledger entry missing xdr".into()))?;
        STANDARD
            .decode(xdr)
            .map_err(|e| SdkError::Xdr(format!("ledger entry xdr: {e}")))
    }

    /// Decode the key XDR from base64.
    pub fn key_bytes(&self) -> Result<Vec<u8>, SdkError> {
        let key = self
            .key
            .as_deref()
            .ok_or_else(|| SdkError::Xdr("ledger entry missing key".into()))?;
        STANDARD
            .decode(key)
            .map_err(|e| SdkError::Xdr(format!("ledger key xdr: {e}")))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct GetLedgerEntriesParams {
    keys: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct GetLedgerEntriesResult {
    #[serde(default)]
    entries: Option<Vec<LedgerEntry>>,
    #[serde(default)]
    latest_ledger: Option<u32>,
}

impl RpcClient {
    /// Fetch raw ledger entries (contract data / WASM) by base64 `LedgerKey` XDR.
    pub async fn get_ledger_entries(
        &self,
        keys: &[LedgerKey],
    ) -> Result<Vec<LedgerEntry>, SdkError> {
        let params = GetLedgerEntriesParams {
            keys: keys.iter().map(|k| k.xdr.clone()).collect(),
        };
        let result: GetLedgerEntriesResult = self.call("getLedgerEntries", Some(params)).await?;
        Ok(result.entries.unwrap_or_default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn key_roundtrip_base64() {
        let key = LedgerKey::from_xdr_bytes(&[1, 2, 3, 4]);
        let parsed = LedgerKey::from_base64(&key.xdr).unwrap();
        assert_eq!(key, parsed);
        assert!(LedgerKey::from_base64("%%%").is_err());
    }

    #[test]
    fn entry_bytes_decode() {
        let xdr = STANDARD.encode([9u8, 8, 7, 6]);
        let entry = LedgerEntry {
            key: Some(STANDARD.encode([1u8; 4])),
            xdr: Some(xdr),
            last_modified_ledger_seq: Some(10),
            live_until_ledger_seq: None,
        };
        assert_eq!(entry.entry_bytes().unwrap(), vec![9, 8, 7, 6]);
        assert_eq!(entry.key_bytes().unwrap().len(), 4);
    }
}
