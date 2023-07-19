use cosmwasm_schema::cw_serde;
use cosmwasm_std::{ensure, Coin, DepsMut, Timestamp, Uint128};
use cw_storage_plus::Item;
use token_bindings::TokenFactoryQuery;

use crate::error::ContractError;
use crate::helpers::{compute_amounts_to_distribute, AmountsToDistribute};
use crate::msg::InstantiateMsg;

#[cw_serde]
pub struct Config {
    pub contract_owner: String,
    pub enabled: bool,
    pub accepted_denom: String,
    pub factory_denom: String,
    pub balance_development_fund_addr: String,
    pub juno_development_fund_addr: String,
    pub dev_addr: String,
    pub burn_permille_u64: u64,
    pub dev_fees_permille_u64: u64,
    pub balance_development_fund_permille_u64: u64,
    pub juno_development_fund_permille_u64: u64,
}

impl Config {
    pub fn validate(
        sender: &str,
        deps: DepsMut<TokenFactoryQuery>,
        mut init_msg: InstantiateMsg,
        factory_denom: String,
    ) -> Result<Self, ContractError> {
        let total_dist = init_msg.burn_permille_u64
            + init_msg.dev_fees_permille_u64
            + init_msg.balance_development_fund_permille_u64
            + init_msg.juno_development_fund_permille_u64;

        ensure!(total_dist == 1_000u64, ContractError::InvalidInitMsg {});

        let validate_range = |value: u64| -> Result<(), ContractError> {
            match value {
                0..=1_000 => Ok(()),
                _ => Err(ContractError::InvalidInitMsg {}),
            }
        };

        validate_range(init_msg.burn_permille_u64)?;
        validate_range(init_msg.dev_fees_permille_u64)?;
        validate_range(init_msg.juno_development_fund_permille_u64)?;
        validate_range(init_msg.balance_development_fund_permille_u64)?;

        init_msg.balance_development_fund_addr = deps
            .api
            .addr_validate(&init_msg.balance_development_fund_addr)?
            .to_string();
        init_msg.juno_development_fund_addr = deps
            .api
            .addr_validate(&init_msg.juno_development_fund_addr)?
            .to_string();
        init_msg.dev_addr = deps.api.addr_validate(&init_msg.dev_addr)?.to_string();

        let config = Config {
            contract_owner: deps.api.addr_validate(sender)?.to_string(),
            enabled: true,
            accepted_denom: init_msg.accepted_denom,
            factory_denom,
            balance_development_fund_addr: init_msg.balance_development_fund_addr,
            juno_development_fund_addr: init_msg.juno_development_fund_addr,
            dev_addr: init_msg.dev_addr,
            burn_permille_u64: init_msg.burn_permille_u64,
            dev_fees_permille_u64: init_msg.dev_fees_permille_u64,
            balance_development_fund_permille_u64: init_msg.balance_development_fund_permille_u64,
            juno_development_fund_permille_u64: init_msg.juno_development_fund_permille_u64,
        };

        // Checks if the distribution works based on the given config
        compute_amounts_to_distribute(&config, Uint128::new(1_000_000u128))?;

        Ok(config)
    }
}

pub const CONFIG: Item<Config> = Item::new("config");

#[cw_serde]
pub struct Statistics {
    pub received: Uint128,
    pub burned: Uint128,
    pub distributed: Uint128,
    pub dev_fees: Uint128,
    pub balance_dev_fund: Uint128,
    pub juno_dev_fund: Uint128,
}

impl Statistics {
    pub fn zero() -> Self {
        Statistics {
            received: Uint128::zero(),
            burned: Uint128::zero(),
            distributed: Uint128::zero(),
            dev_fees: Uint128::zero(),
            balance_dev_fund: Uint128::zero(),
            juno_dev_fund: Uint128::zero(),
        }
    }

    pub fn add(
        &mut self,
        amount_to_distribute: &AmountsToDistribute,
        swap_amount_out: Uint128,
        swap_amount_in: Uint128,
    ) -> &mut Statistics {
        self.burned += amount_to_distribute.burned;
        self.received += swap_amount_in;
        self.distributed += swap_amount_out;
        self.dev_fees += amount_to_distribute.dev;
        self.balance_dev_fund += amount_to_distribute.balance_dev_fund;
        self.juno_dev_fund += amount_to_distribute.juno_dev_fund;
        self
    }
}

pub const STATS: Item<Statistics> = Item::new("stats");

// Only used as a point in time which was using a dead address to burn
#[cw_serde]
pub struct BurnedSnapshot {
    pub denom: String,
    pub amount: Uint128,
    pub snapshot_time: Timestamp,
}
pub const BURNED_REMINTED_SNAPSHOT: Item<BurnedSnapshot> = Item::new("burned_reminted_snapshot");

pub const TO_BURN: Item<Coin> = Item::new("to_burn");
