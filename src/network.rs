//! Stellar network environments, passphrases, and network IDs.
//!
//! The network ID is `SHA-256(passphrase)` and is prepended to the transaction
//! envelope hash before Ed25519 signing (see CAP-0015 / Stellar protocol).

use sha2::{Digest, Sha256};

/// Official passphrase for the Stellar Public Network.
pub const PUBLIC_PASSPHRASE: &str = "Public Global Stellar Network ; September 2015";

/// Official passphrase for SDF Testnet.
pub const TESTNET_PASSPHRASE: &str = "Test SDF Network ; September 2015";

/// Official passphrase for SDF Futurenet.
pub const FUTURENET_PASSPHRASE: &str = "Test SDF Future Network ; October 2022";

/// Well-known Stellar network environments.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NetworkKind {
    /// Stellar Public Network.
    Public,
    /// SDF Testnet.
    Testnet,
    /// SDF Futurenet (protocol preview).
    Futurenet,
    /// Operator-defined network with a custom passphrase.
    Custom,
}

impl NetworkKind {
    /// Returns a stable lowercase identifier for logs and User-Agent tags.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Public => "public",
            Self::Testnet => "testnet",
            Self::Futurenet => "futurenet",
            Self::Custom => "custom",
        }
    }
}

/// Target Stellar network used for URLs, passphrases, and signature hashes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Network {
    /// Stellar Public Network.
    Public,
    /// SDF Testnet.
    Testnet,
    /// SDF Futurenet.
    Futurenet,
    /// Custom network identified by its passphrase.
    Custom(String),
}

impl Network {
    /// Kind discriminant.
    pub fn kind(&self) -> NetworkKind {
        match self {
            Self::Public => NetworkKind::Public,
            Self::Testnet => NetworkKind::Testnet,
            Self::Futurenet => NetworkKind::Futurenet,
            Self::Custom(_) => NetworkKind::Custom,
        }
    }

    /// Network passphrase used in transaction hashes.
    pub fn passphrase(&self) -> &str {
        match self {
            Self::Public => PUBLIC_PASSPHRASE,
            Self::Testnet => TESTNET_PASSPHRASE,
            Self::Futurenet => FUTURENET_PASSPHRASE,
            Self::Custom(p) => p.as_str(),
        }
    }

    /// `SHA-256(passphrase)` — the 32-byte network ID.
    pub fn network_id(&self) -> [u8; 32] {
        sha256(self.passphrase().as_bytes())
    }

    /// Hex-encoded network ID (lowercase).
    pub fn network_id_hex(&self) -> String {
        hex::encode(self.network_id())
    }

    /// Official Horizon URL when this is a well-known SDF network.
    pub fn horizon_url(&self) -> Option<&'static str> {
        match self {
            Self::Public => Some(crate::horizon::PUBLIC_URL),
            Self::Testnet => Some(crate::horizon::TESTNET_URL),
            Self::Futurenet => Some(crate::horizon::FUTURENET_URL),
            Self::Custom(_) => None,
        }
    }

    /// Official Soroban RPC URL when this is a well-known SDF network.
    pub fn rpc_url(&self) -> Option<&'static str> {
        match self {
            Self::Public => Some(crate::rpc::PUBLIC_URL),
            Self::Testnet => Some(crate::rpc::TESTNET_URL),
            Self::Futurenet => Some(crate::rpc::FUTURENET_URL),
            Self::Custom(_) => None,
        }
    }

    /// Hash a transaction envelope for signing:
    /// `sha256(network_id || sha256(tagged_transaction_xdr))`.
    pub fn hash_transaction(&self, tagged_tx_xdr: &[u8]) -> [u8; 32] {
        let mut tagged = Vec::with_capacity(32 + tagged_tx_xdr.len());
        tagged.extend_from_slice(&self.network_id());
        tagged.extend_from_slice(&sha256(tagged_tx_xdr));
        sha256(&tagged)
    }
}

impl Default for Network {
    fn default() -> Self {
        Self::Testnet
    }
}

/// SHA-256 helper used by network ID and transaction hashing.
pub fn sha256(data: &[u8]) -> [u8; 32] {
    let digest = Sha256::digest(data);
    let mut out = [0u8; 32];
    out.copy_from_slice(&digest);
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn kind_labels_are_stable() {
        assert_eq!(NetworkKind::Public.as_str(), "public");
        assert_eq!(NetworkKind::Testnet.as_str(), "testnet");
        assert_eq!(NetworkKind::Futurenet.as_str(), "futurenet");
        assert_eq!(NetworkKind::Custom.as_str(), "custom");
    }

    #[test]
    fn official_passphrases() {
        assert!(Network::Public.passphrase().contains("September 2015"));
        assert!(Network::Testnet.passphrase().contains("Test SDF Network"));
        assert!(Network::Futurenet.passphrase().contains("Future Network"));
        assert_eq!(Network::Custom("n".into()).passphrase(), "n");
    }

    #[test]
    fn network_id_is_sha256_of_passphrase() {
        let id = Network::Testnet.network_id();
        assert_eq!(id, sha256(TESTNET_PASSPHRASE.as_bytes()));
        assert_eq!(id.len(), 32);
        assert_ne!(Network::Public.network_id(), Network::Testnet.network_id());
    }

    #[test]
    fn hash_transaction_is_stable() {
        let a = Network::Testnet.hash_transaction(b"xdr");
        let b = Network::Testnet.hash_transaction(b"xdr");
        let c = Network::Public.hash_transaction(b"xdr");
        assert_eq!(a, b);
        assert_ne!(a, c);
    }
}
