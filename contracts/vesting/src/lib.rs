#![no_std]
use soroban_sdk::{contract, contractimpl, contracttype, token, Address, Env};

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VestingSchedule {
    pub sender: Address,
    pub beneficiary: Address,
    pub token: Address,
    pub total_amount: i128,
    pub start_time: u64,
    pub cliff_time: u64,
    pub end_time: u64,
    pub claimed_amount: i128,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DataKey {
    Schedule(u64),
    ScheduleCount,
}

#[contract]
pub struct VestingContract;

#[contractimpl]
impl VestingContract {
    /// Create a linear token vesting schedule with optional cliff
    pub fn create_schedule(
        env: Env,
        sender: Address,
        beneficiary: Address,
        token: Address,
        total_amount: i128,
        start_time: u64,
        cliff_time: u64,
        end_time: u64,
    ) -> u64 {
        sender.require_auth();

        if total_amount <= 0 {
            panic!("total_amount must be positive");
        }
        if start_time >= end_time {
            panic!("invalid timeframe: start >= end");
        }
        if cliff_time < start_time || cliff_time > end_time {
            panic!("invalid cliff time");
        }

        // Transfer tokens into escrow contract
        let token_client = token::Client::new(&env, &token);
        token_client.transfer(&sender, &env.current_contract_address(), &total_amount);

        let count: u64 = env.storage().instance().get(&DataKey::ScheduleCount).unwrap_or(0);
        let schedule_id = count + 1;

        let schedule = VestingSchedule {
            sender,
            beneficiary,
            token,
            total_amount,
            start_time,
            cliff_time,
            end_time,
            claimed_amount: 0,
        };

        env.storage().persistent().set(&DataKey::Schedule(schedule_id), &schedule);
        env.storage().instance().set(&DataKey::ScheduleCount, &schedule_id);

        schedule_id
    }

    /// Calculate currently vested amount for a schedule
    pub fn get_vested_amount(env: Env, schedule_id: u64) -> i128 {
        let schedule: VestingSchedule = env
            .storage()
            .persistent()
            .get(&DataKey::Schedule(schedule_id))
            .expect("schedule not found");

        let current_time = env.ledger().timestamp();

        if current_time < schedule.cliff_time {
            return 0;
        }

        if current_time >= schedule.end_time {
            return schedule.total_amount;
        }

        let duration = schedule.end_time - schedule.start_time;
        let elapsed = current_time - schedule.start_time;

        (schedule.total_amount * (elapsed as i128)) / (duration as i128)
    }

    /// Claim unlocked vested tokens
    pub fn claim(env: Env, beneficiary: Address, schedule_id: u64) -> i128 {
        beneficiary.require_auth();

        let mut schedule: VestingSchedule = env
            .storage()
            .persistent()
            .get(&DataKey::Schedule(schedule_id))
            .expect("schedule not found");

        if beneficiary != schedule.beneficiary {
            panic!("unauthorized beneficiary");
        }

        let vested = Self::get_vested_amount(env.clone(), schedule_id);
        let claimable = vested - schedule.claimed_amount;

        if claimable <= 0 {
            panic!("no claimable tokens available");
        }

        schedule.claimed_amount += claimable;
        env.storage().persistent().set(&DataKey::Schedule(schedule_id), &schedule);

        let token_client = token::Client::new(&env, &schedule.token);
        token_client.transfer(&env.current_contract_address(), &beneficiary, &claimable);

        claimable
    }

    /// Query vesting schedule details
    pub fn get_schedule(env: Env, schedule_id: u64) -> VestingSchedule {
        env.storage()
            .persistent()
            .get(&DataKey::Schedule(schedule_id))
            .expect("schedule not found")
    }
}

#[cfg(test)]
mod test;
