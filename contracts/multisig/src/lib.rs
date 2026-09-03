#![no_std]
use soroban_sdk::{contract, contractimpl, contracttype, Address, Env, Vec};

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Proposal {
    pub id: u64,
    pub proposer: Address,
    pub target: Address,
    pub amount: i128,
    pub approvals: Vec<Address>,
    pub threshold: u32,
    pub executed: bool,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DataKey {
    Owners,
    Threshold,
    Proposal(u64),
    ProposalCount,
}

#[contract]
pub struct MultisigContract;

#[contractimpl]
impl MultisigContract {
    /// Initialize vault with owners list and approval threshold
    pub fn init(env: Env, owners: Vec<Address>, threshold: u32) {
        if env.storage().instance().has(&DataKey::Owners) {
            panic!("already initialized");
        }
        if threshold == 0 || threshold > owners.len() {
            panic!("invalid threshold");
        }
        env.storage().instance().set(&DataKey::Owners, &owners);
        env.storage().instance().set(&DataKey::Threshold, &threshold);
    }

    /// Submit a new transaction proposal
    pub fn create_proposal(env: Env, proposer: Address, target: Address, amount: i128) -> u64 {
        proposer.require_auth();

        Self::require_owner(&env, &proposer);

        let count: u64 = env.storage().instance().get(&DataKey::ProposalCount).unwrap_or(0);
        let proposal_id = count + 1;
        let threshold: u32 = env.storage().instance().get(&DataKey::Threshold).expect("not initialized");

        let mut initial_approvals = Vec::new(&env);
        initial_approvals.push_back(proposer.clone());

        let proposal = Proposal {
            id: proposal_id,
            proposer,
            target,
            amount,
            approvals: initial_approvals,
            threshold,
            executed: false,
        };

        env.storage().persistent().set(&DataKey::Proposal(proposal_id), &proposal);
        env.storage().instance().set(&DataKey::ProposalCount, &proposal_id);

        proposal_id
    }

    /// Approve a pending proposal
    pub fn approve_proposal(env: Env, owner: Address, proposal_id: u64) {
        owner.require_auth();
        Self::require_owner(&env, &owner);

        let mut proposal: Proposal = env
            .storage()
            .persistent()
            .get(&DataKey::Proposal(proposal_id))
            .expect("proposal not found");

        if proposal.executed {
            panic!("proposal already executed");
        }

        for existing in proposal.approvals.iter() {
            if existing == owner {
                panic!("owner already approved proposal");
            }
        }

        proposal.approvals.push_back(owner);
        env.storage().persistent().set(&DataKey::Proposal(proposal_id), &proposal);
    }

    /// Execute proposal once threshold is met
    pub fn execute_proposal(env: Env, caller: Address, proposal_id: u64) {
        caller.require_auth();
        Self::require_owner(&env, &caller);

        let mut proposal: Proposal = env
            .storage()
            .persistent()
            .get(&DataKey::Proposal(proposal_id))
            .expect("proposal not found");

        if proposal.executed {
            panic!("proposal already executed");
        }

        if proposal.approvals.len() < proposal.threshold {
            panic!("threshold not met");
        }

        proposal.executed = true;
        env.storage().persistent().set(&DataKey::Proposal(proposal_id), &proposal);
    }

    /// Query proposal state
    pub fn get_proposal(env: Env, proposal_id: u64) -> Proposal {
        env.storage()
            .persistent()
            .get(&DataKey::Proposal(proposal_id))
            .expect("proposal not found")
    }

    fn require_owner(env: &Env, account: &Address) {
        let owners: Vec<Address> = env.storage().instance().get(&DataKey::Owners).expect("not initialized");
        let mut is_owner = false;
        for o in owners.iter() {
            if o == *account {
                is_owner = true;
                break;
            }
        }
        if !is_owner {
            panic!("unauthorized: caller is not an owner");
        }
    }
}

#[cfg(test)]
mod test;
