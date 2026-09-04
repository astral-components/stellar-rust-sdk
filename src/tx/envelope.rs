//! Transaction envelope signing and base64 XDR serialization.

use crate::errors::SdkError;
use crate::keypair::Keypair;
use crate::network::Network;
use crate::tx::types::{Memo, Transaction};
use crate::tx::xdr::XdrWriter;
use base64::{engine::general_purpose::STANDARD, Engine as _};

/// Ed25519 decorated signature (hint + 64-byte signature).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DecoratedSignature {
    /// Last 4 bytes of the signer public key.
    pub hint: [u8; 4],
    /// Ed25519 signature bytes.
    pub signature: [u8; 64],
}

/// Signed (or partially signed) transaction envelope.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TransactionEnvelope {
    tx: Transaction,
    signatures: Vec<DecoratedSignature>,
}

impl TransactionEnvelope {
    /// Wrap an unsigned transaction.
    pub fn new(tx: Transaction) -> Self {
        Self {
            tx,
            signatures: Vec::new(),
        }
    }

    /// Inner unsigned transaction.
    pub fn transaction(&self) -> &Transaction {
        &self.tx
    }

    /// Current signatures.
    pub fn signatures(&self) -> &[DecoratedSignature] {
        &self.signatures
    }

    /// SHA-256 transaction hash for `network` (the payload that signers sign).
    pub fn hash(&self, network: &Network) -> [u8; 32] {
        network.hash_transaction(&self.tagged_transaction_xdr())
    }

    /// Sign with `keypair` and append the decorated signature.
    pub fn sign(&mut self, keypair: &Keypair, network: &Network) -> Result<(), SdkError> {
        let hash = self.hash(network);
        let signature = keypair.sign(&hash);
        keypair.verify(&hash, &signature)?;
        self.signatures.push(DecoratedSignature {
            hint: keypair.signature_hint(),
            signature,
        });
        Ok(())
    }

    /// Convenience: consume, sign, and return.
    pub fn signed(mut self, keypair: &Keypair, network: &Network) -> Result<Self, SdkError> {
        self.sign(keypair, network)?;
        Ok(self)
    }

    /// Append an already-produced signature (multi-sig).
    pub fn append_signature(&mut self, sig: DecoratedSignature) {
        self.signatures.push(sig);
    }

    /// Serialize as `TransactionEnvelope` XDR, then base64.
    pub fn to_base64_xdr(&self) -> String {
        STANDARD.encode(self.to_xdr())
    }

    /// Raw `TransactionEnvelope` XDR bytes.
    pub fn to_xdr(&self) -> Vec<u8> {
        let mut w = XdrWriter::new();
        w.write_i32(2); // ENVELOPE_TYPE_TX
        write_transaction(&mut w, &self.tx);
        w.write_u32(self.signatures.len() as u32);
        for sig in &self.signatures {
            w.write_opaque_fixed(&sig.hint);
            w.write_opaque_var(&sig.signature);
        }
        w.into_inner()
    }

    fn tagged_transaction_xdr(&self) -> Vec<u8> {
        let mut w = XdrWriter::new();
        w.write_i32(2); // ENVELOPE_TYPE_TX
        write_transaction(&mut w, &self.tx);
        w.into_inner()
    }
}

pub(crate) fn write_transaction(w: &mut XdrWriter, tx: &Transaction) {
    w.write_muxed_ed25519(tx.source.as_bytes());
    w.write_u32(tx.fee);
    w.write_i64(tx.sequence);
    write_preconditions(w, tx.time_bounds.as_ref());
    write_memo(w, &tx.memo);
    w.write_u32(tx.operations.len() as u32);
    for op in &tx.operations {
        op.write_xdr(w);
    }
    w.write_i32(0); // TransactionExt v = 0
}

fn write_preconditions(w: &mut XdrWriter, bounds: Option<&crate::tx::types::TimeBounds>) {
    match bounds {
        None => w.write_i32(0), // PRECOND_NONE
        Some(b) => {
            w.write_i32(1); // PRECOND_TIME
            w.write_u64(b.min_time);
            w.write_u64(b.max_time);
        }
    }
}

fn write_memo(w: &mut XdrWriter, memo: &Memo) {
    w.write_i32(memo.discriminant());
    match memo {
        Memo::None => {}
        Memo::Text(s) => w.write_string(s),
        Memo::Id(id) => w.write_u64(*id),
        Memo::Hash(h) | Memo::Return(h) => w.write_opaque_fixed(h),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::address::PublicKey;
    use crate::tx::builder::TransactionBuilder;
    use crate::tx::operations::Payment;
    use crate::tx::xlm_to_stroops;

    #[test]
    fn sign_and_encode_base64() {
        let src_kp = Keypair::random().unwrap();
        let dst = PublicKey::from_payload([9u8; 32]).unwrap();
        let tx = TransactionBuilder::new(src_kp.public_key().clone(), 1)
            .add_operation(Payment::native(&dst, xlm_to_stroops(1)))
            .build()
            .unwrap();
        let env = TransactionEnvelope::new(tx)
            .signed(&src_kp, &Network::Testnet)
            .unwrap();
        assert_eq!(env.signatures().len(), 1);
        let b64 = env.to_base64_xdr();
        assert!(!b64.is_empty());
        STANDARD.decode(&b64).unwrap();
        let hash = env.hash(&Network::Testnet);
        src_kp.verify(&hash, &env.signatures()[0].signature).unwrap();
    }

    #[test]
    fn multi_sig_appends() {
        let a = Keypair::random().unwrap();
        let extra = Keypair::random().unwrap();
        let dst = PublicKey::from_payload([8u8; 32]).unwrap();
        let tx = TransactionBuilder::new(a.public_key().clone(), 1)
            .add_operation(Payment::native(&dst, 1))
            .build()
            .unwrap();
        let mut env = TransactionEnvelope::new(tx);
        env.sign(&a, &Network::Testnet).unwrap();
        env.sign(&extra, &Network::Testnet).unwrap();
        assert_eq!(env.signatures().len(), 2);
        assert_ne!(env.signatures()[0].hint, env.signatures()[1].hint);
    }
}
