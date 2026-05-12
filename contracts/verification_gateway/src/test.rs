#![cfg(test)]
use super::*;
use soroban_sdk::{symbol_short, BytesN, Vec, Address, Env, String};
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
        env.storage().instance().get(&symbol_short!("cert_id"))
            .unwrap_or_else(|| BytesN::from_array(&env, &[0u8; 32]))
    }

    pub fn get_certificate(env: Env, _cert_id: BytesN<32>) -> Option<Certificate> {
        env.storage().instance().get(&symbol_short!("cert"))
    }
}

fn setup_gateway(env: &Env) -> (VerificationGatewayClient, Address) {
    let registry_address = env.register_contract(None, MockRegistry);
    let client = VerificationGatewayClient::new(env, &env.register_contract(None, VerificationGateway));
    client.initialize(&registry_address);
    (client, registry_address)
}

fn make_cert(env: &Env, revoked: bool) -> Certificate {
    Certificate {
        id: BytesN::from_array(env, &[2u8; 32]),
        content_hash: BytesN::from_array(env, &[1u8; 32]),
        human_commitment: BytesN::from_array(env, &[3u8; 32]),
        did: String::from_str(env, "did:stellar:GABCDEF123456789"),
        content_type: symbol_short!("text"),
        issued_at: 1234567890,
        revoked_at: if revoked { Some(1234567891) } else { None },
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
    let (client, registry_address) = setup_gateway(&env);
    let content_hash = BytesN::from_array(&env, &[1u8; 32]);
    let cert = make_cert(&env, false);

    env.as_contract(&registry_address, || {
        env.storage().instance().set(&symbol_short!("is_cert"), &true);
        env.storage().instance().set(&symbol_short!("cert_id"), &cert.id);
        env.storage().instance().set(&symbol_short!("cert"), &cert);
    });

    let result = client.verify(&content_hash);
    assert!(matches!(result, VerificationResult::Certified(_)));
    if let VerificationResult::Certified(c) = result {
        assert_eq!(c, cert);
    }
}

#[test]
fn test_verify_not_certified() {
    let env = Env::default();
    let (client, registry_address) = setup_gateway(&env);
    let content_hash = BytesN::from_array(&env, &[1u8; 32]);

    env.as_contract(&registry_address, || {
        env.storage().instance().set(&symbol_short!("is_cert"), &false);
    });

    let result = client.verify(&content_hash);
    assert!(matches!(result, VerificationResult::NotCertified(_)));
}

#[test]
fn test_verify_revoked_certificate() {
    let env = Env::default();
    let (client, registry_address) = setup_gateway(&env);
    let content_hash = BytesN::from_array(&env, &[1u8; 32]);
    let cert = make_cert(&env, true);

    env.as_contract(&registry_address, || {
        env.storage().instance().set(&symbol_short!("is_cert"), &true);
        env.storage().instance().set(&symbol_short!("cert_id"), &cert.id);
        env.storage().instance().set(&symbol_short!("cert"), &cert);
    });

    let result = client.verify(&content_hash);
    assert!(matches!(result, VerificationResult::Revoked(_)));
    if let VerificationResult::Revoked(c) = result {
        assert!(c.revoked_at.is_some());
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

#[test]
fn test_batch_verify() {
    let env = Env::default();
    let (client, registry_address) = setup_gateway(&env);
    let hash1 = BytesN::from_array(&env, &[1u8; 32]);
    let hash2 = BytesN::from_array(&env, &[2u8; 32]);

    env.as_contract(&registry_address, || {
        env.storage().instance().set(&symbol_short!("is_cert"), &false);
    });

    let mut hashes = Vec::new(&env);
    hashes.push_back(hash1);
    hashes.push_back(hash2);

    let results = client.batch_verify(&hashes);
    assert_eq!(results.len(), 2);
    assert!(matches!(results.get(0).unwrap(), VerificationResult::NotCertified(_)));
    assert!(matches!(results.get(1).unwrap(), VerificationResult::NotCertified(_)));
}
