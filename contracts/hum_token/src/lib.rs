#![no_std]
use soroban_sdk::{Address, Env, Map, Symbol};
use humonics_shared::{StakeInfo, TokenConfig, HumonicsError, BALANCE_KEY, STAKE_KEY, CONFIG_KEY, TOTAL_SUPPLY_KEY, TOTAL_STAKED_KEY};

mod test;

#[soroban_sdk::contract]
pub struct HUMToken;

#[soroban_sdk::contractimpl]
impl HUMToken {
    pub fn initialize(env: Env, admin: Address, initial_supply: i128) {

        if env.storage().instance().has(&CONFIG_KEY) {
            panic!("Contract already initialized");
        }

        let config = TokenConfig {
            admin: admin.clone(),
            slashing_enabled: true,
            min_stake_amount: 1_000_000, // 0.001 HUM (assuming 7 decimals)
            max_stake_amount: 1_000_000_000_000_000, // 1B HUM
        };

        env.storage().instance().set(&CONFIG_KEY, &config);
        env.storage().instance().set(&TOTAL_SUPPLY_KEY, &initial_supply);
        env.storage().instance().set(&TOTAL_STAKED_KEY, &0i128);

        let mut balances = Map::<Address, i128>::new(&env);
        balances.set(admin, initial_supply);
        env.storage().instance().set(&BALANCE_KEY, &balances);
    }

    pub fn stake(env: Env, amount: i128, staker: Address) -> Result<(), HumonicsError> {

        if amount <= 0 {
            return Err(HumonicsError::InvalidAmount);
        }

        let config: TokenConfig = env.storage().instance()
            .get(&CONFIG_KEY)
            .unwrap_or_else(|| panic!("Contract not initialized"));

        if amount < config.min_stake_amount || amount > config.max_stake_amount {
            return Err(HumonicsError::InvalidAmount);
        }

        let mut balances: Map<Address, i128> = env.storage().instance()
            .get(&BALANCE_KEY)
            .unwrap_or_else(|| Map::new(&env));

        let current_balance = balances.get(staker.clone()).unwrap_or(0);
        if current_balance < amount {
            return Err(HumonicsError::InsufficientBalance);
        }

        let mut stakes: Map<Address, StakeInfo> = env.storage().instance()
            .get(&STAKE_KEY)
            .unwrap_or_else(|| Map::new(&env));

        let stake_info = stakes.get(staker.clone()).unwrap_or(StakeInfo {
            amount: 0,
            staked_at: env.ledger().timestamp(),
            last_slashed_at: None,
        });

        if stake_info.amount > 0 {
            return Err(HumonicsError::AlreadyStaked);
        }

        balances.set(staker.clone(), current_balance - amount);
        env.storage().instance().set(&BALANCE_KEY, &balances);

        stakes.set(staker.clone(), StakeInfo {
            amount,
            staked_at: env.ledger().timestamp(),
            last_slashed_at: None,
        });
        env.storage().instance().set(&STAKE_KEY, &stakes);

        let total_staked: i128 = env.storage().instance()
            .get(&TOTAL_STAKED_KEY)
            .unwrap_or(0);
        env.storage().instance().set(&TOTAL_STAKED_KEY, &(total_staked + amount));

        Ok(())
    }

    pub fn unstake(env: Env, amount: i128, staker: Address) -> Result<(), HumonicsError> {

        if amount <= 0 {
            return Err(HumonicsError::InvalidAmount);
        }

        let mut stakes: Map<Address, StakeInfo> = env.storage().instance()
            .get(&STAKE_KEY)
            .unwrap_or_else(|| Map::new(&env));

        let mut stake_info = stakes.get(staker.clone())
            .ok_or(HumonicsError::NotStaked)?;

        if stake_info.amount < amount {
            return Err(HumonicsError::InsufficientStake);
        }

        stake_info.amount -= amount;
        if stake_info.amount == 0 {
            stakes.remove(staker.clone());
        } else {
            stakes.set(staker.clone(), stake_info);
        }
        env.storage().instance().set(&STAKE_KEY, &stakes);

        let mut balances: Map<Address, i128> = env.storage().instance()
            .get(&BALANCE_KEY)
            .unwrap_or_else(|| Map::new(&env));

        let current_balance = balances.get(staker.clone()).unwrap_or(0);
        balances.set(staker.clone(), current_balance + amount);
        env.storage().instance().set(&BALANCE_KEY, &balances);

        let total_staked: i128 = env.storage().instance()
            .get(&TOTAL_STAKED_KEY)
            .unwrap_or(0);
        env.storage().instance().set(&TOTAL_STAKED_KEY, &(total_staked - amount));

        Ok(())
    }

