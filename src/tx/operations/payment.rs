//! `CreateAccount` and `Payment` operations with stroop amounts.

use crate::address::PublicKey;
use crate::errors::SdkError;
use crate::tx::operations::{write_asset, Operation};
use crate::tx::types::{parse_amount_stroops, Asset};
use crate::tx::xdr::XdrWriter;
use crate::tx::xlm_to_stroops;

/// Create and fund a new account (`OperationType::CREATE_ACCOUNT = 0`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CreateAccount {
    /// Destination public key (must not already exist).
    pub destination: [u8; 32],
    /// Starting balance in stroops.
    pub starting_balance: i64,
}

impl CreateAccount {
    /// Build from a `G…` destination and XLM starting balance.
    pub fn new(destination: &PublicKey, starting_xlm: i64) -> Self {
        Self {
            destination: *destination.as_bytes(),
            starting_balance: xlm_to_stroops(starting_xlm),
        }
    }

    /// Build from a decimal XLM string.
    pub fn from_xlm_str(destination: &PublicKey, amount: &str) -> Result<Self, SdkError> {
        Ok(Self {
            destination: *destination.as_bytes(),
            starting_balance: parse_amount_stroops(amount)?,
        })
    }

    pub(crate) fn write_xdr(&self, w: &mut XdrWriter) {
        w.write_account_id(&self.destination);
        w.write_i64(self.starting_balance);
    }
}

impl From<CreateAccount> for Operation {
    fn from(op: CreateAccount) -> Self {
        Operation::CreateAccount(op)
    }
}

/// Payment of native or issued assets (`OperationType::PAYMENT = 1`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Payment {
    /// Destination muxed-account ed25519 key (classic `G…`).
    pub destination: [u8; 32],
    /// Asset to send.
    pub asset: Asset,
    /// Amount in stroops.
    pub amount: i64,
}

impl Payment {
    /// Native XLM payment.
    pub fn native(destination: &PublicKey, stroops: i64) -> Self {
        Self {
            destination: *destination.as_bytes(),
            asset: Asset::Native,
            amount: stroops,
        }
    }

    /// Native XLM payment from a decimal string.
    pub fn native_xlm(destination: &PublicKey, xlm: &str) -> Result<Self, SdkError> {
        Ok(Self::native(destination, parse_amount_stroops(xlm)?))
    }

    /// Issued-asset payment.
    pub fn credit(destination: &PublicKey, asset: Asset, stroops: i64) -> Self {
        Self {
            destination: *destination.as_bytes(),
            asset,
            amount: stroops,
        }
    }

    pub(crate) fn write_xdr(&self, w: &mut XdrWriter) {
        w.write_muxed_ed25519(&self.destination);
        write_asset(w, &self.asset);
        w.write_i64(self.amount);
    }
}

impl From<Payment> for Operation {
    fn from(op: Payment) -> Self {
        Operation::Payment(op)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tx::STROOPS_PER_XLM;

    #[test]
    fn native_payment_uses_stroops() {
        let dest = PublicKey::from_payload([1u8; 32]).unwrap();
        let op = Payment::native_xlm(&dest, "1.0000000").unwrap();
        assert_eq!(op.amount, STROOPS_PER_XLM);
        assert_eq!(op.asset, Asset::Native);
    }

    #[test]
    fn create_account_converts_xlm() {
        let dest = PublicKey::from_payload([2u8; 32]).unwrap();
        let op = CreateAccount::new(&dest, 2);
        assert_eq!(op.starting_balance, 2 * STROOPS_PER_XLM);
    }
}
