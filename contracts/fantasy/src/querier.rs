use cosmwasm_bignumber::{Decimal256};
use cosmwasm_std::{
    Api, Coin, Extern, Querier, StdResult, Storage, Uint128,
};
use terra_cosmwasm::TerraQuerier;

pub fn compute_tax<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    coin: &Coin,
) -> StdResult<Uint128> {
    let terra_querier = TerraQuerier::new(&deps.querier);
    let tax_rate = Decimal256::from((terra_querier.query_tax_rate()?).rate);
    let tax_cap = Uint128::from((terra_querier.query_tax_cap(coin.denom.to_string())?).cap);
    let amount = Uint128::from(coin.amount);
    Ok(std::cmp::min(
        amount * (Decimal256::one() - Decimal256::one() / (Decimal256::one() + tax_rate)).into(),
        tax_cap,
    ))
}

pub fn compute_price_with_tax<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    coin: &Coin,
) -> StdResult<Uint128> {
    let terra_querier = TerraQuerier::new(&deps.querier);
    let tax_rate = Decimal256::from((terra_querier.query_tax_rate()?).rate);
    let tax_cap = Uint128::from((terra_querier.query_tax_cap(coin.denom.to_string())?).cap);
    let amount = Uint128::from(coin.amount);
    Ok(std::cmp::min(
        amount * (Decimal256::one() + tax_rate).into(),
        amount + tax_cap,
    ))
}

pub fn deduct_tax<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    coin: Coin,
) -> StdResult<Coin> {
    let tax_amount = compute_tax(deps, &coin).unwrap_or(Uint128::zero());
    Ok(Coin {
        denom: coin.denom,
        amount: (Uint128::from(coin.amount) - tax_amount)?,
    })
}

