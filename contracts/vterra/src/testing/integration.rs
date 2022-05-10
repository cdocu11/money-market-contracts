use cosmwasm_bignumber::Decimal256;
use cosmwasm_std::testing::{mock_env, MockApi, MockStorage};
use cosmwasm_std::{coins, to_binary, Addr};
use cw20::{BalanceResponse, Cw20ExecuteMsg, Cw20QueryMsg};
use moneymarket::market::{
    Cw20HookMsg as MarketCw20HookMsg, ExecuteMsg as MarketExecuteMsg,
    MigrateMsg as MarketMigrateMsg,
};
use moneymarket::overseer::{
    ConfigResponse as OverseerConfigResponse, ExecuteMsg as OverserrExecuteMsg,
    MigrateMsg as OverseerMigrateMsg, QueryMsg as OverseerQueryMsg,
};
use moneymarket::vterra::{
    ConfigResponse as VterraConfigResponse, InstantiateMsg as VterraInstantiateMsg,
    QueryMsg as VterraQueryMsg,
};
use moneymarket_old::distribution_model::InstantiateMsg as DistributionModelInstantiateMsgOld;
use moneymarket_old::interest_model::InstantiateMsg as InterestModelInstantiateMsgOld;
use moneymarket_old::market::{
    ConfigResponse as MarketConfigResponseOld, ExecuteMsg as MarketExecuteMsgOld,
    InstantiateMsg as MarketInstantiateMsgOld, QueryMsg as MarketQueryMsgOld,
};
use moneymarket_old::oracle::InstantiateMsg as OracleInstantiateMsgOld;
use moneymarket_old::overseer::InstantiateMsg as OverseerInstantiateMsgOld;
use std::str::FromStr;
use terra_multi_test::{AppBuilder, BankKeeper, ContractWrapper, Executor, TerraApp, TerraMock};

const ADMIN: &str = "admin";
const OWNER: &str = "owner";
const USER: &str = "user";

#[allow(dead_code)]
struct Addresses {
    market_addr: Option<Addr>,
    overseer_addr: Option<Addr>,
    vterra_addr: Option<Addr>,
    aterra_cw20_addr: Option<Addr>,
    vterra_cw20_addr: Option<Addr>,
    oracle_addr: Option<Addr>,
    interest_model_addr: Option<Addr>,
    distribution_model_addr: Option<Addr>,
    liquidation_addr: Option<Addr>,
    collector_addr: Option<Addr>,
    distributor_addr: Option<Addr>,
}

fn mock_app() -> TerraApp {
    let env = mock_env();
    let api = MockApi::default();
    let bank = BankKeeper::new();
    let storage = MockStorage::new();
    let custom = TerraMock::luna_ust_case();

    AppBuilder::new()
        .with_api(api)
        .with_block(env.block)
        .with_bank(bank)
        .with_storage(storage)
        .with_custom(custom)
        .build()
}

fn store_token_contract_code(app: &mut TerraApp) -> u64 {
    let token_contracct = Box::new(ContractWrapper::new_with_empty(
        cw20_base::contract::execute,
        cw20_base::contract::instantiate,
        cw20_base::contract::query,
    ));

    app.store_code(token_contracct)
}

fn store_old_market_contract_code(app: &mut TerraApp) -> u64 {
    let market_contract = Box::new(
        ContractWrapper::new_with_empty(
            moneymarket_market_old::contract::execute,
            moneymarket_market_old::contract::instantiate,
            moneymarket_market_old::contract::query,
        )
        .with_reply_empty(moneymarket_market_old::contract::reply),
    );

    app.store_code(market_contract)
}

fn store_old_overseer_contract_code(app: &mut TerraApp) -> u64 {
    let overseer_contract = Box::new(ContractWrapper::new_with_empty(
        moneymarket_overseer_old::contract::execute,
        moneymarket_overseer_old::contract::instantiate,
        moneymarket_overseer_old::contract::query,
    ));

    app.store_code(overseer_contract)
}

fn store_old_oracle_contract_code(app: &mut TerraApp) -> u64 {
    let oracle_contract = Box::new(ContractWrapper::new_with_empty(
        moneymarket_oracle_old::contract::execute,
        moneymarket_oracle_old::contract::instantiate,
        moneymarket_oracle_old::contract::query,
    ));

    app.store_code(oracle_contract)
}

