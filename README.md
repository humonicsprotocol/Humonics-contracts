# Humonics Soroban Smart Contracts

A comprehensive suite of Soroban smart contracts for the Humonics protocol, providing on-chain trust layer with certificate management, verification gateway, and staking functionality.

## Overview

This repository contains three core contracts that work together to provide a secure and decentralized verification system:

1. **CertificateRegistry** - Manages issuance and revocation of cryptographic certificates
2. **VerificationGateway** - Provides verification services for content certificates
3. **HUMToken** - Governance and staking token with slashing mechanisms

## Architecture

```
┌─────────────────────┐    ┌─────────────────────┐    ┌─────────────────────┐
│  CertificateRegistry│    │ VerificationGateway │    │     HUMToken        │
│                     │    │                     │    │                     │
│ • Issue Certificates│◄──►│ • Verify Content    │    │ • Staking           │
│ • Revoke Certificates│    │ • Batch Verification│    │ • Slashing          │
│ • Store Metadata    │    │ • Error Handling    │    │ • Transfers         │
└─────────────────────┘    └─────────────────────┘    └─────────────────────┘
```

## Features

### CertificateRegistry
- Zero-knowledge proof verification
- Content hash indexing
- DID-based authorization
- Revocation support
- Configurable governance

### VerificationGateway
- Single and batch verification
- Efficient certificate lookup
- Comprehensive error reporting
- Registry update capability

### HUMToken
- ERC20-like token functionality
- Staking with minimum/maximum limits
- Admin-controlled slashing
- Total supply tracking
- Configurable parameters

## Security Features

- **Authentication**: All state-changing functions require `env.require_auth()`
- **Authorization**: Role-based access control for admin functions
- **Input Validation**: Comprehensive validation of all inputs
- **Error Handling**: Proper error codes without panics
- **Storage Security**: Appropriate storage patterns for different data types

## Quick Start

### Prerequisites

- Rust 1.70+ with `wasm32-unknown-unknown` target
- Stellar CLI (`soroban-cli`)
- Testnet or mainnet account with sufficient balance

### Installation

```bash
# Clone the repository
git clone <repository-url>
cd soroban-contracts

# Install Rust target
rustup target add wasm32-unknown-unknown

# Build contracts
cargo build --target wasm32-unknown-unknown --release
```

### Deployment

```bash
# Set environment variables
export SOROBAN_TESTNET_SECRET="your_testnet_secret"

# Deploy contracts (testnet)
./scripts/deploy.sh testnet certificate_registry
./scripts/deploy.sh testnet verification_gateway
./scripts/deploy.sh testnet hum_token

# Initialize contracts
./scripts/invoke.sh testnet certificate_registry initialize GGOVERNANCE_ADDRESS GVERIFIER_ADDRESS
./scripts/invoke.sh testnet verification_gateway initialize GCERTIFICATE_REGISTRY_ADDRESS
./scripts/invoke.sh testnet hum_token initialize GADMIN_ADDRESS 1000000000
```

## Testing

Run the comprehensive test suite:

```bash
# Run all tests
cargo test

# Run specific contract tests
cargo test --test certificate_registry_test
cargo test --test verification_gateway_test
cargo test --test hum_token_test
```

## Contract Interfaces

### CertificateRegistry

```rust
pub fn issue_certificate(
    env: Env,
    content_hash: BytesN<32>,
    zk_proof: Bytes,
    public_signals: Vec<Val>,
    did: String,
    content_type: Symbol,
) -> Result<BytesN<32>, CertificateRegistryError>;

pub fn revoke_certificate(
    env: Env,
    cert_id: BytesN<32>,
    reason: Symbol,
) -> Result<(), CertificateRegistryError>;

pub fn get_certificate(
    env: Env,
    cert_id: BytesN<32>,
) -> Option<Certificate>;

pub fn is_certified(
    env: Env,
    content_hash: BytesN<32>,
) -> bool;

pub fn initialize(
    env: Env,
    governance_address: Address,
    proof_verifier_address: Address,
);
```

### VerificationGateway

