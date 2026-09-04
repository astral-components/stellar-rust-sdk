//! Core transaction types: assets, memos, time bounds, and the unsigned tx body.

use crate::address::PublicKey;
use crate::errors::SdkError;
use crate::tx::operations::Operation;
use crate::tx::STROOPS_PER_XLM;

/// Stellar asset: native XLM or issued credit.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Asset {
    /// Native lumens.
    Native,
    /// Alphanumeric 4-character issued asset.
    CreditAlphaNum4 {
        /// Asset code (1–4 characters, 0-padded in XDR).
        code: String,
        /// Issuer public key.
        issuer: PublicKey,
    },
    /// Alphanumeric 12-character issued asset.
    CreditAlphaNum12 {
        /// Asset code (5–12 characters, 0-padded in XDR).
        code: String,
        /// Issuer public key.
        issuer: PublicKey,
    },
}

impl Asset {
    /// Native XLM.
    pub fn native() -> Self {
        Self::Native
    }

    /// Issued asset; chooses Alphanum4 vs Alphanum12 from code length.
    pub fn credit(code: &str, issuer: PublicKey) -> Result<Self, SdkError> {
        let code = code.trim();
        if code.is_empty() || code.len() > 12 || !code.chars().all(|c| c.is_ascii_alphanumeric()) {
            return Err(SdkError::message(format!("invalid asset code `{code}`")));
        }
        if code.len() <= 4 {
            Ok(Self::CreditAlphaNum4 {
                code: code.to_string(),
                issuer,
            })
        } else {
            Ok(Self::CreditAlphaNum12 {
                code: code.to_string(),
                issuer,
            })
        }
    }

    /// XDR `AssetType` discriminant.
    pub fn asset_type_discriminant(&self) -> i32 {
        match self {
            Self::Native => 0,
            Self::CreditAlphaNum4 { .. } => 1,
            Self::CreditAlphaNum12 { .. } => 2,
        }
    }

    /// Zero-padded asset code bytes (4 or 12) for XDR.
    pub fn code_bytes(&self) -> Option<Vec<u8>> {
        match self {
            Self::Native => None,
            Self::CreditAlphaNum4 { code, .. } => Some(pad_code(code, 4)),
            Self::CreditAlphaNum12 { code, .. } => Some(pad_code(code, 12)),
        }
    }

    /// Issuer when this is a credit asset.
    pub fn issuer(&self) -> Option<&PublicKey> {
        match self {
            Self::Native => None,
            Self::CreditAlphaNum4 { issuer, .. } | Self::CreditAlphaNum12 { issuer, .. } => {
                Some(issuer)
            }
        }
    }
}

fn pad_code(code: &str, len: usize) -> Vec<u8> {
    let mut out = vec![0u8; len];
    let bytes = code.as_bytes();
    out[..bytes.len()].copy_from_slice(bytes);
    out
}

/// Transaction memo.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Memo {
    /// No memo.
    None,
    /// UTF-8 text, max 28 bytes.
    Text(String),
    /// 64-bit id.
    Id(u64),
    /// 32-byte hash.
    Hash([u8; 32]),
    /// 32-byte return hash.
    Return([u8; 32]),
}

impl Memo {
    /// Text memo; rejects payloads longer than 28 bytes.
    pub fn text(s: impl Into<String>) -> Result<Self, SdkError> {
        let s = s.into();
        if s.len() > 28 {
            return Err(SdkError::message("memo text exceeds 28 bytes"));
        }
        Ok(Self::Text(s))
    }

    /// XDR `MemoType` discriminant.
    pub fn discriminant(&self) -> i32 {
        match self {
            Self::None => 0,
            Self::Text(_) => 1,
            Self::Id(_) => 2,
            Self::Hash(_) => 3,
            Self::Return(_) => 4,
        }
    }
}

/// Inclusive unix-time window during which the transaction is valid.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TimeBounds {
    /// Minimum unix time (0 = none).
    pub min_time: u64,
    /// Maximum unix time (0 = none).
    pub max_time: u64,
}

impl TimeBounds {
    /// Unbounded.
    pub fn none() -> Self {
        Self {
            min_time: 0,
            max_time: 0,
        }
    }

    /// Valid until `max_time` (unix seconds).
    pub fn until(max_time: u64) -> Self {
        Self {
            min_time: 0,
            max_time,
        }
    }
}

/// Unsigned transaction body.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Transaction {
    /// Source account public key.
    pub source: PublicKey,
    /// Sequence number (must be source.sequence + 1 at submission).
    pub sequence: i64,
    /// Total fee in stroops (`base_fee * operations.len()`).
    pub fee: u32,
    /// Optional time bounds.
    pub time_bounds: Option<TimeBounds>,
    /// Memo.
    pub memo: Memo,
    /// Operations to apply.
    pub operations: Vec<Operation>,
}

impl Transaction {
    /// Next sequence after this transaction is consumed.
    pub fn next_sequence(&self) -> i64 {
        self.sequence.saturating_add(1)
    }
}

/// Parse a decimal amount (`"1.5"` or `"1.5000000"`) into stroops.
pub fn parse_amount_stroops(amount: &str) -> Result<i64, SdkError> {
    let amount = amount.trim();
    if amount.is_empty() {
        return Err(SdkError::message("empty amount"));
    }
    let negative = amount.starts_with('-');
    let amount = amount.trim_start_matches('-');
    let (whole, frac) = match amount.split_once('.') {
        Some((w, f)) => (w, f),
        None => (amount, ""),
    };
    if whole.is_empty() || !whole.chars().all(|c| c.is_ascii_digit()) {
        return Err(SdkError::message(format!("invalid amount `{amount}`")));
    }
    if frac.len() > 7 || !frac.chars().all(|c| c.is_ascii_digit()) {
        return Err(SdkError::message(format!(
            "invalid stroop fraction `{frac}`"
        )));
    }
    let whole_i: i64 = whole
        .parse()
        .map_err(|_| SdkError::message("amount overflow"))?;
    let mut frac_padded = frac.to_string();
    while frac_padded.len() < 7 {
        frac_padded.push('0');
    }
    let frac_i: i64 = if frac_padded.is_empty() {
        0
    } else {
        frac_padded
            .parse()
            .map_err(|_| SdkError::message("amount overflow"))?
    };
    let stroops = whole_i
        .checked_mul(STROOPS_PER_XLM)
        .and_then(|v| v.checked_add(frac_i))
        .ok_or_else(|| SdkError::message("amount overflow"))?;
    Ok(if negative { -stroops } else { stroops })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tx::xlm_to_stroops;

    #[test]
    fn parse_amounts() {
        assert_eq!(parse_amount_stroops("1").unwrap(), xlm_to_stroops(1));
        assert_eq!(parse_amount_stroops("1.5").unwrap(), 15_000_000);
        assert_eq!(parse_amount_stroops("0.0000001").unwrap(), 1);
        assert!(parse_amount_stroops("1.12345678").is_err());
    }

    #[test]
    fn memo_text_limit() {
        assert!(Memo::text("hello").is_ok());
        assert!(Memo::text("x".repeat(29)).is_err());
    }
}
