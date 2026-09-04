# stellar-rust-sdk

> Production-grade Rust SDK for Stellar Horizon, Soroban RPC, and typed transaction construction.

[![License: MIT/Apache-2.0](https://img.shields.io/badge/License-MIT_--_Apache_2.0-blue.svg)](LICENSE)
[![Grantfox Bounties](https://img.shields.io/badge/Grantfox-Bounties_Available-5865F2?style=flat-square)](https://grantfox.io)
[![Drips Funding](https://img.shields.io/badge/Drips-Funded_Open_Source-00F5A0?style=flat-square)](https://drips.network)

Maintained by [Astral Components](https://github.com/astral-components) (`Astral-components/stellar-rust-sdk`).

## Installation

```toml
[dependencies]
stellar-rust-sdk = { git = "https://github.com/astral-components/stellar-rust-sdk" }
```

Requires **Rust 1.75+**. HTTP clients use `reqwest` with `rustls` (no system OpenSSL).

```bash
cargo test -p stellar-rust-sdk
cargo run --example transfer_tokens
cargo run --example invoke_soroban
```

Live Testnet / Friendbot checks:

```bash
STELLAR_LIVE_TESTS=1 cargo test -p stellar-rust-sdk --test integration_test
```

## Architecture

```
stellar_rust_sdk
├── errors     SdkError taxonomy + Horizon result_codes parser
├── address    StrKey G/S/C/M wrappers
├── keypair    Ed25519 generate / seed import / sign
├── network    Passphrases + SHA-256 network IDs
├── horizon    Async REST client (accounts, assets, pools, submit)
├── rpc        Soroban JSON-RPC (simulate, entries, events)
├── tx         TransactionBuilder, operations, multi-sig XDR envelopes
└── soroban    ContractInvoker pipeline (simulate → fee → sign → submit)
```

User-Agent on all HTTP/RPC traffic: `Astral-Stellar-Rust-SDK/{version}`.

## Quick example

```rust
use stellar_rust_sdk::horizon::HorizonClient;
use stellar_rust_sdk::keypair::Keypair;
use stellar_rust_sdk::network::Network;
use stellar_rust_sdk::tx::builder::TransactionBuilder;
use stellar_rust_sdk::tx::envelope::TransactionEnvelope;
use stellar_rust_sdk::tx::operations::Payment;
use stellar_rust_sdk::tx::xlm_to_stroops;

# async fn demo() -> Result<(), stellar_rust_sdk::SdkError> {
let source = Keypair::random()?;
let dest = Keypair::random()?;
let client = HorizonClient::testnet()?;
let account = client.accounts().account(source.public_key().as_str()).await?;
let tx = TransactionBuilder::new(source.public_key().clone(), account.sequence_number()?)
    .add_operation(Payment::native(dest.public_key(), xlm_to_stroops(1)))
    .build()?;
let envelope = TransactionEnvelope::new(tx).signed(&source, &Network::Testnet)?;
client.submit_transaction(&envelope.to_base64_xdr()).await?;
# Ok(())
# }
```

Runnable copies live in [`examples/transfer_tokens.rs`](examples/transfer_tokens.rs) and [`examples/invoke_soroban.rs`](examples/invoke_soroban.rs).

## Workspace members

The repository also contains reusable Soroban contracts:

| Path | Role |
| --- | --- |
| `contracts/access-control` | Role-based permissions |
| `contracts/vesting` | Linear / cliff vesting |
| `contracts/multisig` | Threshold vault |
| `contracts/splitter` | Revenue split (Drips-compatible) |

```bash
cargo test --workspace
cargo build --workspace --exclude stellar-rust-sdk --target wasm32-unknown-unknown --release
```

## Grantfox & Drips

- **Grantfox** bounties: issues tagged `bounty` / `grantfox`. See [`.github/ISSUE_TEMPLATE/bounty-issue.md`](.github/ISSUE_TEMPLATE/bounty-issue.md).
- **Drips** streaming: [drips.network](https://drips.network). The on-chain splitter lives in `contracts/splitter`.

## License

Licensed under either of Apache License 2.0 or MIT at your option.
