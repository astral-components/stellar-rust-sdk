//! Simulate a Soroban contract invocation on Testnet RPC.
//!
//! ```text
//! cargo run --example invoke_soroban
//! ```
//!
//! Requires `STELLAR_CONTRACT` (a `C…` id). Optionally set `STELLAR_SECRET`.

use stellar_rust_sdk::address::ContractId;
use stellar_rust_sdk::horizon::HorizonClient;
use stellar_rust_sdk::keypair::Keypair;
use stellar_rust_sdk::rpc::RpcClient;
use stellar_rust_sdk::soroban::{ContractInvokeRequest, ContractInvoker};
use stellar_rust_sdk::tx::builder::TransactionBuilder;
use stellar_rust_sdk::tx::envelope::TransactionEnvelope;
use stellar_rust_sdk::tx::operations::Payment;
use stellar_rust_sdk::network::Network;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let contract = match std::env::var("STELLAR_CONTRACT") {
        Ok(id) => ContractId::from_strkey(&id)?,
        Err(_) => {
            eprintln!("Set STELLAR_CONTRACT to a Testnet C-address to simulate a real invoke.");
            let placeholder = ContractId::from_payload([0u8; 32])?;
            let req = ContractInvokeRequest::new(placeholder, "increment");
            println!(
                "Example request for function `{}` with {} args",
                req.function,
                req.args_xdr.len()
            );
            return Ok(());
        }
    };

    let rpc = RpcClient::testnet()?;
    let health = rpc.get_health().await?;
    println!("RPC health: {} (healthy={})", health.status, health.is_healthy());
    let network = rpc.get_network().await?;
    println!("RPC passphrase: {}", network.passphrase);
    let ledger = rpc.get_latest_ledger().await?;
    println!("Latest ledger: {}", ledger.sequence);

    let invoker = ContractInvoker::testnet()?;
    println!("Invoker network: {:?}", invoker.network().kind());

    let req = ContractInvokeRequest::new(contract, "increment");
    println!("Prepared invoke: {}::{:?}", req.contract_id, req.function);

    // Build a dummy signed envelope so simulateTransaction has bytes to inspect.
    // Real host-function envelopes should be assembled with a Soroban tx builder.
    if let Ok(seed) = std::env::var("STELLAR_SECRET") {
        let kp = Keypair::from_secret_seed(&seed)?;
        let horizon = HorizonClient::testnet()?;
        if let Ok(acc) = horizon.accounts().account(kp.public_key().as_str()).await {
            let seq = acc.sequence_number()?;
            let tx = TransactionBuilder::new(kp.public_key().clone(), seq)
                .add_operation(Payment::native(kp.public_key(), 1))
                .build()?;
            let env = TransactionEnvelope::new(tx).signed(&kp, &Network::Testnet)?;
            match rpc.simulate_transaction(&env.to_base64_xdr()).await {
                Ok(sim) => println!(
                    "simulation ok fee={} cpu={}",
                    sim.min_resource_fee_stroops(),
                    sim.cpu_instructions()
                ),
                Err(err) => eprintln!("simulateTransaction: {err}"),
            }
        }
    }

    Ok(())
}
