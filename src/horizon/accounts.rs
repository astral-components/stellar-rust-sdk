//! Horizon account models and query builder.

use crate::errors::SdkError;
use crate::horizon::client::HorizonClient;
use serde::{Deserialize, Serialize};

/// HAL link object used throughout Horizon.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Link {
    /// Target href.
    pub href: String,
    /// Whether the href is templated.
    #[serde(default)]
    pub templated: bool,
}

/// Account flags (auth required / revocable / immutable / clawback).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct AccountFlags {
    /// Issuer requires authorization.
    #[serde(default)]
    pub auth_required: bool,
    /// Issuer can revoke authorization.
    #[serde(default)]
    pub auth_revocable: bool,
    /// Flags can no longer be changed.
    #[serde(default)]
    pub auth_immutable: bool,
    /// Issuer can claw back trustlines.
    #[serde(default)]
    pub auth_clawback_enabled: bool,
}

/// Trustline or native balance on an account.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Balance {
    /// Balance amount as a decimal string.
    pub balance: String,
    /// Buying liabilities.
    #[serde(default)]
    pub buying_liabilities: Option<String>,
    /// Selling liabilities.
    #[serde(default)]
    pub selling_liabilities: Option<String>,
    /// Limit for credit assets.
    #[serde(default)]
    pub limit: Option<String>,
    /// Asset type (`native`, `credit_alphanum4`, `credit_alphanum12`).
    pub asset_type: String,
    /// Asset code when non-native.
    #[serde(default)]
    pub asset_code: Option<String>,
    /// Asset issuer when non-native.
    #[serde(default)]
    pub asset_issuer: Option<String>,
    /// Liquidity pool id when this is a pool share.
    #[serde(default)]
    pub liquidity_pool_id: Option<String>,
    /// Whether the account is authorized for this trustline.
    #[serde(default)]
    pub is_authorized: Option<bool>,
}

/// Signer attached to an account.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AccountSigner {
    /// Weighted threshold for this signer.
    pub weight: u32,
    /// Signer key (`G…`, pre-auth, hash-x, signed payload).
    pub key: String,
    /// Signer type (`ed25519_public_key`, …).
    #[serde(rename = "type")]
    pub signer_type: String,
}

/// Horizon account resource.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Account {
    /// Account id (`G…`).
    pub id: String,
    /// Account id (same as `id` on modern Horizon).
    pub account_id: String,
    /// Sequence number as a decimal string.
    pub sequence: String,
    /// Sequence ledger.
    #[serde(default)]
    pub sequence_ledger: Option<u32>,
    /// Sequence time (unix seconds as string).
    #[serde(default)]
    pub sequence_time: Option<String>,
    /// Subentry count.
    #[serde(default)]
    pub subentry_count: u32,
    /// Number of sponsored entries.
    #[serde(default)]
    pub num_sponsoring: Option<u32>,
    /// Number of entries this account is sponsored by.
    #[serde(default)]
    pub num_sponsored: Option<u32>,
    /// Home domain.
    #[serde(default)]
    pub home_domain: Option<String>,
    /// Last modified ledger.
    #[serde(default)]
    pub last_modified_ledger: Option<u32>,
    /// Thresholds.
    #[serde(default)]
    pub thresholds: Option<AccountThresholds>,
    /// Account flags.
    #[serde(default)]
    pub flags: AccountFlags,
    /// Balances including native XLM.
    #[serde(default)]
    pub balances: Vec<Balance>,
    /// Signers.
    #[serde(default)]
    pub signers: Vec<AccountSigner>,
}

impl Account {
    /// Parsed sequence number.
    pub fn sequence_number(&self) -> Result<i64, SdkError> {
        self.sequence
            .parse::<i64>()
            .map_err(|e| SdkError::message(format!("invalid sequence `{}`: {e}", self.sequence)))
    }

    /// Native (XLM) balance as a decimal string, if present.
    pub fn native_balance(&self) -> Option<&str> {
        self.balances
            .iter()
            .find(|b| b.asset_type == "native")
            .map(|b| b.balance.as_str())
    }
}

/// Master / low / medium / high thresholds.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct AccountThresholds {
    /// Low threshold.
    #[serde(default)]
    pub low_threshold: u8,
    /// Medium threshold.
    #[serde(default)]
    pub med_threshold: u8,
    /// High threshold.
    #[serde(default)]
    pub high_threshold: u8,
}

/// HAL collection wrapper for `/accounts`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HorizonAccountResponse {
    /// Embedded records.
    #[serde(rename = "_embedded")]
    pub embedded: AccountEmbedded,
}

/// `_embedded` object for account collections.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountEmbedded {
    /// Account records.
    pub records: Vec<Account>,
}