fn store_old_interest_model_code(app: &mut TerraApp) -> u64 {
    let interest_model_contract = Box::new(ContractWrapper::new_with_empty(
        moneymarket_interest_model_old::contract::execute,
        moneymarket_interest_model_old::contract::instantiate,
        moneymarket_interest_model_old::contract::query,
    ));

    app.store_code(interest_model_contract)
}

fn store_old_distribution_model_code(app: &mut TerraApp) -> u64 {
    let distribution_model_contract = Box::new(ContractWrapper::new_with_empty(
        moneymarket_distribution_model_old::contract::execute,
        moneymarket_distribution_model_old::contract::instantiate,
        moneymarket_distribution_model_old::contract::query,
    ));

    app.store_code(distribution_model_contract)
}

fn store_vterra_contract_code(app: &mut TerraApp) -> u64 {
    let vterra_contract = Box::new(
        ContractWrapper::new_with_empty(
            vterra::contract::execute,
            vterra::contract::instantiate,
            vterra::contract::query,
        )
        .with_reply_empty(vterra::contract::reply),
    );

    app.store_code(vterra_contract)
}

fn store_market_contract_code(app: &mut TerraApp) -> u64 {
    let market_contract = Box::new(
        ContractWrapper::new_with_empty(
            moneymarket_market::contract::execute,
            moneymarket_market::contract::instantiate,
            moneymarket_market::contract::query,
        )
        .with_reply_empty(moneymarket_market::contract::reply)
        .with_migrate_empty(moneymarket_market::contract::migrate),
    );

    app.store_code(market_contract)
}

fn store_overseer_contract_code(app: &mut TerraApp) -> u64 {
    let overseer_contract = Box::new(
        ContractWrapper::new_with_empty(
            moneymarket_overseer::contract::execute,
            moneymarket_overseer::contract::instantiate,
            moneymarket_overseer::contract::query,
        )
        .with_migrate_empty(moneymarket_overseer::contract::migrate),
    );

    app.store_code(overseer_contract)
}

