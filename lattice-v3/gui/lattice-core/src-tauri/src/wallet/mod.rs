use std::sync::Arc;
use tokio::sync::RwLock;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use ed25519_dalek::{SigningKey, VerifyingKey, Signer, Signature as Ed25519Signature};
use keyring::Entry;
use argon2::{
    password_hash::{PasswordHash, PasswordHasher, SaltString},
    Argon2,
};
use aes_gcm::{
    aead::{Aead, AeadCore, KeyInit},
    Aes256Gcm, Nonce, Key,
};
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use rand::rngs::OsRng;
use rand::RngCore;
use lattice_consensus::types::{Transaction, Hash, PublicKey, Signature};
use tracing::{info, error};
use bip39::{Mnemonic, Language};

const KEYRING_SERVICE: &str = "lattice-core";
const KEYRING_USER: &str = "wallet";

/// Secure wallet manager with OS keychain integration
pub struct WalletManager {
    accounts: Arc<RwLock<Vec<Account>>>,
    keystore: Arc<SecureKeyStore>,
    active_account: Arc<RwLock<Option<usize>>>,
}

impl WalletManager {
    pub fn new() -> Result<Self> {
        let keystore = Arc::new(SecureKeyStore::new()?);
        let accounts = Arc::new(RwLock::new(Self::load_accounts(&keystore)?));
        
        Ok(Self {
            accounts,
            keystore,
            active_account: Arc::new(RwLock::new(None)),
        })
    }

    pub async fn create_account(&self, label: String, password: &str) -> Result<Account> {
        // Generate 128-bit entropy and build a 12-word mnemonic
        let mut entropy = [0u8; 16];
        OsRng.fill_bytes(&mut entropy);
        let mnemonic = Mnemonic::from_entropy_in(Language::English, &entropy)?;
        // Derive a deterministic 32-byte key from the mnemonic phrase
        let mut sk_bytes = [0u8; 32];
        {
            use sha3::{Digest, Keccak256};
            let mut hasher = Keccak256::new();
            hasher.update(mnemonic.to_string().as_bytes());
            let digest = hasher.finalize();
            sk_bytes.copy_from_slice(&digest[..32]);
        }
        let signing_key = SigningKey::from_bytes(&sk_bytes);
        let verifying_key = signing_key.verifying_key();
        
        // Derive address from public key
        let address = self.derive_address(&verifying_key);
        
        // Store encrypted private key in OS keychain
        self.keystore.store_key(&address, &signing_key, password)?;
        // Verify roundtrip immediately to guarantee correct format
        let _ = self.keystore.get_key(&address, password)?;
        
        // Create account
        let account = Account {
            address: address.clone(),
            label,
            public_key: hex::encode(verifying_key.as_bytes()),
            balance: 0,
            nonce: 0,
            created_at: chrono::Utc::now().timestamp() as u64,
        };
        
        // Add to accounts list
        self.accounts.write().await.push(account.clone());
        self.save_accounts().await?;
        
        info!("Created new account: {}", address);
        Ok(account)
    }

    /// Create account and return credentials (mnemonic & private key)
    pub async fn create_account_with_credentials(&self, label: String, password: &str) -> Result<(Account, String, String)> {
        let mut entropy = [0u8; 16];
        OsRng.fill_bytes(&mut entropy);
        let mnemonic = Mnemonic::from_entropy_in(Language::English, &entropy)?;
        let mut sk_bytes = [0u8; 32];
        {
            use sha3::{Digest, Keccak256};
            let mut hasher = Keccak256::new();
            hasher.update(mnemonic.to_string().as_bytes());
            let digest = hasher.finalize();
            sk_bytes.copy_from_slice(&digest[..32]);
        }
        let signing_key = SigningKey::from_bytes(&sk_bytes);
        let verifying_key = signing_key.verifying_key();
        let address = self.derive_address(&verifying_key);

        self.keystore.store_key(&address, &signing_key, password)?;
        let _ = self.keystore.get_key(&address, password)?;

        let account = Account {
            address: address.clone(),
            label,
            public_key: hex::encode(verifying_key.as_bytes()),
            balance: 0,
            nonce: 0,
            created_at: chrono::Utc::now().timestamp() as u64,
        };
        self.accounts.write().await.push(account.clone());
        self.save_accounts().await?;

        Ok((account, hex::encode(signing_key.to_bytes()), mnemonic.to_string()))
    }

