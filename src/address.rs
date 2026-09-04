//! Stellar StrKey address wrappers (`G…`, `S…`, `C…`, `M…`).
//!
//! All constructors validate checksums through `stellar-strkey` and reject
//! payloads whose version byte does not match the expected kind.

use crate::errors::SdkError;
use stellar_strkey::{ed25519, DecodeError, Strkey};

/// Discriminant for a Stellar StrKey payload.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AddressKind {
    /// Account public key (`G…`).
    PublicKey,
    /// Secret seed (`S…`).
    SecretSeed,
    /// Contract identifier (`C…`).
    Contract,
    /// Muxed account (`M…`, ed25519 key plus 64-bit id).
    MuxedAccount,
}

impl AddressKind {
    /// Human-readable prefix character for this kind.
    pub fn prefix(self) -> char {
        match self {
            Self::PublicKey => 'G',
            Self::SecretSeed => 'S',
            Self::Contract => 'C',
            Self::MuxedAccount => 'M',
        }
    }
}

/// Ed25519 account public key (`G…`).
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct PublicKey {
    bytes: [u8; 32],
    encoded: String,
}

impl std::fmt::Debug for PublicKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("PublicKey").field(&self.encoded).finish()
    }
}

impl PublicKey {
    /// Parse a `G…` StrKey.
    pub fn from_strkey(s: &str) -> Result<Self, SdkError> {
        let key = ed25519::PublicKey::from_string(s).map_err(|e| strkey_err(s, e))?;
        Ok(Self {
            bytes: key.0,
            encoded: s.to_string(),
        })
    }

    /// Encode raw 32-byte ed25519 public key bytes as a `G…` StrKey.
    pub fn from_payload(bytes: [u8; 32]) -> Result<Self, SdkError> {
        let key = ed25519::PublicKey(bytes);
        Ok(Self {
            bytes,
            encoded: key.to_string(),
        })
    }

    /// 32-byte ed25519 public key.
    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.bytes
    }

    /// Canonical `G…` encoding.
    pub fn as_str(&self) -> &str {
        &self.encoded
    }

    /// Last 4 bytes of the public key, used as a Stellar signature hint.
    pub fn signature_hint(&self) -> [u8; 4] {
        let mut hint = [0u8; 4];
        hint.copy_from_slice(&self.bytes[28..32]);
        hint
    }
}

impl std::fmt::Display for PublicKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.encoded)
    }
}

impl std::str::FromStr for PublicKey {
    type Err = SdkError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::from_strkey(s)
    }
}

/// Secret seed (`S…`). Zeroized on drop as best-effort (the backing `String`
/// is cleared). Prefer keeping seeds in [`crate::keypair::Keypair`].
#[derive(Clone)]
pub struct SecretSeed {
    bytes: [u8; 32],
    encoded: String,
}

impl std::fmt::Debug for SecretSeed {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("SecretSeed(REDACTED)")
    }
}

impl SecretSeed {
    /// Parse an `S…` StrKey.
    pub fn from_strkey(s: &str) -> Result<Self, SdkError> {
        let key = ed25519::PrivateKey::from_string(s).map_err(|e| strkey_err(s, e))?;
        Ok(Self {
            bytes: key.0,
            encoded: s.to_string(),
        })
    }

    /// Encode a raw 32-byte seed as an `S…` StrKey.
    pub fn from_payload(bytes: [u8; 32]) -> Result<Self, SdkError> {
        let key = ed25519::PrivateKey(bytes);
        Ok(Self {
            bytes,
            encoded: key.to_string(),
        })
    }

    /// 32-byte ed25519 seed.
    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.bytes
    }

    /// Canonical `S…` encoding.
    pub fn as_str(&self) -> &str {
        &self.encoded
    }
}

impl std::fmt::Display for SecretSeed {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.encoded)
    }
}

impl Drop for SecretSeed {
    fn drop(&mut self) {
        self.bytes.fill(0);
        // Best-effort: `String::clear` drops the UTF-8 view; the heap buffer
        // is not guaranteed to be overwritten without `unsafe`.
        self.encoded.clear();
        self.encoded.shrink_to_fit();
    }
}

/// Contract identifier (`C…`).
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct ContractId {
    bytes: [u8; 32],
    encoded: String,
}

impl std::fmt::Debug for ContractId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("ContractId").field(&self.encoded).finish()
    }
}

impl ContractId {
    /// Parse a `C…` StrKey.
    pub fn from_strkey(s: &str) -> Result<Self, SdkError> {
        match Strkey::from_string(s).map_err(|e| strkey_err(s, e))? {
            Strkey::Contract(c) => Ok(Self {
                bytes: c.0,
                encoded: s.to_string(),
            }),
            other => Err(SdkError::InvalidAddress(format!(
                "expected contract C-address, got {}",
                kind_name(&other)
            ))),
        }
    }

    /// Encode raw 32-byte contract hash as a `C…` StrKey.
    pub fn from_payload(bytes: [u8; 32]) -> Result<Self, SdkError> {
        let encoded = Strkey::Contract(stellar_strkey::Contract(bytes)).to_string();
        Ok(Self { bytes, encoded })
    }

    /// 32-byte contract identifier.
    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.bytes
    }

    /// Canonical `C…` encoding.
    pub fn as_str(&self) -> &str {
        &self.encoded
    }
}

impl std::fmt::Display for ContractId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.encoded)
    }
}

