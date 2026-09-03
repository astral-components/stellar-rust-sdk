# `stellar-primitives/soroban-contracts`

> **Production-grade, modular Rust building blocks for Soroban smart contracts on Stellar.**

[![Rust CI](https://github.com/stellar-primitives/soroban-contracts/actions/workflows/ci.yml/badge.svg)](https://github.com/stellar-primitives/soroban-contracts/actions/workflows/ci.yml)
[![Grantfox Bounties](https://img.shields.io/badge/Grantfox-Bounties_Available-5865F2?style=flat-square&logo=github)](https://grantfox.io)
[![Drips Funding](https://img.shields.io/badge/Drips-Funded_Open_Source-00F5A0?style=flat-square)](https://drips.network)
[![License: MIT/Apache-2.0](https://img.shields.io/badge/License-MIT_--_Apache_2.0-blue.svg)](LICENSE)

---

## 🚀 Overview

`soroban-contracts` is an open-source library maintained by `stellar-primitives`. It provides secure, gas-optimized, and reusable Rust smart contracts for the Soroban smart contract platform on Stellar.

Whether you are building DeFi primitives, DAO governance tools, token distribution mechanics, or fine-grained access control, these contracts serve as audited, battle-tested foundations.

---

## 📦 Repository Structure

```
soroban-contracts/
├── contracts/                  # Soroban Rust Contracts (Cargo Workspace)
│   ├── vesting/                # Linear/Cliff Token Vesting & Escrow
│   ├── multisig/               # Multi-Party Threshold Vault
│   ├── splitter/               # Revenue & Fee Distribution (Drips-compatible)
│   └── access-control/         # Granular Role-Based Permissions (Admin, Minter, Operator)
├── packages/                   # Shared Client Packages & SDKs
│   ├── sdk/                    # TypeScript SDK for interacting with contracts (@stellar/stellar-sdk)
│   └── config/                 # Shared TypeScript & Tailwind configurations
├── apps/                       # Web & Documentation Frontends
│   └── docs/                   # Interactive Documentation & Playground (Next.js 14, Tailwind CSS)
├── scripts/                    # Deployment & Developer Automation Tooling
│   ├── deploy.sh               # Shell script for local/testnet deployment via stellar-cli
│   └── setup-identity.sh      # Stellar keys & network setup helpers
└── .github/                    # CI/CD and Open-Source Governance
    ├── workflows/              # GitHub Actions (Rust tests, WASM build checks)
    └── ISSUE_TEMPLATE/         # Grantfox bounty issue templates
```

---

## ⚙️ Quickstart & Local Development

### Prerequisites

- **Rust**: `rustup target add wasm32-unknown-unknown`
- **Stellar CLI**: Install using `cargo install --locked stellar-cli --features opt`
- **Node.js**: `v18+` and `pnpm v8+`

### 1. Compile & Test Soroban Contracts

```bash
# Run unit tests across all smart contracts
cargo test

# Compile all contracts to release WASM binaries
cargo build --target wasm32-unknown-unknown --release
```

### 2. Deploy to Testnet

```bash
# Setup identity & request Friendbot funds
bash ./scripts/setup-identity.sh deployer

# Build and deploy all contracts to Stellar Testnet
bash ./scripts/deploy.sh testnet
```

### 3. Launch Interactive Documentation Site

```bash
pnpm install
pnpm dev:docs
```

Open [http://localhost:3000](http://localhost:3000) to view the API documentation and interactive playground.

---

## 🦊 Grantfox Bounties & Contribution Rules

We actively fund open-source development through **Grantfox** bounties!

1. **Browse Bounties**: Look for issues tagged with `bounty` or `grantfox` in the issue tracker.
2. **Submit a Proposal**: Comment on the issue stating your proposed timeline and approach.
3. **Submit a PR**:
   - Ensure `cargo test` passes cleanly.
   - Include unit tests in `src/test.rs` for any new logic.
   - Follow `#![no_std]` compliance.
4. **Get Paid**: Upon PR review and approval, bounties are released directly to your Stellar account!

See [.github/ISSUE_TEMPLATE/bounty-issue.md](.github/ISSUE_TEMPLATE/bounty-issue.md) for bounty issue definitions.

---

## 💧 Drips Streaming & Sustained Funding

`stellar-primitives` leverages **Drips** to split continuous revenue streams among contributors and open-source dependencies.

- Learn more about continuous funding on [Drips Network](https://drips.network).
- View the splitter contract implementation at [`contracts/splitter`](contracts/splitter).

---

## 🛡️ License

Licensed under either of [Apache License, Version 2.0](LICENSE-APACHE) or [MIT License](LICENSE-MIT) at your option.
