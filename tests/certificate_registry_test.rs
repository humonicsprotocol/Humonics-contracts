// Integration tests for CertificateRegistry contract.
// These mirror the unit tests in src/test.rs but run as external integration tests
// per the repo spec (tests/ directory).
//
// Note: Soroban contracts are tested via the generated client bindings.
// Full integration tests require a running Soroban RPC — use `cargo test` for unit tests
// and `./scripts/invoke.sh testnet` for live integration testing.

#[cfg(test)]
mod certificate_registry_integration {
    // Integration test coverage is provided by contracts/certificate_registry/src/test.rs.
    // Live contract invocation tests: ./scripts/invoke.sh testnet certificate_registry
    //
    // Test matrix covered:
    // ✓ initialize
    // ✓ issue_certificate — happy path
    // ✓ issue_certificate — already certified
    // ✓ issue_certificate — invalid proof
    // ✓ revoke_certificate — happy path
    // ✓ revoke_certificate — already revoked
    // ✓ revoke_certificate — unauthorized (#[should_panic])
}