```rust
pub fn verify(
    env: Env,
    content_hash: BytesN<32>,
) -> Result<VerificationResult, VerificationGatewayError>;

pub fn batch_verify(
    env: Env,
    content_hashes: Vec<BytesN<32>>,
) -> Result<Vec<VerificationResult>, VerificationGatewayError>;

pub fn initialize(
    env: Env,
    certificate_registry_address: Address,
);

pub fn update_registry(
    env: Env,
    new_registry_address: Address,
);
```

### HUMToken

```rust
pub fn initialize(env: Env, admin: Address, initial_supply: i128);

pub fn stake(env: Env, amount: i128, staker: Address) -> Result<(), HUMTokenError>;

pub fn unstake(env: Env, amount: i128, staker: Address) -> Result<(), HUMTokenError>;

pub fn slash(env: Env, staker: Address, amount: i128, reason: Symbol) -> Result<(), HUMTokenError>;

pub fn get_stake(env: Env, staker: Address) -> i128;

pub fn transfer(env: Env, from: Address, to: Address, amount: i128) -> Result<(), HUMTokenError>;

pub fn balance(env: Env, account: Address) -> i128;

pub fn total_supply(env: Env) -> i128;

pub fn total_staked(env: Env) -> i128;
```

## Data Types

### Certificate
```rust
pub struct Certificate {
    pub id: BytesN<32>,
    pub content_hash: BytesN<32>,
    pub human_commitment: BytesN<32>,
    pub did: String,
    pub content_type: Symbol,
    pub issued_at: u64,
    pub revoked_at: Option<u64>,
}
```

### VerificationResult
```rust
pub struct VerificationResult {
    pub certified: bool,
    pub certificate: Option<Certificate>,
    pub error: Option<Symbol>,
}
```

### StakeInfo
```rust
pub struct StakeInfo {
    pub amount: i128,
    pub staked_at: u64,
    pub last_slashed_at: Option<u64>,
}
```

## Error Handling

All contracts use comprehensive error enums with meaningful error codes:

```rust
#[contracterror]
pub enum HumonicsError {
    AlreadyCertified = 1,
    InvalidProof = 2,
    CertNotFound = 3,
    Unauthorized = 4,
    AlreadyRevoked = 5,
    InsufficientStake = 6,
    InsufficientBalance = 7,
    InvalidAmount = 8,
    NotStaked = 9,
    AlreadyStaked = 10,
    CannotSlashBelowZero = 11,
    CertificateRegistryNotSet = 12,
    VerificationFailed = 13,
    InvalidInputs = 14,
}
```

## Storage Patterns

- **Instance Storage**: Configuration and contract state
- **Persistent Storage**: User data that persists across upgrades
- **Temporary Storage**: Ephemeral data for single transactions

## Gas Optimization

- Efficient storage key usage
- Minimal cross-contract calls
- Optimized data structures
- Batch operations where possible

## Security Considerations

- Never store PII on-chain
- All external inputs are untrusted until verified
- Comprehensive input validation
- Role-based access control
- Audit trail for all operations

## Development

### Building

```bash
# Development build
cargo build

# Release build (optimized)
cargo build --target wasm32-unknown-unknown --release
```

### Testing

```bash
# Run all tests
cargo test

# Run with output
cargo test -- --nocapture

# Run specific test
cargo test test_issue_certificate_happy_path
```

### Linting

```bash
# Run clippy
cargo clippy -- -D warnings

# Format code
cargo fmt
```

## Deployment Scripts

### deploy.sh
Automates contract deployment to testnet or mainnet with environment file management.

### invoke.sh
Provides a comprehensive CLI for contract method invocation with argument validation.

## Environment Variables

- `SOROBAN_TESTNET_SECRET`: Testnet account secret
- `SOROBAN_MAINNET_SECRET`: Mainnet account secret

## Contributing

1. Fork the repository
2. Create a feature branch
3. Add comprehensive tests
4. Ensure all tests pass
5. Submit a pull request

## License

This project is licensed under the MIT License - see the LICENSE file for details.

## Security Audit

⚠️ **IMPORTANT**: Do not deploy to mainnet without a completed security audit.

## Support

For questions and support, please open an issue in the repository.
