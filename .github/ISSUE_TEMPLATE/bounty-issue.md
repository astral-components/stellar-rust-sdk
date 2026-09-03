---
name: Grantfox Bounty Task
about: Template for Grantfox open-source bounty issues and community tasks
title: '[BOUNTY]: '
labels: 'bounty, grantfox'
assignees: ''
---

## 🦊 Task Description

Provide a clear and concise description of the feature, optimization, or bugfix required for `stellar-primitives`.

### Target Module
- [ ] `contracts/vesting`
- [ ] `contracts/multisig`
- [ ] `contracts/splitter`
- [ ] `contracts/access-control`
- [ ] `packages/sdk`
- [ ] `apps/docs`

---

## 🎯 Acceptance Criteria

- [ ] All Rust smart contract code strictly enforces `#![no_std]` compliance.
- [ ] Unit tests added in `src/test.rs` covering happy and edge path scenarios.
- [ ] `cargo test` and `cargo build --target wasm32-unknown-unknown --release` pass cleanly with zero warnings.
- [ ] Documentation updated in `apps/docs`.

---

## 💰 Estimated Bounty & Funding

- **Estimated Bounty Amount**: `[e.g. 500 XLM / 100 USDC]`
- **Funding Platform**: Grantfox / Drips Network
- **Payout Account**: `[Contributor Stellar Address]`

---

## 🚀 How to Claim & Submit

1. Comment on this issue expressing intent to work on the task with your estimated delivery date.
2. Submit a Pull Request referencing this issue number.
3. Upon PR approval and merge, bounty will be transferred directly on Stellar Testnet/Mainnet!
