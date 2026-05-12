# Contributing to Humonics Contracts

## Prerequisites

- Rust (stable) with `wasm32-unknown-unknown` target: `rustup target add wasm32-unknown-unknown`
- Stellar CLI: `cargo install stellar-cli`

## Local setup

```bash
git clone git@github.com:humonicsprotocol/Humonics-contracts.git
cd Humonics-contracts
cargo test
```

No external services needed — all tests use `soroban_sdk::testutils`.

## Rules

- **No `unwrap()` in contract code** — handle all Results explicitly
- **Always call `env.require_auth()`** for any state-changing function
- **Never store PII on-chain** — only cryptographic commitments and hashes
- **Never change a function signature** without bumping the contract version
- **Never deploy to mainnet** without a completed security audit

## After changing a circuit

Any change to contract logic that affects the on-chain interface requires:
1. A version bump in the contract's `Cargo.toml`
2. Updated tests covering the new behaviour
3. Re-deployment via `./scripts/deploy.sh testnet <contract>`

## Branch naming

`feat/`, `fix/`, `chore/`, `docs/` — PRs target `main`.

## PR checklist

- [ ] `cargo test` passes
- [ ] `cargo clippy -- -D warnings` clean
- [ ] No `unwrap()` in contract code
- [ ] `env.require_auth()` present on all state-changing functions
- [ ] New functions have tests for happy path + unauthorized caller
