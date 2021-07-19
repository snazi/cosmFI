use cosmwasm_std::{HumanAddr, CanonicalAddr, Uint128};
use cosmwasm_bignumber::{Uint256, Decimal256};
use crate::state::{TokenData};
use cw20::{Cw20ReceiveMsg};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InitMsg {
    /// Stable coin denomination. 
    pub stable_denom: String,
    // anchor contract address for depositing the rewards
    pub anchor_addr: HumanAddr,
    // terrand contract address for calling Oracle's DRand
    pub terrand_addr: HumanAddr,
    // athlete token data (optional)
    pub tokens: Option<Vec<TokenData>>,
    // Number of Player NFTs to be pulled per pack
    pub pack_len: Uint128,
}

/// This is like Cw721HandleMsg but we add a Mint command for an owner
/// to make this stand-alone. You will likely want to remove mint and
/// use other control logic in any contract that inherits this.
#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
#[serde(rename_all = "snake_case")]
pub enum HandleMsg {
    /// Purchase an athlete token pack
    PurchasePack {},
    /// Deposit Stablecoins into the contract to receive an athlete token
    DepositStable {},
    /// Redeem Stablecoins (UST) from Anchor
    RedeemStable {
        //amount in uusd to be redeemed from Anchor
        amount: Uint128,
    },
    /// Add athlete token contract address
    AddToken {
        tokens: Vec<TokenData>,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    /// Returns the state for the Fantasy Contract
    State {},
    /// Returns the contract address of the corresponding token id
    TokenContract {
        athlete_id: String,
    },
    /// Returns the total deposited stable coin amount to Anchor
    TotalDeposit {},
    /// Returns a boolean if the token is mintable using the Athlete Contract's IsMintable{} Query
    IsTokenMintable {
        athlete_id: String,
    },
    /// Returns the total number of Athlete Contracts saved 
    ContractCount {},
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
pub struct ContractCountResponse {
    pub count: u64,
}

/// Athlete Token Message
#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
#[serde(rename_all = "snake_case")]
pub enum TokenMsg {
    Mint {
        /// The owner of the newly minter NFT
        owner: HumanAddr,
        /// Describes the rank of the NFT 
        rank: String,
    },
}

/// IsMintable Message
#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
#[serde(rename_all = "snake_case")]
pub enum QueryMintMsg {
    IsMintable {
        /// Describes the rank of the NFT 
        rank: String,
    },
}

/// Anchor Messages
#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
#[serde(rename_all = "snake_case")]
pub enum AnchorMsg {
    Config {},
    DepositStable {},
    Receive(Cw20ReceiveMsg),
    RedeemStable {},
}

/// Anchor Responses
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct StateResponse {
    pub total_liabilities: Decimal256,
    pub total_reserves: Decimal256,
    pub last_interest_updated: u64,
    pub last_reward_updated: u64,
    pub global_interest_index: Decimal256,
    pub global_reward_index: Decimal256,
    pub anc_emission_rate: Decimal256,
    pub prev_aterra_supply: Uint256,
    pub prev_exchange_rate: Decimal256,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ConfigResponse {
    pub contract_addr: CanonicalAddr,
    pub owner_addr: CanonicalAddr,
    pub aterra_contract: CanonicalAddr,
    pub interest_model: CanonicalAddr,
    pub distribution_model: CanonicalAddr,
    pub overseer_contract: CanonicalAddr,
    pub collector_contract: CanonicalAddr,
    pub distributor_contract: CanonicalAddr,
    pub stable_denom: String,
    pub max_borrow_factor: Decimal256,
}