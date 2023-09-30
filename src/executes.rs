use cosmwasm_std::{ensure, Coin, Decimal, DepsMut, Env, MessageInfo, Response};
use token_bindings::{TokenFactoryMsg, TokenFactoryQuery};

use crate::constants::{BALANCE_MAX_SUPPLY, JUNO_MAX_SUPPLY};
use crate::error::ContractError;
use crate::helpers::{compute_amounts_to_distribute, validate_coin_received};
use crate::state::{CONFIG, STATS, TO_BURN};

pub fn swap(
    deps: DepsMut<TokenFactoryQuery>,
    _env: Env,
    info: MessageInfo,
) -> Result<Response<TokenFactoryMsg>, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    let mut stats = STATS.load(deps.storage)?;

    validate_coin_received(&config.accepted_denom, &info)?;

    ensure!(
        stats.received + info.funds[0].amount <= JUNO_MAX_SUPPLY,
        ContractError::MaxSupplyReceivedReached {}
    );

    let mut response = Response::new();

    let amounts_to_distribute = compute_amounts_to_distribute(&config, info.funds[0].amount)?;

    // Send Balance to the sender
    let token_amount_to_send =
        info.funds[0].amount * Decimal::from_ratio(BALANCE_MAX_SUPPLY, JUNO_MAX_SUPPLY);
    ensure!(
        stats.distributed + token_amount_to_send <= BALANCE_MAX_SUPPLY,
        ContractError::MaxSupplyReached {}
    );

    // Update statistics
    let stats = stats.add(
        &amounts_to_distribute,
        token_amount_to_send,
        info.funds[0].amount,
    );
    STATS.save(deps.storage, stats)?;

    // Burn: currently keeps the Juno in the contract which will be burned with `Burn` msg later
    // TODO: To be changed when the new burn module is live on Juno
    TO_BURN.update(
        deps.storage,
        |mut updated_to_burn| -> Result<_, ContractError> {
            updated_to_burn.amount += amounts_to_distribute.burned;
            Ok(updated_to_burn)
        },
    )?;

    // Dev funds
    response = response.add_message(cosmwasm_std::BankMsg::Send {
        to_address: config.balance_development_fund_addr.clone(),
        amount: vec![Coin {
            denom: config.accepted_denom.clone(),
            amount: amounts_to_distribute.balance_dev_fund,
        }],
    });

    // Vesting
    response = response.add_message(cosmwasm_std::BankMsg::Send {
        to_address: config.juno_development_fund_addr.clone(),
        amount: vec![Coin {
            denom: config.accepted_denom.clone(),
            amount: amounts_to_distribute.juno_dev_fund,
        }],
    });

    // Dev
    response = response.add_message(cosmwasm_std::BankMsg::Send {
        to_address: config.dev_addr.clone(),
        amount: vec![Coin {
            denom: config.accepted_denom.clone(),
            amount: amounts_to_distribute.dev,
        }],
    });

    ensure!(
        token_amount_to_send + stats.distributed <= BALANCE_MAX_SUPPLY,
        ContractError::MaxSupplyReached {}
    );
    let mint_tokens_msg = TokenFactoryMsg::mint_contract_tokens(
        config.factory_denom,
        token_amount_to_send,
        info.sender.to_string(),
    );

    response = response.add_message(mint_tokens_msg);

    Ok(response)
}
