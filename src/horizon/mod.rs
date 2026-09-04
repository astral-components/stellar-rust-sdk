//! Asynchronous Horizon REST client and endpoint builders.

mod client;

pub use client::{HorizonClient, HorizonClientBuilder};

/// Official Horizon base URL for the Stellar Public Network.
pub const PUBLIC_URL: &str = "https://horizon.stellar.org";

/// Official Horizon base URL for SDF Testnet.
pub const TESTNET_URL: &str = "https://horizon-testnet.stellar.org";

/// Official Horizon base URL for SDF Futurenet.
pub const FUTURENET_URL: &str = "https://horizon-futurenet.stellar.org";

/// Returns the Horizon base URL for a well-known network label.
pub fn url_for(network: &str) -> Option<&'static str> {
    match network {
        "public" | "mainnet" => Some(PUBLIC_URL),
        "testnet" => Some(TESTNET_URL),
        "futurenet" => Some(FUTURENET_URL),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn well_known_horizon_urls() {
        assert_eq!(url_for("testnet"), Some(TESTNET_URL));
        assert_eq!(url_for("public"), Some(PUBLIC_URL));
        assert_eq!(url_for("futurenet"), Some(FUTURENET_URL));
        assert_eq!(url_for("local"), None);
    }
}