/// Pagination order.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Order {
    /// Ascending (`asc`).
    Asc,
    /// Descending (`desc`).
    Desc,
}

impl Order {
    fn as_str(self) -> &'static str {
        match self {
            Self::Asc => "asc",
            Self::Desc => "desc",
        }
    }
}

/// Query builder for `GET /accounts` and `GET /accounts/{id}`.
#[derive(Debug, Clone)]
pub struct AccountsRequestBuilder<'a> {
    client: &'a HorizonClient,
    cursor: Option<String>,
    limit: Option<u32>,
    order: Option<Order>,
    signer: Option<String>,
    asset: Option<String>,
    sponsor: Option<String>,
    liquidity_pool: Option<String>,
}

impl<'a> AccountsRequestBuilder<'a> {
    /// Bind this builder to a Horizon client.
    pub fn new(client: &'a HorizonClient) -> Self {
        Self {
            client,
            cursor: None,
            limit: None,
            order: None,
            signer: None,
            asset: None,
            sponsor: None,
            liquidity_pool: None,
        }
    }

    /// Paging cursor.
    pub fn cursor(mut self, cursor: impl Into<String>) -> Self {
        self.cursor = Some(cursor.into());
        self
    }

    /// Page size (Horizon max is 200).
    pub fn limit(mut self, limit: u32) -> Self {
        self.limit = Some(limit.min(200));
        self
    }

    /// Sort order.
    pub fn order(mut self, order: Order) -> Self {
        self.order = Some(order);
        self
    }

    /// Filter accounts that have `signer` as a signer.
    pub fn for_signer(mut self, signer: impl Into<String>) -> Self {
        self.signer = Some(signer.into());
        self
    }

    /// Filter accounts holding `asset` (`native` or `CODE:ISSUER`).
    pub fn for_asset(mut self, asset: impl Into<String>) -> Self {
        self.asset = Some(asset.into());
        self
    }

    /// Filter accounts sponsored by `sponsor`.
    pub fn for_sponsor(mut self, sponsor: impl Into<String>) -> Self {
        self.sponsor = Some(sponsor.into());
        self
    }

    /// Filter accounts holding shares of a liquidity pool.
    pub fn for_liquidity_pool(mut self, pool_id: impl Into<String>) -> Self {
        self.liquidity_pool = Some(pool_id.into());
        self
    }

    /// Fetch a single account by id (`G…`).
    pub async fn account(&self, account_id: &str) -> Result<Account, SdkError> {
        self.client
            .get_json(&format!("accounts/{account_id}"))
            .await
    }

    /// Fetch a page of accounts matching the configured filters.
    pub async fn call(&self) -> Result<HorizonAccountResponse, SdkError> {
        let mut url = self.client.url("accounts")?;
        {
            let mut pairs = url.query_pairs_mut();
            if let Some(c) = &self.cursor {
                pairs.append_pair("cursor", c);
            }
            if let Some(l) = self.limit {
                pairs.append_pair("limit", &l.to_string());
            }
            if let Some(o) = self.order {
                pairs.append_pair("order", o.as_str());
            }
            if let Some(s) = &self.signer {
                pairs.append_pair("signer", s);
            }
            if let Some(a) = &self.asset {
                pairs.append_pair("asset", a);
            }
            if let Some(s) = &self.sponsor {
                pairs.append_pair("sponsor", s);
            }
            if let Some(p) = &self.liquidity_pool {
                pairs.append_pair("liquidity_pool", p);
            }
        }
        self.client.get_url_json(url).await
    }
}

impl HorizonClient {
    /// Start an accounts query builder.
    pub fn accounts(&self) -> AccountsRequestBuilder<'_> {
        AccountsRequestBuilder::new(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sequence_number_parses() {
        let acc = Account {
            id: "G".into(),
            account_id: "G".into(),
            sequence: "12345".into(),
            sequence_ledger: None,
            sequence_time: None,
            subentry_count: 0,
            num_sponsoring: None,
            num_sponsored: None,
            home_domain: None,
            last_modified_ledger: None,
            thresholds: None,
            flags: AccountFlags::default(),
            balances: vec![Balance {
                balance: "10.0000000".into(),
                buying_liabilities: None,
                selling_liabilities: None,
                limit: None,
                asset_type: "native".into(),
                asset_code: None,
                asset_issuer: None,
                liquidity_pool_id: None,
                is_authorized: None,
            }],
            signers: vec![],
        };
        assert_eq!(acc.sequence_number().unwrap(), 12_345);
        assert_eq!(acc.native_balance(), Some("10.0000000"));
    }

    #[test]
    fn order_strings() {
        assert_eq!(Order::Asc.as_str(), "asc");
        assert_eq!(Order::Desc.as_str(), "desc");
    }
}