    pub async fn import_account(&self, private_key: &str, label: String, password: &str) -> Result<Account> {
        // Parse private key
        let key_bytes = hex::decode(private_key)?;
        if key_bytes.len() != 32 {
            return Err(anyhow::anyhow!("Invalid private key length"));
        }
        
        let signing_key = SigningKey::from_bytes(&key_bytes.try_into().unwrap());
        let verifying_key = signing_key.verifying_key();
        
        // Derive address
        let address = self.derive_address(&verifying_key);
        
        // Check if account already exists
        if self.accounts.read().await.iter().any(|a| a.address == address) {
            return Err(anyhow::anyhow!("Account already exists"));
        }
        
        // Store in keychain
        self.keystore.store_key(&address, &signing_key, password)?;
        // Verify roundtrip immediately
        let _ = self.keystore.get_key(&address, password)?;
        
        // Create account
        let account = Account {
            address: address.clone(),
            label,
            public_key: hex::encode(verifying_key.as_bytes()),
            balance: 0,
            nonce: 0,
            created_at: chrono::Utc::now().timestamp() as u64,
        };
        
        // Add to accounts list
        self.accounts.write().await.push(account.clone());
        self.save_accounts().await?;
        
        info!("Imported account: {}", address);
        Ok(account)
    }

    pub async fn import_account_from_mnemonic(&self, mnemonic_phrase: &str, label: String, password: &str) -> Result<Account> {
        let mnemonic = Mnemonic::parse_in_normalized(Language::English, mnemonic_phrase)
            .map_err(|e| anyhow::anyhow!("Invalid mnemonic: {}", e))?;
        let mut sk_bytes = [0u8; 32];
        {
            use sha3::{Digest, Keccak256};
            let mut hasher = Keccak256::new();
            hasher.update(mnemonic.to_string().as_bytes());
            let digest = hasher.finalize();
            sk_bytes.copy_from_slice(&digest[..32]);
        }
        let signing_key = SigningKey::from_bytes(&sk_bytes);
        let verifying_key = signing_key.verifying_key();
        let address = self.derive_address(&verifying_key);
        if self.accounts.read().await.iter().any(|a| a.address == address) {
            return Err(anyhow::anyhow!("Account already exists"));
        }
        self.keystore.store_key(&address, &signing_key, password)?;
        let _ = self.keystore.get_key(&address, password)?;
        let account = Account {
            address: address.clone(),
            label,
            public_key: hex::encode(verifying_key.as_bytes()),
            balance: 0,
            nonce: 0,
            created_at: chrono::Utc::now().timestamp() as u64,
        };
        self.accounts.write().await.push(account.clone());
        self.save_accounts().await?;
        Ok(account)
    }

    pub async fn export_private_key(&self, address: &str, password: &str) -> Result<String> {
        let signing_key = self.keystore.get_key(address, password)?;
        Ok(hex::encode(signing_key.to_bytes()))
    }

    pub async fn get_accounts(&self) -> Vec<Account> {
        self.accounts.read().await.clone()
    }

    pub async fn get_account(&self, address: &str) -> Option<Account> {
        self.accounts.read().await
            .iter()
            .find(|a| a.address == address)
            .cloned()
    }

    pub async fn send_transaction(&self, request: TransactionRequest, password: &str) -> Result<String> {
        let tx = self.create_signed_transaction(request, password).await?;
        let tx_hash = hex::encode(tx.hash.as_bytes());
        info!("Transaction sent: {}", tx_hash);
        Ok(tx_hash)
    }

