#![no_std]
use soroban_sdk::{symbol_short, Address, BytesN, Symbol, Vec, Env, IntoVal, Val};
use humonics_shared::{Certificate, VerificationResult, GatewayConfig, HumonicsError, CONFIG_KEY};

mod test;

#[soroban_sdk::contract]
pub struct VerificationGateway;

#[soroban_sdk::contractimpl]
impl VerificationGateway {
    pub fn verify(
        env: Env,
        content_hash: BytesN<32>,
    ) -> Result<VerificationResult, HumonicsError> {
        if content_hash == BytesN::from_array(&env, &[0u8; 32]) {
            return Err(HumonicsError::InvalidInputs);
        }

        let config: GatewayConfig = env.storage().instance()
            .get(&CONFIG_KEY)
            .ok_or(HumonicsError::CertificateRegistryNotSet)?;

        let cert_registry_address = config.certificate_registry_address;

        let mut args: Vec<Val> = Vec::new(&env);
        args.push_back(content_hash.clone().into_val(&env));
        let is_certified: bool = env.invoke_contract::<bool>(
            &cert_registry_address,
            &Symbol::new(&env, "is_certified"),
            args,
        );

        if !is_certified {
            return Ok(VerificationResult::NotCertified(symbol_short!("not_cert")));
        }

        let mut args: Vec<Val> = Vec::new(&env);
        args.push_back(content_hash.into_val(&env));
        let cert_id: BytesN<32> = env.invoke_contract::<BytesN<32>>(
            &cert_registry_address,
            &Symbol::new(&env, "get_cert_id_by_hash"),
            args,
        );

        if cert_id == BytesN::from_array(&env, &[0u8; 32]) {
            return Ok(VerificationResult::NotCertified(symbol_short!("not_found")));
        }

        let mut args: Vec<Val> = Vec::new(&env);
        args.push_back(cert_id.into_val(&env));
        let certificate: Option<Certificate> = env.invoke_contract::<Option<Certificate>>(
            &cert_registry_address,
            &Symbol::new(&env, "get_certificate"),
            args,
        );

        match certificate {
            Some(cert) => {
                if cert.revoked_at.is_some() {
                    Ok(VerificationResult::Revoked(cert))
                } else {
                    Ok(VerificationResult::Certified(cert))
                }
            }
            None => Ok(VerificationResult::NotCertified(symbol_short!("not_found"))),
        }
    }

    pub fn batch_verify(
        env: Env,
        content_hashes: Vec<BytesN<32>>,
    ) -> Result<Vec<VerificationResult>, HumonicsError> {
        if content_hashes.is_empty() {
            return Err(HumonicsError::InvalidInputs);
        }

        let mut results = Vec::new(&env);

        for content_hash in content_hashes.iter() {
            let result = Self::verify(env.clone(), content_hash)?;
            results.push_back(result);
        }

        Ok(results)
    }

    pub fn initialize(
        env: Env,
        certificate_registry_address: Address,
    ) {

        if env.storage().instance().has(&CONFIG_KEY) {
            panic!("Contract already initialized");
        }

        let config = GatewayConfig {
            certificate_registry_address,
        };

        env.storage().instance().set(&CONFIG_KEY, &config);
    }

    pub fn update_registry(
        env: Env,
        new_registry_address: Address,
    ) {

        let mut config: GatewayConfig = env.storage().instance()
            .get(&CONFIG_KEY)
            .unwrap_or_else(|| panic!("Contract not initialized"));

        config.certificate_registry_address = new_registry_address;
        env.storage().instance().set(&CONFIG_KEY, &config);
    }

    pub fn get_config(env: Env) -> GatewayConfig {
        env.storage().instance()
            .get(&CONFIG_KEY)
            .unwrap_or_else(|| panic!("Contract not initialized"))
    }
}