    pub fn slash(env: Env, staker: Address, amount: i128, _reason: Symbol) -> Result<(), HumonicsError> {

        if amount <= 0 {
            return Err(HumonicsError::InvalidAmount);
        }

        let config: TokenConfig = env.storage().instance()
            .get(&CONFIG_KEY)
            .unwrap_or_else(|| panic!("Contract not initialized"));

        config.admin.require_auth();

        if !config.slashing_enabled {
            return Err(HumonicsError::Unauthorized);
        }

        let mut stakes: Map<Address, StakeInfo> = env.storage().instance()
            .get(&STAKE_KEY)
            .unwrap_or_else(|| Map::new(&env));

        let mut stake_info = stakes.get(staker.clone())
            .ok_or(HumonicsError::NotStaked)?;

        if stake_info.amount < amount {
            return Err(HumonicsError::CannotSlashBelowZero);
        }

        stake_info.amount -= amount;
        stake_info.last_slashed_at = Some(env.ledger().timestamp());

        if stake_info.amount == 0 {
            stakes.remove(staker.clone());
        } else {
            stakes.set(staker.clone(), stake_info);
        }
        env.storage().instance().set(&STAKE_KEY, &stakes);

        let total_staked: i128 = env.storage().instance()
            .get(&TOTAL_STAKED_KEY)
            .unwrap_or(0);
        env.storage().instance().set(&TOTAL_STAKED_KEY, &(total_staked - amount));

        Ok(())
    }

    pub fn get_stake(env: Env, staker: Address) -> i128 {
        let stakes: Map<Address, StakeInfo> = env.storage().instance()
            .get(&STAKE_KEY)
            .unwrap_or_else(|| Map::new(&env));

        stakes.get(staker)
            .map(|info| info.amount)
            .unwrap_or(0)
    }

    pub fn get_stake_info(env: Env, staker: Address) -> Option<StakeInfo> {
        let stakes: Map<Address, StakeInfo> = env.storage().instance()
            .get(&STAKE_KEY)
            .unwrap_or_else(|| Map::new(&env));

        stakes.get(staker)
    }

    pub fn transfer(env: Env, from: Address, to: Address, amount: i128) -> Result<(), HumonicsError> {

        if amount <= 0 {
            return Err(HumonicsError::InvalidAmount);
        }

        let mut balances: Map<Address, i128> = env.storage().instance()
            .get(&BALANCE_KEY)
            .unwrap_or_else(|| Map::new(&env));

        let from_balance = balances.get(from.clone()).unwrap_or(0);
        if from_balance < amount {
            return Err(HumonicsError::InsufficientBalance);
        }

        let to_balance = balances.get(to.clone()).unwrap_or(0);

        balances.set(from.clone(), from_balance - amount);
        balances.set(to.clone(), to_balance + amount);
        env.storage().instance().set(&BALANCE_KEY, &balances);

        Ok(())
    }

    pub fn balance(env: Env, account: Address) -> i128 {
        let balances: Map<Address, i128> = env.storage().instance()
            .get(&BALANCE_KEY)
            .unwrap_or_else(|| Map::new(&env));

        balances.get(account).unwrap_or(0)
    }

    pub fn total_supply(env: Env) -> i128 {
        env.storage().instance()
            .get(&TOTAL_SUPPLY_KEY)
            .unwrap_or(0)
    }

    pub fn total_staked(env: Env) -> i128 {
        env.storage().instance()
            .get(&TOTAL_STAKED_KEY)
            .unwrap_or(0)
    }

    pub fn get_config(env: Env) -> TokenConfig {
        env.storage().instance()
            .get(&CONFIG_KEY)
            .unwrap_or_else(|| panic!("Contract not initialized"))
    }

    pub fn update_config(env: Env, new_config: TokenConfig) {

        let current_config: TokenConfig = env.storage().instance()
            .get(&CONFIG_KEY)
            .unwrap_or_else(|| panic!("Contract not initialized"));

        current_config.admin.require_auth();

        env.storage().instance().set(&CONFIG_KEY, &new_config);
    }
}
