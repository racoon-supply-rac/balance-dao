#![cfg(test)]
mod tests {
    use cosmwasm_std::{coin, Addr, Coin, Decimal, StdResult, Uint128};
    use cw_multi_test::{BankSudo, Contract, ContractWrapper, Executor, SudoMsg};
    use token_bindings::{TokenFactoryMsg, TokenFactoryQuery};
    use token_bindings_test::TokenFactoryApp;

    use crate::constants::{BALANCE_MAX_SUPPLY, JUNO_MAX_SUPPLY};
    use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
    use crate::state::{BurnedSnapshot, Config, Statistics};

    pub const ADMIN: &str = "juno1admin";
    pub const JUNO_DENOM: &str = "ujuno";
    pub const INVALID_DENOM: &str = "uinvalid";
    pub const WALLET1: &str = "juno1wallet1";
    pub const BAL_DEV_FUND: &str = "juno1balancefund";
    pub const JUNO_DEV_FUND: &str = "juno1junofund";
    pub const DEV: &str = "juno1dev";

    fn mock_app() -> TokenFactoryApp {
        TokenFactoryApp::default()
    }

    pub fn contract_box_def() -> Box<dyn Contract<TokenFactoryMsg, TokenFactoryQuery>> {
        let contract = ContractWrapper::new(
            crate::contract::execute,
            crate::contract::instantiate,
            crate::contract::query,
        )
        .with_migrate(crate::contract::migrate);
        Box::new(contract)
    }

