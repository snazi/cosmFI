use cosmwasm_std::{
    from_binary, log, to_binary, Api, Binary, CosmosMsg, Env, Extern, HandleResponse, HumanAddr,
    InitResponse, Order, Querier, StdError, StdResult, Storage, WasmMsg, Uint128,
};

use cw0::{calc_range_start_human, calc_range_start_string};
use cw2::set_contract_version;
use cw721::{
    AllNftInfoResponse, ApprovedForAllResponse, Expiration, NftInfoResponse, OwnerOfResponse,
    NumTokensResponse, TokensResponse,
};

use crate::msg::{HandleMsg, InitMsg, MinterResponse, QueryMsg};
use crate::state::{
    contract_info, contract_info_read, mint, mint_read, operators,
    operators_read, tokens, tokens_read, Approval, TokenInfo, ContractInfo,
    base_num_tokens, increment_base_tokens,
    silver_num_tokens, increment_silver_tokens,
    gold_num_tokens, increment_gold_tokens,
};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:cw721-base";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

pub fn init<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    _env: Env,
    msg: InitMsg,
) -> StdResult<InitResponse> {
    set_contract_version(&mut deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    let info = ContractInfo {
        name: msg.name,
        symbol: msg.symbol,
        base_cap: msg.base_cap,
        silver_cap: msg.silver_cap,
        gold_cap: msg.gold_cap,
    };
    contract_info(&mut deps.storage).save(&info)?;
    let minter = deps.api.canonical_address(&msg.minter)?;
    mint(&mut deps.storage).save(&minter)?;
    Ok(InitResponse::default())
}

pub fn handle<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    msg: HandleMsg,
) -> StdResult<HandleResponse> {
    match msg {
        HandleMsg::Mint {
            owner,
            rank
        } => handle_mint(deps, env, owner, rank),
        HandleMsg::BaseToSilver {
            base_1,
            base_2,
            base_3,
        } => handle_base_to_silver(deps, env, base_1, base_2, base_3),
        HandleMsg::SilverToGold {
            silver_1,
            silver_2,
            silver_3,
            silver_4,
            silver_5,
        } => handle_silver_to_gold(deps, env, silver_1, silver_2, silver_3, silver_4, silver_5),
        HandleMsg::Approve {
            spender,
            token_id,
            expires,
        } => handle_approve(deps, env, spender, token_id, expires),
        HandleMsg::Revoke { spender, token_id } => handle_revoke(deps, env, spender, token_id),
        HandleMsg::ApproveAll { operator, expires } => {
            handle_approve_all(deps, env, operator, expires)
        }
        HandleMsg::RevokeAll { operator } => handle_revoke_all(deps, env, operator),
        HandleMsg::TransferNft {
            recipient,
            token_id,
        } => handle_transfer_nft(deps, env, recipient, token_id),
        HandleMsg::SendNft {
            contract,
            token_id,
            msg,
        } => handle_send_nft(deps, env, contract, token_id, msg),
        HandleMsg::UpdateMinter {
            minter,
        } => handle_update_minter(deps, env, minter),
    }
}

pub fn handle_mint<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    owner: HumanAddr,
    rank: String,
) -> StdResult<HandleResponse> {
    let rank_copy_1 = rank.clone();
    let rank_copy_2 = rank.clone();
    let rank_copy_3 = rank.clone();
    let minter = mint(&mut deps.storage).load()?;
    let sender_raw = deps.api.canonical_address(&env.message.sender)?;

    if sender_raw != minter {
        if sender_raw !=  deps.api.canonical_address(&env.contract.address)? {
            return Err(StdError::unauthorized());
        }
    } 

    // create the token
    let token = TokenInfo {
        owner: deps.api.canonical_address(&owner)?,
        approvals: vec![],
        rank,
    };

    if !query_mintable(deps, rank_copy_1).unwrap() {
        return Err(StdError::generic_err("Minting cannot exceed the cap"))
    }

    // generate a token id based on rank
    let token_id = generate_token_id(deps, rank_copy_2).unwrap();

    tokens(&mut deps.storage).update(token_id.as_bytes(), |old| match old {
        Some(_) => Err(StdError::generic_err("token_id already claimed")),
        None => Ok(token),
    })?;

    if rank_copy_3.eq("S"){
        increment_silver_tokens(&mut deps.storage)?;
    } else if rank_copy_3.eq("G"){
        increment_gold_tokens(&mut deps.storage)?;
    } else  {
        increment_base_tokens(&mut deps.storage)?;
    }

    Ok(HandleResponse {
        messages: vec![],
        log: vec![
            log("action", "mint"),
            log("minter", env.message.sender),
            log("token_id", token_id),
            log("rank", rank_copy_3),
        ],
        data: None,
    })
}

pub fn handle_update_minter<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    minter: HumanAddr,
) -> StdResult<HandleResponse> {
    let prev_minter = mint(&mut deps.storage).load()?;
    let sender_raw = deps.api.canonical_address(&env.message.sender)?;

    if sender_raw != prev_minter {
        return Err(StdError::unauthorized());
    }

    let new_minter = deps.api.canonical_address(&minter)?;
    mint(&mut deps.storage).save(&new_minter)?;

    Ok(HandleResponse {
        messages: vec![],
        log: vec![
            log("action", "update_minter"),
            log("minter", env.message.sender),
        ],
        data: None,
    })
}

