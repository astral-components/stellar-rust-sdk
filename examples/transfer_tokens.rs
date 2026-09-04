//! Transfer native XLM on SDF Testnet.
//!
//! ```text
//! cargo run --example transfer_tokens
//! ```
//!
//! Set `STELLAR_SECRET` to an `S…` seed that already has testnet funds, or the
//! example will generate a key and print a Friendbot URL.

use stellar_rust_sdk::horizon::HorizonClient;
use stellar_rust_sdk::keypair::Keypair;
use stellar_rust_sdk::network::Network;
use stellar_rust_sdk::tx::builder::TransactionBuilder;
use stellar_rust_sdk::tx::envelope::TransactionEnvelope;
use stellar_rust_sdk::tx::operations::Payment;
use stellar_rust_sdk::tx::types::Memo;
use stellar_rust_sdk::tx::xlm_to_stroops;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let source = match std::env::var("STELLAR_SECRET") {
        Ok(seed) => Keypair::from_secret_seed(&seed)?,
        Err(_) => {
            let kp = Keypair::random()?;
            println!(
                "Generated source account {}\nFund it via https://friendbot.stellar.org/?addr={}\nSet STELLAR_SECRET={} to reuse it.",
                kp.public_key(),
                kp.public_key(),
                kp.secret_seed()?.as_str()
            );
            kp
        }
    };

    let destination = match std::env::var("STELLAR_DEST") {
        Ok(g) => stellar_rust_sdk::address::PublicKey::from_strkey(&g)?,
        Err(_) => {
            let kp = Keypair::random()?;
            println!("Generated destination {}", kp.public_key());
            kp.public_key().clone()
        }
    };

    let client = HorizonClient::testnet()?;
    let account = match client.accounts().account(source.public_key().as_str()).await {
        Ok(acc) => acc,
        Err(err) => {
            eprintln!("Source account not found on Testnet ({err}). Fund it with Friendbot first.");
            return Ok(());
        }
    };

    let seq = account.sequence_number()?;
    let tx = TransactionBuilder::new(source.public_key().clone(), seq)
        .memo(Memo::text("astral-sdk")?)
        .add_operation(Payment::native(&destination, xlm_to_stroops(1)))
        .build()?;
    let envelope = TransactionEnvelope::new(tx).signed(&source, &Network::Testnet)?;
    let xdr = envelope.to_base64_xdr();
    println!("Submitting envelope ({} bytes base64)…", xdr.len());
    match client.submit_transaction(&xdr).await {
        Ok(resp) => println!("Submitted hash={:?} ledger={:?}", resp.hash, resp.ledger),
        Err(err) => eprintln!("submit failed: {err}"),
    }
    Ok(())
}
