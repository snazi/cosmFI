use cosmwasm_std::{
    from_binary, log, to_binary, Api, Binary, Env, Extern, HandleResponse, HumanAddr,
    InitResponse, Querier, WasmQuery, StdError, StdResult, Storage, Uint128, Coin,
};
use cosmwasm_storage::to_length_prefixed;
use cosmwasm_bignumber::{Decimal256};
//use rand::distributions::{Distribution, Uniform};

use cw2::set_contract_version;
use cw20::{Cw20HandleMsg};


use crate::msg::{HandleMsg, InitMsg, QueryMsg, AnchorMsg, TokenMsg, ConfigResponse, StateResponse,
                 QueryMintMsg, ContractCountResponse,                    
};
use crate::state::{
    increase_deposit, total_deposit, total_deposit_read, state, state_read, State, TokenData,
    token_addresses, token_addresses_read,
    get_contract_count, increment_contract_count,
};
use crate::helpers::{
    encode_msg_execute,
    encode_raw_query,
};
use crate::querier::{deduct_tax};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:fantasy";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

pub fn init<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    msg: InitMsg,
) -> StdResult<InitResponse> {
    set_contract_version(&mut deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    let info = State {
        stable_denom: msg.stable_denom,
        anchor_addr: msg.anchor_addr,
        terrand_addr: msg.terrand_addr,
        pack_len: msg.pack_len,
    };

    match msg.tokens {
        Some(m) => handle_add_token(deps, env, m)?,
        None => HandleResponse {
            messages: vec![],
            log: vec![],
            data: None,
        },
    };

    state(&mut deps.storage).save(&info)?;
    total_deposit(&mut deps.storage).save(&0)?;
    Ok(InitResponse::default())
}

pub fn handle<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    msg: HandleMsg,
) -> StdResult<HandleResponse> {
    match msg {
        HandleMsg::PurchasePack {} => handle_purchase(deps, env),
        HandleMsg::DepositStable {} => handle_deposit(deps, env),
        HandleMsg::RedeemStable {
            amount,
        } => handle_redeem(deps, env, amount),
        HandleMsg::AddToken {
            tokens
        } => handle_add_token(deps, env, tokens),
    }
}

pub fn handle_purchase<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
) -> StdResult<HandleResponse> {
    let sender_raw = deps.api.canonical_address(&env.message.sender)?;
    let sender = deps.api.human_address(&sender_raw)?;

    // TODO: Generate N token ids based on the pack_len using Terrand
    let athlete_pack = generate_pack(&deps).unwrap();
    let mut mint_responses = vec![];

    for athlete in athlete_pack.iter() {
        let token_address = query_token_address(&deps, athlete.clone().to_string()).unwrap();

        //TODO: Handle error from query_token_address. Ensure that the generated token ids are a subset of the token addresses
        
        let mint_msg = TokenMsg::Mint {
            owner: sender.clone(),
            rank: "B".to_string(),
        };

        let mint_res = encode_msg_execute(
            to_binary(&mint_msg).unwrap(),
            token_address.clone(),
            vec![]
        )?;

        // TODO: handle error from mint_res

        mint_responses.push(mint_res);
    }

    let response = HandleResponse {
        messages: mint_responses,
        log: vec![
            log("action", "purchase"),
            log("from", &sender),
        ],
        data: None,
    };
    
    Ok(response)
}

pub fn handle_deposit<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
) -> StdResult<HandleResponse> {
    let sender_raw = deps.api.canonical_address(&env.message.sender)?;
    let sender = deps.api.human_address(&sender_raw)?;

    let deposit_amount: Uint128 = env
        .message
        .sent_funds
        .iter()
        .find(|c| c.denom == "uusd")
        .map(|c| Uint128::from(c.amount))
        .unwrap_or_else(Uint128::zero);
    
    // coin deposit minus tax
    let coin_deposit = deduct_tax(
        &deps, 
        Coin {
            denom: "uusd".to_string(),
            amount: deposit_amount
        }
    )?;
    let contract = query_state(&deps).unwrap().anchor_addr;
    
    // execute anchor's deposit stable contract
    let deposit_msg = to_binary(&AnchorMsg::DepositStable{})?;
    let anchor_res = encode_msg_execute(
        deposit_msg,
        contract.clone(),
        vec![coin_deposit.clone()]
    )?;

    increase_deposit(&mut deps.storage, coin_deposit.amount.u128() as u64)?;

    let response = HandleResponse {
        messages: vec![anchor_res],
        log: vec![
            log("action", "deposit"),
            log("from", &sender),
            log("to", &contract),
            log("deposit_amount", &coin_deposit.amount),
        ],
        data: None,
    };
    
    Ok(response)
}

