//! Transaction operations.

mod claimable_balances;
mod offers;
mod payment;

pub use claimable_balances::{ClaimClaimableBalance, ClaimPredicate, Claimant, CreateClaimableBalance};
pub use offers::{ManageBuyOffer, ManageSellOffer, Price};
pub use payment::{CreateAccount, Payment};

use crate::tx::types::Asset;
use crate::tx::xdr::XdrWriter;

/// Any supported transaction operation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Operation {
    /// Create and fund a new account.
    CreateAccount(CreateAccount),
    /// Payment of native or issued assets.
    Payment(Payment),
    /// Create a claimable balance.
    CreateClaimableBalance(CreateClaimableBalance),
    /// Claim a claimable balance.
    ClaimClaimableBalance(ClaimClaimableBalance),
    /// Create / update / delete a sell offer.
    ManageSellOffer(ManageSellOffer),
    /// Create / update / delete a buy offer.
    ManageBuyOffer(ManageBuyOffer),
}

impl Operation {
    /// XDR `OperationType` discriminant.
    pub fn discriminant(&self) -> i32 {
        match self {
            Self::CreateAccount(_) => 0,
            Self::Payment(_) => 1,
            Self::ManageSellOffer(_) => 3,
            Self::ManageBuyOffer(_) => 12,
            Self::CreateClaimableBalance(_) => 14,
            Self::ClaimClaimableBalance(_) => 15,
        }
    }

    /// Encode the operation (including optional source account = none).
    pub fn write_xdr(&self, w: &mut XdrWriter) {
        w.write_bool(false); // sourceAccount not present
        w.write_i32(self.discriminant());
        match self {
            Self::CreateAccount(op) => op.write_xdr(w),
            Self::Payment(op) => op.write_xdr(w),
            Self::CreateClaimableBalance(op) => op.write_xdr(w),
            Self::ClaimClaimableBalance(op) => op.write_xdr(w),
            Self::ManageSellOffer(op) => op.write_xdr(w),
            Self::ManageBuyOffer(op) => op.write_xdr(w),
        }
    }
}

pub(crate) fn write_asset(w: &mut XdrWriter, asset: &Asset) {
    w.write_i32(asset.asset_type_discriminant());
    match asset {
        Asset::Native => {}
        Asset::CreditAlphaNum4 { issuer, .. } | Asset::CreditAlphaNum12 { issuer, .. } => {
            let code = asset.code_bytes().expect("credit asset has code");
            w.write_opaque_fixed(&code);
            w.write_account_id(issuer.as_bytes());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn discriminants_match_stellar_xdr() {
        // Values from Stellar `OperationType` in Stellar-transaction.x
        assert_eq!(Operation::CreateAccount(CreateAccount {
            destination: [0u8; 32],
            starting_balance: 1,
        }).discriminant(), 0);
        assert_eq!(
            Operation::ManageBuyOffer(ManageBuyOffer {
                selling: Asset::Native,
                buying: Asset::Native,
                buy_amount: 1,
                price: Price { n: 1, d: 1 },
                offer_id: 0,
            })
            .discriminant(),
            12
        );
    }
}