/// Muxed account (`M…`): ed25519 public key plus a 64-bit mux id.
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct MuxedAccount {
    ed25519: [u8; 32],
    id: u64,
    encoded: String,
}

impl std::fmt::Debug for MuxedAccount {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MuxedAccount")
            .field("encoded", &self.encoded)
            .field("id", &self.id)
            .finish()
    }
}

impl MuxedAccount {
    /// Parse an `M…` StrKey.
    pub fn from_strkey(s: &str) -> Result<Self, SdkError> {
        let muxed = ed25519::MuxedAccount::from_string(s).map_err(|e| strkey_err(s, e))?;
        Ok(Self {
            ed25519: muxed.ed25519,
            id: muxed.id,
            encoded: s.to_string(),
        })
    }

    /// Build a muxed account from a public key and mux id.
    pub fn from_account(pk: &PublicKey, id: u64) -> Result<Self, SdkError> {
        let muxed = ed25519::MuxedAccount {
            ed25519: *pk.as_bytes(),
            id,
        };
        Ok(Self {
            ed25519: muxed.ed25519,
            id,
            encoded: muxed.to_string(),
        })
    }

    /// Underlying ed25519 public key bytes.
    pub fn ed25519_bytes(&self) -> &[u8; 32] {
        &self.ed25519
    }

    /// 64-bit mux identifier.
    pub fn id(&self) -> u64 {
        self.id
    }

    /// Canonical `M…` encoding.
    pub fn as_str(&self) -> &str {
        &self.encoded
    }
}

impl std::fmt::Display for MuxedAccount {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.encoded)
    }
}

/// Any publicly shareable Stellar address kind.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StellarAddress {
    /// Classic account (`G…`).
    Account(PublicKey),
    /// Soroban contract (`C…`).
    Contract(ContractId),
    /// Muxed account (`M…`).
    Muxed(MuxedAccount),
}

impl StellarAddress {
    /// Parse `G…`, `C…`, or `M…` (secret seeds are rejected).
    pub fn parse(s: &str) -> Result<Self, SdkError> {
        match Strkey::from_string(s).map_err(|e| strkey_err(s, e))? {
            Strkey::PublicKeyEd25519(pk) => Ok(Self::Account(PublicKey {
                bytes: pk.0,
                encoded: s.to_string(),
            })),
            Strkey::Contract(c) => Ok(Self::Contract(ContractId {
                bytes: c.0,
                encoded: s.to_string(),
            })),
            Strkey::MuxedAccountEd25519(m) => Ok(Self::Muxed(MuxedAccount {
                ed25519: m.ed25519,
                id: m.id,
                encoded: s.to_string(),
            })),
            Strkey::PrivateKeyEd25519(_) => Err(SdkError::InvalidAddress(
                "secret seeds are not valid public addresses".into(),
            )),
            other => Err(SdkError::InvalidAddress(format!(
                "unsupported StrKey kind {}",
                kind_name(&other)
            ))),
        }
    }

    /// Address kind discriminant.
    pub fn kind(&self) -> AddressKind {
        match self {
            Self::Account(_) => AddressKind::PublicKey,
            Self::Contract(_) => AddressKind::Contract,
            Self::Muxed(_) => AddressKind::MuxedAccount,
        }
    }

    /// Canonical StrKey encoding.
    pub fn as_str(&self) -> &str {
        match self {
            Self::Account(a) => a.as_str(),
            Self::Contract(c) => c.as_str(),
            Self::Muxed(m) => m.as_str(),
        }
    }
}

impl std::fmt::Display for StellarAddress {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl std::str::FromStr for StellarAddress {
    type Err = SdkError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::parse(s)
    }
}

fn strkey_err(input: &str, err: DecodeError) -> SdkError {
    SdkError::StrKey(format!("failed to decode `{input}`: {err}"))
}

fn kind_name(key: &Strkey) -> &'static str {
    match key {
        Strkey::PublicKeyEd25519(_) => "public_key",
        Strkey::PrivateKeyEd25519(_) => "secret_seed",
        Strkey::MuxedAccountEd25519(_) => "muxed_account",
        Strkey::PreAuthTx(_) => "pre_auth_tx",
        Strkey::HashX(_) => "hash_x",
        Strkey::SignedPayloadEd25519(_) => "signed_payload",
        Strkey::Contract(_) => "contract",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn prefixes_match_stellar_strkey_spec() {
        assert_eq!(AddressKind::PublicKey.prefix(), 'G');
        assert_eq!(AddressKind::SecretSeed.prefix(), 'S');
        assert_eq!(AddressKind::Contract.prefix(), 'C');
        assert_eq!(AddressKind::MuxedAccount.prefix(), 'M');
    }

    #[test]
    fn public_key_roundtrip_from_payload() {
        let pk = PublicKey::from_payload([7u8; 32]).unwrap();
        assert!(pk.as_str().starts_with('G'));
        let parsed = PublicKey::from_strkey(pk.as_str()).unwrap();
        assert_eq!(parsed.as_bytes(), pk.as_bytes());
        assert_eq!(parsed.signature_hint(), [7, 7, 7, 7]);
    }

    #[test]
    fn rejects_account_string_as_contract() {
        let pk = PublicKey::from_payload([1u8; 32]).unwrap();
        let err = ContractId::from_strkey(pk.as_str()).unwrap_err();
        assert!(err.is_user_error());
    }
}
