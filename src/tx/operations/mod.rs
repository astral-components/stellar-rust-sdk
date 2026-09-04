//! Transaction operations.

use crate::address::PublicKey;

/// Any supported transaction operation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Operation {
    /// Skeleton variant used by [`TransactionBuilder`] until typed ops land.
    Skeleton,
}

/// Placeholder payment builder; replaced by the full Payment type next commit.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Payment;

impl Payment {
    /// Native XLM payment skeleton (amount recorded when the full op lands).
    pub fn native(_destination: &PublicKey, _stroops: i64) -> Operation {
        Operation::Skeleton
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn skeleton_payment_converts() {
        let pk = crate::address::PublicKey::from_payload([1u8; 32]).unwrap();
        let op: Operation = Payment::native(&pk, 1);
        assert_eq!(op, Operation::Skeleton);
    }
}
