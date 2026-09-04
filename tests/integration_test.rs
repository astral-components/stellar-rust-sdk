//! End-to-end workflow tests against Stellar Testnet and Friendbot.
//!
//! Live network tests run when `STELLAR_LIVE_TESTS=1`. Default `cargo test`
//! executes construction / signing checks that do not require the network.

use stellar_rust_sdk::address::PublicKey;
use stellar_rust_sdk::horizon::HorizonClient;
use stellar_rust_sdk::keypair::Keypair;
use stellar_rust_sdk::network::Network;
use stellar_rust_sdk::tx::builder::TransactionBuilder;
use stellar_rust_sdk::tx::envelope::TransactionEnvelope;
use stellar_rust_sdk::tx::operations::{CreateAccount, Payment};
use stellar_rust_sdk::tx::xlm_to_stroops;
use stellar_rust_sdk::TESTNET_HORIZON_URL;

fn live_enabled() -> bool {
    std::env::var("STELLAR_LIVE_TESTS").ok().as_deref() == Some("1")
}

#[test]
fn builds_and_signs_create_account_plus_payment() {
    let source = Keypair::random().unwrap();
    let dest = Keypair::random().unwrap();
    let tx = TransactionBuilder::new(source.public_key().clone(), 0)
        .base_fee(100)
        .add_operation(CreateAccount::new(dest.public_key(), 10))
        .add_operation(Payment::native(dest.public_key(), xlm_to_stroops(1)))
        .build()
        .unwrap();
    assert_eq!(tx.sequence, 1);
    assert_eq!(tx.fee, 200);
    let env = TransactionEnvelope::new(tx)
        .signed(&source, &Network::Testnet)
        .unwrap();
    assert_eq!(env.signatures().len(), 1);
    let xdr = env.to_base64_xdr();
    assert!(xdr.len() > 32);
}

#[tokio::test]
async fn friendbot_horizon_payment_flow() {
    if !live_enabled() {
        eprintln!("skipping live test; set STELLAR_LIVE_TESTS=1 to hit Testnet");
        return;
    }

    let source = Keypair::random().unwrap();
    let dest = Keypair::random().unwrap();
    let client = HorizonClient::testnet().unwrap();
    assert_eq!(client.base_url(), TESTNET_HORIZON_URL);

    fund_via_friendbot(source.public_key()).await.unwrap();
    fund_via_friendbot(dest.public_key()).await.unwrap();

    let account = client
        .accounts()
        .account(source.public_key().as_str())
        .await
        .unwrap();
    let seq = account.sequence_number().unwrap();
    let before = account.native_balance().map(str::to_string);

    let tx = TransactionBuilder::new(source.public_key().clone(), seq)
        .add_operation(Payment::native(dest.public_key(), xlm_to_stroops(1)))
        .build()
        .unwrap();
    let env = TransactionEnvelope::new(tx)
        .signed(&source, &Network::Testnet)
        .unwrap();
    let submit = client
        .submit_transaction(&env.to_base64_xdr())
        .await
        .expect("submit payment");
    assert!(submit.hash.is_some() || submit.successful != Some(false));

    let updated = client
        .accounts()
        .account(source.public_key().as_str())
        .await
        .unwrap();
    let new_seq = updated.sequence_number().unwrap();
    assert!(new_seq > seq, "sequence should bump after a successful tx");
    let _ = before;
}

async fn fund_via_friendbot(pk: &PublicKey) -> Result<(), Box<dyn std::error::Error>> {
    let url = format!("https://friendbot.stellar.org/?addr={}", pk.as_str());
    let resp = reqwest::Client::new().get(url).send().await?;
    if !resp.status().is_success() {
        let body = resp.text().await.unwrap_or_default();
        return Err(format!("friendbot failed: {body}").into());
    }
    Ok(())
}