    /// Create and sign a transaction, update nonce, and return the full Transaction
    pub async fn create_signed_transaction(&self, request: TransactionRequest, password: &str) -> Result<Transaction> {
        // Get account
        let account = self.get_account(&request.from).await
            .ok_or_else(|| anyhow::anyhow!("Account not found"))?;
        
        // Create transaction
        // Parse numeric fields from strings
        let value_u128: u128 = request.value.parse().unwrap_or(0);
        let gas_price_u64: u64 = request.gas_price.parse().unwrap_or(0);

        let mut tx = Transaction {
            hash: Hash::new([0u8; 32]), // Will be computed after signing
            nonce: account.nonce,
            from: PublicKey::new([0u8; 32]), // Will be set during signing
            to: request.to.map(|addr| {
                let mut bytes = [0u8; 32];
                hex::decode(addr.trim_start_matches("0x"))
                    .unwrap_or_default()
                    .iter()
                    .take(32)
                    .enumerate()
                    .for_each(|(i, b)| bytes[i] = *b);
                PublicKey::new(bytes)
            }),
            value: value_u128,
            gas_limit: request.gas_limit,
            gas_price: gas_price_u64,
            data: hex::decode(&request.data.trim_start_matches("0x")).unwrap_or_default(),
            signature: Signature::new([0u8; 64]),
            tx_type: None,
        };
        
        // Sign transaction
        self.sign_transaction(&mut tx, &request.from, &password).await?;
        
        // Update nonce
        self.update_nonce(&request.from, account.nonce + 1).await?;
        Ok(tx)
    }
    
    pub async fn sign_transaction(&self, tx: &mut Transaction, address: &str, password: &str) -> Result<()> {
        // Get private key from keystore
        let signing_key = self.keystore.get_key(address, password)?;
        
        // Build canonical bytes and sign them
        let msg = self.canonical_tx_bytes(tx);
        let signature = signing_key.sign(&msg);
        
        // Update transaction
        tx.signature = Signature::new(signature.to_bytes());
        tx.from = PublicKey::new(signing_key.verifying_key().to_bytes());
        
        // Update hash (Keccak of canonical bytes for id/display)
        {
            use sha3::{Digest, Keccak256};
            let mut hasher = Keccak256::new();
            hasher.update(&msg);
            let digest = hasher.finalize();
            tx.hash = Hash::from_bytes(&digest);
        }
        
        Ok(())
    }

    pub async fn sign_message(&self, message: &[u8], address: &str, password: &str) -> Result<String> {
        let signing_key = self.keystore.get_key(address, password)?;
        let signature = signing_key.sign(message);
        Ok(hex::encode(signature.to_bytes()))
    }

    pub async fn verify_signature(&self, message: &[u8], signature: &str, address: &str) -> Result<bool> {
        let account = self.get_account(address).await
            .ok_or_else(|| anyhow::anyhow!("Account not found"))?;
        
        let public_key_bytes = hex::decode(&account.public_key)?;
        let verifying_key = VerifyingKey::from_bytes(&public_key_bytes.try_into().unwrap())?;
        
        let signature_bytes = hex::decode(signature)?;
        let signature = Ed25519Signature::from_bytes(&signature_bytes.try_into().unwrap());
        
        Ok(verifying_key.verify_strict(message, &signature).is_ok())
    }

    pub async fn update_balance(&self, address: &str, balance: u128) -> Result<()> {
        let mut accounts = self.accounts.write().await;
        if let Some(account) = accounts.iter_mut().find(|a| a.address == address) {
            account.balance = balance;
            drop(accounts);
            self.save_accounts().await?;
        }
        Ok(())
    }

