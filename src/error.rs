use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("NotImplementedYet")]
    NotImplementedYet {},

    #[error("SwapDisabled")]
    SwapDisabled {},

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("InvalidInitMsg")]
    InvalidInitMsg {},

    #[error("InvalidFundsReceived")]
    InvalidFundsReceived {},

    #[error("InvalidAmountsDistribution")]
    InvalidAmountsDistribution {},

    #[error("MaxSupplyReached")]
    MaxSupplyReceivedReached {},

    #[error("MaxSupplyReached")]
    MaxSupplyReached {},
}
