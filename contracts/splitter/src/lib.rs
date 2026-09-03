#![no_std]
use soroban_sdk::{contract, contractimpl, contracttype, token, Address, Env, Vec};

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Share {
    pub recipient: Address,
    pub bps: u32, // Basis points (e.g. 5000 = 50%)
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DataKey {
    Admin,
    Shares,
}

#[contract]
pub struct SplitterContract;

#[contractimpl]
impl SplitterContract {
    /// Initialize revenue splitter with recipients and basis points allocations (must sum to 10,000)
    pub fn init(env: Env, admin: Address, shares: Vec<Share>) {
        if env.storage().instance().has(&DataKey::Admin) {
            panic!("already initialized");
        }
        admin.require_auth();

        let mut total_bps: u32 = 0;
        for share in shares.iter() {
            if share.bps == 0 {
                panic!("share bps must be greater than 0");
            }
            total_bps += share.bps;
        }

        if total_bps != 10_000 {
            panic!("total basis points must equal 10000");
        }

        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage().instance().set(&DataKey::Shares, &shares);
    }

    /// Split and distribute incoming tokens directly to recipients based on configured shares
    pub fn distribute(env: Env, sender: Address, token: Address, amount: i128) {
        sender.require_auth();

        if amount <= 0 {
            panic!("amount must be positive");
        }

        let shares: Vec<Share> = env.storage().instance().get(&DataKey::Shares).expect("not initialized");
        let token_client = token::Client::new(&env, &token);

        // Transfer funds from sender to contract
        token_client.transfer(&sender, &env.current_contract_address(), &amount);

        // Distribute to recipients according to BPS
        let mut distributed_total: i128 = 0;
        let last_idx = shares.len() - 1;

        for (i, share) in shares.iter().enumerate() {
            let recipient_amount = if i as u32 == last_idx {
                // Ensure no dust tokens remain due to rounding
                amount - distributed_total
            } else {
                (amount * (share.bps as i128)) / 10_000
            };

            if recipient_amount > 0 {
                token_client.transfer(&env.current_contract_address(), &share.recipient, &recipient_amount);
                distributed_total += recipient_amount;
            }
        }
    }

    /// Query configured shares
    pub fn get_shares(env: Env) -> Vec<Share> {
        env.storage().instance().get(&DataKey::Shares).expect("not initialized")
    }

    /// Query admin address
    pub fn get_admin(env: Env) -> Address {
        env.storage().instance().get(&DataKey::Admin).expect("not initialized")
    }
}

#[cfg(test)]
mod test;
