
use near_sdk::json_types::{U128};
use near_sdk::{Balance};
use near_sdk_sim::{call, to_yocto, ContractAccount, UserAccount};

// use near_sdk_sim::transaction::ExecutionStatus;
use ref_exchange::{ContractContract as TestRef};
use test_token::ContractContract as TestToken;
use ref_farming_v2::{ContractContract as Farming};
use ref_farming_v2::{HRSimpleFarmTerms};
use near_sdk::serde_json::Value;

use super::init::*;
use super::utils::*;

#[allow(dead_code)]
pub(crate) fn prepair_pool_and_liquidity(
    root: &UserAccount, 
    owner: &UserAccount,
    farming_id: String,
    lps: Vec<&UserAccount>,
) -> (ContractAccount<TestRef>, ContractAccount<TestToken>, ContractAccount<TestToken>) {
    let pool = deploy_pool(&root, swap(), owner.account_id());
    let token1 = deploy_token(&root, dai(), vec![swap()]);
    let token2 = deploy_token(&root, eth(), vec![swap()]);
    call!(owner, pool.extend_whitelisted_tokens(vec![to_va(dai()), to_va(eth())]))
    .assert_success();
    call!(root,
        pool.add_simple_pool(vec![to_va(dai()), to_va(eth())], 25),
        deposit = to_yocto("1")
    ).assert_success();
    call!(root, pool.mft_register(":0".to_string(), to_va(farming_id)), deposit = to_yocto("1"))
    .assert_success();
    for lp in lps {
        add_liquidity(lp, &pool, &token1, &token2, 0);
    }
    (pool,token1, token2)
}

#[allow(dead_code)]
pub(crate) fn prepair_pool(
    root: &UserAccount, 
    owner: &UserAccount, 
) -> (ContractAccount<TestRef>, ContractAccount<TestToken>, ContractAccount<TestToken>) {
    let pool = deploy_pool(&root, swap(), owner.account_id());
    let token1 = deploy_token(&root, dai(), vec![swap()]);
    let token2 = deploy_token(&root, eth(), vec![swap()]);
    call!(
        owner,
        pool.extend_whitelisted_tokens(vec![to_va(dai()), to_va(eth())])
    );
    call!(
        root,
        pool.add_simple_pool(vec![to_va(dai()), to_va(eth())], 25),
        deposit = to_yocto("1")
    )
    .assert_success();
    (pool, token1, token2)
}

#[allow(dead_code)]
pub(crate) fn prepair_farm(
    root: &UserAccount, 
    owner: &UserAccount,
    token: &ContractAccount<TestToken>,
    total_reward: Balance,
) -> (ContractAccount<Farming>, String) {
    // create farm
    
    let farming = deploy_farming(&root, farming_id(), owner.account_id());
    let out_come = call!(
        owner,
        farming.create_simple_farm(HRSimpleFarmTerms{
            seed_id: format!("{}@0", swap()),
            reward_token: to_va(token.account_id()),
            start_at: 0,
            reward_per_session: to_yocto("1").into(),
            session_interval: 60,
        }, Some(U128(1000000000000000000))),
        deposit = to_yocto("1")
    );
    out_come.assert_success();
    let farm_id: String;
    if let Value::String(farmid) = out_come.unwrap_json_value() {
        farm_id = farmid.clone();
    } else {
        farm_id = String::from("N/A");
    }
    // println!("    Farm {} created at Height#{}", farm_id.clone(), root.borrow_runtime().current_block().block_height);
    
    // deposit reward token
    call!(
        root,
        token.storage_deposit(Some(to_va(farming_id())), None),
        deposit = to_yocto("1")
    )
    .assert_success();
    mint_token(&token, &root, total_reward.into());
    call!(
        root,
        token.ft_transfer_call(to_va(farming_id()), total_reward.into(), None, generate_reward_msg(farm_id.clone())),
        deposit = 1
    )
    .assert_success();
    // println!("    Farm running at Height#{}", root.borrow_runtime().current_block().block_height);

    (farming, farm_id)
}

