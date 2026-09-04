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
```

## Architecture

The crate is a Cargo workspace member at the repository root. Shared constants
include `VERSION`, `USER_AGENT` (`Astral-Stellar-Rust-SDK/{version}`), and the
official Horizon / Soroban testnet URLs.

Planned modules:

- `errors` — unified `SdkError` taxonomy
- `address` — StrKey `G…` / `S…` / `C…` / `M…` wrappers
- `keypair` — Ed25519 generate / seed import / sign
- `network` — passphrases and SHA-256 network IDs
- `horizon` — async REST client
- `rpc` — Soroban JSON-RPC
- `tx` — transaction builder, operations, envelopes
- `soroban` — high-level contract invoker

## Workspace members

The repository also contains reusable Soroban contracts under `contracts/`.

```bash
cargo test -p stellar-rust-sdk
cargo build --workspace --exclude stellar-rust-sdk --target wasm32-unknown-unknown --release
```

## Grantfox & Drips

- **Grantfox** bounties: issues tagged `bounty` / `grantfox`.
- **Drips** streaming: [drips.network](https://drips.network).

## License

Licensed under either of Apache License 2.0 or MIT at your option.