pub fn handle_base_to_silver<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    base_1: String,
    base_2: String,
    base_3: String,
) -> StdResult<HandleResponse> {
    let sender_raw = deps.api.canonical_address(&env.message.sender)?;
    let sender = deps.api.human_address(&sender_raw)?;

    // the contract address is used as the burn address
    _transfer_nft(deps, &env, &env.contract.address, &base_1)?;
    _transfer_nft(deps, &env, &env.contract.address, &base_2)?;
    _transfer_nft(deps, &env, &env.contract.address, &base_3)?;

    // create msg for minting
    let mint_msg = HandleMsg::Mint {
        owner: sender.clone(), 
        rank: "S".into()
    };

    // Have the contract execute the mint function since the sender is not authorized to mint
    let _mint_response = StdResult::<CosmosMsg>::from(Ok(WasmMsg::Execute {
        contract_addr: env.contract.address,
        msg: to_binary(&mint_msg).unwrap(),
        send: vec![],
    }.into()))?;

    Ok(HandleResponse {
        messages: vec![],
        log: vec![
            log("action", "base_to_gold"),
            log("sender", &sender),
            log("base_1", base_1),
            log("base_2", base_2),
            log("base_3", base_3),
        ],
        data: None,
    })
}

pub fn handle_silver_to_gold<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    silver_1: String,
    silver_2: String,
    silver_3: String,
    silver_4: String,
    silver_5: String,
) -> StdResult<HandleResponse> {
    let sender_raw = deps.api.canonical_address(&env.message.sender)?;
    let sender = deps.api.human_address(&sender_raw)?;

    // the contract address is used as the burn address
    _transfer_nft(deps, &env, &env.contract.address, &silver_1)?;
    _transfer_nft(deps, &env, &env.contract.address, &silver_2)?;
    _transfer_nft(deps, &env, &env.contract.address, &silver_3)?;
    _transfer_nft(deps, &env, &env.contract.address, &silver_4)?;
    _transfer_nft(deps, &env, &env.contract.address, &silver_5)?;

    // create msg for minting
    let mint_msg = HandleMsg::Mint {
        owner: sender.clone(), 
        rank: "G".into()
    };

    // Have the contract execute the mint function since the sender is not authorized to mint
    let _mint_response = StdResult::<CosmosMsg>::from(Ok(WasmMsg::Execute {
        contract_addr: env.contract.address,
        msg: to_binary(&mint_msg).unwrap(),
        send: vec![],
    }.into()))?;

    Ok(HandleResponse {
        messages: vec![],
        log: vec![
            log("action", "base_to_gold"),
            log("sender", &sender),
            log("silver_1", silver_1),
            log("silver_2", silver_2),
            log("silver_3", silver_3),
            log("silver_4", silver_4),
            log("silver_5", silver_5),
        ],
        data: None,
    })
}

pub fn handle_transfer_nft<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    recipient: HumanAddr,
    token_id: String,
) -> StdResult<HandleResponse> {
    _transfer_nft(deps, &env, &recipient, &token_id)?;

    Ok(HandleResponse {
        messages: vec![],
        log: vec![
            log("action", "transfer_nft"),
            log("sender", env.message.sender),
            log("recipient", recipient),
            log("token_id", token_id),
        ],
        data: None,
    })
}

pub fn handle_send_nft<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    contract: HumanAddr,
    token_id: String,
    msg: Option<Binary>,
) -> StdResult<HandleResponse> {
    // Unwrap message first
    let msgs: Vec<CosmosMsg> = match &msg {
        None => vec![],
        Some(msg) => vec![from_binary(msg)?],
    };

    // Transfer token
    _transfer_nft(deps, &env, &contract, &token_id)?;

    // Send message
    Ok(HandleResponse {
        messages: msgs,
        log: vec![
            log("action", "send_nft"),
            log("sender", env.message.sender),
            log("recipient", contract),
            log("token_id", token_id),
        ],
        data: None,
    })
}

pub fn _transfer_nft<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: &Env,
    recipient: &HumanAddr,
    token_id: &str,
) -> StdResult<TokenInfo> {
    let mut token = tokens(&mut deps.storage).load(token_id.as_bytes())?;
    // ensure we have permissions
    check_ownership(&deps, env, &token)?;
    // set owner and remove existing approvals
    token.owner = deps.api.canonical_address(recipient)?;
    token.approvals = vec![];
    tokens(&mut deps.storage).save(token_id.as_bytes(), &token)?;
    Ok(token)
}

pub fn handle_approve<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    spender: HumanAddr,
    token_id: String,
    expires: Option<Expiration>,
) -> StdResult<HandleResponse> {
    _update_approvals(deps, &env, &spender, &token_id, true, expires)?;

    Ok(HandleResponse {
        messages: vec![],
        log: vec![
            log("action", "approve"),
            log("sender", env.message.sender),
            log("spender", spender),
            log("token_id", token_id),
        ],
        data: None,
    })
}

