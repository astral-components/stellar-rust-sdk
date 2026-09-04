//! Horizon liquidity pool queries: reserves, shares, and participants.

use crate::errors::SdkError;
use crate::horizon::client::HorizonClient;
use serde::{Deserialize, Serialize};

/// Reserve held by a constant-product liquidity pool.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LiquidityPoolReserve {
    /// Asset type.
    pub asset: String,
    /// Reserve amount as a decimal string.
    pub amount: String,
}

/// Liquidity pool resource.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LiquidityPool {
    /// Pool id (hex).
    pub id: String,
    /// Paging token.
    #[serde(default)]
    pub paging_token: Option<String>,
    /// Fee in basis points (typically 30).
    #[serde(default)]
    pub fee_bp: Option<u32>,
    /// Pool type (`constant_product`).
    #[serde(default, rename = "type")]
    pub pool_type: Option<String>,
    /// Total shares issued.
    #[serde(default)]
    pub total_shares: Option<String>,
    /// Number of trustlines (participants).
    #[serde(default)]
    pub total_trustlines: Option<u64>,
    /// Reserves in the pool.
    #[serde(default)]
    pub reserves: Vec<LiquidityPoolReserve>,
}

impl LiquidityPool {
    /// Share of the pool held as `amount / total_shares` when both parse as `f64`.
    pub fn share_fraction(&self, amount: &str) -> Option<f64> {
        let shares: f64 = self.total_shares.as_ref()?.parse().ok()?;
        let held: f64 = amount.parse().ok()?;
        if shares == 0.0 {
            None
        } else {
            Some(held / shares)
        }
    }
}

/// HAL collection for `/liquidity_pools`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PoolsPage {
    /// Embedded records.
    #[serde(rename = "_embedded")]
    pub embedded: PoolsEmbedded,
}

/// `_embedded` object for pool collections.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PoolsEmbedded {
    /// Pool records.
    pub records: Vec<LiquidityPool>,
}

/// Query builder for `GET /liquidity_pools`.
#[derive(Debug, Clone)]
pub struct PoolsRequestBuilder<'a> {
    client: &'a HorizonClient,
    reserves: Option<String>,
    account: Option<String>,
    cursor: Option<String>,
    limit: Option<u32>,
}

impl<'a> PoolsRequestBuilder<'a> {
    /// Bind this builder to a Horizon client.
    pub fn new(client: &'a HorizonClient) -> Self {
        Self {
            client,
            reserves: None,
            account: None,
            cursor: None,
            limit: None,
        }
    }

    /// Filter by reserve assets (`native,USDC:G…`).
    pub fn reserves(mut self, reserves: impl Into<String>) -> Self {
        self.reserves = Some(reserves.into());
        self
    }

    /// Filter pools that `account` participates in.
    pub fn for_account(mut self, account: impl Into<String>) -> Self {
        self.account = Some(account.into());
        self
    }

    /// Paging cursor.
    pub fn cursor(mut self, cursor: impl Into<String>) -> Self {
        self.cursor = Some(cursor.into());
        self
    }

    /// Page size (max 200).
    pub fn limit(mut self, limit: u32) -> Self {
        self.limit = Some(limit.min(200));
        self
    }

    /// Fetch a single pool by hex id.
    pub async fn pool(&self, pool_id: &str) -> Result<LiquidityPool, SdkError> {
        self.client
            .get_json(&format!("liquidity_pools/{pool_id}"))
            .await
    }

    /// Execute the list query.
    pub async fn call(&self) -> Result<PoolsPage, SdkError> {
        let mut url = self.client.url("liquidity_pools")?;
        {
            let mut pairs = url.query_pairs_mut();
            if let Some(r) = &self.reserves {
                pairs.append_pair("reserves", r);
            }
            if let Some(a) = &self.account {
                pairs.append_pair("account", a);
            }
            if let Some(c) = &self.cursor {
                pairs.append_pair("cursor", c);
            }
            if let Some(l) = self.limit {
                pairs.append_pair("limit", &l.to_string());
            }
        }
        self.client.get_url_json(url).await
    }
}

impl HorizonClient {
    /// Start a liquidity-pool query builder.
    pub fn liquidity_pools(&self) -> PoolsRequestBuilder<'_> {
        PoolsRequestBuilder::new(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn share_fraction() {
        let pool = LiquidityPool {
            id: "abc".into(),
            paging_token: None,
            fee_bp: Some(30),
            pool_type: Some("constant_product".into()),
            total_shares: Some("100.0000000".into()),
            total_trustlines: Some(4),
            reserves: vec![
                LiquidityPoolReserve {
                    asset: "native".into(),
                    amount: "50.0000000".into(),
                },
                LiquidityPoolReserve {
                    asset: "USDC:GABC".into(),
                    amount: "50.0000000".into(),
                },
            ],
        };
        assert_eq!(pool.share_fraction("25.0000000"), Some(0.25));
        assert_eq!(pool.reserves.len(), 2);
    }
}
