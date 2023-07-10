use cosmwasm_std::{
    ensure, entry_point, to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult,
};
use cw2::set_contract_version;
use token_bindings::{TokenFactoryMsg, TokenFactoryQuery, TokenMsg};

use crate::error::ContractError;
use crate::executes::swap;
use crate::msg::{ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg};
use crate::state::{Config, Statistics, CONFIG, STATS};

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

    let config = &Config::validate(
        info.sender.as_ref(),
        deps.branch(),
        init_msg,
        format!("factory/{}/{}", env.contract.address, "balance"),
    )?;

    CONFIG.save(deps.storage, config)?;

    STATS.save(deps.storage, &Statistics::zero())?;

    Ok(Response::new().add_message(TokenMsg::CreateDenom {
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
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps<TokenFactoryQuery>, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetConfig {} => to_binary(&CONFIG.load(deps.storage)?),
        QueryMsg::GetStats {} => to_binary(&STATS.load(deps.storage)?),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(
    _deps: DepsMut<TokenFactoryQuery>,
    _env: Env,
    _msg: MigrateMsg,
) -> Result<Response<TokenFactoryMsg>, ContractError> {
    Ok(Response::default())
}