pub fn handle_revoke<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    spender: HumanAddr,
    token_id: String,
) -> StdResult<HandleResponse> {
    _update_approvals(deps, &env, &spender, &token_id, false, None)?;

    Ok(HandleResponse {
        messages: vec![],
        log: vec![
            log("action", "revoke"),
            log("sender", env.message.sender),
            log("spender", spender),
            log("token_id", token_id),
        ],
        data: None,
    })
}

pub fn _update_approvals<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: &Env,
    spender: &HumanAddr,
    token_id: &str,
    // if add == false, remove. if add == true, remove then set with this expiration
    add: bool,
    expires: Option<Expiration>,
) -> StdResult<TokenInfo> {
    let mut token = tokens(&mut deps.storage).load(token_id.as_bytes())?;
    // ensure we have permissions
    check_can_approve(&deps, &env, &token)?;

    // update the approval list (remove any for the same spender before adding)
    let spender_raw = deps.api.canonical_address(&spender)?;
    token.approvals = token
        .approvals
        .into_iter()
        .filter(|apr| apr.spender != spender_raw)
        .collect();

    // only difference between approve and revoke
    if add {
        // reject expired data as invalid
        let expires = expires.unwrap_or_default();
        if expires.is_expired(&env.block) {
            return Err(StdError::generic_err(
                "Cannot set approval that is already expired",
            ));
        }
        let approval = Approval {
            spender: spender_raw,
            expires,
        };
        token.approvals.push(approval);
    }

    tokens(&mut deps.storage).save(token_id.as_bytes(), &token)?;

    Ok(token)
}

pub fn handle_approve_all<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    operator: HumanAddr,
    expires: Option<Expiration>,
) -> StdResult<HandleResponse> {
    // reject expired data as invalid
    let expires = expires.unwrap_or_default();
    if expires.is_expired(&env.block) {
        return Err(StdError::generic_err(
            "Cannot set approval that is already expired",
        ));
    }

    // set the operator for us
    let sender_raw = deps.api.canonical_address(&env.message.sender)?;
    let operator_raw = deps.api.canonical_address(&operator)?;
    operators(&mut deps.storage, &sender_raw).save(operator_raw.as_slice(), &expires)?;

    Ok(HandleResponse {
        messages: vec![],
        log: vec![
            log("action", "approve_all"),
            log("sender", env.message.sender),
            log("operator", operator),
        ],
        data: None,
    })
}

pub fn handle_revoke_all<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    operator: HumanAddr,
) -> StdResult<HandleResponse> {
    let sender_raw = deps.api.canonical_address(&env.message.sender)?;
    let operator_raw = deps.api.canonical_address(&operator)?;
    operators(&mut deps.storage, &sender_raw).remove(operator_raw.as_slice());

    Ok(HandleResponse {
        messages: vec![],
        log: vec![
            log("action", "revoke_all"),
            log("sender", env.message.sender),
            log("operator", operator),
        ],
        data: None,
    })
}

/// returns true iff the sender can execute approve or reject on the contract
fn check_can_approve<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    env: &Env,
    token: &TokenInfo,
) -> StdResult<()> {
    // owner can approve
    let sender_raw = deps.api.canonical_address(&env.message.sender)?;
    if token.owner == sender_raw {
        return Ok(());
    }
    // operator can approve
    let op = operators_read(&deps.storage, &token.owner).may_load(sender_raw.as_slice())?;
    match op {
        Some(ex) => {
            if ex.is_expired(&env.block) {
                Err(StdError::unauthorized())
            } else {
                Ok(())
            }
        }
        None => Err(StdError::unauthorized()),
    }
}

/// returns true iff the sender can transfer ownership of the token
fn check_ownership<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    env: &Env,
    token: &TokenInfo,
) -> StdResult<()> {
    // owner can send
    let sender_raw = deps.api.canonical_address(&env.message.sender)?;
    if token.owner == sender_raw {
        return Ok(());
    }

    // contract address can send
    if env.contract.address == env.message.sender {
        return Ok(());
    }

    // any non-expired token approval can send
    if token
        .approvals
        .iter()
        .any(|apr| apr.spender == sender_raw && !apr.expires.is_expired(&env.block))
    {
        return Ok(());
    }

    // operator can send
    let op = operators_read(&deps.storage, &token.owner).may_load(sender_raw.as_slice())?;
    match op {
        Some(ex) => {
            if ex.is_expired(&env.block) {
                Err(StdError::unauthorized())
            } else {
                Ok(())
            }
        }
        None => Err(StdError::unauthorized()),
    }
}

fn generate_token_id<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    rank: String,
) -> StdResult<String> {

    let contract_info = query_contract_info(&deps).unwrap();
    let mut token_id: String = contract_info.symbol.to_owned();

    if rank.eq("S"){
        let rank_string: &str = "S";
        let silver_count = query_silver_tokens(&deps).unwrap().count + 1;

        token_id.push_str(rank_string);
        token_id.push_str(&silver_count.to_string());
    } else if rank.eq("G"){
        let rank_string: &str = "G";
        let gold_count = query_gold_tokens(&deps).unwrap().count + 1;

        token_id.push_str(rank_string);
        token_id.push_str(&gold_count.to_string());
    } else  {
        let rank_string: &str = "B";
        let base_count = query_base_tokens(&deps).unwrap().count + 1;

        token_id.push_str(rank_string);
        token_id.push_str(&base_count.to_string());
    }

    Ok(token_id)
}

