#![cfg(test)]
use super::*;
use soroban_sdk::{BytesN, Symbol, Vec, Val, Address, Env, IntoVal, Bytes, String};
use soroban_sdk::testutils::{Address as _};
use humonics_shared::{Certificate, RegistryConfig, HumonicsError};

// Mock Verifier
#[soroban_sdk::contract]
pub struct MockVerifier;

#[soroban_sdk::contractimpl]
impl MockVerifier {
    pub fn verify(env: Env, _proof: Bytes, _public_signals: Vec<Val>) -> bool {
        env.storage().instance().get(&symbol_short!("result")).unwrap_or(false)
    }
}

#[test]
fn test_initialize() {
    let env = Env::default();
    let admin = Address::generate(&env);
    let verifier = Address::generate(&env);
    
    let client = CertificateRegistryClient::new(&env, &env.register_contract(None, CertificateRegistry));
    
    client.initialize(&admin, &verifier);
}

#[test]
fn test_issue_certificate_happy_path() {
    let env = Env::default();
    let admin = Address::generate(&env);
    let verifier_address = env.register_contract(None, MockVerifier);
    
    let client = CertificateRegistryClient::new(&env, &env.register_contract(None, CertificateRegistry));
    client.initialize(&admin, &verifier_address);
    
    let content_hash = BytesN::from_array(&env, &[1u8; 32]);
    let zk_proof = soroban_sdk::Bytes::from_slice(&env, b"proof_data");
    let mut public_signals = Vec::new(&env);
    public_signals.push_back(BytesN::from_array(&env, &[2u8; 32]).into_val(&env)); // human_commitment
    public_signals.push_back(content_hash.clone().into_val(&env)); // content_hash
    public_signals.push_back(1234567890u64.into_val(&env)); // timestamp
    
    let did = Address::generate(&env).to_string();
    let content_type = symbol_short!("text");
    
    // Setup mock verifier
    env.as_contract(&verifier_address, || {
        env.storage().instance().set(&symbol_short!("result"), &true);
    });
    
    env.mock_all_auths();
    
    let cert_id = client.issue_certificate(
        &content_hash,
        &zk_proof,
        &public_signals,
        &did,
        &content_type,
    );
    
    assert_ne!(cert_id, BytesN::from_array(&env, &[0u8; 32]));
    
    let certificate = client.get_certificate(&cert_id).unwrap();
    assert_eq!(certificate.content_hash, content_hash);
    assert_eq!(certificate.did, did);
    assert_eq!(certificate.content_type, content_type);
    assert_eq!(certificate.revoked_at, None);
}

#[test]
fn test_issue_certificate_already_certified() {
    let env = Env::default();
    let admin = Address::generate(&env);
    let verifier_address = env.register_contract(None, MockVerifier);
    
    let client = CertificateRegistryClient::new(&env, &env.register_contract(None, CertificateRegistry));
    client.initialize(&admin, &verifier_address);
    
    let content_hash = BytesN::from_array(&env, &[1u8; 32]);
    let zk_proof = soroban_sdk::Bytes::from_slice(&env, b"proof_data");
    let mut public_signals = Vec::new(&env);
    public_signals.push_back(BytesN::from_array(&env, &[2u8; 32]).into_val(&env));
    public_signals.push_back(content_hash.clone().into_val(&env));
    public_signals.push_back(1234567890u64.into_val(&env));
    
    let did = Address::generate(&env).to_string();
    let content_type = symbol_short!("text");
    
    env.as_contract(&verifier_address, || {
        env.storage().instance().set(&symbol_short!("result"), &true);
    });
    
    env.mock_all_auths();
    
    client.issue_certificate(
        &content_hash,
        &zk_proof,
        &public_signals,
        &did,
        &content_type,
    );
    
    let result = client.try_issue_certificate(
        &content_hash,
        &zk_proof,
        &public_signals,
        &did,
        &content_type,
    );
    
    assert_eq!(result, Err(Ok(HumonicsError::AlreadyCertified)));
}

#[test]
fn test_revoke_certificate_happy_path() {
    let env = Env::default();
    let admin = Address::generate(&env);
    let verifier_address = env.register_contract(None, MockVerifier);
    
    let contract_id = env.register_contract(None, CertificateRegistry);
    let client = CertificateRegistryClient::new(&env, &contract_id);
    client.initialize(&admin, &verifier_address);
    
    let content_hash = BytesN::from_array(&env, &[1u8; 32]);
    let zk_proof = soroban_sdk::Bytes::from_slice(&env, b"proof_data");
    let mut public_signals = Vec::new(&env);
    public_signals.push_back(BytesN::from_array(&env, &[2u8; 32]).into_val(&env));
    public_signals.push_back(content_hash.clone().into_val(&env));
    public_signals.push_back(1234567890u64.into_val(&env));
    
    let did_address = Address::generate(&env);
    let did = did_address.to_string();
    let content_type = symbol_short!("text");
    
    env.as_contract(&verifier_address, || {
        env.storage().instance().set(&symbol_short!("result"), &true);
    });
    
    env.mock_all_auths();
    
    let cert_id = client.issue_certificate(
        &content_hash,
        &zk_proof,
        &public_signals,
        &did,
        &content_type,
    );
    
    // Revoke as admin (governance)
    client.revoke_certificate(&cert_id, &symbol_short!("fraud"));
    
    let certificate = client.get_certificate(&cert_id).unwrap();
    assert!(certificate.revoked_at.is_some());
}
