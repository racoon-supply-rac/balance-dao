use cosmwasm_schema::cw_serde;

#[cw_serde]
pub struct InstantiateMsg {
    pub accepted_denom: String,
    pub balance_development_fund_addr: String,
    pub juno_development_fund_addr: String,
    pub dev_addr: String,
    pub burn_permille_u64: u64,
    pub dev_fees_permille_u64: u64,
    pub balance_development_fund_permille_u64: u64,
    pub juno_development_fund_permille_u64: u64,
}

#[cw_serde]
pub enum ExecuteMsg {
    Swap {},
    EnableDisable {},
    Burn {},
}

#[cw_serde]
pub enum QueryMsg {
    GetConfig {},
    GetStats {},
    GetBurnedSnapshot {},
    GetToBurn {},
}

#[cw_serde]
pub struct MigrateMsg {}