pub fn query<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    msg: QueryMsg,
) -> StdResult<Binary> {
    match msg {
        QueryMsg::Minter {} => to_binary(&query_minter(deps)?),
        QueryMsg::ContractInfo {} => to_binary(&query_contract_info(deps)?),
        QueryMsg::NftInfo { token_id } => to_binary(&query_nft_info(deps, token_id)?),
        QueryMsg::OwnerOf { token_id } => to_binary(&query_owner_of(deps, token_id)?),
        QueryMsg::AllNftInfo { token_id } => to_binary(&query_all_nft_info(deps, token_id)?),
        QueryMsg::ApprovedForAll {
            owner,
            start_after,
            limit,
        } => to_binary(&query_all_approvals(deps, owner, start_after, limit)?),
        QueryMsg::BaseTokens {} => to_binary(&query_base_tokens(deps)?),
        QueryMsg::AllBaseTokens { start_after, limit } => {
            to_binary(&query_all_base_tokens(deps, start_after, limit)?)
        },
        QueryMsg::SilverTokens {} => to_binary(&query_silver_tokens(deps)?),
        QueryMsg::AllSilverTokens { start_after, limit } => {
            to_binary(&query_all_silver_tokens(deps, start_after, limit)?)
        },
        QueryMsg::GoldTokens {} => to_binary(&query_gold_tokens(deps)?),
        QueryMsg::AllGoldTokens { start_after, limit } => {
            to_binary(&query_all_gold_tokens(deps, start_after, limit)?)
        },
        QueryMsg::IsMintable { rank } => to_binary(&query_mintable(deps, rank)?),
    }
}

fn query_mintable<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    rank: String,
) -> StdResult<bool> {

    let contract_info = query_contract_info(&deps).unwrap();
    let mut is_mintable = true;

    if rank.eq("S"){
        let silver_count = query_silver_tokens(&deps).unwrap();

        if Uint128::from(silver_count.count) == contract_info.silver_cap {
            is_mintable = false
        }
    } else if rank.eq("G"){
        let gold_count = query_gold_tokens(&deps).unwrap();

        if Uint128::from(gold_count.count) == contract_info.gold_cap {
            is_mintable = false
        }
    } else  {
        let base_count = query_base_tokens(&deps).unwrap();

        if Uint128::from(base_count.count) == contract_info.base_cap {
            is_mintable = false
        }
    }

    Ok(is_mintable)
}

fn query_minter<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
) -> StdResult<MinterResponse> {
    let minter_raw = mint_read(&deps.storage).load()?;
    let minter = deps.api.human_address(&minter_raw)?;
    Ok(MinterResponse { minter })
}

fn query_contract_info<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
) -> StdResult<ContractInfo> {
    contract_info_read(&deps.storage).load()
}

fn query_base_tokens<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
) -> StdResult<NumTokensResponse> {
    let count = base_num_tokens(&deps.storage)?;
    Ok(NumTokensResponse { count })
}

fn query_silver_tokens<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
) -> StdResult<NumTokensResponse> {
    let count = silver_num_tokens(&deps.storage)?;
    Ok(NumTokensResponse { count })
}

fn query_gold_tokens<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
) -> StdResult<NumTokensResponse> {
    let count = gold_num_tokens(&deps.storage)?;
    Ok(NumTokensResponse { count })
}

fn query_nft_info<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    token_id: String,
) -> StdResult<NftInfoResponse> {
    let info = tokens_read(&deps.storage).load(token_id.as_bytes())?;
    Ok(NftInfoResponse {
        rank: info.rank,
    })
}

fn query_owner_of<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    token_id: String,
) -> StdResult<OwnerOfResponse> {
    let info = tokens_read(&deps.storage).load(token_id.as_bytes())?;
    Ok(OwnerOfResponse {
        owner: deps.api.human_address(&info.owner)?,
        approvals: humanize_approvals(deps.api, &info)?,
    })
}

const DEFAULT_LIMIT: u32 = 10;
const MAX_LIMIT: u32 = 30;

fn query_all_approvals<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    owner: HumanAddr,
    start_after: Option<HumanAddr>,
    limit: Option<u32>,
) -> StdResult<ApprovedForAllResponse> {
    let owner_raw = deps.api.canonical_address(&owner)?;
    let limit = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT) as usize;
    let start = calc_range_start_human(deps.api, start_after)?;

    let res: StdResult<Vec<_>> = operators_read(&deps.storage, &owner_raw)
        .range(start.as_deref(), None, Order::Ascending)
        .take(limit)
        .map(|item| {
            item.and_then(|(k, expires)| {
                Ok(cw721::Approval {
                    spender: deps.api.human_address(&k.into())?,
                    expires,
                })
            })
        })
        .collect();
    Ok(ApprovedForAllResponse { operators: res? })
}