    #[test]
    fn integration_tests() {
        let mut app = mock_app();

        // Send JUNO + INVALID denom to 2 wallets
        for i in vec![ADMIN, WALLET1].iter() {
            app.sudo(SudoMsg::Bank({
                BankSudo::Mint {
                    to_address: i.to_string(),
                    amount: vec![coin(200_000_000_000_000u128, JUNO_DENOM.clone())],
                }
            }))
            .unwrap();
            app.sudo(SudoMsg::Bank({
                BankSudo::Mint {
                    to_address: i.to_string(),
                    amount: vec![coin(200_000_000_000_000u128, INVALID_DENOM.clone())],
                }
            }))
            .unwrap();
        }

        // Init contract
        let contract_id = app.store_code(contract_box_def());
        let contract_addr = app
            .instantiate_contract(
                contract_id.clone(),
                Addr::unchecked(ADMIN),
                &InstantiateMsg {
                    accepted_denom: JUNO_DENOM.to_string(),
                    balance_development_fund_addr: BAL_DEV_FUND.to_string(),
                    juno_development_fund_addr: JUNO_DEV_FUND.to_string(),
                    dev_addr: DEV.to_string(),
                    burn_permille_u64: 780,
                    dev_fees_permille_u64: 20,
                    balance_development_fund_permille_u64: 100,
                    juno_development_fund_permille_u64: 100,
                },
                &[],
                "balance_swap",
                Some(ADMIN.to_string()),
            )
            .unwrap();

        // Swap invalid input denom
        let execute_outcome = app.execute_contract(
            Addr::unchecked(WALLET1),
            contract_addr.clone(),
            &ExecuteMsg::Swap {},
            &[Coin {
                denom: INVALID_DENOM.to_string(),
                amount: Uint128::new(1_000_000u128),
            }],
        );
        assert_eq!(
            execute_outcome.unwrap_err().source().unwrap().to_string(),
            "InvalidFundsReceived".to_string()
        );

        // Swap more than max supply
        let execute_outcome = app.execute_contract(
            Addr::unchecked(WALLET1),
            contract_addr.clone(),
            &ExecuteMsg::Swap {},
            &[Coin {
                denom: JUNO_DENOM.to_string(),
                amount: Uint128::new(190_000_000_000_000u128),
            }],
        );
        assert_eq!(
            execute_outcome.unwrap_err().source().unwrap().to_string(),
            "MaxSupplyReached".to_string()
        );

        // Try to enable disable without being admin
        let execute_outcome = app.execute_contract(
            Addr::unchecked(WALLET1),
            contract_addr.clone(),
            &ExecuteMsg::EnableDisable {},
            &[],
        );
        assert_eq!(
            execute_outcome.unwrap_err().source().unwrap().to_string(),
            "Unauthorized".to_string()
        );

        // Validate the stats
        let config_query: Statistics = app
            .wrap()
            .query_wasm_smart(contract_addr.clone(), &QueryMsg::GetStats {})
            .unwrap();
        assert_eq!(config_query.distributed, Uint128::zero());
        assert_eq!(config_query.received, Uint128::zero());
        assert_eq!(config_query.juno_dev_fund, Uint128::zero());
        assert_eq!(config_query.burned, Uint128::zero());
        assert_eq!(config_query.balance_dev_fund, Uint128::zero());
        assert_eq!(config_query.dev_fees, Uint128::zero());

        // Validate the config
        let config_query: Config = app
            .wrap()
            .query_wasm_smart(contract_addr.clone(), &QueryMsg::GetConfig {})
            .unwrap();
        assert_eq!(config_query.enabled, true);
        assert_eq!(config_query.accepted_denom, JUNO_DENOM);
        assert_eq!(config_query.juno_development_fund_permille_u64, 100);
        assert_eq!(config_query.balance_development_fund_permille_u64, 100);
        assert_eq!(config_query.burn_permille_u64, 780);
        assert_eq!(config_query.dev_fees_permille_u64, 20);
        assert_eq!(config_query.dev_addr.as_str(), DEV);
        assert_eq!(config_query.balance_development_fund_addr, BAL_DEV_FUND);
        assert_eq!(config_query.juno_development_fund_addr, JUNO_DEV_FUND);
        assert_eq!(config_query.contract_owner, ADMIN);
        assert_eq!(
            config_query.factory_denom,
            "factory/contract0/balance".to_string()
        );

        // EnableDisable
        let execute_outcome = app.execute_contract(
            Addr::unchecked(ADMIN),
            contract_addr.clone(),
            &ExecuteMsg::EnableDisable {},
            &[],
        );
        assert!(execute_outcome.is_ok());

        // Validate the config
        let config_query: Config = app
            .wrap()
            .query_wasm_smart(contract_addr.clone(), &QueryMsg::GetConfig {})
            .unwrap();
        assert_eq!(config_query.enabled, false);

        // Initial amount in every wallets
        let initial_dev: Coin = app
            .wrap()
            .query_balance(DEV, JUNO_DENOM.to_string())
            .unwrap();
        assert_eq!(initial_dev.amount, Uint128::zero());

        let initial_dev_fund: Coin = app
            .wrap()
            .query_balance(BAL_DEV_FUND, JUNO_DENOM.to_string())
            .unwrap();
        assert_eq!(initial_dev_fund.amount, Uint128::zero());

        let initial_vesting: Coin = app
            .wrap()
            .query_balance(JUNO_DEV_FUND, JUNO_DENOM.to_string())
            .unwrap();
        assert_eq!(initial_vesting.amount, Uint128::zero());

        let initial_wallet1_juno: Coin = app
            .wrap()
            .query_balance(WALLET1, JUNO_DENOM.to_string())
            .unwrap();
        assert_eq!(
            initial_wallet1_juno.amount,
            Uint128::new(200_000_000_000_000u128)
        );

        let initial_wallet1_balance: Coin = app
            .wrap()
            .query_balance(WALLET1, "factory/contract0/balance".to_string())
            .unwrap();
        assert_eq!(initial_wallet1_balance.amount, Uint128::zero());

        // Swap 100 $JUNO for $BALANCE
        let execute_outcome = app.execute_contract(
            Addr::unchecked(WALLET1),
            contract_addr.clone(),
            &ExecuteMsg::Swap {},
            &[Coin {
                denom: JUNO_DENOM.to_string(),
                amount: Uint128::new(100_000_000u128),
            }],
        );
        assert_eq!(
            execute_outcome.unwrap_err().source().unwrap().to_string(),
            "SwapDisabled".to_string()
        );

        // EnableDisable
        let execute_outcome = app.execute_contract(
            Addr::unchecked(ADMIN),
            contract_addr.clone(),
            &ExecuteMsg::EnableDisable {},
            &[],
        );
        assert!(execute_outcome.is_ok());

        // Swap 100 $JUNO for $BALANCE
        let execute_outcome = app.execute_contract(
            Addr::unchecked(WALLET1),
            contract_addr.clone(),
            &ExecuteMsg::Swap {},
            &[Coin {
                denom: JUNO_DENOM.to_string(),
                amount: Uint128::new(100_000_000u128),
            }],
        );
        assert!(execute_outcome.is_ok());

        // Final amount in every wallets
        let final_dev: Coin = app
            .wrap()
            .query_balance(DEV, JUNO_DENOM.to_string())
            .unwrap();
        // Should have 2%
        assert_eq!(final_dev.amount, Uint128::new(2_000_000u128));

        let final_dev_fund: Coin = app
            .wrap()
            .query_balance(BAL_DEV_FUND, JUNO_DENOM.to_string())
            .unwrap();
        // Should have 10%
        assert_eq!(final_dev_fund.amount, Uint128::new(10_000_000u128));

        let final_vesting: Coin = app
            .wrap()
            .query_balance(JUNO_DEV_FUND, JUNO_DENOM.to_string())
            .unwrap();
        // Should have 10%
        assert_eq!(final_vesting.amount, Uint128::new(10_000_000u128));

        let contract_final: Coin = app
            .wrap()
            .query_balance(contract_addr.to_string(), JUNO_DENOM.to_string())
            .unwrap();
        // Should have 78_000_000 because it will now be burned later
        assert_eq!(contract_final.amount, Uint128::new(78_000_000u128));

        let final_wallet1_juno: Coin = app
            .wrap()
            .query_balance(WALLET1, JUNO_DENOM.to_string())
            .unwrap();
        assert_eq!(
            final_wallet1_juno.amount,
            Uint128::new(200_000_000_000_000u128 - 100_000_000u128)
        );

        let final_wallet1_balance: Coin = app
            .wrap()
            .query_balance(WALLET1, "factory/contract0/balance".to_string())
            .unwrap();
        assert_eq!(final_wallet1_balance.amount, Uint128::new(11316955u128));

        // Validate the stats
        let config_query: Statistics = app
            .wrap()
            .query_wasm_smart(contract_addr.clone(), &QueryMsg::GetStats {})
            .unwrap();
        assert_eq!(config_query.distributed, Uint128::new(11_316_955u128));
        assert_eq!(config_query.received, Uint128::new(100_000_000u128));
        assert_eq!(config_query.juno_dev_fund, Uint128::new(10_000_000u128));
        assert_eq!(config_query.burned, Uint128::new(78_000_000u128));
        assert_eq!(config_query.balance_dev_fund, Uint128::new(10_000_000u128));
        assert_eq!(config_query.dev_fees, Uint128::new(2_000_000u128));

        // Validate the config
        let config_query: Config = app
            .wrap()
            .query_wasm_smart(contract_addr.clone(), &QueryMsg::GetConfig {})
            .unwrap();
        assert_eq!(config_query.enabled, true);
        assert_eq!(config_query.accepted_denom, JUNO_DENOM);
        assert_eq!(config_query.juno_development_fund_permille_u64, 100);
        assert_eq!(config_query.balance_development_fund_permille_u64, 100);
        assert_eq!(config_query.burn_permille_u64, 780);
        assert_eq!(config_query.dev_fees_permille_u64, 20);
        assert_eq!(config_query.dev_addr.as_str(), DEV);
        assert_eq!(config_query.balance_development_fund_addr, BAL_DEV_FUND);
        assert_eq!(config_query.juno_development_fund_addr, JUNO_DEV_FUND);
        assert_eq!(config_query.contract_owner, ADMIN);
        assert_eq!(
            config_query.factory_denom,
            "factory/contract0/balance".to_string()
        );

        // Make several buy to check the flow
        let mut total_bought = Uint128::zero();
        for i in 33..55 {
            total_bought += Uint128::new(i * 100_000_000u128);
            let execute_outcome = app.execute_contract(
                Addr::unchecked(WALLET1),
                contract_addr.clone(),
                &ExecuteMsg::Swap {},
                &[Coin {
                    denom: JUNO_DENOM.to_string(),
                    amount: Uint128::new(i * 100_000_000u128),
                }],
            );
            assert!(execute_outcome.is_ok());
        }

        // Validate the stats
        let config_query: Statistics = app
            .wrap()
            .query_wasm_smart(contract_addr.clone(), &QueryMsg::GetStats {})
            .unwrap();
        // Check bound - rounding
        assert!(
            config_query.distributed
                <= Uint128::new(11_316_955u128)
                    + (total_bought * Decimal::from_ratio(BALANCE_MAX_SUPPLY, JUNO_MAX_SUPPLY))
        );
        assert!(
            config_query.distributed
                > Uint128::new(11_316_900u128)
                    + (total_bought * Decimal::from_ratio(BALANCE_MAX_SUPPLY, JUNO_MAX_SUPPLY))
        );
        // 33 + ... + 54 = 957
        assert_eq!(
            config_query.received,
            Uint128::new(100_000_000u128) + Uint128::new(95_700_000_000u128)
        );
        assert_eq!(
            config_query.juno_dev_fund,
            Uint128::new(10_000_000u128) + Uint128::new(9_570_000_000u128)
        );
        assert_eq!(
            config_query.burned,
            Uint128::new(78_000_000u128) + Uint128::new(74_646_000_000u128)
        );
        assert_eq!(
            config_query.juno_dev_fund,
            Uint128::new(10_000_000u128) + Uint128::new(9_570_000_000u128)
        );
        assert_eq!(
            config_query.dev_fees,
            Uint128::new(2_000_000u128) + Uint128::new(1_914_000_000u128)
        );

        // Contract migration
        // Check if the current state are as expected
        // Snapshot should be at 0 as no migration happened yet
        let snapshot_query: StdResult<BurnedSnapshot> = app
            .wrap()
            .query_wasm_smart(contract_addr.clone(), &QueryMsg::GetBurnedSnapshot {});
        // Does not exist
        assert!(snapshot_query.is_err());

        // The amount to be burned so far should be at 78_000_000 + 74_646_000_000 but it will be 0 after the migration
        let to_burn_query: Coin = app
            .wrap()
            .query_wasm_smart(contract_addr.clone(), &QueryMsg::GetToBurn {})
            .unwrap();
        assert_eq!(
            to_burn_query.amount,
            Uint128::new(78_000_000u128) + Uint128::new(74_646_000_000u128)
        );
        assert_eq!(to_burn_query.denom, JUNO_DENOM);

        // Same Code ID (migration was already there)
        let migrate_outcome = app.migrate_contract(
            Addr::unchecked(ADMIN),
            contract_addr.clone(),
            &crate::msg::MigrateMsg { states_update: true },
            contract_id,
        );
        assert!(migrate_outcome.is_ok());
        let migration_time = app.block_info().time;

        // After the migration
        let snapshot_query: BurnedSnapshot = app
            .wrap()
            .query_wasm_smart(contract_addr.clone(), &QueryMsg::GetBurnedSnapshot {})
            .unwrap();
        assert_eq!(snapshot_query.snapshot_time, migration_time);
        assert_eq!(
            snapshot_query.amount,
            Uint128::new(78_000_000u128) + Uint128::new(74_646_000_000u128)
        );
        assert_eq!(snapshot_query.denom, JUNO_DENOM);

        // Should now be 0 as this is what will be burned later
        let to_burn_query: Coin = app
            .wrap()
            .query_wasm_smart(contract_addr.clone(), &QueryMsg::GetToBurn {})
            .unwrap();
        assert_eq!(to_burn_query.amount, Uint128::zero());
        assert_eq!(to_burn_query.denom, JUNO_DENOM);

        // We now make a swap and check the states
        let execute_outcome = app.execute_contract(
            Addr::unchecked(WALLET1),
            contract_addr.clone(),
            &ExecuteMsg::Swap {},
            &[Coin {
                denom: JUNO_DENOM.to_string(),
                amount: Uint128::new(100_000_000u128),
            }],
        );
        app.next_block();
        assert!(execute_outcome.is_ok());

        // Should be no change
        let snapshot_query: BurnedSnapshot = app
            .wrap()
            .query_wasm_smart(contract_addr.clone(), &QueryMsg::GetBurnedSnapshot {})
            .unwrap();
        assert_eq!(snapshot_query.snapshot_time, migration_time);
        assert_eq!(
            snapshot_query.amount,
            Uint128::new(78_000_000u128) + Uint128::new(74_646_000_000u128)
        );
        assert_eq!(snapshot_query.denom, JUNO_DENOM);

        // Should now be at 78_000_000 as this is what will be burned later
        let to_burn_query: Coin = app
            .wrap()
            .query_wasm_smart(contract_addr.clone(), &QueryMsg::GetToBurn {})
            .unwrap();
        assert_eq!(to_burn_query.amount, Uint128::new(78_000_000u128));
        assert_eq!(to_burn_query.denom, JUNO_DENOM);

        // But the stats should remain intact
        let stats_query: Statistics = app
            .wrap()
            .query_wasm_smart(contract_addr.clone(), &QueryMsg::GetStats {})
            .unwrap();
        // Check bound - rounding
        assert!(
            stats_query.distributed
                <= Uint128::new(11_316_955u128)
                    + ((total_bought + Uint128::new(100_000_000u128))
                        * Decimal::from_ratio(BALANCE_MAX_SUPPLY, JUNO_MAX_SUPPLY))
        );
        assert!(
            stats_query.distributed
                > Uint128::new(11_316_900u128)
                    + ((total_bought + Uint128::new(100_000_000u128))
                        * Decimal::from_ratio(BALANCE_MAX_SUPPLY, JUNO_MAX_SUPPLY))
        );
        // 33 + ... + 54 = 957
        assert_eq!(
            stats_query.received,
            Uint128::new(100_000_000u128)
                + Uint128::new(95_700_000_000u128)
                + Uint128::new(100_000_000u128)
        );
        assert_eq!(
            stats_query.juno_dev_fund,
            Uint128::new(10_000_000u128)
                + Uint128::new(9_570_000_000u128)
                + Uint128::new(10_000_000u128)
        );
        assert_eq!(
            stats_query.burned,
            Uint128::new(78_000_000u128)
                + Uint128::new(74_646_000_000u128)
                + Uint128::new(78_000_000u128)
        );
        assert_eq!(
            stats_query.juno_dev_fund,
            Uint128::new(10_000_000u128)
                + Uint128::new(9_570_000_000u128)
                + Uint128::new(10_000_000u128)
        );
        assert_eq!(
            stats_query.dev_fees,
            Uint128::new(2_000_000u128)
                + Uint128::new(1_914_000_000u128)
                + Uint128::new(2_000_000u128)
        );

        // Try to burn - should not work
        let execute_outcome = app.execute_contract(
            Addr::unchecked(WALLET1),
            contract_addr.clone(),
            &ExecuteMsg::Burn {},
            &[],
        );
        assert!(execute_outcome.is_err());

        // Should now have more in the contract
        let final_contract: Coin = app
            .wrap()
            .query_balance(contract_addr, JUNO_DENOM.to_string())
            .unwrap();
        assert_eq!(final_contract.amount, Uint128::new(78_000_000u128 + 74_646_000_000u128 + 78_000_000u128));
    }
}
