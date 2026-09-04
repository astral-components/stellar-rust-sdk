//! Horizon account model and query-builder unit tests.

use stellar_rust_sdk::horizon::{
    Account, AccountFlags, AccountSigner, Balance, HorizonClient, Order,
};

#[test]
fn account_sequence_and_native_balance() {
    let account = Account {
        id: "GTEST".into(),
        account_id: "GTEST".into(),
        sequence: "999".into(),
        sequence_ledger: Some(1),
        sequence_time: None,
        subentry_count: 1,
        num_sponsoring: Some(0),
        num_sponsored: Some(0),
        home_domain: None,
        last_modified_ledger: Some(10),
        thresholds: None,
        flags: AccountFlags {
            auth_required: true,
            ..AccountFlags::default()
        },
        balances: vec![Balance {
            balance: "42.0000000".into(),
            buying_liabilities: Some("0".into()),
            selling_liabilities: Some("0".into()),
            limit: None,
            asset_type: "native".into(),
            asset_code: None,
            asset_issuer: None,
            liquidity_pool_id: None,
            is_authorized: None,
        }],
        signers: vec![AccountSigner {
            weight: 1,
            key: "GTEST".into(),
            signer_type: "ed25519_public_key".into(),
        }],
    };
    assert_eq!(account.sequence_number().unwrap(), 999);
    assert_eq!(account.native_balance(), Some("42.0000000"));
    assert!(account.flags.auth_required);
    assert_eq!(account.signers[0].weight, 1);
}

#[test]
fn accounts_builder_is_constructed_from_client() {
    let client = HorizonClient::testnet().unwrap();
    let builder = client
        .accounts()
        .limit(10)
        .order(Order::Desc)
        .cursor("now")
        .for_signer("GTEST");
    // Type exists and is chainable; network is not hit here.
    let _ = builder;
}

#[test]
fn json_roundtrip_account_flags() {
    let flags = AccountFlags {
        auth_revocable: true,
        auth_clawback_enabled: true,
        ..AccountFlags::default()
    };
    let v = serde_json::to_value(flags).unwrap();
    let back: AccountFlags = serde_json::from_value(v).unwrap();
    assert!(back.auth_revocable);
    assert!(back.auth_clawback_enabled);
}