fn query_all_base_tokens<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    start_after: Option<String>,
    limit: Option<u32>,
) -> StdResult<TokensResponse> {
    let limit = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT) as usize;
    let start = calc_range_start_string(start_after);

    let tokens: StdResult<Vec<String>> = tokens_read(&deps.storage)
        .range(start.as_deref(), None, Order::Ascending)
        .take(limit)
        .map(|item| item.map(|(k, _)| String::from_utf8_lossy(&k).to_string()))
        .collect();
    Ok(TokensResponse { tokens: tokens? })
}

fn query_all_silver_tokens<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    start_after: Option<String>,
    limit: Option<u32>,
) -> StdResult<TokensResponse> {
    let limit = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT) as usize;
    let start = calc_range_start_string(start_after);

    let tokens: StdResult<Vec<String>> = tokens_read(&deps.storage)
        .range(start.as_deref(), None, Order::Ascending)
        .take(limit)
        .map(|item| item.map(|(k, _)| String::from_utf8_lossy(&k).to_string()))
        .collect();
    Ok(TokensResponse { tokens: tokens? })
}

fn query_all_gold_tokens<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    start_after: Option<String>,
    limit: Option<u32>,
) -> StdResult<TokensResponse> {
    let limit = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT) as usize;
    let start = calc_range_start_string(start_after);

    let tokens: StdResult<Vec<String>> = tokens_read(&deps.storage)
        .range(start.as_deref(), None, Order::Ascending)
        .take(limit)
        .map(|item| item.map(|(k, _)| String::from_utf8_lossy(&k).to_string()))
        .collect();
    Ok(TokensResponse { tokens: tokens? })
}

fn query_all_nft_info<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    token_id: String,
) -> StdResult<AllNftInfoResponse> {
    let info = tokens_read(&deps.storage).load(token_id.as_bytes())?;
    Ok(AllNftInfoResponse {
        access: OwnerOfResponse {
            owner: deps.api.human_address(&info.owner)?,
            approvals: humanize_approvals(deps.api, &info)?,
        },
        info: NftInfoResponse {
            rank: info.rank,
        },
    })
}

fn humanize_approvals<A: Api>(api: A, info: &TokenInfo) -> StdResult<Vec<cw721::Approval>> {
    info.approvals
        .iter()
        .map(|apr| humanize_approval(api, apr))
        .collect()
}

fn humanize_approval<A: Api>(api: A, approval: &Approval) -> StdResult<cw721::Approval> {
    Ok(cw721::Approval {
        spender: api.human_address(&approval.spender)?,
        expires: approval.expires,
    })
}


#[cfg(test)]
mod tests {
    use cosmwasm_std::testing::{mock_dependencies, mock_env};
    use cosmwasm_std::{StdError, WasmMsg, Uint128};

    use super::*;
    use cw721::ApprovedForAllResponse;

    const MINTER: &str = "cosmos2contract";
    const CONTRACT_NAME: &str = "Lebron Token";
    const SYMBOL: &str = "LBJ";
    const BASE_CAP: Uint128 = Uint128(2);
    const SILVER_CAP: Uint128 = Uint128(2);
    const GOLD_CAP: Uint128 = Uint128(2);

    fn setup_contract<S: Storage, A: Api, Q: Querier>(deps: &mut Extern<S, A, Q>) {
        let msg = InitMsg {
            name: CONTRACT_NAME.to_string(),
            symbol: SYMBOL.to_string(),
            minter: MINTER.into(),
            base_cap: BASE_CAP,
            silver_cap: SILVER_CAP,
            gold_cap: GOLD_CAP,
        };
        let env = mock_env("creator", &[]);
        let res = init(deps, env, msg).unwrap();
        assert_eq!(0, res.messages.len());
    }

    #[test]
    fn proper_initialization() {
        let mut deps = mock_dependencies(20, &[]);

        let msg = InitMsg {
            name: CONTRACT_NAME.to_string(),
            symbol: SYMBOL.to_string(),
            minter: MINTER.into(),
            base_cap: BASE_CAP,
            silver_cap: SILVER_CAP,
            gold_cap: GOLD_CAP,
        };
        let env = mock_env("creator", &[]);

        // we can just call .unwrap() to assert this was a success
        let res = init(&mut deps, env, msg).unwrap();
        assert_eq!(0, res.messages.len());

        // it worked, let's query the state
        let res = query_minter(&deps).unwrap();
        assert_eq!(MINTER, res.minter.as_str());
        let info = query_contract_info(&deps).unwrap();
        assert_eq!(
            info,
            ContractInfo {
                name: CONTRACT_NAME.to_string(),
                symbol: SYMBOL.to_string(),
                base_cap: BASE_CAP,
                silver_cap: SILVER_CAP,
                gold_cap: GOLD_CAP,
            }
        );

        let count = query_base_tokens(&deps).unwrap();
        assert_eq!(0, count.count);

        // list the token_ids
        let tokens = query_all_base_tokens(&deps, None, None).unwrap();
        assert_eq!(0, tokens.tokens.len());
    }