    pub async fn update_nonce(&self, address: &str, nonce: u64) -> Result<()> {
        let mut accounts = self.accounts.write().await;
        if let Some(account) = accounts.iter_mut().find(|a| a.address == address) {
            account.nonce = nonce;
            drop(accounts);
            self.save_accounts().await?;
        }
        Ok(())
    }

    fn derive_address(&self, public_key: &VerifyingKey) -> String {
        // Use keccak256 hash of public key for Ethereum-compatible address
        use sha3::{Digest, Keccak256};
        let hash = Keccak256::digest(public_key.as_bytes());
        format!("0x{}", hex::encode(&hash[12..]))
    }

    fn canonical_tx_bytes(&self, tx: &Transaction) -> Vec<u8> {
        let mut buf = Vec::new();
        buf.extend_from_slice(&tx.nonce.to_le_bytes());
        buf.extend_from_slice(tx.from.as_bytes());
        if let Some(to) = &tx.to {
            buf.push(1);
            buf.extend_from_slice(to.as_bytes());
        } else {
            buf.push(0);
        }
        buf.extend_from_slice(&tx.value.to_le_bytes());
        buf.extend_from_slice(&tx.gas_limit.to_le_bytes());
        buf.extend_from_slice(&tx.gas_price.to_le_bytes());
        buf.extend_from_slice(&(tx.data.len() as u32).to_le_bytes());
        buf.extend_from_slice(&tx.data);
        buf
    }

    fn load_accounts(_keystore: &SecureKeyStore) -> Result<Vec<Account>> {
        let accounts_path = Self::accounts_path();
        if accounts_path.exists() {
            let accounts_str = std::fs::read_to_string(accounts_path)?;
            Ok(serde_json::from_str(&accounts_str)?)
        } else {
            Ok(Vec::new())
        }
    }

    async fn save_accounts(&self) -> Result<()> {
        let accounts_path = Self::accounts_path();
        if let Some(parent) = accounts_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let accounts = self.accounts.read().await.clone();
        let accounts_str = serde_json::to_string_pretty(&accounts)?;
        std::fs::write(accounts_path, accounts_str)?;
        Ok(())
    }

    fn accounts_path() -> std::path::PathBuf {
        dirs::data_dir()
            .unwrap_or_else(|| std::path::PathBuf::from("."))
            .join("lattice-core")
            .join("accounts.json")
    }
}

/// Secure key storage using OS keychain and encryption
struct SecureKeyStore {
    entry: Entry,
}

impl SecureKeyStore {
    fn new() -> Result<Self> {
        let entry = Entry::new(KEYRING_SERVICE, KEYRING_USER)?;
        Ok(Self { entry })
    }

    fn store_key(&self, address: &str, signing_key: &SigningKey, password: &str) -> Result<()> {
        // Derive encryption key from password with a per-key random salt
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();
        let password_hash = argon2
            .hash_password(password.as_bytes(), &salt)
            .map_err(|e| anyhow::anyhow!("Failed to hash password: {}", e))?;
        let hash_output = password_hash
            .hash
            .ok_or_else(|| anyhow::anyhow!("Argon2 produced no hash output"))?;
        let key_bytes = hash_output.as_bytes();
        let key = Key::<Aes256Gcm>::from_slice(&key_bytes[..32]);

        // Encrypt private key
        let cipher = Aes256Gcm::new(key);
        let nonce = Aes256Gcm::generate_nonce(&mut OsRng);
        let ciphertext = cipher
            .encrypt(&nonce, signing_key.to_bytes().as_ref())
            .map_err(|e| anyhow::anyhow!("Encryption failed: {}", e))?;

        // Store as JSON so we keep the salt and nonce
        #[derive(Serialize)]
        struct StoredKey<'a> {
            v: u8,
            salt: &'a str,        // PHC salt string
            nonce: String,        // base64
            ct: String,           // base64
        }
        let record = StoredKey {
            v: 1,
            salt: salt.as_str(),
            nonce: BASE64.encode(&nonce),
            ct: BASE64.encode(&ciphertext),
        };
        let encoded = serde_json::to_string(&record)?;

