#![cfg(test)]

use super::*;
use soroban_sdk::{testutils::Address as _, token, Address, Env};

#[test]
fn test_vesting_flow() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, VestingContract);
    let client = VestingContractClient::new(&env, &contract_id);

    let sender = Address::generate(&env);
    let beneficiary = Address::generate(&env);

    // Mock SAC token using admin
    let token_admin = Address::generate(&env);
    let token_contract = env.register_stellar_asset_contract(token_admin.clone());
    let token_client = token::Client::new(&env, &token_contract);
    let token_admin_client = token::StellarAssetClient::new(&env, &token_contract);

    token_admin_client.mint(&sender, &1000);
    assert_eq!(token_client.balance(&sender), 1000);

    let start_time = 1000;
    let cliff_time = 1500;
    let end_time = 2000;
    let total_amount = 1000;

    let schedule_id = client.create_schedule(
        &sender,
        &beneficiary,
        &token_contract,
        &total_amount,
        &start_time,
        &cliff_time,
        &end_time,
    );

    assert_eq!(schedule_id, 1);
    assert_eq!(token_client.balance(&contract_id), 1000);

    // Set time before cliff
    env.ledger().set_timestamp(1200);
    assert_eq!(client.get_vested_amount(&schedule_id), 0);

    // Set time at cliff/midpoint
    env.ledger().set_timestamp(1500);
    let vested = client.get_vested_amount(&schedule_id);
    assert_eq!(vested, 500);

    // Claim midpoint tokens
    let claimed = client.claim(&beneficiary, &schedule_id);
    assert_eq!(claimed, 500);
    assert_eq!(token_client.balance(&beneficiary), 500);

    // Set time to completion
    env.ledger().set_timestamp(2000);
    let claimed_final = client.claim(&beneficiary, &schedule_id);
    assert_eq!(claimed_final, 500);
    assert_eq!(token_client.balance(&beneficiary), 1000);
}
