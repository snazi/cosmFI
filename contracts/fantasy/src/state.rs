use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{HumanAddr, ReadonlyStorage, Storage, StdResult, Uint128};
use cosmwasm_storage::{
    bucket, bucket_read, singleton, singleton_read, Bucket, ReadonlyBucket, ReadonlySingleton,
    Singleton,
};

pub const STATE_KEY: &[u8] = b"state";
pub const TOTAL_DEPOSIT_KEY: &[u8] = b"total_deposit";
pub const ANCHOR_ADDR_KEY: &[u8] = b"anchor_addr";
pub const TERRAND_ADDR_KEY: &[u8] = b"terrand_addr";
pub const PACK_LEN_KEY: &[u8] = b"pack_len";
pub const TOKEN_ADDRESSES_PREFIX: &[u8] = b"token_addresses";
pub const CONTRACT_COUNT_KEY: &[u8] = b"contract_count";

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct State {
    /// Stable coin denomination. 
    pub stable_denom: String,
    // anchor contract address for depositing the rewards
    pub anchor_addr: HumanAddr,
    // terrand contract address for calling Oracle's DRand
    pub terrand_addr: HumanAddr,
    /// number of NFT players to be pulled per pack
    pub pack_len: Uint128,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct TokenData {
    // ID of the athlete token
    pub athlete_id: String,
    // Contract address of the athlete token
    pub contract_addr: HumanAddr,
}

pub fn state<S: Storage>(storage: &mut S) -> Singleton<S, State> {
    singleton(storage, STATE_KEY)
}

pub fn state_read<S: ReadonlyStorage>(
    storage: &S,
) -> ReadonlySingleton<S, State> {
    singleton_read(storage, STATE_KEY)
}

pub fn total_deposit<S: Storage>(storage: &mut S) -> Singleton<S, u64> {
    singleton(storage, TOTAL_DEPOSIT_KEY)
}

pub fn total_deposit_read<S: ReadonlyStorage>(storage: &S) -> ReadonlySingleton<S, u64> {
    singleton_read(storage, TOTAL_DEPOSIT_KEY)
}

pub fn increase_deposit<S: Storage>(storage: &mut S, amount: u64) -> StdResult<u64> {
    let val = total_deposit_read(storage).load()? + amount;
    total_deposit(storage).save(&val)?;
    Ok(val)
}

pub fn reduce_deposit<S: Storage>(storage: &mut S, amount: u64) -> StdResult<u64> {
    let val = total_deposit_read(storage).load()? - amount;
    total_deposit(storage).save(&val)?;
    Ok(val)
}

pub fn token_addresses<S: Storage>(storage: &mut S) -> Bucket<S, HumanAddr> {
    bucket(TOKEN_ADDRESSES_PREFIX, storage)
}

pub fn token_addresses_read<S: ReadonlyStorage>(storage: &S) -> ReadonlyBucket<S, HumanAddr> {
    bucket_read(TOKEN_ADDRESSES_PREFIX, storage)
}

fn contract_count<S: Storage>(storage: &mut S) -> Singleton<S, u64> {
    singleton(storage, CONTRACT_COUNT_KEY)
}

fn contract_count_read<S: ReadonlyStorage>(storage: &S) -> ReadonlySingleton<S, u64> {
    singleton_read(storage, CONTRACT_COUNT_KEY)
}

pub fn get_contract_count<S: ReadonlyStorage>(storage: &S) -> StdResult<u64> {
    Ok(contract_count_read(storage).may_load()?.unwrap_or_default())
}

pub fn increment_contract_count<S: Storage>(storage: &mut S) -> StdResult<u64> {
    let val = get_contract_count(storage)? + 1;
    contract_count(storage).save(&val)?;
    Ok(val)
}

pub fn anchor_addr<S: Storage>(storage: &mut S) -> Singleton<S, HumanAddr> {
    singleton(storage, ANCHOR_ADDR_KEY)
}

pub fn anchor_addr_read<S: ReadonlyStorage>(storage: &S) -> ReadonlySingleton<S, HumanAddr> {
    singleton_read(storage, ANCHOR_ADDR_KEY)
}

pub fn terrand_addr<S: Storage>(storage: &mut S) -> Singleton<S, HumanAddr> {
    singleton(storage, TERRAND_ADDR_KEY)
}

pub fn terrand_addr_read<S: ReadonlyStorage>(storage: &S) -> ReadonlySingleton<S, HumanAddr> {
    singleton_read(storage, TERRAND_ADDR_KEY)
}

pub fn pack_len<S: Storage>(storage: &mut S) -> Singleton<S, HumanAddr> {
    singleton(storage, PACK_LEN_KEY)
}

pub fn pack_len_read<S: ReadonlyStorage>(storage: &S) -> ReadonlySingleton<S, HumanAddr> {
    singleton_read(storage, PACK_LEN_KEY)
}
