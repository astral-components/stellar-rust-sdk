//! High-level Soroban contract invocation pipeline.

mod invoker;

pub use invoker::{ContractInvokeRequest, ContractInvoker, InvokeResult};

/// Default CPU instruction limit used when a simulation does not supply one.
pub const DEFAULT_CPU_INSTRUCTIONS: u64 = 10_000_000;

/// Default memory-bytes limit used when a simulation does not supply one.
pub const DEFAULT_MEMORY_BYTES: u64 = 40_000;

/// Resource ceiling applied when assembling a Soroban transaction.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ResourceBudget {
    /// Maximum CPU instructions the host may consume.
    pub cpu_instructions: u64,
    /// Maximum ledger-entry read/write memory in bytes.
    pub memory_bytes: u64,
}

impl Default for ResourceBudget {
    fn default() -> Self {
        Self {
            cpu_instructions: DEFAULT_CPU_INSTRUCTIONS,
            memory_bytes: DEFAULT_MEMORY_BYTES,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_budget_matches_constants() {
        let budget = ResourceBudget::default();
        assert_eq!(budget.cpu_instructions, DEFAULT_CPU_INSTRUCTIONS);
        assert_eq!(budget.memory_bytes, DEFAULT_MEMORY_BYTES);
    }
}
