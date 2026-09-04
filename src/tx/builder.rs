//! Type-safe [`TransactionBuilder`].

use crate::address::PublicKey;
use crate::errors::SdkError;
use crate::tx::operations::Operation;
use crate::tx::types::{Memo, TimeBounds, Transaction};
use crate::tx::MIN_BASE_FEE;

/// Fluent builder for an unsigned [`Transaction`].
#[derive(Debug, Clone)]
pub struct TransactionBuilder {
    source: PublicKey,
    sequence: i64,
    base_fee: u32,
    time_bounds: Option<TimeBounds>,
    memo: Memo,
    operations: Vec<Operation>,
}

impl TransactionBuilder {
    /// Start a builder for `source` with the account's *current* sequence.
    ///
    /// The transaction will use `sequence + 1`, matching Horizon semantics.
    pub fn new(source: PublicKey, current_sequence: i64) -> Self {
        Self {
            source,
            sequence: current_sequence.saturating_add(1),
            base_fee: MIN_BASE_FEE,
            time_bounds: None,
            memo: Memo::None,
            operations: Vec::new(),
        }
    }

    /// Override the sequence number actually placed in the envelope.
    pub fn sequence(mut self, sequence: i64) -> Self {
        self.sequence = sequence;
        self
    }

    /// Base fee per operation in stroops (protocol minimum is 100).
    pub fn base_fee(mut self, stroops: u32) -> Self {
        self.base_fee = stroops.max(MIN_BASE_FEE);
        self
    }

    /// Set time bounds.
    pub fn time_bounds(mut self, bounds: TimeBounds) -> Self {
        self.time_bounds = Some(bounds);
        self
    }

    /// Valid until unix `max_time`.
    pub fn valid_until(mut self, max_time: u64) -> Self {
        self.time_bounds = Some(TimeBounds::until(max_time));
        self
    }

    /// Attach a memo.
    pub fn memo(mut self, memo: Memo) -> Self {
        self.memo = memo;
        self
    }

    /// Append an operation.
    pub fn add_operation(mut self, op: impl Into<Operation>) -> Self {
        self.operations.push(op.into());
        self
    }

    /// Build the unsigned transaction.
    pub fn build(self) -> Result<Transaction, SdkError> {
        if self.operations.is_empty() {
            return Err(SdkError::message(
                "transaction must contain at least one operation",
            ));
        }
        if self.operations.len() > 100 {
            return Err(SdkError::message("transaction exceeds 100 operations"));
        }
        let fee = self
            .base_fee
            .saturating_mul(self.operations.len() as u32);
        Ok(Transaction {
            source: self.source,
            sequence: self.sequence,
            fee,
            time_bounds: self.time_bounds,
            memo: self.memo,
            operations: self.operations,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tx::operations::Payment;
    use crate::tx::xlm_to_stroops;

    #[test]
    fn builds_payment_with_fee_and_sequence() {
        let src = PublicKey::from_payload([1u8; 32]).unwrap();
        let dst = PublicKey::from_payload([2u8; 32]).unwrap();
        let tx = TransactionBuilder::new(src, 10)
            .base_fee(200)
            .memo(Memo::text("hi").unwrap())
            .time_bounds(TimeBounds::until(1_700_000_000))
            .add_operation(Payment::native(&dst, xlm_to_stroops(1)))
            .build()
            .unwrap();
        assert_eq!(tx.sequence, 11);
        assert_eq!(tx.fee, 200);
        assert_eq!(tx.operations.len(), 1);
        assert!(matches!(tx.memo, Memo::Text(_)));
    }

    #[test]
    fn rejects_empty_operations() {
        let src = PublicKey::from_payload([1u8; 32]).unwrap();
        assert!(TransactionBuilder::new(src, 0).build().is_err());
    }
}