    #[test]
    fn minting() {
        let mut deps = mock_dependencies(100, &[]);
        setup_contract(&mut deps);

        let token_id = "petrify".to_string();
        let token_id2 = "second".to_string();
        //let token_id3 = "cap".to_string();
        // let name = "Petrify with Gaze".to_string();
        // let description = "Allows the owner to petrify anyone looking at him or her".to_string();
        let rank = "base".to_string();

        let mint_msg = HandleMsg::Mint {
            //token_id: token_id.clone(),
            owner: "medusa".into(),
            // name: name.clone(),
            // description: Some(description.clone()),
            // image: None,
            rank: rank.clone(),
        };

        // random cannot mint
        let random = mock_env("random", &[]);
        let err = handle(&mut deps, random, mint_msg.clone()).unwrap_err();
        match err {
            StdError::Unauthorized { .. } => {}
            e => panic!("unexpected error: {}", e),
        }

        // minter can mint
        let allowed = mock_env(MINTER, &[]);
        let _ = handle(&mut deps, allowed, mint_msg.clone()).unwrap();

        // ensure num tokens increases
        let count = query_base_tokens(&deps).unwrap();
        assert_eq!(1, count.count);

        // unknown nft returns error
        let _ = query_nft_info(&deps, "unknown".to_string()).unwrap_err();

        // this nft info is correct
        let info = query_nft_info(&deps, token_id.clone()).unwrap();
        assert_eq!(
            info,
            NftInfoResponse {
                // name: name.clone(),
                // description: description.clone(),
                // image: None,
                rank: rank.clone(),
            }
        );

        // owner info is correct
        let owner = query_owner_of(&deps, token_id.clone()).unwrap();
        assert_eq!(
            owner,
            OwnerOfResponse {
                owner: "medusa".into(),
                approvals: vec![],
            }
        );

        // Cannot mint same token_id again
        let mint_msg2 = HandleMsg::Mint {
            //token_id: token_id.clone(),
            owner: "hercules".into(),
            // name: "copy cat".into(),
            // description: None,
            // image: None,
            rank: "base".into(),
        };

        let allowed = mock_env(MINTER, &[]);
        let err = handle(&mut deps, allowed, mint_msg2).unwrap_err();
        match err {
            StdError::GenericErr { msg, .. } => {
                assert_eq!(msg.as_str(), "token_id already claimed")
            }
            e => panic!("unexpected error: {}", e),
        }

        let mint_msg3 = HandleMsg::Mint {
            //token_id: token_id2.clone(),
            owner: "hercules".into(),
            // name: "below cap".into(),
            // description: None,
            // image: None,
            rank: "base".into(),
        };
        let allowed = mock_env(MINTER, &[]);
        let _ = handle(&mut deps, allowed, mint_msg3).unwrap();

        // Cannot mint more than the cap
        let mint_msg4 = HandleMsg::Mint {
            //token_id: token_id3.clone(),
            owner: "hercules".into(),
            // name: "over the cap".into(),
            // description: None,
            // image: None,
            rank: "base".into(),
        };
        let allowed = mock_env(MINTER, &[]);
        let err = handle(&mut deps, allowed, mint_msg4).unwrap_err();
        match err {
            StdError::GenericErr { msg, .. } => {
                assert_eq!(msg.as_str(), "Minting cannot exceed the cap")
            }
            e => panic!("unexpected error: {}", e),
        }

        // list the token_ids
        let tokens = query_all_base_tokens(&deps, None, None).unwrap();
        assert_eq!(2, tokens.tokens.len());
        assert_eq!(vec![token_id.clone(), token_id2.clone()], tokens.tokens);

        
        // list the number of tokens
        // burnt token must be deducted to the number of tokens
        let num_tokens = query_base_tokens(&deps).unwrap();
        assert_eq!(1, num_tokens.count);

        // burnt token is now owned by the burn address
        let owner = query_owner_of(&deps, token_id2.clone()).unwrap();
        assert_eq!(
            owner,
            OwnerOfResponse {
                owner: "terra1jrg2hv92xpjl4wwgd84jcm4cs2pfmzdxl6y2sx".into(),
                approvals: vec![],
            }
        );
    }

    #[test]
    fn update_nft() {
        let mut deps = mock_dependencies(20, &[]);
        setup_contract(&mut deps);

        // Mint a token
        let token_id = "melt".to_string();
        //let name = "Melting power".to_string();
        //let description = "Allows the owner to melt anyone looking at him or her".to_string();

        let mint_msg = HandleMsg::Mint {
            //token_id: token_id.clone(),
            owner: "venus".into(),
            // name: name.clone(),
            // description: Some(description.clone()),
            // image: None,
            rank: "base".into()
        };

        let minter = mock_env(MINTER, &[]);
        handle(&mut deps, minter, mint_msg).unwrap();

        // random cannot transfer
        let random = mock_env("random", &[]);
        let transfer_msg = HandleMsg::TransferNft {
            recipient: "random".into(),
            token_id: token_id.clone(),
        };

        let err = handle(&mut deps, random, transfer_msg.clone()).unwrap_err();

        match err {
            StdError::Unauthorized { .. } => {}
            e => panic!("unexpected error: {}", e),
        }

        // owner can transfer
        let random = mock_env("venus", &[]);
        let transfer_msg = HandleMsg::TransferNft {
            recipient: "random".into(),
            token_id: token_id.clone(),
        };

        let res = handle(&mut deps, random, transfer_msg.clone()).unwrap();

        assert_eq!(
            res,
            HandleResponse {
                messages: vec![],
                log: vec![
                    log("action", "transfer_nft"),
                    log("sender", "venus"),
                    log("recipient", "random"),
                    log("token_id", token_id.clone()),
                ],
                data: None,
            }
        );
    }

