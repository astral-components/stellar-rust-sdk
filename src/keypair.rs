//! Ed25519 keypair generation, seed import, and payload signing.
//!
//! Stellar transaction signatures are Ed25519 over the 32-byte SHA-256
//! transaction hash (`network_id || sha256(tagged_tx)`). [`Keypair::sign`]
//! accepts any byte slice so callers can pass either a raw hash or other
//! payloads.

use crate::address::{PublicKey, SecretSeed};
use crate::errors::SdkError;
use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};
use rand::rngs::OsRng;

/// Ed25519 account keypair (32-byte seed + derived public key).
#[derive(Clone)]
pub struct Keypair {
    signing: SigningKey,
    public: PublicKey,
}

impl std::fmt::Debug for Keypair {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Keypair")
            .field("public", &self.public)
            .finish_non_exhaustive()
    }
}

impl Keypair {
    /// Generate a cryptographically random keypair from the OS CSPRNG.
    pub fn random() -> Result<Self, SdkError> {
        let signing = SigningKey::generate(&mut OsRng);
        Self::from_signing(signing)
    }

    /// Import from a 32-byte seed.
    pub fn from_seed_bytes(seed: [u8; 32]) -> Result<Self, SdkError> {
        Self::from_signing(SigningKey::from_bytes(&seed))
    }

    /// Import from an `S…` secret seed StrKey.
    pub fn from_secret_seed(seed: &str) -> Result<Self, SdkError> {
        let secret = SecretSeed::from_strkey(seed)?;
        Self::from_seed_bytes(*secret.as_bytes())
    }

    fn from_signing(signing: SigningKey) -> Result<Self, SdkError> {
        let vk: VerifyingKey = signing.verifying_key();
        let public = PublicKey::from_payload(vk.to_bytes())?;
        Ok(Self { signing, public })
    }

    /// Corresponding `G…` public key.
    pub fn public_key(&self) -> &PublicKey {
        &self.public
    }

    /// 32-byte secret seed.
    pub fn seed_bytes(&self) -> [u8; 32] {
        self.signing.to_bytes()
    }

    /// `S…` StrKey encoding of the secret seed.
    pub fn secret_seed(&self) -> Result<SecretSeed, SdkError> {
        SecretSeed::from_payload(self.seed_bytes())
    }

    /// Sign a raw byte payload (typically a 32-byte SHA-256 transaction hash).
    pub fn sign(&self, payload: &[u8]) -> [u8; 64] {
        let sig: Signature = self.signing.sign(payload);
        sig.to_bytes()
    }

    /// Verify a signature produced by [`Keypair::sign`].
    pub fn verify(&self, payload: &[u8], signature: &[u8; 64]) -> Result<(), SdkError> {
        let sig = Signature::from_bytes(signature);
        let vk = VerifyingKey::from_bytes(self.public.as_bytes()).map_err(|e| {
            SdkError::Signing(format!("invalid verifying key: {e}"))
        })?;
        vk.verify(payload, &sig)
            .map_err(|e| SdkError::Signing(format!("signature verification failed: {e}")))
    }

    /// Stellar signature hint (last 4 bytes of the public key).
    pub fn signature_hint(&self) -> [u8; 4] {
        self.public.signature_hint()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sha2::{Digest, Sha256};

    #[test]
    fn random_keypair_signs_and_verifies() {
        let kp = Keypair::random().unwrap();
        let msg = b"stellar-rust-sdk";
        let sig = kp.sign(msg);
        kp.verify(msg, &sig).unwrap();
        assert!(kp.public_key().as_str().starts_with('G'));
        assert!(kp.secret_seed().unwrap().as_str().starts_with('S'));
    }

    #[test]
    fn seed_import_is_deterministic() {
        let seed = [42u8; 32];
        let a = Keypair::from_seed_bytes(seed).unwrap();
        let b = Keypair::from_seed_bytes(seed).unwrap();
        assert_eq!(a.public_key().as_bytes(), b.public_key().as_bytes());
        assert_eq!(a.sign(b"hash"), b.sign(b"hash"));
    }

    #[test]
    fn strkey_seed_roundtrip() {
        let kp = Keypair::from_seed_bytes([9u8; 32]).unwrap();
        let s = kp.secret_seed().unwrap();
        let restored = Keypair::from_secret_seed(s.as_str()).unwrap();
        assert_eq!(kp.public_key().as_str(), restored.public_key().as_str());
    }

    #[test]
    fn signs_sha256_transaction_hash_payload() {
        let kp = Keypair::random().unwrap();
        let digest = Sha256::digest(b"network-id-plus-envelope");
        let sig = kp.sign(&digest);
        kp.verify(&digest, &sig).unwrap();
        assert_eq!(sig.len(), 64);
    }

    #[test]
    fn rejects_tampered_signature() {
        let kp = Keypair::random().unwrap();
        let mut sig = kp.sign(b"payload");
        sig[0] ^= 0xff;
        assert!(kp.verify(b"payload", &sig).is_err());
    }
}
