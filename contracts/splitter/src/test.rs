#![cfg(test)]

use super::*;
use soroban_sdk::{testutils::Address as _, token, vec, Address, Env};

#[test]
fn test_splitter_distribution() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, SplitterContract);
    let client = SplitterContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let sender = Address::generate(&env);
    let recipient1 = Address::generate(&env);
    let recipient2 = Address::generate(&env);

    // Mock SAC token
    let token_admin = Address::generate(&env);
    let token_contract = env.register_stellar_asset_contract(token_admin.clone());
    let token_client = token::Client::new(&env, &token_contract);
    let token_admin_client = token::StellarAssetClient::new(&env, &token_contract);

    token_admin_client.mint(&sender, &1000);

    let shares = vec![
        &env,
        Share {
            recipient: recipient1.clone(),
            bps: 7000, // 70%
        },
        Share {
            recipient: recipient2.clone(),
            bps: 3000, // 30%
        },
    ];

    client.init(&admin, &shares);

    client.distribute(&sender, &token_contract, &1000);

    assert_eq!(token_client.balance(&sender), 0);
    assert_eq!(token_client.balance(&recipient1), 700);
    assert_eq!(token_client.balance(&recipient2), 300);
}
