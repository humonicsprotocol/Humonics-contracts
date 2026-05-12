#![cfg(test)]
use super::*;
use soroban_sdk::{symbol_short, BytesN, Symbol, Vec, Address, Env, IntoVal, String};
use soroban_sdk::testutils::{Address as _};
use humonics_shared::{VerificationResult, Certificate, HumonicsError};

// Mock Certificate Registry
#[soroban_sdk::contract]
pub struct MockRegistry;

#[soroban_sdk::contractimpl]
impl MockRegistry {
    pub fn is_certified(env: Env, _content_hash: BytesN<32>) -> bool {
        env.storage().instance().get(&symbol_short!("is_cert")).unwrap_or(false)
    }

    pub fn get_cert_id_by_hash(env: Env, _content_hash: BytesN<32>) -> BytesN<32> {
        env.storage().instance().get(&symbol_short!("cert_id")).unwrap_or_else(|| BytesN::from_array(&env, &[0u8; 32]))
    }

    pub fn get_certificate(env: Env, _cert_id: BytesN<32>) -> Option<Certificate> {
        env.storage().instance().get(&symbol_short!("cert"))
    }
}

#[test]
fn test_initialize() {
    let env = Env::default();
    let registry_address = Address::generate(&env);
    
    let client = VerificationGatewayClient::new(&env, &env.register_contract(None, VerificationGateway));
    
    client.initialize(&registry_address);
    
    let config = client.get_config();
    assert_eq!(config.certificate_registry_address, registry_address);
}

#[test]
fn test_verify_certified_content() {
    let env = Env::default();
    let registry_address = env.register_contract(None, MockRegistry);
    
    let client = VerificationGatewayClient::new(&env, &env.register_contract(None, VerificationGateway));
    client.initialize(&registry_address);
    
    let content_hash = BytesN::from_array(&env, &[1u8; 32]);
    let cert_id = BytesN::from_array(&env, &[2u8; 32]);
    
    let certificate = Certificate {
        id: cert_id.clone(),
        content_hash: content_hash.clone(),
        human_commitment: BytesN::from_array(&env, &[3u8; 32]),
        did: String::from_str(&env, "did:stellar:GABCDEF123456789"),
        content_type: symbol_short!("text"),
        issued_at: 1234567890,
        revoked_at: None,
    };

    // Setup mock data
    env.as_contract(&registry_address, || {
        env.storage().instance().set(&symbol_short!("is_cert"), &true);
        env.storage().instance().set(&symbol_short!("cert_id"), &cert_id);
        env.storage().instance().set(&symbol_short!("cert"), &certificate);
    });
    
    let result = client.verify(&content_hash);
    
    match result {
        VerificationResult::Certified(cert) => assert_eq!(cert, certificate),
        _ => panic!("Expected Certified"),
    }
}

#[test]
fn test_verify_not_certified() {
    let env = Env::default();
    let registry_address = env.register_contract(None, MockRegistry);
    
    let client = VerificationGatewayClient::new(&env, &env.register_contract(None, VerificationGateway));
    client.initialize(&registry_address);
    
    let content_hash = BytesN::from_array(&env, &[1u8; 32]);
    
    // Setup mock data
    env.as_contract(&registry_address, || {
        env.storage().instance().set(&symbol_short!("is_cert"), &false);
    });
    
    let result = client.verify(&content_hash);
    
    match result {
        VerificationResult::NotCertified(s) => assert_eq!(s, symbol_short!("not_cert")),
        _ => panic!("Expected NotCertified"),
    }
}

#[test]
fn test_verify_revoked_certificate() {
    let env = Env::default();
    let registry_address = env.register_contract(None, MockRegistry);
    
    let client = VerificationGatewayClient::new(&env, &env.register_contract(None, VerificationGateway));
    client.initialize(&registry_address);
    
    let content_hash = BytesN::from_array(&env, &[1u8; 32]);
    let cert_id = BytesN::from_array(&env, &[2u8; 32]);
    
    let certificate = Certificate {
        id: cert_id.clone(),
        content_hash: content_hash.clone(),
        human_commitment: BytesN::from_array(&env, &[3u8; 32]),
        did: String::from_str(&env, "did:stellar:GABCDEF123456789"),
        content_type: symbol_short!("text"),
        issued_at: 1234567890,
        revoked_at: Some(1234567891),
    };

    // Setup mock data
    env.as_contract(&registry_address, || {
        env.storage().instance().set(&symbol_short!("is_cert"), &true);
        env.storage().instance().set(&symbol_short!("cert_id"), &cert_id);
        env.storage().instance().set(&symbol_short!("cert"), &certificate);
    });
    
    let result = client.verify(&content_hash);
    
    match result {
        VerificationResult::Revoked(cert) => assert_eq!(cert, certificate),
        _ => panic!("Expected Revoked"),
    }
}

#[test]
fn test_verify_invalid_inputs() {
    let env = Env::default();
    let zero_hash = BytesN::from_array(&env, &[0u8; 32]);
    
    let client = VerificationGatewayClient::new(&env, &env.register_contract(None, VerificationGateway));
    
    let result = client.try_verify(&zero_hash);
    assert_eq!(result, Err(Ok(HumonicsError::InvalidInputs)));
}