pub fn handle_redeem<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    amount: Uint128,
) -> StdResult<HandleResponse> {
    let sender_raw = deps.api.canonical_address(&env.message.sender)?;
    let sender = deps.api.human_address(&sender_raw)?;
    let anchor_contract = query_state(&deps).unwrap().anchor_addr;

     // get exchange rate from anchor state
     let state_bin: Binary = encode_raw_query(
        &deps, 
        Binary::from(to_length_prefixed(b"state")),
        anchor_contract.clone(),
    )?;

    // transform binary response to state response
    let state_response: StateResponse = from_binary(&state_bin)?;
    let exchange_rate: Decimal256 = state_response.prev_exchange_rate.into();

    let contract_msg = to_binary(&AnchorMsg::RedeemStable{})?;
    let aust_amount = amount * (Decimal256::one() / exchange_rate).into();

    let logs = vec![
        log("action", "receive"),
        log("from", &sender),
        log("to", &anchor_contract),
        log("amount", &amount),
        log("aust_amount", &aust_amount),
    ];

    // get anchor usd (aust) contract address from anchor config
    let config_bin: Binary = encode_raw_query(
        &deps,
        Binary::from(to_length_prefixed(b"config")),
        anchor_contract.clone(),
    )?;

    // transform binary response to state response
    let config_response: ConfigResponse = from_binary(&config_bin)?;
    let aterra_contract = deps.api.human_address(&config_response.aterra_contract)?;

    // create a send message
    let msg = to_binary(&Cw20HandleMsg::Send{
        amount: aust_amount,
        contract: anchor_contract,
        msg: Some(contract_msg)
    })?;

    let anchor_response = encode_msg_execute(
        msg,
        aterra_contract,
        vec![]
    )?;

    let res = HandleResponse {
        messages: vec![anchor_response],
        log: logs,
        data: None,
    };
    
    Ok(res)
}

pub fn handle_add_token<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    _env: Env,
    tokens: Vec<TokenData>,
) -> StdResult<HandleResponse> {

    for token in tokens.iter() {
        token_addresses(&mut deps.storage).update(token.athlete_id.as_bytes(), |old| match old {
            Some(_) => Err(StdError::generic_err("athlete_id already claimed")),
            None => Ok(token.contract_addr.clone()),
        })?;

        increment_contract_count(&mut deps.storage)?;
    }

    let response = HandleResponse {
        messages: vec![],
        log: vec![
            log("action", "add_token"),
        ],
        data: None,
    };
    
    Ok(response)
}

pub fn query<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    msg: QueryMsg,
) -> StdResult<Binary> {
    match msg {
        QueryMsg::State {} => to_binary(&query_state(deps)?),
        QueryMsg::TotalDeposit {} => to_binary(&query_total_deposit(deps)?),
        QueryMsg::TokenContract {
            athlete_id
        } => to_binary(&query_token_address(deps, athlete_id)?),
        QueryMsg::IsTokenMintable {
            athlete_id
        } => to_binary(&query_token_mintable(deps, athlete_id)?),
        QueryMsg::ContractCount {} => to_binary(&query_contract_count(deps)?),
    }
}

fn query_state<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
) -> StdResult<State> {
    state_read(&deps.storage).load()
}

fn query_total_deposit<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
) -> StdResult<Uint128> {
    Ok(Uint128(total_deposit_read(&deps.storage).load()? as u128))
}

fn query_contract_count<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
) -> StdResult<ContractCountResponse> {
    let count = get_contract_count(&deps.storage)?;
    Ok(ContractCountResponse { count })
}

fn query_token_address<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    athlete_id: String
) -> StdResult<HumanAddr> {
    token_addresses_read(&deps.storage).load(athlete_id.as_bytes())
}

fn query_token_mintable<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    athlete_id: String
) -> StdResult<bool> {
    let token_address = query_token_address(&deps, athlete_id).unwrap();

    // TODO: Query token_address if mintable using the NFT contract's IsMintable{} query
    let msg = QueryMintMsg::IsMintable { rank: "B".to_string() };
    let wasm = WasmQuery::Smart {
        contract_addr: token_address,
        msg: to_binary(&msg)?,
    };
    let is_mintable: bool = deps.querier.query(&wasm.into())?;

    Ok(is_mintable)
}

fn generate_pack<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
) -> StdResult<Vec<String>> {
    let contract_info = query_state(&deps).unwrap();
    let pack_len = contract_info.pack_len;
    let contract_count = query_contract_count(&deps).unwrap().count;
   
    let mut pack: Vec<String> = Vec::new();
    //let mut roll = rand::thread_rng();
    //let rolls = Uniform::from(0..5);
    //let rolls = Uniform::from(0..contract_count);
    let mut mintable_tokens = Uint128::from(0 as u128);

    while mintable_tokens < pack_len {
        //let pull = rolls.sample(&mut roll) as u8;
        let pull = 1;
        
        if query_token_mintable(&deps, pull.to_string()).unwrap(){    
            // let token_data = TokenData {
            //     athlete_id: pull.to_string(),
            //     contract_addr: query_token_address(&deps, pull.clone().to_string()).unwrap()
            // };
            pack.push(pull.to_string());
            mintable_tokens += Uint128::from(1 as u128);
        }
    }

    Ok(pack)
}