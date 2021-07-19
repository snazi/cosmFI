use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{CanonicalAddr, ReadonlyStorage, StdResult, Storage, Uint128};
use cosmwasm_storage::{
    bucket, bucket_read, singleton, singleton_read, Bucket, ReadonlyBucket, ReadonlySingleton,
    Singleton,
};
use cw721::{Expiration};

pub const CONFIG_KEY: &[u8] = b"config";
pub const MINTER_KEY: &[u8] = b"minter";
pub const CONTRACT_INFO_KEY: &[u8] = b"nft_info";
pub const BASE_TOKENS_KEY: &[u8] = b"base_tokens";
pub const SILVER_TOKENS_KEY: &[u8] = b"silver_tokens";
pub const GOLD_TOKENS_KEY: &[u8] = b"gold_tokens";

pub const TOKEN_PREFIX: &[u8] = b"tokens";
pub const OPERATOR_PREFIX: &[u8] = b"operators";

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct TokenInfo {
    /// The owner of the newly minter NFT
    pub owner: CanonicalAddr,
    /// approvals are stored here, as we clear them all upon transfer and cannot accumulate much
    pub approvals: Vec<Approval>,
    /// Describes the rank of the NFT 
    pub rank: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ContractInfo {
    /// name of the NFT contract
    pub name: String,
    /// symbol of the NFT contract
    pub symbol: String, // Becomes LB
    /// cap is the maximum number of tokens that could be minted
    pub base_cap: Uint128,
    pub silver_cap: Uint128,
    pub gold_cap: Uint128,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
pub struct Approval {
    /// Account that can transfer/send the token
    pub spender: CanonicalAddr,
    /// When the Approval expires (maybe Expiration::never)
    pub expires: Expiration,
}

pub fn contract_info<S: Storage>(storage: &mut S) -> Singleton<S, ContractInfo> {
    singleton(storage, CONTRACT_INFO_KEY)
}

pub fn contract_info_read<S: ReadonlyStorage>(
    storage: &S,
) -> ReadonlySingleton<S, ContractInfo> {
    singleton_read(storage, CONTRACT_INFO_KEY)
}

pub fn mint<S: Storage>(storage: &mut S) -> Singleton<S, CanonicalAddr> {
    singleton(storage, MINTER_KEY)
}

pub fn mint_read<S: ReadonlyStorage>(storage: &S) -> ReadonlySingleton<S, CanonicalAddr> {
    singleton_read(storage, MINTER_KEY)
}

// BASE TOKEN
fn base_count<S: Storage>(storage: &mut S) -> Singleton<S, u64> {
    singleton(storage, BASE_TOKENS_KEY)
}

fn base_count_read<S: ReadonlyStorage>(storage: &S) -> ReadonlySingleton<S, u64> {
    singleton_read(storage, BASE_TOKENS_KEY)
}

pub fn base_num_tokens<S: ReadonlyStorage>(storage: &S) -> StdResult<u64> {
    Ok(base_count_read(storage).may_load()?.unwrap_or_default())
}

pub fn increment_base_tokens<S: Storage>(storage: &mut S) -> StdResult<u64> {
    let val = base_num_tokens(storage)? + 1;
    base_count(storage).save(&val)?;
    Ok(val)
}

// SILVER TOKEN
fn silver_count<S: Storage>(storage: &mut S) -> Singleton<S, u64> {
    singleton(storage, SILVER_TOKENS_KEY)
}

fn silver_count_read<S: ReadonlyStorage>(storage: &S) -> ReadonlySingleton<S, u64> {
    singleton_read(storage, SILVER_TOKENS_KEY)
}

pub fn silver_num_tokens<S: ReadonlyStorage>(storage: &S) -> StdResult<u64> {
    Ok(silver_count_read(storage).may_load()?.unwrap_or_default())
}

pub fn increment_silver_tokens<S: Storage>(storage: &mut S) -> StdResult<u64> {
    let val = silver_num_tokens(storage)? + 1;
    silver_count(storage).save(&val)?;
    Ok(val)
}

// GOLD TOKEN
fn gold_count<S: Storage>(storage: &mut S) -> Singleton<S, u64> {
    singleton(storage, GOLD_TOKENS_KEY)
}

fn gold_count_read<S: ReadonlyStorage>(storage: &S) -> ReadonlySingleton<S, u64> {
    singleton_read(storage, GOLD_TOKENS_KEY)
}

pub fn gold_num_tokens<S: ReadonlyStorage>(storage: &S) -> StdResult<u64> {
    Ok(gold_count_read(storage).may_load()?.unwrap_or_default())
}

pub fn increment_gold_tokens<S: Storage>(storage: &mut S) -> StdResult<u64> {
    let val = gold_num_tokens(storage)? + 1;
    gold_count(storage).save(&val)?;
    Ok(val)
}

pub fn tokens<S: Storage>(storage: &mut S) -> Bucket<S, TokenInfo> {
    bucket(TOKEN_PREFIX, storage)
}

pub fn tokens_read<S: ReadonlyStorage>(storage: &S) -> ReadonlyBucket<S, TokenInfo> {
    bucket_read(TOKEN_PREFIX, storage)
}

pub fn operators<'a, S: Storage>(
    storage: &'a mut S,
    owner: &CanonicalAddr,
) -> Bucket<'a, S, Expiration> {
    Bucket::multilevel(&[OPERATOR_PREFIX, owner.as_slice()], storage)
}

pub fn operators_read<'a, S: ReadonlyStorage>(
    storage: &'a S,
    owner: &CanonicalAddr,
) -> ReadonlyBucket<'a, S, Expiration> {
    ReadonlyBucket::multilevel(&[OPERATOR_PREFIX, owner.as_slice()], storage)
}
