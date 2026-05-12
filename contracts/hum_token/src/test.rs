#![cfg(test)]
use super::*;
use soroban_sdk::{symbol_short, Address, Env};
use soroban_sdk::testutils::{Address as _};
use humonics_shared::{TokenConfig, StakeInfo, HumonicsError};

#[test]
fn test_initialize() {
    let env = Env::default();
    let admin = Address::generate(&env);
    let initial_supply = 1000000000i128;
    
    let client = HUMTokenClient::new(&env, &env.register_contract(None, HUMToken));
    
    client.initialize(&admin, &initial_supply);
    
    let config = client.get_config();
    assert_eq!(config.admin, admin);
    assert!(config.slashing_enabled);
    assert_eq!(client.total_supply(), initial_supply);
    assert_eq!(client.balance(&admin), initial_supply);
}

#[test]
fn test_stake_happy_path() {
    let env = Env::default();
    let admin = Address::generate(&env);
    let staker = Address::generate(&env);
    let initial_supply = 1000000000i128;
    let stake_amount = 1000000i128;
    
    let client = HUMTokenClient::new(&env, &env.register_contract(None, HUMToken));
    client.initialize(&admin, &initial_supply);
    
    client.transfer(&admin, &staker, &stake_amount);
    
    env.mock_all_auths();
    
    client.stake(&stake_amount, &staker);
    
    assert_eq!(client.get_stake(&staker), stake_amount);
    assert_eq!(client.balance(&staker), 0);
    assert_eq!(client.total_staked(), stake_amount);
}

#[test]
fn test_stake_insufficient_balance() {
    let env = Env::default();
    let admin = Address::generate(&env);
    let staker = Address::generate(&env);
    let initial_supply = 1000000000i128;
    let stake_amount = 2000000i128;
    
    let client = HUMTokenClient::new(&env, &env.register_contract(None, HUMToken));
    client.initialize(&admin, &initial_supply);
    
    client.transfer(&admin, &staker, &1000000i128);
    
    env.mock_all_auths();
    
    let result = client.try_stake(&stake_amount, &staker);
    assert_eq!(result, Err(Ok(HumonicsError::InsufficientBalance)));
}

#[test]
fn test_unstake_happy_path() {
    let env = Env::default();
    let admin = Address::generate(&env);
    let staker = Address::generate(&env);
    let initial_supply = 1000000000i128;
    let stake_amount = 1000000i128;
    
    let client = HUMTokenClient::new(&env, &env.register_contract(None, HUMToken));
    client.initialize(&admin, &initial_supply);
    
    client.transfer(&admin, &staker, &stake_amount);
    
    env.mock_all_auths();
    
    client.stake(&stake_amount, &staker);
    
    client.unstake(&stake_amount, &staker);
    
    assert_eq!(client.get_stake(&staker), 0);
    assert_eq!(client.balance(&staker), stake_amount);
    assert_eq!(client.total_staked(), 0);
}

#[test]
fn test_slash_happy_path() {
    let env = Env::default();
    let admin = Address::generate(&env);
    let staker = Address::generate(&env);
    let initial_supply = 1000000000i128;
    let stake_amount = 1000000i128;
    let slash_amount = 500000i128;
    
    let client = HUMTokenClient::new(&env, &env.register_contract(None, HUMToken));
    client.initialize(&admin, &initial_supply);
    
    client.transfer(&admin, &staker, &stake_amount);
    
    env.mock_all_auths();
    
    client.stake(&stake_amount, &staker);
    
    let reason = symbol_short!("fraud");
    client.slash(&staker, &slash_amount, &reason);
    
    assert_eq!(client.get_stake(&staker), stake_amount - slash_amount);
    assert_eq!(client.total_staked(), stake_amount - slash_amount);
    
    let stake_info = client.get_stake_info(&staker).unwrap();
    assert!(stake_info.last_slashed_at.is_some());
}

#[test]
fn test_transfer_happy_path() {
    let env = Env::default();
    let admin = Address::generate(&env);
    let recipient = Address::generate(&env);
    let initial_supply = 1000000000i128;
    let transfer_amount = 1000000i128;
    
    let client = HUMTokenClient::new(&env, &env.register_contract(None, HUMToken));
    client.initialize(&admin, &initial_supply);
    
    env.mock_all_auths();
    
    client.transfer(&admin, &recipient, &transfer_amount);
    
    assert_eq!(client.balance(&admin), initial_supply - transfer_amount);
    assert_eq!(client.balance(&recipient), transfer_amount);
}
