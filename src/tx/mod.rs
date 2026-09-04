//! Transaction construction, operations, envelopes, and XDR.

pub mod builder;
pub mod operations;
pub mod types;
mod xdr;

pub use builder::TransactionBuilder;
pub use types::{Memo, TimeBounds, Transaction};

/// One stroop is `10^-7` XLM.
pub const STROOPS_PER_XLM: i64 = 10_000_000;

/// Protocol default base fee in stroops (100 stroops = 0.00001 XLM).
pub const MIN_BASE_FEE: u32 = 100;

/// Converts whole XLM into stroops.
pub fn xlm_to_stroops(xlm: i64) -> i64 {
    xlm.saturating_mul(STROOPS_PER_XLM)
}

/// Converts stroops into whole XLM, truncating toward zero.
pub fn stroops_to_xlm(stroops: i64) -> i64 {
    stroops / STROOPS_PER_XLM
}

/// Converts a decimal XLM string (`"1.5000000"`) into stroops.
pub fn parse_stroops(amount: &str) -> Result<i64, crate::errors::SdkError> {
    types::parse_amount_stroops(amount)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stroop_conversion_roundtrip_whole_xlm() {
        assert_eq!(xlm_to_stroops(1), 10_000_000);
        assert_eq!(stroops_to_xlm(10_000_000), 1);
        assert_eq!(MIN_BASE_FEE, 100);
        assert_eq!(parse_stroops("2.0000000").unwrap(), 20_000_000);
    }
}
