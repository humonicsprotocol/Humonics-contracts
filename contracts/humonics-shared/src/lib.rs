#![no_std]
pub use soroban_sdk::{contracttype, contracterror, BytesN, Symbol, Address, String, Vec};

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Certificate {
    pub id: BytesN<32>,
    pub content_hash: BytesN<32>,
    pub human_commitment: BytesN<32>,
    pub did: String,
    pub content_type: Symbol,
    pub issued_at: u64,
    pub revoked_at: Option<u64>,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum VerificationResult {
    Certified(Certificate),
    NotCertified(Symbol),
    Revoked(Certificate),
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StakeInfo {
    pub amount: i128,
    pub staked_at: u64,
    pub last_slashed_at: Option<u64>,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RegistryConfig {
    pub governance_address: Address,
    pub proof_verifier_address: Address,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TokenConfig {
    pub admin: Address,
    pub slashing_enabled: bool,
    pub min_stake_amount: i128,
    pub max_stake_amount: i128,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GatewayConfig {
    pub certificate_registry_address: Address,
}

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
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

pub const CERT_KEY: Symbol = soroban_sdk::symbol_short!("CERT");
pub const HASH_INDEX: Symbol = soroban_sdk::symbol_short!("HIDX");
pub const STAKE_KEY: Symbol = soroban_sdk::symbol_short!("STAKE");
pub const CONFIG_KEY: Symbol = soroban_sdk::symbol_short!("CFG");
pub const TOTAL_SUPPLY_KEY: Symbol = soroban_sdk::symbol_short!("TOTAL");
pub const TOTAL_STAKED_KEY: Symbol = soroban_sdk::symbol_short!("TSTAKED");
pub const BALANCE_KEY: Symbol = soroban_sdk::symbol_short!("BAL");
