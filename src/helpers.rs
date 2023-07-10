use crate::error::ContractError;
use crate::state::Config;
use cosmwasm_schema::cw_serde;
use cosmwasm_std::{ensure, Decimal, MessageInfo, Uint128};

#[cw_serde]
pub struct AmountsToDistribute {
    pub burned: Uint128,
    pub balance_dev_fund: Uint128,
    pub juno_dev_fund: Uint128,
    pub dev: Uint128,
}

impl AmountsToDistribute {
    pub fn init() -> AmountsToDistribute {
        AmountsToDistribute {
            burned: Uint128::zero(),
            balance_dev_fund: Uint128::zero(),
            juno_dev_fund: Uint128::zero(),
            dev: Uint128::zero(),
        }
    }
    pub fn total_value(&self) -> Uint128 {
        self.dev + self.balance_dev_fund + self.juno_dev_fund + self.burned
    }
}

pub fn validate_coin_received(
    accepted_denom: &String,
    info: &MessageInfo,
) -> Result<(), ContractError> {
    ensure!(
        info.funds.len() == 1,
        ContractError::InvalidFundsReceived {}
    );
    ensure!(
        &info.funds[0].denom == accepted_denom,
        ContractError::InvalidFundsReceived {}
    );
    ensure!(
        info.funds[0].amount > Uint128::zero(),
        ContractError::InvalidFundsReceived {}
    );
    Ok(())
}

pub fn compute_amounts_to_distribute(
    config: &Config,
    amount_received: Uint128,
) -> Result<AmountsToDistribute, ContractError> {
    let mut amounts_to_send = AmountsToDistribute::init();

    amounts_to_send.burned = Decimal::permille(config.burn_permille_u64) * amount_received;
    amounts_to_send.dev = Decimal::permille(config.dev_fees_permille_u64) * amount_received;
    amounts_to_send.balance_dev_fund =
        Decimal::permille(config.balance_development_fund_permille_u64) * amount_received;
    amounts_to_send.juno_dev_fund =
        Decimal::permille(config.juno_development_fund_permille_u64) * amount_received;

    ensure!(
        amounts_to_send.total_value() == amount_received,
        ContractError::InvalidAmountsDistribution {}
    );

    Ok(amounts_to_send)
}
