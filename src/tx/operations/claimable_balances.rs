//! Claimable balance operations.

use crate::address::PublicKey;
use crate::tx::operations::{write_asset, Operation};
use crate::tx::types::Asset;
use crate::tx::xdr::XdrWriter;

/// Predicate that must be satisfied to claim a balance.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ClaimPredicate {
    /// Always claimable.
    Unconditional,
}

impl ClaimPredicate {
    pub(crate) fn write_xdr(&self, w: &mut XdrWriter) {
        match self {
            Self::Unconditional => w.write_i32(0), // CLAIM_PREDICATE_UNCONDITIONAL
        }
    }
}

/// Claimant of a claimable balance.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Claimant {
    /// Destination account.
    pub destination: [u8; 32],
    /// Predicate.
    pub predicate: ClaimPredicate,
}

impl Claimant {
    /// Unconditional claimant.
    pub fn unconditional(destination: &PublicKey) -> Self {
        Self {
            destination: *destination.as_bytes(),
            predicate: ClaimPredicate::Unconditional,
        }
    }

    pub(crate) fn write_xdr(&self, w: &mut XdrWriter) {
        w.write_i32(0); // CLAIMANT_TYPE_V0
        w.write_account_id(&self.destination);
        self.predicate.write_xdr(w);
    }
}

/// Create a claimable balance (`OperationType = 14`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CreateClaimableBalance {
    /// Asset to lock.
    pub asset: Asset,
    /// Amount in stroops.
    pub amount: i64,
    /// Claimants (max 10 per protocol).
    pub claimants: Vec<Claimant>,
}

impl CreateClaimableBalance {
    /// Construct with a single unconditional claimant.
    pub fn new(asset: Asset, amount: i64, claimant: &PublicKey) -> Self {
        Self {
            asset,
            amount,
            claimants: vec![Claimant::unconditional(claimant)],
        }
    }

    pub(crate) fn write_xdr(&self, w: &mut XdrWriter) {
        write_asset(w, &self.asset);
        w.write_i64(self.amount);
        w.write_u32(self.claimants.len() as u32);
        for c in &self.claimants {
            c.write_xdr(w);
        }
    }
}

impl From<CreateClaimableBalance> for Operation {
    fn from(op: CreateClaimableBalance) -> Self {
        Operation::CreateClaimableBalance(op)
    }
}

/// Claim a claimable balance (`OperationType = 15`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClaimClaimableBalance {
    /// 32-byte balance id payload (claimable balance id without StrKey wrapper).
    pub balance_id: [u8; 32],
}

impl ClaimClaimableBalance {
    /// Construct from raw 32-byte balance id.
    pub fn new(balance_id: [u8; 32]) -> Self {
        Self { balance_id }
    }

    pub(crate) fn write_xdr(&self, w: &mut XdrWriter) {
        // ClaimableBalanceID: type HASH (0) + Hash
        w.write_i32(0);
        w.write_opaque_fixed(&self.balance_id);
    }
}

impl From<ClaimClaimableBalance> for Operation {
    fn from(op: ClaimClaimableBalance) -> Self {
        Operation::ClaimClaimableBalance(op)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unconditional_claimant() {
        let pk = PublicKey::from_payload([3u8; 32]).unwrap();
        let op = CreateClaimableBalance::new(Asset::Native, 1_000, &pk);
        assert_eq!(op.claimants.len(), 1);
        assert_eq!(op.claimants[0].predicate, ClaimPredicate::Unconditional);
    }
}