    #[test]
    fn sending_nft() {
        let mut deps = mock_dependencies(20, &[]);
        setup_contract(&mut deps);

        // Mint a token
        let token_id = "melt".to_string();
        //let name = "Melting power".to_string();
        //let description = "Allows the owner to melt anyone looking at him or her".to_string();

        let mint_msg = HandleMsg::Mint {
            //token_id: token_id.clone(),
            owner: "venus".into(),
            // name: name.clone(),
            // description: Some(description.clone()),
            // image: None,
            rank: "base".into()
        };

        let minter = mock_env(MINTER, &[]);
        handle(&mut deps, minter, mint_msg).unwrap();

        // random cannot send
        let inner_msg = WasmMsg::Execute {
            contract_addr: "another_contract".into(),
            msg: to_binary("You now have the melting power").unwrap(),
            send: vec![],
        };
        let msg: CosmosMsg = CosmosMsg::Wasm(inner_msg);

        let send_msg = HandleMsg::SendNft {
            contract: "another_contract".into(),
            token_id: token_id.clone(),
            msg: Some(to_binary(&msg).unwrap()),
        };

        let random = mock_env("random", &[]);
        let err = handle(&mut deps, random, send_msg.clone()).unwrap_err();
        match err {
            StdError::Unauthorized { .. } => {}
            e => panic!("unexpected error: {}", e),
        }

        // but owner can
        let random = mock_env("venus", &[]);
        let res = handle(&mut deps, random, send_msg).unwrap();
        assert_eq!(
            res,
            HandleResponse {
                messages: vec![msg],
                log: vec![
                    log("action", "send_nft"),
                    log("sender", "venus"),
                    log("recipient", "another_contract"),
                    log("token_id", token_id),
                ],
                data: None,
            }
        );
    }

    #[test]
    fn approving_revoking() {
        let mut deps = mock_dependencies(20, &[]);
        setup_contract(&mut deps);

        // Mint a token
        let token_id = "grow".to_string();
        //let name = "Growing power".to_string();
        //let description = "Allows the owner to grow anything".to_string();

        let mint_msg = HandleMsg::Mint {
            //token_id: token_id.clone(),
            owner: "demeter".into(),
            // name: name.clone(),
            // description: Some(description.clone()),
            // image: None,
            rank: "base".into()
        };

        let minter = mock_env(MINTER, &[]);
        handle(&mut deps, minter, mint_msg).unwrap();

        // Give random transferring power
        let approve_msg = HandleMsg::Approve {
            spender: "random".into(),
            token_id: token_id.clone(),
            expires: None,
        };
        let owner = mock_env("demeter", &[]);
        let res = handle(&mut deps, owner, approve_msg).unwrap();
        assert_eq!(
            res,
            HandleResponse {
                messages: vec![],
                log: vec![
                    log("action", "approve"),
                    log("sender", "demeter"),
                    log("spender", "random"),
                    log("token_id", token_id.clone()),
                ],
                data: None,
            }
        );

        // random can now transfer
        let random = mock_env("random", &[]);
        let transfer_msg = HandleMsg::TransferNft {
            recipient: "person".into(),
            token_id: token_id.clone(),
        };
        handle(&mut deps, random, transfer_msg).unwrap();

        // Approvals are removed / cleared
        let query_msg = QueryMsg::OwnerOf {
            token_id: token_id.clone(),
        };
        let res: OwnerOfResponse = from_binary(&query(&deps, query_msg.clone()).unwrap()).unwrap();
        assert_eq!(
            res,
            OwnerOfResponse {
                owner: "person".into(),
                approvals: vec![],
            }
        );

        // Approve, revoke, and check for empty, to test revoke
        let approve_msg = HandleMsg::Approve {
            spender: "random".into(),
            token_id: token_id.clone(),
            expires: None,
        };
        let owner = mock_env("person", &[]);
        handle(&mut deps, owner.clone(), approve_msg).unwrap();

        let revoke_msg = HandleMsg::Revoke {
            spender: "random".into(),
            token_id: token_id.clone(),
        };
        handle(&mut deps, owner, revoke_msg).unwrap();

        // Approvals are now removed / cleared
        let res: OwnerOfResponse = from_binary(&query(&deps, query_msg).unwrap()).unwrap();
        assert_eq!(
            res,
            OwnerOfResponse {
                owner: "person".into(),
                approvals: vec![],
            }
        );
    }

