#![no_std]
use soroban_sdk::{contract, contractimpl, contracttype, symbol_short, Address, Env, Symbol};

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DataKey {
    Admin,
    Role(Address, Symbol),
}

#[contract]
pub struct AccessControlContract;

#[contractimpl]
impl AccessControlContract {
    /// Initialize the contract with an admin address
    pub fn init(env: Env, admin: Address) {
        if env.storage().instance().has(&DataKey::Admin) {
            panic!("already initialized");
        }
        admin.require_auth();
        env.storage().instance().set(&DataKey::Admin, &admin);

        // Grant Admin role to initial admin
        let admin_role = symbol_short!("admin");
        env.storage().persistent().set(&DataKey::Role(admin.clone(), admin_role), &true);
    }

    /// Retrieve current admin address
    pub fn get_admin(env: Env) -> Address {
        env.storage().instance().get(&DataKey::Admin).expect("not initialized")
    }

    /// Check if account has a given role
    pub fn has_role(env: Env, account: Address, role: Symbol) -> bool {
        env.storage().persistent().get(&DataKey::Role(account, role)).unwrap_or(false)
    }

    /// Grant a role to an account (requires admin auth)
    pub fn grant_role(env: Env, caller: Address, account: Address, role: Symbol) {
        caller.require_auth();
        let admin = Self::get_admin(env.clone());
        if caller != admin {
            panic!("only admin can grant roles");
        }
        env.storage().persistent().set(&DataKey::Role(account, role), &true);
    }

    /// Revoke a role from an account (requires admin auth)
    pub fn revoke_role(env: Env, caller: Address, account: Address, role: Symbol) {
        caller.require_auth();
        let admin = Self::get_admin(env.clone());
        if caller != admin {
            panic!("only admin can revoke roles");
        }
        env.storage().persistent().set(&DataKey::Role(account, role), &false);
    }

    /// Require that an account possesses a specific role or panic
    pub fn require_role(env: Env, account: Address, role: Symbol) {
        if !Self::has_role(env, account, role) {
            panic!("unauthorized: missing role");
        }
    }
}

#[cfg(test)]
mod test;
