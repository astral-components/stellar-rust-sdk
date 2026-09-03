#![cfg(test)]

use super::*;
use soroban_sdk::{testutils::Address as _, vec, Address, Env};

#[test]
fn test_multisig_workflow() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, MultisigContract);
    let client = MultisigContractClient::new(&env, &contract_id);

    let owner1 = Address::generate(&env);
    let owner2 = Address::generate(&env);
    let owner3 = Address::generate(&env);
    let target = Address::generate(&env);

    let owners = vec![&env, owner1.clone(), owner2.clone(), owner3.clone()];
    client.init(&owners, &2);

    let proposal_id = client.create_proposal(&owner1, &target, &500);

    let proposal = client.get_proposal(&proposal_id);
    assert_eq!(proposal.approvals.len(), 1);
    assert!(!proposal.executed);

    // Second approval reaches threshold (2 of 3)
    client.approve_proposal(&owner2, &proposal_id);

    let proposal_after = client.get_proposal(&proposal_id);
    assert_eq!(proposal_after.approvals.len(), 2);

    // Execute proposal
    client.execute_proposal(&owner1, &proposal_id);

    let executed_proposal = client.get_proposal(&proposal_id);
    assert!(executed_proposal.executed);
}
