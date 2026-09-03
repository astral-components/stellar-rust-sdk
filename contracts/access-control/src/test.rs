#![cfg(test)]

use super::*;
use soroban_sdk::{symbol_short, testutils::Address as _, Env};

#[test]
fn test_access_control_workflow() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, AccessControlContract);
    let client = AccessControlContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let operator = Address::generate(&env);

    client.init(&admin);
    assert_eq!(client.get_admin(), admin);

    let minter_role = symbol_short!("minter");
    let operator_role = symbol_short!("operator");

    assert!(!client.has_role(&operator, &minter_role));

    // Grant role
    client.grant_role(&admin, &operator, &minter_role);
    assert!(client.has_role(&operator, &minter_role));

    client.grant_role(&admin, &operator, &operator_role);
    assert!(client.has_role(&operator, &operator_role));

    // Revoke role
    client.revoke_role(&admin, &operator, &minter_role);
    assert!(!client.has_role(&operator, &minter_role));
    assert!(client.has_role(&operator, &operator_role));
}