fn create_old_contracts(app: &mut TerraApp) -> Addresses {
    let owner = Addr::unchecked(OWNER);
    let admin = Addr::unchecked(ADMIN);

    app.init_bank_balance(&owner, coins(1000000, "uust"))
        .unwrap();

    // these 3 contracts are not needed for now
    let liquidation_addr = Addr::unchecked("liquidation_addr");
    let collector_addr = Addr::unchecked("collector_addr");
    let distributor_addr = Addr::unchecked("distributor_addr");

    // store contract codes
    let token_code_id = store_token_contract_code(app);
    let oracle_code_id = store_old_oracle_contract_code(app);
    let interest_model_code_id = store_old_interest_model_code(app);
    let distribution_model_code_id = store_old_distribution_model_code(app);
    let market_code_id = store_old_market_contract_code(app);
    let overseer_code_id = store_old_overseer_contract_code(app);

    // instantiate oracle contract
    let msg = OracleInstantiateMsgOld {
        owner: owner.to_string(),
        base_asset: "uust".to_string(),
    };
    let oracle_addr = app
        .instantiate_contract(
            oracle_code_id,
            owner.clone(),
            &msg,
            &[],
            String::from("ORACLE"),
            Some(admin.to_string()),
        )
        .unwrap();

    // instantiate interest model contract
    let msg = InterestModelInstantiateMsgOld {
        owner: owner.to_string(),
        base_rate: Decimal256::percent(10),
        interest_multiplier: Decimal256::percent(10),
    };
    let interest_model_addr = app
        .instantiate_contract(
            interest_model_code_id,
            owner.clone(),
            &msg,
            &[],
            String::from("INTEREST MODEL"),
            Some(admin.to_string()),
        )
        .unwrap();

    // instantiate distribution model contract
    let msg = DistributionModelInstantiateMsgOld {
        owner: owner.to_string(),
        emission_cap: Decimal256::from_uint256(100u64),
        emission_floor: Decimal256::from_uint256(10u64),
        increment_multiplier: Decimal256::percent(110),
        decrement_multiplier: Decimal256::percent(90),
    };
    let distribution_model_addr = app
        .instantiate_contract(
            distribution_model_code_id,
            owner.clone(),
            &msg,
            &[],
            String::from("INTEREST MODEL"),
            Some(admin.to_string()),
        )
        .unwrap();

    // instantitate market contract
    let msg = MarketInstantiateMsgOld {
        owner_addr: owner.to_string(),
        stable_denom: "uust".to_string(),
        aterra_code_id: token_code_id,
        anc_emission_rate: Decimal256::one(),
        max_borrow_factor: Decimal256::one(),
    };
    let market_addr = app
        .instantiate_contract(
            market_code_id,
            owner.clone(),
            &msg,
            &coins(1000000, "uust"),
            String::from("MARKET"),
            Some(admin.to_string()),
        )
        .unwrap();

    // instantiate overseer contract
    let msg = OverseerInstantiateMsgOld {
        owner_addr: owner.to_string(),
        oracle_contract: oracle_addr.to_string(),
        market_contract: market_addr.to_string(),
        liquidation_contract: liquidation_addr.to_string(),
        collector_contract: collector_addr.to_string(),
        stable_denom: "uusd".to_string(),
        epoch_period: 86400u64,
        threshold_deposit_rate: Decimal256::permille(3),
        target_deposit_rate: Decimal256::permille(5),
        buffer_distribution_factor: Decimal256::percent(20),
        anc_purchase_factor: Decimal256::percent(20),
        price_timeframe: 60u64,
        dyn_rate_epoch: 8600u64,
        dyn_rate_maxchange: Decimal256::permille(5),
        dyn_rate_yr_increase_expectation: Decimal256::permille(1),
        dyn_rate_min: Decimal256::from_ratio(1000000000000u64, 1000000000000000000u64),
        dyn_rate_max: Decimal256::from_ratio(1200000000000u64, 1000000000000000000u64),
    };
    let overseer_addr = app
        .instantiate_contract(
            overseer_code_id,
            owner.clone(),
            &msg,
            &[],
            String::from("OVERSEER"),
            Some(admin.to_string()),
        )
        .unwrap();

    // register contracts to market
    let msg = MarketExecuteMsgOld::RegisterContracts {
        overseer_contract: overseer_addr.to_string(),
        interest_model: interest_model_addr.to_string(),
        distribution_model: distribution_model_addr.to_string(),
        collector_contract: collector_addr.to_string(),
        distributor_contract: distributor_addr.to_string(),
    };

    app.execute_contract(owner.clone(), market_addr.clone(), &msg, &[])
        .unwrap();

    // query aterra contract address
    let res: MarketConfigResponseOld = app
        .wrap()
        .query_wasm_smart(market_addr.clone(), &MarketQueryMsgOld::Config {})
        .unwrap();

    Addresses {
        market_addr: Some(market_addr),
        overseer_addr: Some(overseer_addr),
        vterra_addr: None,
        aterra_cw20_addr: Some(Addr::unchecked(res.aterra_contract)),
        vterra_cw20_addr: None,
        oracle_addr: Some(oracle_addr),
        interest_model_addr: Some(interest_model_addr),
        distribution_model_addr: Some(distribution_model_addr),
        liquidation_addr: Some(liquidation_addr),
        collector_addr: Some(collector_addr),
        distributor_addr: Some(distributor_addr),
    }
}

fn create_new_contract(app: &mut TerraApp, mut addresses: Addresses) -> Addresses {
    let owner = Addr::unchecked(OWNER);
    let admin = Addr::unchecked(ADMIN);

    // store new contract code
    let token_code_id = store_token_contract_code(app);
    let vterra_code_id = store_vterra_contract_code(app);

    // instantiate vterra contract
    let msg = VterraInstantiateMsg {
        owner_addr: owner.to_string(),
        vterra_code_id: token_code_id,
        market_addr: addresses.market_addr.as_ref().unwrap().to_string(),
        overseer_addr: addresses.overseer_addr.as_ref().unwrap().to_string(),
        aterra_contract: addresses.aterra_cw20_addr.as_ref().unwrap().to_string(),
        stable_denom: "uust".to_string(),
        target_share: Decimal256::percent(80),
        max_pos_change: Decimal256::permille(1),
        max_neg_change: Decimal256::permille(1),
        max_rate: Decimal256::from_str("1.20").unwrap(),
        min_rate: Decimal256::from_str("1.01").unwrap(),
        diff_multiplier: Decimal256::percent(5),
        initial_premium_rate: Decimal256::percent(2),
        premium_rate_epoch: 10,
        min_gross_rate: Decimal256::from_str("1.185").unwrap(),
    };

    addresses.vterra_addr = Some(
        app.instantiate_contract(
            vterra_code_id,
            owner.clone(),
            &msg,
            &[],
            String::from("VTERRA"),
            Some(admin.to_string()),
        )
        .unwrap(),
    );

    // query vterra cw20 contract address
    let res: VterraConfigResponse = app
        .wrap()
        .query_wasm_smart(
            addresses.vterra_addr.as_ref().unwrap().clone(),
            &VterraQueryMsg::Config {},
        )
        .unwrap();

    addresses.vterra_cw20_addr = Some(Addr::unchecked(res.vterra_contract));

    addresses
}

