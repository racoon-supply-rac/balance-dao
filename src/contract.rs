use cosmwasm_std::{
    ensure, entry_point, to_binary, Binary, Coin, Deps, DepsMut, Env, MessageInfo, Response,
    StdResult, Uint128,
};
use cw2::set_contract_version;
use token_bindings::{TokenFactoryMsg, TokenFactoryQuery};

use crate::error::ContractError;
use crate::executes::swap;
use crate::msg::{ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg};
use crate::state::{
    BurnedSnapshot, Config, Statistics, BURNED_REMINTED_SNAPSHOT, CONFIG, STATS, TO_BURN,
};

pub const CONTRACT_NAME: &str = "crates.io:balance-token-swap";
pub const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    mut deps: DepsMut<TokenFactoryQuery>,
    env: Env,
    info: MessageInfo,
    init_msg: InstantiateMsg,
) -> Result<Response<TokenFactoryMsg>, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    let config = Config::validate(
        info.sender.as_ref(),
        deps.branch(),
        init_msg,
        format!("factory/{}/{}", env.contract.address, "balance"),
    )?;

    CONFIG.save(deps.storage, &config)?;

    STATS.save(deps.storage, &Statistics::zero())?;

    TO_BURN.save(
        deps.storage,
        &Coin {
            denom: config.accepted_denom,
            amount: Uint128::zero(),
        },
    )?;

    Ok(Response::new().add_message(TokenFactoryMsg::CreateDenom {
        subdenom: "balance".to_string(),
        metadata: None, // exponent 6
    }))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut<TokenFactoryQuery>,
    _env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response<TokenFactoryMsg>, ContractError> {
    match msg {
        ExecuteMsg::Swap {} => {
            let config = CONFIG.load(deps.storage)?;
            ensure!(config.enabled, ContractError::SwapDisabled {});
            swap(deps, _env, info)
        }
        ExecuteMsg::EnableDisable {} => {
            // Alts swapping
            let mut config = CONFIG.load(deps.storage)?;
            ensure!(
                info.sender.as_str() == config.contract_owner,
                ContractError::Unauthorized {}
            );
            config.enabled = !config.enabled;
            CONFIG.save(deps.storage, &config)?;
            Ok(Response::default())
        }
        // Will be changed when the new burn module is live
        ExecuteMsg::Burn {} => Err(ContractError::NotImplementedYet {}),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps<TokenFactoryQuery>, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetConfig {} => to_binary(&CONFIG.load(deps.storage)?),
        QueryMsg::GetStats {} => to_binary(&STATS.load(deps.storage)?),
        QueryMsg::GetBurnedSnapshot {} => to_binary(&BURNED_REMINTED_SNAPSHOT.load(deps.storage)?),
        QueryMsg::GetToBurn {} => to_binary(&TO_BURN.load(deps.storage)?),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(
    deps: DepsMut<TokenFactoryQuery>,
    env: Env,
    _msg: MigrateMsg,
) -> Result<Response<TokenFactoryMsg>, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    let stats = STATS.load(deps.storage)?;

    // Starts from 0 for the new burn module
    TO_BURN.save(
        deps.storage,
        &Coin {
            denom: config.accepted_denom.clone(),
            amount: Uint128::zero(),
        },
    )?;

    // This was burned but will be reminted - need to check if Juno can burn it when the new module
    // happens
    BURNED_REMINTED_SNAPSHOT.save(
        deps.storage,
        &BurnedSnapshot {
            denom: config.accepted_denom,
            amount: stats.burned,
            snapshot_time: env.block.time,
        },
    )?;

    Ok(Response::default())
}
