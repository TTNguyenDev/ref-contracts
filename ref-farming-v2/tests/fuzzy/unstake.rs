use near_sdk_sim::{call, to_yocto, ContractAccount, UserAccount};
use ref_farming_v2::{ContractContract as Farming, FarmInfo};
use ref_exchange::{ContractContract as TestRef};
use rand_pcg::Pcg32;
use crate::fuzzy::{
    constant::*,
    utils::*,
    types::*,
};

pub fn do_unstake(ctx: &mut FarmInfo, _rng: &mut Pcg32, root: &UserAccount, operator: &Operator, farming :&ContractAccount<Farming>, pool :&ContractAccount<TestRef>){
    let farm_id = FARM_ID.to_string();
    let stake_count = match show_user_seed_amounts(&farming, operator.user.account_id(), false).get(&String::from("swap@0")){
        Some(r) => r.0,
        None => 0
    };
    println!("current farmer stake : {}", stake_count);
    if stake_count != 0 {
        let unclaim = show_unclaim(&farming, operator.user.account_id(), farm_id.clone(), false);
        ctx.claimed_reward.0 += unclaim.0;
        ctx.unclaimed_reward.0 -= unclaim.0;
        
        let farm_info = show_farminfo(&farming, farm_id.clone(), false);
        let out_come = call!(
            operator.user,
            farming.withdraw_seed(farm_info.seed_id.clone(), to_yocto("1").into()),
            deposit = 1
        );
        out_come.assert_success();
    } else {
        println!("----->> {} staking lpt.", operator.user.account_id());
        let out_come = call!(
            operator.user,
            pool.mft_transfer_call(":0".to_string(), to_va(farming_id()), to_yocto("1").into(), None, "".to_string()),
            deposit = 1
        );
        out_come.assert_success();
        println!("<<----- {} staked liquidity at #{}, ts:{}.", 
        operator.user.account_id(),
        root.borrow_runtime().current_block().block_height, 
        root.borrow_runtime().current_block().block_timestamp);
    }
    ctx.last_round = ctx.cur_round;
    let farm_info = show_farminfo(&farming, farm_id.clone(), false);
    assert_farming(&farm_info, "Running".to_string(), to_yocto(&OPERATION_NUM.to_string()), ctx.cur_round, ctx.last_round, ctx.claimed_reward.0, ctx.unclaimed_reward.0, ctx.beneficiary_reward.0);
    ctx.cur_round += 1;
}