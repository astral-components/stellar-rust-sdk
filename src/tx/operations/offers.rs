//! DEX manage-sell and manage-buy offer operations.

use crate::tx::operations::{write_asset, Operation};
use crate::tx::types::Asset;
use crate::tx::xdr::XdrWriter;

/// Rational price `n/d` in terms of the buying asset per selling asset.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Price {
    /// Numerator.
    pub n: i32,
    /// Denominator.
    pub d: i32,
}

impl Price {
    /// Construct a price, rejecting a zero denominator.
    pub fn new(n: i32, d: i32) -> Result<Self, crate::errors::SdkError> {
        if d == 0 {
            return Err(crate::errors::SdkError::message("price denominator is 0"));
        }
        Ok(Self { n, d })
    }

    pub(crate) fn write_xdr(self, w: &mut XdrWriter) {
        w.write_i32(self.n);
        w.write_i32(self.d);
    }
}

/// Create / update / delete a sell offer (`OperationType = 3`).
///
/// Set `amount = 0` to delete; set `offer_id = 0` to create.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ManageSellOffer {
    /// Asset being sold.
    pub selling: Asset,
    /// Asset being bought.
    pub buying: Asset,
    /// Amount of `selling` in stroops (0 deletes).
    pub amount: i64,
    /// Price of selling in terms of buying.
    pub price: Price,
    /// Existing offer id, or 0 to create.
    pub offer_id: i64,
}

impl ManageSellOffer {
    /// Create a new sell offer.
    pub fn create(selling: Asset, buying: Asset, amount: i64, price: Price) -> Self {
        Self {
            selling,
            buying,
            amount,
            price,
            offer_id: 0,
        }
    }

    pub(crate) fn write_xdr(&self, w: &mut XdrWriter) {
        write_asset(w, &self.selling);
        write_asset(w, &self.buying);
        w.write_i64(self.amount);
        self.price.write_xdr(w);
        w.write_i64(self.offer_id);
    }
}

impl From<ManageSellOffer> for Operation {
    fn from(op: ManageSellOffer) -> Self {
        Operation::ManageSellOffer(op)
    }
}

/// Create / update / delete a buy offer (`OperationType = 12`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ManageBuyOffer {
    /// Asset being sold (the quote the buyer pays).
    pub selling: Asset,
    /// Asset being bought.
    pub buying: Asset,
    /// Amount of `buying` to purchase in stroops.
    pub buy_amount: i64,
    /// Price of buying in terms of selling.
    pub price: Price,
    /// Existing offer id, or 0 to create.
    pub offer_id: i64,
}

impl ManageBuyOffer {
    /// Create a new buy offer.
    pub fn create(selling: Asset, buying: Asset, buy_amount: i64, price: Price) -> Self {
        Self {
            selling,
            buying,
            buy_amount,
            price,
            offer_id: 0,
        }
    }

    pub(crate) fn write_xdr(&self, w: &mut XdrWriter) {
        write_asset(w, &self.selling);
        write_asset(w, &self.buying);
        w.write_i64(self.buy_amount);
        self.price.write_xdr(w);
        w.write_i64(self.offer_id);
    }
}

impl From<ManageBuyOffer> for Operation {
    fn from(op: ManageBuyOffer) -> Self {
        Operation::ManageBuyOffer(op)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn price_rejects_zero_denominator() {
        assert!(Price::new(1, 0).is_err());
        assert_eq!(Price::new(1, 2).unwrap(), Price { n: 1, d: 2 });
    }

    #[test]
    fn create_sets_offer_id_zero() {
        let op = ManageSellOffer::create(Asset::Native, Asset::Native, 100, Price { n: 1, d: 1 });
        assert_eq!(op.offer_id, 0);
        let buy = ManageBuyOffer::create(Asset::Native, Asset::Native, 50, Price { n: 1, d: 1 });
        assert_eq!(buy.offer_id, 0);
    }
}