#[allow(dead_code)]
pub(crate) fn prepair_multi_farms(
    root: &UserAccount, 
    owner: &UserAccount,
    token: &ContractAccount<TestToken>,
    total_reward: Balance,
    farm_count: u32,
) -> (ContractAccount<Farming>, Vec<String>) {
    // create farms
    
    let farming = deploy_farming(&root, farming_id(), owner.account_id());
    let mut farm_ids: Vec<String> = vec![];

    // register farming contract to reward token
    call!(
        root,
        token.storage_deposit(Some(to_va(farming_id())), None),
        deposit = to_yocto("1")
    )
    .assert_success();

    mint_token(&token, &root, to_yocto("100000"));

    for _ in 0..farm_count {
        let out_come = call!(
            owner,
            farming.create_simple_farm(HRSimpleFarmTerms{
                seed_id: format!("{}@0", swap()),
                reward_token: to_va(token.account_id()),
                start_at: 0,
                reward_per_session: to_yocto("1").into(),
                session_interval: 60,
            }, Some(U128(1000000000000000000))),
            deposit = to_yocto("1")
        );
        out_come.assert_success();
        let farm_id: String;
        if let Value::String(farmid) = out_come.unwrap_json_value() {
            farm_id = farmid.clone();
        } else {
            farm_id = String::from("N/A");
        }
        call!(
            root,
            token.ft_transfer_call(to_va(farming_id()), total_reward.into(), None, generate_reward_msg(farm_id.clone())),
            deposit = 1
        )
        .assert_success();

        farm_ids.push(farm_id.clone());

        println!("  Farm {} created and running at Height#{}", farm_id.clone(), root.borrow_runtime().current_block().block_height);
    }
    if farm_count >= 32 {
        let out_come = call!(
            owner,
            farming.create_simple_farm(HRSimpleFarmTerms{
                seed_id: format!("{}@0", swap()),
                reward_token: to_va(token.account_id()),
                start_at: 0,
                reward_per_session: to_yocto("1").into(),
                session_interval: 60,
            }, Some(U128(1000000000000000000))),
            deposit = to_yocto("1")
        );
        assert!(!out_come.is_ok());
        let ex_status = format!("{:?}", out_come.promise_errors()[0].as_ref().unwrap().status());
        assert!(ex_status.contains("E36: the number of farms has reached its limit"));
    }
    
    (farming, farm_ids)
}

pub(crate) fn add_liquidity(
    user: &UserAccount, 
    pool: &ContractAccount<TestRef>, 
    token1: &ContractAccount<TestToken>, 
    token2: &ContractAccount<TestToken>, 
    pool_id: u64,
) {
    mint_token(&token1, user, to_yocto("105"));
    mint_token(&token2, user, to_yocto("105"));
    call!(
        user,
        pool.storage_deposit(None, None),
        deposit = to_yocto("1")
    )
    .assert_success();
    call!(
        user,
        token1.ft_transfer_call(to_va(swap()), to_yocto("100").into(), None, "".to_string()),
        deposit = 1
    )
    .assert_success();
    call!(
        user,
        token2.ft_transfer_call(to_va(swap()), to_yocto("100").into(), None, "".to_string()),
        deposit = 1
    )
    .assert_success();
    call!(
        user,
        pool.add_liquidity(pool_id, vec![U128(to_yocto("100")), U128(to_yocto("100"))], None),
        deposit = to_yocto("0.01")
    )
    .assert_success();
}

pub(crate) fn mint_token(token: &ContractAccount<TestToken>, user: &UserAccount, amount: Balance) {
    // call!(
    //     user,
    //     token.storage_deposit(None, None),
    //     deposit = to_yocto("1")
    // )
    // .assert_success();
    call!(
        user,
        token.mint(to_va(user.account_id.clone()), amount.into())
    ).assert_success();
}
