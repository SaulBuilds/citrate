// lattice-v3/core/execution/src/state/account.rs

// Account manager for handling account states
use crate::types::{AccountState, Address, ExecutionError, ModelId};
use dashmap::DashMap;
use lattice_consensus::types::Hash;
use primitive_types::U256;
use std::sync::Arc;
use tracing::{debug, info};

/// Account manager for handling account states
pub struct AccountManager {
    accounts: Arc<DashMap<Address, AccountState>>,
    dirty: Arc<DashMap<Address, bool>>,
}

impl AccountManager {
    pub fn new() -> Self {
        Self {
            accounts: Arc::new(DashMap::new()),
            dirty: Arc::new(DashMap::new()),
        }
    }

    /// Get account state
    pub fn get_account(&self, address: &Address) -> AccountState {
        self.accounts
            .get(address)
            .map(|a| a.clone())
            .unwrap_or_default()
    }

    /// Set account state
    pub fn set_account(&self, address: Address, state: AccountState) {
        self.accounts.insert(address, state);
        self.dirty.insert(address, true);
    }

    /// Get balance
    pub fn get_balance(&self, address: &Address) -> U256 {
        self.get_account(address).balance
    }

    /// Set balance
    pub fn set_balance(&self, address: Address, balance: U256) {
        let mut account = self.get_account(&address);
        account.balance = balance;
        self.set_account(address, account);
    }

    /// Transfer value between accounts
    pub fn transfer(
        &self,
        from: &Address,
        to: &Address,
        value: U256,
    ) -> Result<(), ExecutionError> {
        if from == to {
            return Ok(());
        }

        let from_balance = self.get_balance(from);
        if from_balance < value {
            return Err(ExecutionError::InsufficientBalance {
                need: value,
                have: from_balance,
            });
        }

        // Deduct from sender
        self.set_balance(*from, from_balance - value);

        // Add to receiver
        let to_balance = self.get_balance(to);
        self.set_balance(*to, to_balance + value);

        debug!("Transferred {} from {} to {}", value, from, to);
        Ok(())
    }

    /// Get nonce
    pub fn get_nonce(&self, address: &Address) -> u64 {
        self.get_account(address).nonce
    }

    /// Set nonce
    pub fn set_nonce(&self, address: Address, nonce: u64) {
        let mut account = self.get_account(&address);
        account.nonce = nonce;
        self.set_account(address, account);
    }

    /// Increment nonce
    pub fn increment_nonce(&self, address: &Address) {
        let mut account = self.get_account(address);
        account.nonce += 1;
        self.set_account(*address, account);
    }

    /// Check and increment nonce
    pub fn check_and_increment_nonce(
        &self,
        address: &Address,
        expected: u64,
    ) -> Result<(), ExecutionError> {
        let current = self.get_nonce(address);
        if current != expected {
            return Err(ExecutionError::InvalidNonce {
                expected: current,
                got: expected,
            });
        }
        self.increment_nonce(address);
        Ok(())
    }

    /// Get code hash
    pub fn get_code_hash(&self, address: &Address) -> Hash {
        self.get_account(address).code_hash
    }

    /// Set code hash
    pub fn set_code_hash(&self, address: Address, code_hash: Hash) {
        let mut account = self.get_account(&address);
        account.code_hash = code_hash;
        self.set_account(address, account);
    }

    /// Check if account exists
    pub fn exists(&self, address: &Address) -> bool {
        self.accounts.contains_key(address)
    }

    /// Create account if not exists
    pub fn create_account_if_not_exists(&self, address: Address) {
        if !self.exists(&address) {
            self.set_account(address, AccountState::default());
            info!("Created new account: {}", address);
        }
    }

    /// Add model permission
    pub fn add_model_permission(&self, address: Address, model_id: ModelId) {
        let mut account = self.get_account(&address);
        if !account.model_permissions.contains(&model_id) {
            account.model_permissions.push(model_id);
            self.set_account(address, account);
        }
    }

    /// Check model permission
    pub fn has_model_permission(&self, address: &Address, model_id: &ModelId) -> bool {
        self.get_account(address)
            .model_permissions
            .contains(model_id)
    }

    /// Get dirty accounts
    pub fn get_dirty_accounts(&self) -> Vec<Address> {
        self.dirty
            .iter()
            .filter(|e| *e.value())
            .map(|e| *e.key())
            .collect()
    }

    /// Clear dirty flags
    pub fn clear_dirty(&self) {
        self.dirty.clear();
    }

    /// Create snapshot for rollback
    pub fn snapshot(&self) -> AccountSnapshot {
        AccountSnapshot {
            accounts: self
                .accounts
                .iter()
                .map(|e| (*e.key(), e.value().clone()))
                .collect(),
        }
    }

    /// Restore from snapshot
    pub fn restore(&self, snapshot: AccountSnapshot) {
        self.accounts.clear();
        for (addr, state) in snapshot.accounts {
            self.accounts.insert(addr, state);
        }
        self.dirty.clear();
    }
}

impl Default for AccountManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Account snapshot for rollback
pub struct AccountSnapshot {
    accounts: Vec<(Address, AccountState)>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_account_creation() {
        let manager = AccountManager::new();
        let addr = Address([1; 20]);

        assert!(!manager.exists(&addr));

        manager.create_account_if_not_exists(addr);
        assert!(manager.exists(&addr));

        let account = manager.get_account(&addr);
        assert_eq!(account.nonce, 0);
        assert_eq!(account.balance, U256::zero());
    }

    #[test]
    fn test_transfer() {
        let manager = AccountManager::new();
        let alice = Address([1; 20]);
        let bob = Address([2; 20]);

        // Setup alice with balance
        manager.set_balance(alice, U256::from(1000));

        // Transfer
        manager.transfer(&alice, &bob, U256::from(300)).unwrap();

        assert_eq!(manager.get_balance(&alice), U256::from(700));
        assert_eq!(manager.get_balance(&bob), U256::from(300));
    }

    #[test]
    fn test_insufficient_balance() {
        let manager = AccountManager::new();
        let alice = Address([1; 20]);
        let bob = Address([2; 20]);

        manager.set_balance(alice, U256::from(100));

        let result = manager.transfer(&alice, &bob, U256::from(200));
        assert!(matches!(
            result,
            Err(ExecutionError::InsufficientBalance { .. })
        ));
    }

    #[test]
    fn test_nonce_management() {
        let manager = AccountManager::new();
        let addr = Address([1; 20]);

        assert_eq!(manager.get_nonce(&addr), 0);

        manager.check_and_increment_nonce(&addr, 0).unwrap();
        assert_eq!(manager.get_nonce(&addr), 1);

        // Wrong nonce should fail
        let result = manager.check_and_increment_nonce(&addr, 0);
        assert!(matches!(result, Err(ExecutionError::InvalidNonce { .. })));
    }
}
