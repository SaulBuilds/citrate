use lattice_consensus::{Hash, PublicKey, Transaction};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ValidationError {
    #[error("Invalid signature")]
    InvalidSignature,
    
    #[error("Insufficient balance: required {required}, available {available}")]
    InsufficientBalance { required: u128, available: u128 },
    
    #[error("Invalid nonce: expected {expected}, got {got}")]
    InvalidNonce { expected: u64, got: u64 },
    
    #[error("Gas limit too high: max {max}, got {got}")]
    GasLimitTooHigh { max: u64, got: u64 },
    
    #[error("Gas price too low: min {min}, got {got}")]
    GasPriceTooLow { min: u64, got: u64 },
    
    #[error("Transaction expired")]
    Expired,
    
    #[error("Invalid recipient")]
    InvalidRecipient,
    
    #[error("Data too large: max {max} bytes, got {got}")]
    DataTooLarge { max: usize, got: usize },
    
    #[error("Blacklisted address: {0:?}")]
    BlacklistedAddress(PublicKey),
    
    #[error("Rate limit exceeded")]
    RateLimitExceeded,
}

/// Transaction validation rules
#[derive(Debug, Clone)]
pub struct ValidationRules {
    /// Minimum gas price
    pub min_gas_price: u64,
    
    /// Maximum gas limit per transaction
    pub max_gas_limit: u64,
    
    /// Maximum transaction data size
    pub max_data_size: usize,
    
    /// Transaction expiry time in seconds
    pub tx_expiry_secs: u64,
    
    /// Enable signature verification
    pub verify_signatures: bool,
    
    /// Enable balance checks
    pub check_balance: bool,
    
    /// Enable nonce validation
    pub check_nonce: bool,
    
    /// Rate limit per sender (txs per minute)
    pub rate_limit: u32,
    /// Rate limit window length in seconds
    pub rate_limit_window_secs: u64,
}

impl Default for ValidationRules {
    fn default() -> Self {
        Self {
            min_gas_price: 1_000_000_000, // 1 gwei
            max_gas_limit: 10_000_000, // 10M gas
            max_data_size: 128 * 1024, // 128 KB
            tx_expiry_secs: 3600, // 1 hour
            verify_signatures: true,
            check_balance: true,
            check_nonce: true,
            rate_limit: 100, // 100 txs per minute
            rate_limit_window_secs: 60,
        }
    }
}

/// Account state for validation
#[derive(Debug, Clone)]
pub struct AccountState {
    pub balance: u128,
    pub nonce: u64,
    pub code_hash: Option<Hash>,
}

impl AccountState {
    pub fn new(balance: u128, nonce: u64) -> Self {
        Self {
            balance,
            nonce,
            code_hash: None,
        }
    }
}

/// State provider trait for account lookups
#[async_trait::async_trait]
pub trait StateProvider: Send + Sync {
    async fn get_account(&self, address: &PublicKey) -> Option<AccountState>;
    async fn get_balance(&self, address: &PublicKey) -> u128;
    async fn get_nonce(&self, address: &PublicKey) -> u64;
}

/// Mock state provider for testing
pub struct MockStateProvider {
    accounts: Arc<RwLock<HashMap<PublicKey, AccountState>>>,
}

