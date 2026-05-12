#![no_std]
use soroban_sdk::{symbol_short, Address, BytesN, Bytes, Symbol, Vec, Val, Env, IntoVal, TryFromVal, xdr::ToXdr, String};
use humonics_shared::{Certificate, RegistryConfig, HumonicsError, CONFIG_KEY};

mod test;

#[soroban_sdk::contract]
pub struct CertificateRegistry;

#[soroban_sdk::contractimpl]
impl CertificateRegistry {
    pub fn issue_certificate(
        env: Env,
        content_hash: BytesN<32>,
        zk_proof: Bytes,
        public_signals: Vec<Val>,
        did: String,
        content_type: Symbol,
    ) -> Result<BytesN<32>, HumonicsError> {

        if content_hash == BytesN::from_array(&env, &[0u8; 32]) || did.len() == 0 {
            return Err(HumonicsError::InvalidInputs);
        }

        let config: RegistryConfig = env.storage().instance().get(&CONFIG_KEY).ok_or(HumonicsError::Unauthorized)?;

        if Self::is_certified(env.clone(), content_hash.clone()) {
            return Err(HumonicsError::AlreadyCertified);
        }

        if !Self::verify_zk_proof(env.clone(), config.proof_verifier_address, zk_proof, public_signals.clone()) {
            return Err(HumonicsError::InvalidProof);
        }

        let cert_tuple = (content_hash.clone(), did.clone(), content_type.clone(), env.ledger().timestamp());
        let cert_id = env.crypto().sha256(&cert_tuple.to_xdr(&env));

        let human_commitment: BytesN<32> = public_signals.get(0).unwrap().into_val(&env);
        let content_hash_signal: BytesN<32> = public_signals.get(1).unwrap().into_val(&env);
        let timestamp: u64 = public_signals.get(2).unwrap().into_val(&env);

        if content_hash_signal != content_hash {
            return Err(HumonicsError::InvalidProof);
        }

        let certificate = Certificate {
            id: cert_id.clone().into(),
            content_hash: content_hash.clone(),
            human_commitment,
            did: did.clone(),
            content_type,
            issued_at: timestamp,
            revoked_at: None,
        };

        env.storage().persistent().set(&cert_id, &certificate);
        env.storage().persistent().set(&content_hash, &cert_id);

        Ok(cert_id.into())
    }

    pub fn revoke_certificate(
        env: Env,
        cert_id: BytesN<32>,
        _reason: Symbol,
    ) -> Result<(), HumonicsError> {

        let mut certificate: Certificate = env.storage().persistent()
            .get(&cert_id)
            .ok_or(HumonicsError::CertNotFound)?;

        if certificate.revoked_at.is_some() {
            return Err(HumonicsError::AlreadyRevoked);
        }

        let config: RegistryConfig = env.storage().instance().get(&CONFIG_KEY).ok_or(HumonicsError::Unauthorized)?;

        config.governance_address.require_auth();

        certificate.revoked_at = Some(env.ledger().timestamp());
        env.storage().persistent().set(&cert_id, &certificate);

        Ok(())
    }

    pub fn get_certificate(
        env: Env,
        cert_id: BytesN<32>,
    ) -> Option<Certificate> {
        env.storage().persistent().get(&cert_id)
    }

    pub fn is_certified(
        env: Env,
        content_hash: BytesN<32>,
    ) -> bool {
        if let Some(cert_id) = env.storage().persistent().get::<_, BytesN<32>>(&content_hash) {
            if let Some(certificate) = env.storage().persistent().get::<_, Certificate>(&cert_id) {
                return certificate.revoked_at.is_none();
            }
        }
        false
    }

    pub fn get_cert_id_by_hash(
        env: Env,
        content_hash: BytesN<32>,
    ) -> BytesN<32> {
        env.storage().persistent().get(&content_hash).unwrap_or_else(|| BytesN::from_array(&env, &[0u8; 32]))
    }

    pub fn initialize(
        env: Env,
        governance_address: Address,
        proof_verifier_address: Address,
    ) {

        if env.storage().instance().has(&CONFIG_KEY) {
            panic!("Contract already initialized");
        }

        let config = RegistryConfig {
            governance_address,
            proof_verifier_address,
        };

        env.storage().instance().set(&CONFIG_KEY, &config);
    }

    fn verify_zk_proof(
        env: Env,
        verifier_address: Address,
        proof: Bytes,
        public_signals: Vec<Val>,
    ) -> bool {
        let mut args = Vec::new(&env);
        args.push_back(proof.into_val(&env));
        args.push_back(public_signals.into_val(&env));
        let result_val: Val = env.invoke_contract(
            &verifier_address,
            &symbol_short!("verify"),
            args,
        );
        let result = bool::try_from_val(&env, &result_val).unwrap_or(false);
        result
    }
}
