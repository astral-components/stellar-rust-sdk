//! Horizon asset list and issuer-detail queries.

use crate::errors::SdkError;
use crate::horizon::client::HorizonClient;
use serde::{Deserialize, Serialize};

/// Asset identity as returned by Horizon.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HorizonAsset {
    /// `native`, `credit_alphanum4`, or `credit_alphanum12`.
    pub asset_type: String,
    /// Asset code (absent for native).
    #[serde(default)]
    pub asset_code: Option<String>,
    /// Issuer account (absent for native).
    #[serde(default)]
    pub asset_issuer: Option<String>,
}

impl HorizonAsset {
    /// Native XLM asset.
    pub fn native() -> Self {
        Self {
            asset_type: "native".into(),
            asset_code: None,
            asset_issuer: None,
        }
    }

    /// Issued credit asset (`CODE:ISSUER` query form).
    pub fn credit(code: impl Into<String>, issuer: impl Into<String>) -> Self {
        let code = code.into();
        let asset_type = if code.len() <= 4 {
            "credit_alphanum4"
        } else {
            "credit_alphanum12"
        };
        Self {
            asset_type: asset_type.into(),
            asset_code: Some(code),
            asset_issuer: Some(issuer.into()),
        }
    }

    /// Horizon `asset` query parameter (`native` or `CODE:ISSUER`).
    pub fn query_param(&self) -> String {
        match (&self.asset_code, &self.asset_issuer) {
            (Some(code), Some(issuer)) => format!("{code}:{issuer}"),
            _ => "native".to_string(),
        }
    }
}

/// Asset statistics record from `GET /assets`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AssetRecord {
    /// Asset type.
    pub asset_type: String,
    /// Asset code.
    #[serde(default)]
    pub asset_code: Option<String>,
    /// Issuer account.
    #[serde(default)]
    pub asset_issuer: Option<String>,
    /// Number of accounts holding this asset.
    #[serde(default)]
    pub num_accounts: Option<u64>,
    /// Number of claimable balances.
    #[serde(default)]
    pub num_claimable_balances: Option<u64>,
    /// Number of liquidity pools.
    #[serde(default)]
    pub num_liquidity_pools: Option<u64>,
    /// Amount held in accounts.
    #[serde(default)]
    pub amount: Option<String>,
    /// Amount in claimable balances.
    #[serde(default)]
    pub claimable_balances_amount: Option<String>,
    /// Amount in liquidity pools.
    #[serde(default)]
    pub liquidity_pools_amount: Option<String>,
    /// Flags on the issuer account.
    #[serde(default)]
    pub flags: Option<crate::horizon::accounts::AccountFlags>,
}

/// HAL collection for `/assets`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetsPage {
    /// Embedded records.
    #[serde(rename = "_embedded")]
    pub embedded: AssetsEmbedded,
}

/// `_embedded` object for asset collections.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetsEmbedded {
    /// Asset records.
    pub records: Vec<AssetRecord>,
}

/// Query builder for `GET /assets`.
#[derive(Debug, Clone)]
pub struct AssetsRequestBuilder<'a> {
    client: &'a HorizonClient,
    asset_code: Option<String>,
    asset_issuer: Option<String>,
    cursor: Option<String>,
    limit: Option<u32>,
}

impl<'a> AssetsRequestBuilder<'a> {
    /// Bind this builder to a Horizon client.
    pub fn new(client: &'a HorizonClient) -> Self {
        Self {
            client,
            asset_code: None,
            asset_issuer: None,
            cursor: None,
            limit: None,
        }
    }

    /// Filter by asset code.
    pub fn asset_code(mut self, code: impl Into<String>) -> Self {
        self.asset_code = Some(code.into());
        self
    }

    /// Filter by issuer account.
    pub fn asset_issuer(mut self, issuer: impl Into<String>) -> Self {
        self.asset_issuer = Some(issuer.into());
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

    /// Execute the query.
    pub async fn call(&self) -> Result<AssetsPage, SdkError> {
        let mut url = self.client.url("assets")?;
        {
            let mut pairs = url.query_pairs_mut();
            if let Some(c) = &self.asset_code {
                pairs.append_pair("asset_code", c);
            }
            if let Some(i) = &self.asset_issuer {
                pairs.append_pair("asset_issuer", i);
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
    /// Start an assets query builder.
    pub fn assets(&self) -> AssetsRequestBuilder<'_> {
        AssetsRequestBuilder::new(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn credit_query_param_and_type() {
        let usdc = HorizonAsset::credit("USDC", "GA5ZSEJYB37JRC5AVCIA5MOP4RHTM335X2KGX3IHOJAPP5RE34K4KZVN");
        assert_eq!(usdc.asset_type, "credit_alphanum4");
        assert!(usdc.query_param().starts_with("USDC:G"));
        let long = HorizonAsset::credit("LONGERTOKENX", "GTEST");
        assert_eq!(long.asset_type, "credit_alphanum12");
        assert_eq!(HorizonAsset::native().query_param(), "native");
    }
}