        // Create a unique entry for this address
        let entry = Entry::new(KEYRING_SERVICE, &format!("wallet_{}", address))?;
        entry.set_password(&encoded)?;

        Ok(())
    }

    fn get_key(&self, address: &str, password: &str) -> Result<SigningKey> {
        // Retrieve from keychain using address-specific entry
        let entry = Entry::new(KEYRING_SERVICE, &format!("wallet_{}", address))?;
        let stored = entry
            .get_password()
            .map_err(|_| anyhow::anyhow!("Key not found for address"))?;

        // Try JSON format first
        #[derive(Deserialize)]
        struct StoredKeyOwned {
            v: u8,
            salt: String,
            nonce: String,
            ct: String,
        }
        if let Ok(record) = serde_json::from_str::<StoredKeyOwned>(&stored) {
            // Derive key using stored salt
            let argon2 = Argon2::default();
            let salt = SaltString::from_b64(&record.salt)
                .map_err(|e| anyhow::anyhow!("Invalid stored salt: {}", e))?;
            let password_hash = argon2
                .hash_password(password.as_bytes(), &salt)
                .map_err(|e| anyhow::anyhow!("Failed to hash password: {}", e))?;
            let hash_output = password_hash
                .hash
                .ok_or_else(|| anyhow::anyhow!("Argon2 produced no hash output"))?;
            let key_bytes = hash_output.as_bytes();
            let key = Key::<Aes256Gcm>::from_slice(&key_bytes[..32]);

            // Decrypt
            let nonce_bytes = BASE64
                .decode(&record.nonce)
                .map_err(|_| anyhow::anyhow!("Invalid stored nonce"))?;
            if nonce_bytes.len() != 12 {
                return Err(anyhow::anyhow!("Invalid nonce length"));
            }
            let nonce = Nonce::from_slice(&nonce_bytes);
            let ciphertext = BASE64
                .decode(&record.ct)
                .map_err(|_| anyhow::anyhow!("Invalid stored ciphertext"))?;

            let cipher = Aes256Gcm::new(key);
            let plaintext = cipher
                .decrypt(nonce, ciphertext.as_ref())
                .map_err(|_| anyhow::anyhow!("Invalid password"))?;
            if plaintext.len() != 32 {
                return Err(anyhow::anyhow!("Invalid key data"));
            }
            return Ok(SigningKey::from_bytes(&plaintext.try_into().unwrap()));
        }

        // Legacy format fallback (nonce||ciphertext base64) - cannot verify without salt
        Err(anyhow::anyhow!(
            "Stored key uses legacy format; please re-import or re-create the account"
        ))
    }

    fn delete_key(&self, address: &str) -> Result<()> {
        // Delete the address-specific entry
        let entry = Entry::new(KEYRING_SERVICE, &format!("wallet_{}", address))?;
        entry.delete_password()
            .map_err(|e| anyhow::anyhow!("Failed to delete key: {}", e))?;
        info!("Deleted key for address: {}", address);
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Account {
    pub address: String,
    pub label: String,
    pub public_key: String,
    #[serde(serialize_with = "serialize_u128", deserialize_with = "deserialize_u128")]
    pub balance: u128,
    pub nonce: u64,
    pub created_at: u64,
}

// Custom serializer for u128 to string
fn serialize_u128<S>(value: &u128, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    serializer.serialize_str(&value.to_string())
}

fn deserialize_u128<'de, D>(deserializer: D) -> Result<u128, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    s.parse::<u128>().map_err(serde::de::Error::custom)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TransactionRequest {
    pub from: String,
    pub to: Option<String>,
    // Accept large integers as decimal strings for JSON compatibility
    pub value: String,
    pub gas_limit: u64,
    pub gas_price: String,
    pub data: String,
}