impl MockStateProvider {
    pub fn new() -> Self {
        Self {
            accounts: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    pub async fn set_account(&self, address: PublicKey, state: AccountState) {
        self.accounts.write().await.insert(address, state);
    }
}

impl Default for MockStateProvider {
    fn default() -> Self { Self::new() }
}

#[async_trait::async_trait]
impl StateProvider for MockStateProvider {
    async fn get_account(&self, address: &PublicKey) -> Option<AccountState> {
        self.accounts.read().await.get(address).cloned()
    }
    
    async fn get_balance(&self, address: &PublicKey) -> u128 {
        self.accounts
            .read()
            .await
            .get(address)
            .map(|a| a.balance)
            .unwrap_or(0)
    }
    
    async fn get_nonce(&self, address: &PublicKey) -> u64 {
        self.accounts
            .read()
            .await
            .get(address)
            .map(|a| a.nonce)
            .unwrap_or(0)
    }
}

/// Transaction validator
pub struct TxValidator<S: StateProvider> {
    rules: ValidationRules,
    state_provider: Arc<S>,
    blacklist: Arc<RwLock<HashMap<PublicKey, bool>>>,
    rate_limiter: Arc<RwLock<HashMap<PublicKey, RateLimitEntry>>>,
}

#[derive(Debug, Clone)]
struct RateLimitEntry {
    count: u32,
    window_start: u64,
}

impl<S: StateProvider> TxValidator<S> {
    pub fn new(rules: ValidationRules, state_provider: Arc<S>) -> Self {
        Self {
            rules,
            state_provider,
            blacklist: Arc::new(RwLock::new(HashMap::new())),
            rate_limiter: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    /// Validate a transaction
    pub async fn validate(&self, tx: &Transaction) -> Result<(), ValidationError> {
        // Check blacklist
        if self.is_blacklisted(&tx.from).await {
            return Err(ValidationError::BlacklistedAddress(tx.from));
        }
        
        // Check rate limit
        self.check_rate_limit(&tx.from).await?;
        
        // Basic validation
        self.validate_basic(tx)?;
        
        // Signature validation
        if self.rules.verify_signatures {
            self.validate_signature(tx)?;
        }
        
        // State validation
        if self.rules.check_balance || self.rules.check_nonce {
            self.validate_state(tx).await?;
        }
        
        Ok(())
    }
    
    /// Basic validation without state lookups
    fn validate_basic(&self, tx: &Transaction) -> Result<(), ValidationError> {
        // Check gas price
        if tx.gas_price < self.rules.min_gas_price {
            return Err(ValidationError::GasPriceTooLow {
                min: self.rules.min_gas_price,
                got: tx.gas_price,
            });
        }
        
        // Check gas limit
        if tx.gas_limit > self.rules.max_gas_limit {
            return Err(ValidationError::GasLimitTooHigh {
                max: self.rules.max_gas_limit,
                got: tx.gas_limit,
            });
        }
        
        // Check data size
        if tx.data.len() > self.rules.max_data_size {
            return Err(ValidationError::DataTooLarge {
                max: self.rules.max_data_size,
                got: tx.data.len(),
            });
        }
        
        // Check recipient (optional field validation)
        // Contract creation has no recipient
        
        Ok(())
    }
    
    /// Validate transaction signature
    fn validate_signature(&self, tx: &Transaction) -> Result<(), ValidationError> {
        // Use real cryptographic signature verification
        match lattice_consensus::crypto::verify_transaction(tx) {
            Ok(true) => Ok(()),
            Ok(false) => Err(ValidationError::InvalidSignature),
            Err(e) => {
                warn!("Signature verification error: {}", e);
                Err(ValidationError::InvalidSignature)
            }
        }
    }
    
    /// Validate against current state
    async fn validate_state(&self, tx: &Transaction) -> Result<(), ValidationError> {
        let account = self.state_provider.get_account(&tx.from).await
            .unwrap_or_else(|| AccountState::new(0, 0));
        
        // Check nonce
        if self.rules.check_nonce && tx.nonce != account.nonce {
            return Err(ValidationError::InvalidNonce {
                expected: account.nonce,
                got: tx.nonce,
            });
        }
        
        // Check balance
        if self.rules.check_balance {
            let required = tx.value + (tx.gas_limit * tx.gas_price) as u128;
            if account.balance < required {
                return Err(ValidationError::InsufficientBalance {
                    required,
                    available: account.balance,
                });
            }
        }
        
        Ok(())
    }
    
    /// Check if address is blacklisted
    async fn is_blacklisted(&self, address: &PublicKey) -> bool {
        self.blacklist.read().await.get(address).copied().unwrap_or(false)
    }
    
    /// Add address to blacklist
    pub async fn blacklist_address(&self, address: PublicKey) {
        self.blacklist.write().await.insert(address, true);
        warn!("Blacklisted address: {:?}", address);
    }
    
    /// Remove address from blacklist
    pub async fn unblacklist_address(&self, address: &PublicKey) {
        self.blacklist.write().await.remove(address);
        info!("Unblacklisted address: {:?}", address);
    }
    
    /// Check rate limit for sender
    async fn check_rate_limit(&self, sender: &PublicKey) -> Result<(), ValidationError> {
        let current_time = chrono::Utc::now().timestamp() as u64;
        let window_size = self.rules.rate_limit_window_secs;
        
        let mut rate_limiter = self.rate_limiter.write().await;
        
        let entry = rate_limiter.entry(*sender).or_insert(RateLimitEntry {
            count: 0,
            window_start: current_time,
        });
        
        // Reset window if expired
        if current_time - entry.window_start >= window_size {
            entry.count = 0;
            entry.window_start = current_time;
        }
        
        // Check limit
        if entry.count >= self.rules.rate_limit {
            return Err(ValidationError::RateLimitExceeded);
        }
        
        entry.count += 1;
        Ok(())
    }
    
    /// Batch validate multiple transactions
    pub async fn validate_batch(&self, transactions: &[Transaction]) -> Vec<Result<(), ValidationError>> {
        let mut results = Vec::new();
        
        for tx in transactions {
            results.push(self.validate(tx).await);
        }
        
        results
    }
}

/// Validation pipeline for processing transactions
pub struct ValidationPipeline<S: StateProvider> {
    validator: Arc<TxValidator<S>>,
    parallel_validation: bool,
    #[allow(dead_code)]
    batch_size: usize,
}

impl<S: StateProvider> ValidationPipeline<S> {
    pub fn new(validator: Arc<TxValidator<S>>) -> Self {
        Self {
            validator,
            parallel_validation: true,
            batch_size: 100,
        }
    }
    
    /// Process a batch of transactions
    pub async fn process(&self, transactions: Vec<Transaction>) -> (Vec<Transaction>, Vec<(Transaction, ValidationError)>) {
        let mut valid = Vec::new();
        let mut invalid = Vec::new();
        
        if self.parallel_validation {
            // Parallel validation for independent checks
            let futures: Vec<_> = transactions
                .into_iter()
                .map(|tx| {
                    let validator = self.validator.clone();
                    async move {
                        match validator.validate(&tx).await {
                            Ok(()) => (Some(tx), None),
                            Err(e) => (None, Some((tx, e))),
                        }
                    }
                })
                .collect();
            
            let results = futures::future::join_all(futures).await;
            
            for (valid_tx, invalid_tx) in results {
                if let Some(tx) = valid_tx {
                    valid.push(tx);
                }
                if let Some((tx, err)) = invalid_tx {
                    invalid.push((tx, err));
                }
            }
        } else {
            // Sequential validation
            for tx in transactions {
                match self.validator.validate(&tx).await {
                    Ok(()) => valid.push(tx),
                    Err(e) => invalid.push((tx, e)),
                }
            }
        }
        
        debug!("Validation complete: {} valid, {} invalid", valid.len(), invalid.len());
        
        (valid, invalid)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use lattice_consensus::types::Signature;
    
    fn create_test_tx(nonce: u64, gas_price: u64, value: u128) -> Transaction {
        Transaction {
            hash: Hash::new([nonce as u8; 32]),
            nonce,
            from: PublicKey::new([1; 32]),
            to: Some(PublicKey::new([2; 32])),
            value,
            gas_limit: 21000,
            gas_price,
            data: vec![],
            signature: Signature::new([1; 64]), // Non-zero for validation
            tx_type: None,
        }
    }
    
    #[tokio::test]
    async fn test_basic_validation() {
        let rules = ValidationRules {
            check_balance: false,  // Disable balance check for basic test
            verify_signatures: false, // Disable signature verification
            ..Default::default()
        };
        let state_provider = Arc::new(MockStateProvider::new());
        let validator = TxValidator::new(rules, state_provider);
        
        let tx = create_test_tx(0, 2_000_000_000, 1000);
        assert!(validator.validate(&tx).await.is_ok());
    }
    
    #[tokio::test]
    async fn test_gas_price_validation() {
        let rules = ValidationRules {
            min_gas_price: 1_000_000_000,
            verify_signatures: false,
            check_balance: false,
            check_nonce: false,
            ..Default::default()
        };
        let state_provider = Arc::new(MockStateProvider::new());
        let validator = TxValidator::new(rules, state_provider);
        
        let tx = create_test_tx(0, 500_000_000, 1000); // Too low gas price
        assert!(matches!(
            validator.validate(&tx).await,
            Err(ValidationError::GasPriceTooLow { .. })
        ));
    }
    
    #[tokio::test]
    async fn test_balance_validation() {
        let rules = ValidationRules {
            verify_signatures: false,
            check_balance: true,
            check_nonce: false,
            ..Default::default()
        };
        let state_provider = Arc::new(MockStateProvider::new());
        
        // Set account with insufficient balance
        state_provider.set_account(
            PublicKey::new([1; 32]),
            AccountState::new(1000, 0),
        ).await;
        
        let validator = TxValidator::new(rules, state_provider);
        
        let tx = create_test_tx(0, 2_000_000_000, 100000); // Requires more than available
        assert!(matches!(
            validator.validate(&tx).await,
            Err(ValidationError::InsufficientBalance { .. })
        ));
    }
    
    #[tokio::test]
    async fn test_nonce_validation() {
        let rules = ValidationRules {
            verify_signatures: false,
            check_balance: false,
            check_nonce: true,
            ..Default::default()
        };
        let state_provider = Arc::new(MockStateProvider::new());
        
        // Set account with nonce 5
        state_provider.set_account(
            PublicKey::new([1; 32]),
            AccountState::new(1000000, 5),
        ).await;
        
        let validator = TxValidator::new(rules, state_provider);
        
        // Wrong nonce
        let tx = create_test_tx(3, 2_000_000_000, 1000);
        assert!(matches!(
            validator.validate(&tx).await,
            Err(ValidationError::InvalidNonce { .. })
        ));
        
        // Correct nonce
        let tx = create_test_tx(5, 2_000_000_000, 1000);
        assert!(validator.validate(&tx).await.is_ok());
    }
    
    #[tokio::test]
    async fn test_blacklist() {
        let rules = ValidationRules {
            verify_signatures: false,
            check_balance: false,
            check_nonce: false,
            ..Default::default()
        };
        let state_provider = Arc::new(MockStateProvider::new());
        let validator = TxValidator::new(rules, state_provider);
        
        let sender = PublicKey::new([1; 32]);
        validator.blacklist_address(sender).await;
        
        let tx = create_test_tx(0, 2_000_000_000, 1000);
        assert!(matches!(
            validator.validate(&tx).await,
            Err(ValidationError::BlacklistedAddress(_))
        ));
    }
    
    #[tokio::test]
    async fn test_rate_limiting() {
        let rules = ValidationRules {
            verify_signatures: false,
            check_balance: false,
            check_nonce: false,
            rate_limit: 2, // Only 2 txs per window
            rate_limit_window_secs: 60,
            ..Default::default()
        };
        let state_provider = Arc::new(MockStateProvider::new());
        let validator = TxValidator::new(rules, state_provider);
        
        let tx1 = create_test_tx(0, 2_000_000_000, 1000);
        let tx2 = create_test_tx(1, 2_000_000_000, 1000);
        let tx3 = create_test_tx(2, 2_000_000_000, 1000);
        
        assert!(validator.validate(&tx1).await.is_ok());
        assert!(validator.validate(&tx2).await.is_ok());
        assert!(matches!(
            validator.validate(&tx3).await,
            Err(ValidationError::RateLimitExceeded)
        ));
    }

    #[tokio::test]
    async fn test_rate_limit_resets_after_window() {
        let rules = ValidationRules {
            verify_signatures: false,
            check_balance: false,
            check_nonce: false,
            rate_limit: 1,
            rate_limit_window_secs: 1, // 1-second window
            ..Default::default()
        };
        let state_provider = Arc::new(MockStateProvider::new());
        let validator = TxValidator::new(rules, state_provider);

        let tx1 = create_test_tx(0, 2_000_000_000, 1000);
        let tx2 = create_test_tx(1, 2_000_000_000, 1000);

        assert!(validator.validate(&tx1).await.is_ok());
        assert!(matches!(
            validator.validate(&tx2).await,
            Err(ValidationError::RateLimitExceeded)
        ));

        // Wait for window reset
        tokio::time::sleep(std::time::Duration::from_millis(1100)).await;
        let tx3 = create_test_tx(2, 2_000_000_000, 1000);
        assert!(validator.validate(&tx3).await.is_ok());
    }
}