    #[test]
    fn approving_all_revoking_all() {
        let mut deps = mock_dependencies(20, &[]);
        setup_contract(&mut deps);

        // Mint a couple tokens (from the same owner)
        let token_id1 = "grow1".to_string();
        //let name1 = "Growing power".to_string();
        //let description1 = "Allows the owner the power to grow anything".to_string();
        let token_id2 = "grow2".to_string();
        //let name2 = "More growing power".to_string();
        //let description2 = "Allows the owner the power to grow anything even faster".to_string();

        let mint_msg1 = HandleMsg::Mint {
            //token_id: token_id1.clone(),
            owner: "demeter".into(),
            // name: name1.clone(),
            // description: Some(description1.clone()),
            // image: None,
            rank: "base".into()
        };

        let minter = mock_env(MINTER, &[]);
        handle(&mut deps, minter.clone(), mint_msg1).unwrap();

        let mint_msg2 = HandleMsg::Mint {
            //token_id: token_id2.clone(),
            owner: "demeter".into(),
            // name: name2.clone(),
            // description: Some(description2.clone()),
            // image: None,
            rank: "base".into()
        };

        handle(&mut deps, minter, mint_msg2).unwrap();

        // paginate the token_ids
        let tokens = query_all_base_tokens(&deps, None, Some(1)).unwrap();
        assert_eq!(1, tokens.tokens.len());
        assert_eq!(vec![token_id1.clone()], tokens.tokens);
        let tokens = query_all_base_tokens(&deps, Some(token_id1.clone()), Some(3)).unwrap();
        assert_eq!(1, tokens.tokens.len());
        assert_eq!(vec![token_id2.clone()], tokens.tokens);

        // demeter gives random full (operator) power over her tokens
        let approve_all_msg = HandleMsg::ApproveAll {
            operator: "random".into(),
            expires: None,
        };
        let owner = mock_env("demeter", &[]);
        let res = handle(&mut deps, owner, approve_all_msg).unwrap();
        assert_eq!(
            res,
            HandleResponse {
                messages: vec![],
                log: vec![
                    log("action", "approve_all"),
                    log("sender", "demeter"),
                    log("operator", "random"),
                ],
                data: None,
            }
        );

        // random can now transfer
        let random = mock_env("random", &[]);
        let transfer_msg = HandleMsg::TransferNft {
            recipient: "person".into(),
            token_id: token_id1.clone(),
        };
        handle(&mut deps, random.clone(), transfer_msg).unwrap();

        // random can now send
        let inner_msg = WasmMsg::Execute {
            contract_addr: "another_contract".into(),
            msg: to_binary("You now also have the growing power").unwrap(),
            send: vec![],
        };
        let msg: CosmosMsg = CosmosMsg::Wasm(inner_msg);

        let send_msg = HandleMsg::SendNft {
            contract: "another_contract".into(),
            token_id: token_id2.clone(),
            msg: Some(to_binary(&msg).unwrap()),
        };
        handle(&mut deps, random, send_msg).unwrap();

        // Approve_all, revoke_all, and check for empty, to test revoke_all
        let approve_all_msg = HandleMsg::ApproveAll {
            operator: "operator".into(),
            expires: None,
        };
        // person is now the owner of the tokens
        let owner = mock_env("person", &[]);
        handle(&mut deps, owner.clone(), approve_all_msg).unwrap();

        let res = query_all_approvals(&deps, "person".into(), None, None).unwrap();
        assert_eq!(
            res,
            ApprovedForAllResponse {
                operators: vec![cw721::Approval {
                    spender: "operator".into(),
                    expires: Expiration::Never {}
                }]
            }
        );

        // second approval
        let buddy_expires = Expiration::AtHeight(1234567);
        let approve_all_msg = HandleMsg::ApproveAll {
            operator: "buddy".into(),
            expires: Some(buddy_expires),
        };
        let owner = mock_env("person", &[]);
        handle(&mut deps, owner.clone(), approve_all_msg).unwrap();

        // and paginate queries
        let res = query_all_approvals(&deps, "person".into(), None, Some(1)).unwrap();
        assert_eq!(
            res,
            ApprovedForAllResponse {
                operators: vec![cw721::Approval {
                    spender: "buddy".into(),
                    expires: buddy_expires,
                }]
            }
        );
        let res =
            query_all_approvals(&deps, "person".into(), Some("buddy".into()), Some(2)).unwrap();
        assert_eq!(
            res,
            ApprovedForAllResponse {
                operators: vec![cw721::Approval {
                    spender: "operator".into(),
                    expires: Expiration::Never {}
                }]
            }
        );

        let revoke_all_msg = HandleMsg::RevokeAll {
            operator: "operator".into(),
        };
        handle(&mut deps, owner, revoke_all_msg).unwrap();

        // Approvals are removed / cleared without affecting others
        let res = query_all_approvals(&deps, "person".into(), None, None).unwrap();
        assert_eq!(
            res,
            ApprovedForAllResponse {
                operators: vec![cw721::Approval {
                    spender: "buddy".into(),
                    expires: buddy_expires,
                }]
            }
        );
    }
}