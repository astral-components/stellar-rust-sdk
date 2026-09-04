//! Integration tests for StrKey address wrappers.

use stellar_rust_sdk::address::{ContractId, MuxedAccount, PublicKey, SecretSeed, StellarAddress};

#[test]
fn public_key_rejects_wrong_prefix() {
    let seed = SecretSeed::from_payload([5u8; 32]).unwrap();
    assert!(PublicKey::from_strkey(seed.as_str()).is_err());
}

#[test]
fn stellar_address_parses_account_and_contract() {
    let pk = PublicKey::from_payload([11u8; 32]).unwrap();
    let addr = StellarAddress::parse(pk.as_str()).unwrap();
    assert!(matches!(addr, StellarAddress::Account(_)));

    let cid = ContractId::from_payload([12u8; 32]).unwrap();
    let addr = StellarAddress::parse(cid.as_str()).unwrap();
    assert!(matches!(addr, StellarAddress::Contract(_)));
}

#[test]
fn muxed_account_roundtrip() {
    let pk = PublicKey::from_payload([13u8; 32]).unwrap();
    let muxed = MuxedAccount::from_account(&pk, 99).unwrap();
    assert!(muxed.as_str().starts_with('M'));
    assert_eq!(muxed.id(), 99);
    let parsed = MuxedAccount::from_strkey(muxed.as_str()).unwrap();
    assert_eq!(parsed.ed25519_bytes(), pk.as_bytes());
}

#[test]
fn garbage_string_is_strkey_error() {
    let err = StellarAddress::parse("not-a-key").unwrap_err();
    assert!(err.to_string().contains("strkey") || err.to_string().contains("failed"));
}
