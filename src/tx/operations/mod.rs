//! Transaction operations.

mod payment;

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
}

impl Operation {
    /// XDR `OperationType` discriminant.
    pub fn discriminant(&self) -> i32 {
        match self {
            Self::CreateAccount(_) => 0,
            Self::Payment(_) => 1,
        }
    }

    /// Encode the operation (including optional source account = none).
    pub fn write_xdr(&self, w: &mut XdrWriter) {
        w.write_bool(false);
        w.write_i32(self.discriminant());
        match self {
            Self::CreateAccount(op) => op.write_xdr(w),
            Self::Payment(op) => op.write_xdr(w),
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
        assert_eq!(
            Operation::CreateAccount(CreateAccount {
                destination: [0u8; 32],
                starting_balance: 1,
            })
            .discriminant(),
            0
        );
        assert_eq!(
            Operation::Payment(Payment {
                destination: [0u8; 32],
                asset: Asset::Native,
                amount: 1,
            })
            .discriminant(),
            1
        );
    }
}
