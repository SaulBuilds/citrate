use crate::errors::WalletError;
use crate::keystore::KeyStore;
use crate::rpc_client::RpcClient;
use lattice_consensus::types::{Hash, PublicKey};
use lattice_execution::types::Address;
use primitive_types::U256;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Account information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Account {
    pub index: usize,
    pub address: Address,
    pub public_key: PublicKey,
    pub alias: Option<String>,
    pub balance: U256,
    pub nonce: u64,
}

/// Wallet configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletConfig {
    pub keystore_path: PathBuf,
    pub rpc_url: String,
    pub chain_id: u64,
    pub default_gas_price: u64,
    pub default_gas_limit: u64,
}

impl Default for WalletConfig {
    fn default() -> Self {
        let mut keystore_path = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
        keystore_path.push(".lattice");
        keystore_path.push("keystore.json");

        Self {
            keystore_path,
            rpc_url: "http://localhost:8545".to_string(),
            chain_id: 1337,
            default_gas_price: 1_000_000_000, // 1 gwei
            default_gas_limit: 21_000,
        }
    }
}

/// Main wallet structure
pub struct Wallet {
    config: WalletConfig,
    keystore: KeyStore,
    rpc_client: RpcClient,
    accounts: Vec<Account>,
}

impl Wallet {
    /// Create new wallet with config
    pub fn new(config: WalletConfig) -> Result<Self, WalletError> {
        // Ensure parent directory exists
        if let Some(parent) = config.keystore_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let keystore = KeyStore::new(&config.keystore_path)?;
        let rpc_client = RpcClient::new(&config.rpc_url);

        Ok(Self {
            config,
            keystore,
            rpc_client,
            accounts: Vec::new(),
        })
    }

    /// Create new account
    pub fn create_account(
        &mut self,
        password: &str,
        alias: Option<String>,
    ) -> Result<Account, WalletError> {
        let verifying_key = self.keystore.generate_key(password, alias.clone())?;

        let public_key = PublicKey::new(verifying_key.to_bytes());
        let address = Address::from_public_key(&public_key);

        let index = self.accounts.len();
        let account = Account {
            index,
            address,
            public_key,
            alias,
            balance: U256::zero(),
            nonce: 0,
        };

        self.accounts.push(account.clone());

        Ok(account)
    }

    /// Import account from private key
    pub fn import_account(
        &mut self,
        private_key_hex: &str,
        password: &str,
        alias: Option<String>,
    ) -> Result<Account, WalletError> {
        let verifying_key = self
            .keystore
            .import_key(private_key_hex, password, alias.clone())?;

        let public_key = PublicKey::new(verifying_key.to_bytes());
        let address = Address::from_public_key(&public_key);

        let index = self.accounts.len();
        let account = Account {
            index,
            address,
            public_key,
            alias,
            balance: U256::zero(),
            nonce: 0,
        };

        self.accounts.push(account.clone());

        Ok(account)
    }

    /// Unlock wallet
    pub fn unlock(&mut self, password: &str) -> Result<(), WalletError> {
        self.keystore.unlock(password)?;
        self.refresh_accounts()?;
        Ok(())
    }

    /// Lock wallet
    pub fn lock(&mut self) {
        self.keystore.lock();
    }

    /// Refresh account list from keystore
    pub fn refresh_accounts(&mut self) -> Result<(), WalletError> {
        self.accounts.clear();

        for (index, public_key_bytes, alias) in self.keystore.list_accounts() {
            let mut pk_array = [0u8; 32];
            pk_array.copy_from_slice(&public_key_bytes[..32.min(public_key_bytes.len())]);
            let public_key = PublicKey::new(pk_array);
            let address = Address::from_public_key(&public_key);

            self.accounts.push(Account {
                index,
                address,
                public_key,
                alias,
                balance: U256::zero(),
                nonce: 0,
            });
        }

        Ok(())
    }

    /// Update account balances from chain
    pub async fn update_balances(&mut self) -> Result<(), WalletError> {
        for account in &mut self.accounts {
            let balance = self.rpc_client.get_balance(&account.address).await?;
            let nonce = self.rpc_client.get_nonce(&account.address).await?;

            account.balance = balance;
            account.nonce = nonce;
        }

        Ok(())
    }

    /// Get account by index
    pub fn get_account(&self, index: usize) -> Option<&Account> {
        self.accounts.get(index)
    }

    /// Get account by address
    pub fn get_account_by_address(&self, address: &Address) -> Option<&Account> {
        self.accounts.iter().find(|a| a.address == *address)
    }

    /// List all accounts
    pub fn list_accounts(&self) -> &[Account] {
        &self.accounts
    }

    /// Send transaction
    pub async fn send_transaction(
        &self,
        from_index: usize,
        to: Address,
        value: U256,
        data: Vec<u8>,
        gas_price: Option<u64>,
        gas_limit: Option<u64>,
    ) -> Result<Hash, WalletError> {
        // Get account
        let account = self
            .get_account(from_index)
            .ok_or_else(|| WalletError::AccountNotFound(format!("Index {}", from_index)))?;

        // Check balance
        let gas_price = gas_price.unwrap_or(self.config.default_gas_price);
        let gas_limit = gas_limit.unwrap_or(self.config.default_gas_limit);
        let gas_cost = U256::from(gas_price) * U256::from(gas_limit);
        let total_cost = value + gas_cost;

        if account.balance < total_cost {
            return Err(WalletError::InsufficientBalance {
                need: format_latt(total_cost),
                have: format_latt(account.balance),
            });
        }

        // Get signing key
        let signing_key = self.keystore.get_signing_key(from_index)?;

        // Build and sign transaction
        let tx = crate::transaction::TransactionBuilder::new()
            .from(account.public_key)
            .to(Some(to))
            .value(value)
            .data(data)
            .nonce(account.nonce)
            .gas_price(gas_price)
            .gas_limit(gas_limit)
            .chain_id(self.config.chain_id)
            .build_and_sign(signing_key)?;

        // Send transaction
        let tx_hash = self.rpc_client.send_transaction(tx).await?;

        Ok(tx_hash)
    }

    /// Transfer tokens
    pub async fn transfer(
        &self,
        from_index: usize,
        to: Address,
        amount: U256,
    ) -> Result<Hash, WalletError> {
        self.send_transaction(from_index, to, amount, Vec::new(), None, None)
            .await
    }

    /// Get transaction receipt
    pub async fn get_transaction_receipt(
        &self,
        tx_hash: &Hash,
    ) -> Result<Option<serde_json::Value>, WalletError> {
        self.rpc_client.get_transaction_receipt(tx_hash).await
    }

    /// Export private key
    pub fn export_private_key(&self, index: usize) -> Result<String, WalletError> {
        self.keystore.export_private_key(index)
    }

    /// Get config
    pub fn config(&self) -> &WalletConfig {
        &self.config
    }

    /// Get RPC client
    pub fn rpc_client(&self) -> &RpcClient {
        &self.rpc_client
    }
}

/// Format U256 as LATT with decimals
fn format_latt(value: U256) -> String {
    let decimals = U256::from(10).pow(U256::from(18));
    let whole = value / decimals;
    let fraction = value % decimals;

    // Format with up to 6 decimal places
    let fraction_str = format!("{:018}", fraction);
    let fraction_trimmed = fraction_str[..6].trim_end_matches('0');

    if fraction_trimmed.is_empty() {
        format!("{}", whole)
    } else {
        format!("{}.{}", whole, fraction_trimmed)
    }
}