fn migrate_contracts(app: &mut TerraApp, addresses: &Addresses) {
    let admin = Addr::unchecked(ADMIN);

    // store new contract code
    let market_code_id = store_market_contract_code(app);
    let overseer_code_id = store_overseer_contract_code(app);

    // migrate market contract
    let msg = MarketMigrateMsg {
        vterra_anchor_addr: addresses.vterra_addr.as_ref().unwrap().to_string(),
        vterra_cw20_addr: addresses.vterra_cw20_addr.as_ref().unwrap().to_string(),
    };
    app.migrate_contract(
        admin.clone(),
        addresses.market_addr.as_ref().unwrap().clone(),
        &msg,
        market_code_id,
    )
    .unwrap();

    // migrate overseer contract
    let msg = OverseerMigrateMsg {
        dyn_rate_epoch: 8600u64,
        dyn_rate_maxchange: Decimal256::permille(5),
        dyn_rate_yr_increase_expectation: Decimal256::permille(1),
        dyn_rate_min: Decimal256::from_ratio(1000000000000u64, 1000000000000000000u64),
        dyn_rate_max: Decimal256::from_ratio(1200000000000u64, 1000000000000000000u64),
        vterra_contract_addr: addresses.vterra_addr.as_ref().unwrap().to_string(),
    };
    app.migrate_contract(
        admin.clone(),
        addresses.overseer_addr.as_ref().unwrap().clone(),
        &msg,
        overseer_code_id,
    )
    .unwrap();
}

fn proper_initialization(app: &mut TerraApp) -> Addresses {
    let addresses = create_old_contracts(app);
    let addresses = create_new_contract(app, addresses);
    migrate_contracts(app, &addresses);
    addresses
}

#[test]
fn test_migration() {
    let mut app = mock_app();
    proper_initialization(&mut app);
}

#[test]
fn deposit_aterra_and_withdraw() {
    let mut app = mock_app();
    let addresses = proper_initialization(&mut app);
    let user = Addr::unchecked(USER);
    let market_addr = addresses.market_addr.unwrap();
    let overseer_addr = addresses.overseer_addr.unwrap();
    let aterra_cw20_addr = addresses.aterra_cw20_addr.unwrap();

    // give the user 101 UST
    app.init_bank_balance(&user, coins(101_000_000, "uust"))
        .unwrap();

    // deposit 100 UST to get aUST
    let msg = MarketExecuteMsg::DepositStable {};
    app.execute_contract(
        user.clone(),
        market_addr.clone(),
        &msg,
        &coins(100_000_000, "uust"),
    )
    .unwrap();

    // query epoch_period of overseer
    let res: OverseerConfigResponse = app
        .wrap()
        .query_wasm_smart(overseer_addr.clone(), &OverseerQueryMsg::Config {})
        .unwrap();
    let epoch_period = res.epoch_period;

    // call overseer's ExecuteEpochOperations every epoch_period blocks
    app.update_block(|b| {
        b.height += epoch_period;
    });
    let msg = OverserrExecuteMsg::ExecuteEpochOperations {};
    // @TODO: fix the overflow error
    app.execute_contract(user.clone(), overseer_addr.clone(), &msg, &[])
        .unwrap();

    // query user's amount of aUST
    let res: BalanceResponse = app
        .wrap()
        .query_wasm_smart(
            aterra_cw20_addr.clone(),
            &Cw20QueryMsg::Balance {
                address: user.to_string(),
            },
        )
        .unwrap();
    let aterra_balance = res.balance;

    // redeem all his deposited UST
    let msg = Cw20ExecuteMsg::Send {
        contract: market_addr.to_string(),
        amount: aterra_balance,
        msg: to_binary(&MarketCw20HookMsg::RedeemStable {}).unwrap(),
    };
    app.execute_contract(user.clone(), aterra_cw20_addr.clone(), &msg, &[])
        .unwrap();

    // check user's UST balance
    assert_eq!(
        app.wrap().query_all_balances(user).unwrap(),
        coins(101_000_000, "uust")
    );
}
