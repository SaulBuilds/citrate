use aes_gcm::{
    aead::{Aead, AeadCore, KeyInit},
    Aes256Gcm, Key, Nonce,
};
use anyhow::Result;
use argon2::{
    password_hash::{PasswordHasher, SaltString},
    Argon2,
};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};
use bip39::{Language, Mnemonic};
use ed25519_dalek::{Signature as Ed25519Signature, Signer, SigningKey, VerifyingKey};
use hmac::{Hmac, Mac};
use keyring::Entry;
use citrate_consensus::types::{Hash, PublicKey, Signature, Transaction};
use rand::rngs::OsRng;
use rand::RngCore;
use serde::{Deserialize, Serialize};
use sha2::Sha512;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tracing::{info, warn, error};

const KEYRING_SERVICE: &str = "citrate-core";
const KEYRING_USER: &str = "wallet";

// BIP32/BIP44 constants
const BIP44_PURPOSE: u32 = 44;
const BIP44_COIN_TYPE_ED25519: u32 = 501; // Ed25519-based (similar to Solana)
const HARDENED_OFFSET: u32 = 0x80000000;

// Security constants for rate limiting and session management
const MAX_FAILED_ATTEMPTS: u32 = 5;           // Maximum failed password attempts before lockout
const LOCKOUT_DURATION_SECS: u64 = 300;       // 5 minutes lockout after max failures
const RATE_LIMIT_WINDOW_SECS: u64 = 60;       // 1 minute sliding window for rate limiting
const MAX_OPERATIONS_PER_WINDOW: u32 = 10;    // Max sensitive operations per window
const SESSION_TIMEOUT_SECS: u64 = 900;        // 15 minute session timeout for unlocked wallet
const REAUTH_THRESHOLD_SALT: u128 = 10_000_000_000_000_000_000; // 10 SALT - high-value tx threshold

/// Operation types for rate limiting
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SensitiveOperation {
    PasswordAttempt,
    KeyExport,
    SignTransaction,
    SignMessage,
    DeleteAccount,
}

impl std::fmt::Display for SensitiveOperation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SensitiveOperation::PasswordAttempt => write!(f, "password_attempt"),
            SensitiveOperation::KeyExport => write!(f, "key_export"),
            SensitiveOperation::SignTransaction => write!(f, "sign_transaction"),
            SensitiveOperation::SignMessage => write!(f, "sign_message"),
            SensitiveOperation::DeleteAccount => write!(f, "delete_account"),
        }
    }
}

/// Rate limiter for sensitive operations
/// Uses sliding window algorithm to prevent brute force attacks
pub struct RateLimiter {
    // Operation timestamps for sliding window
    operation_times: HashMap<(String, SensitiveOperation), Vec<Instant>>,
    // Failed password attempts per address with lockout tracking
    failed_attempts: HashMap<String, (u32, Option<Instant>)>,
}

impl RateLimiter {
    pub fn new() -> Self {
        Self {
            operation_times: HashMap::new(),
            failed_attempts: HashMap::new(),
        }
    }

    /// Check if an operation is allowed under rate limiting rules
    pub fn check_rate_limit(&mut self, address: &str, op: SensitiveOperation) -> Result<(), String> {
        let key = (address.to_string(), op);
        let now = Instant::now();
        let window = Duration::from_secs(RATE_LIMIT_WINDOW_SECS);

        // Clean up old timestamps outside the window
        if let Some(times) = self.operation_times.get_mut(&key) {
            times.retain(|t| now.duration_since(*t) < window);
        }

        // Check current count
        let times = self.operation_times.entry(key.clone()).or_insert_with(Vec::new);
        if times.len() >= MAX_OPERATIONS_PER_WINDOW as usize {
            let oldest = times.first().copied();
            let wait_time = oldest
                .map(|t| window.saturating_sub(now.duration_since(t)))
                .unwrap_or(Duration::ZERO);
            return Err(format!(
                "Rate limit exceeded for {}. Please wait {} seconds.",
                op,
                wait_time.as_secs()
            ));
        }

        // Record this operation
        times.push(now);
        Ok(())
    }

    /// Record a failed password attempt and check for lockout
    pub fn record_failed_attempt(&mut self, address: &str) -> Result<(), String> {
        let now = Instant::now();
        let (count, lockout_until) = self.failed_attempts
            .entry(address.to_string())
            .or_insert((0, None));

        // Check if currently locked out
        if let Some(until) = lockout_until {
            if now < *until {
                let remaining = until.duration_since(now).as_secs();
                return Err(format!(
                    "Account locked due to too many failed attempts. Please wait {} seconds.",
                    remaining
                ));
            } else {
                // Lockout expired, reset
                *count = 0;
                *lockout_until = None;
            }
        }

        // Increment failed count
        *count += 1;
        warn!("Failed password attempt {} of {} for address {}", count, MAX_FAILED_ATTEMPTS, address);

        if *count >= MAX_FAILED_ATTEMPTS {
            let lockout_time = now + Duration::from_secs(LOCKOUT_DURATION_SECS);
            *lockout_until = Some(lockout_time);
            error!("Account {} locked for {} seconds due to {} failed attempts",
                   address, LOCKOUT_DURATION_SECS, count);
            return Err(format!(
                "Account locked due to {} failed attempts. Please wait {} seconds.",
                MAX_FAILED_ATTEMPTS,
                LOCKOUT_DURATION_SECS
            ));
        }

        Ok(())
    }

    /// Reset failed attempts on successful authentication
    pub fn reset_failed_attempts(&mut self, address: &str) {
        self.failed_attempts.remove(address);
    }

    /// Check if an address is currently locked out
    pub fn is_locked_out(&self, address: &str) -> bool {
        if let Some((count, lockout_until)) = self.failed_attempts.get(address) {
            if *count >= MAX_FAILED_ATTEMPTS {
                if let Some(until) = lockout_until {
                    return Instant::now() < *until;
                }
            }
        }
        false
    }

    /// Get remaining lockout time in seconds
    pub fn get_lockout_remaining(&self, address: &str) -> Option<u64> {
        if let Some((count, Some(until))) = self.failed_attempts.get(address) {
            if *count >= MAX_FAILED_ATTEMPTS {
                let now = Instant::now();
                if now < *until {
                    return Some(until.duration_since(now).as_secs());
                }
            }
        }
        None
    }
}

/// Session manager for wallet unlock state
/// Tracks when wallet was unlocked and enforces timeouts
pub struct SessionManager {
    // Address -> (unlock time, last activity time)
    sessions: HashMap<String, (Instant, Instant)>,
}

impl SessionManager {
    pub fn new() -> Self {
        Self {
            sessions: HashMap::new(),
        }
    }

    /// Create a new session for an address
    pub fn create_session(&mut self, address: &str) {
        let now = Instant::now();
        self.sessions.insert(address.to_string(), (now, now));
        info!("Created session for address: {}", address);
    }

    /// Update the last activity time for a session
    pub fn touch_session(&mut self, address: &str) {
        if let Some((_, last_activity)) = self.sessions.get_mut(address) {
            *last_activity = Instant::now();
        }
    }

    /// Check if a session is valid (not timed out)
    pub fn is_session_valid(&self, address: &str) -> bool {
        if let Some((_, last_activity)) = self.sessions.get(address) {
            let elapsed = Instant::now().duration_since(*last_activity);
            return elapsed.as_secs() < SESSION_TIMEOUT_SECS;
        }
        false
    }

    /// End a session (lock the wallet)
    pub fn end_session(&mut self, address: &str) {
        self.sessions.remove(address);
        info!("Ended session for address: {}", address);
    }

    /// Get remaining session time in seconds
    pub fn get_session_remaining(&self, address: &str) -> Option<u64> {
        if let Some((_, last_activity)) = self.sessions.get(address) {
            let elapsed = Instant::now().duration_since(*last_activity);
            let timeout = Duration::from_secs(SESSION_TIMEOUT_SECS);
            if elapsed < timeout {
                return Some((timeout - elapsed).as_secs());
            }
        }
        None
    }

    /// Clean up expired sessions
    pub fn cleanup_expired(&mut self) {
        let now = Instant::now();
        let timeout = Duration::from_secs(SESSION_TIMEOUT_SECS);
        self.sessions.retain(|addr, (_, last_activity)| {
            let valid = now.duration_since(*last_activity) < timeout;
            if !valid {
                info!("Session expired for address: {}", addr);
            }
            valid
        });
    }
}

/// Re-authentication requirement check
pub struct ReauthChecker;

impl ReauthChecker {
    /// Check if a transaction requires re-authentication
    /// High-value transactions and sensitive operations require fresh password
    pub fn requires_reauth(value: u128, op: SensitiveOperation) -> bool {
        match op {
            SensitiveOperation::KeyExport => true,
            SensitiveOperation::DeleteAccount => true,
            SensitiveOperation::SignTransaction => value >= REAUTH_THRESHOLD_SALT,
            _ => false,
        }
    }

    /// Get the threshold amount that requires re-auth (in SALT)
    pub fn get_reauth_threshold() -> u128 {
        REAUTH_THRESHOLD_SALT
    }
}

/// Password strength requirements
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PasswordStrength {
    pub is_valid: bool,
    pub score: u8, // 0-100
    pub issues: Vec<String>,
}

/// Validates password strength according to security requirements
/// Requirements:
/// - Minimum 12 characters
/// - At least one uppercase letter
/// - At least one lowercase letter
/// - At least one digit
/// - At least one special character
pub fn validate_password_strength(password: &str) -> PasswordStrength {
    let mut issues = Vec::new();
    let mut score: u8 = 0;

    // Length check (minimum 12 characters)
    let len = password.len();
    if len < 12 {
        issues.push(format!("Password must be at least 12 characters (currently {})", len));
    } else {
        score += 25;
        // Bonus for longer passwords
        if len >= 16 {
            score += 10;
        }
        if len >= 20 {
            score += 5;
        }
    }

    // Uppercase check
    if !password.chars().any(|c| c.is_ascii_uppercase()) {
        issues.push("Password must contain at least one uppercase letter".to_string());
    } else {
        score += 15;
    }

    // Lowercase check
    if !password.chars().any(|c| c.is_ascii_lowercase()) {
        issues.push("Password must contain at least one lowercase letter".to_string());
    } else {
        score += 15;
    }

    // Digit check
    if !password.chars().any(|c| c.is_ascii_digit()) {
        issues.push("Password must contain at least one digit".to_string());
    } else {
        score += 15;
    }

    // Special character check
    let special_chars = "!@#$%^&*()_+-=[]{}|;':\",./<>?`~";
    if !password.chars().any(|c| special_chars.contains(c)) {
        issues.push("Password must contain at least one special character (!@#$%^&*...)".to_string());
    } else {
        score += 20;
    }

    // Check for common weak patterns
    let password_lower = password.to_lowercase();
    let weak_patterns = [
        "password", "123456", "qwerty", "admin", "letmein", "welcome",
        "monkey", "dragon", "master", "login", "abc123", "111111"
    ];

    for pattern in &weak_patterns {
        if password_lower.contains(pattern) {
            issues.push(format!("Password contains common weak pattern: '{}'", pattern));
            score = score.saturating_sub(20);
        }
    }

    // Check for sequential characters
    let has_sequential = password.as_bytes().windows(3).any(|w| {
        (w[0] as i16 + 1 == w[1] as i16 && w[1] as i16 + 1 == w[2] as i16) ||
        (w[0] as i16 - 1 == w[1] as i16 && w[1] as i16 - 1 == w[2] as i16)
    });

    if has_sequential {
        issues.push("Password should avoid sequential characters (abc, 123, etc.)".to_string());
        score = score.saturating_sub(10);
    }

    PasswordStrength {
        is_valid: issues.is_empty(),
        score: score.min(100),
        issues,
    }
}

/// BIP32 Ed25519 key derivation following SLIP-0010
/// Derives an Ed25519 signing key from a BIP39 seed using hierarchical deterministic derivation
fn derive_ed25519_from_seed(seed: &[u8], path: &[u32]) -> Result<SigningKey> {
    // SLIP-0010: Use HMAC-SHA512 with "ed25519 seed" as key
    type HmacSha512 = Hmac<Sha512>;

    let mut mac = <HmacSha512 as Mac>::new_from_slice(b"ed25519 seed")
        .map_err(|e| anyhow::anyhow!("HMAC initialization failed: {}", e))?;
    mac.update(seed);
    let result = mac.finalize().into_bytes();

    let mut key = [0u8; 32];
    let mut chain_code = [0u8; 32];
    key.copy_from_slice(&result[..32]);
    chain_code.copy_from_slice(&result[32..]);

    // Derive through path (all hardened for Ed25519 per SLIP-0010)
    for &index in path {
        let hardened_index = index | HARDENED_OFFSET;

        let mut mac = <HmacSha512 as Mac>::new_from_slice(&chain_code)
            .map_err(|e| anyhow::anyhow!("HMAC initialization failed: {}", e))?;

        // For Ed25519, always use hardened derivation: 0x00 || key || index
        mac.update(&[0x00]);
        mac.update(&key);
        mac.update(&hardened_index.to_be_bytes());

        let result = mac.finalize().into_bytes();
        key.copy_from_slice(&result[..32]);
        chain_code.copy_from_slice(&result[32..]);
    }

    Ok(SigningKey::from_bytes(&key))
}

/// Derives an Ed25519 key using BIP44 path: m/44'/501'/account'/0'/0'
/// Uses coin type 501 (similar to Solana for Ed25519-based chains)
fn derive_bip44_ed25519(seed: &[u8], account: u32) -> Result<SigningKey> {
    // BIP44 path: m/44'/501'/account'/0'/0'
    // All indices are hardened for Ed25519
    let path = [
        BIP44_PURPOSE,          // 44'
        BIP44_COIN_TYPE_ED25519, // 501' (Ed25519 coin type)
        account,                 // account'
        0,                       // 0' (external chain)
        0,                       // 0' (address index)
    ];

    derive_ed25519_from_seed(seed, &path)
}

/// Result of first-time wallet setup
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FirstTimeSetupResult {
    pub primary_address: String,
    pub mnemonic: String,
    pub warning_message: String,
}

/// Secure wallet manager with OS keychain integration
/// Includes rate limiting, session management, and re-authentication checks
pub struct WalletManager {
    accounts: Arc<RwLock<Vec<Account>>>,
    keystore: Arc<SecureKeyStore>,
    #[allow(dead_code)]
    active_account: Arc<RwLock<Option<usize>>>,
    rate_limiter: Arc<RwLock<RateLimiter>>,
    session_manager: Arc<RwLock<SessionManager>>,
}

impl WalletManager {
    pub fn new() -> Result<Self> {
        let keystore = Arc::new(SecureKeyStore::new()?);
        let accounts = Arc::new(RwLock::new(Self::load_accounts(&keystore)?));

        Ok(Self {
            accounts,
            keystore,
            active_account: Arc::new(RwLock::new(None)),
            rate_limiter: Arc::new(RwLock::new(RateLimiter::new())),
            session_manager: Arc::new(RwLock::new(SessionManager::new())),
        })
    }

    // ========== Security: Rate Limiting & Session Management ==========

    /// Check rate limit for a sensitive operation
    /// Returns an error if rate limit is exceeded
    pub async fn check_rate_limit(&self, address: &str, op: SensitiveOperation) -> Result<()> {
        let mut limiter = self.rate_limiter.write().await;
        limiter.check_rate_limit(address, op)
            .map_err(|e| anyhow::anyhow!("{}", e))
    }

    /// Record a failed password attempt (for brute force protection)
    pub async fn record_failed_password_attempt(&self, address: &str) -> Result<()> {
        let mut limiter = self.rate_limiter.write().await;
        limiter.record_failed_attempt(address)
            .map_err(|e| anyhow::anyhow!("{}", e))
    }

    /// Reset failed attempts after successful authentication
    pub async fn reset_failed_attempts(&self, address: &str) {
        let mut limiter = self.rate_limiter.write().await;
        limiter.reset_failed_attempts(address);
    }

    /// Check if an address is currently locked out
    pub async fn is_locked_out(&self, address: &str) -> bool {
        let limiter = self.rate_limiter.read().await;
        limiter.is_locked_out(address)
    }

    /// Get remaining lockout time in seconds
    pub async fn get_lockout_remaining(&self, address: &str) -> Option<u64> {
        let limiter = self.rate_limiter.read().await;
        limiter.get_lockout_remaining(address)
    }

    /// Create a session after successful unlock
    pub async fn create_session(&self, address: &str) {
        let mut session_mgr = self.session_manager.write().await;
        session_mgr.create_session(address);
    }

    /// Update session activity timestamp
    pub async fn touch_session(&self, address: &str) {
        let mut session_mgr = self.session_manager.write().await;
        session_mgr.touch_session(address);
    }

    /// Check if a session is valid
    pub async fn is_session_valid(&self, address: &str) -> bool {
        let session_mgr = self.session_manager.read().await;
        session_mgr.is_session_valid(address)
    }

    /// End a session (lock wallet)
    pub async fn lock_wallet(&self, address: &str) {
        let mut session_mgr = self.session_manager.write().await;
        session_mgr.end_session(address);
        info!("Wallet locked for address: {}", address);
    }

    /// Get remaining session time in seconds
    pub async fn get_session_remaining(&self, address: &str) -> Option<u64> {
        let session_mgr = self.session_manager.read().await;
        session_mgr.get_session_remaining(address)
    }

    /// Cleanup expired sessions
    pub async fn cleanup_expired_sessions(&self) {
        let mut session_mgr = self.session_manager.write().await;
        session_mgr.cleanup_expired();
    }

    /// Check if re-authentication is required for an operation
    pub fn requires_reauth(value: u128, op: SensitiveOperation) -> bool {
        ReauthChecker::requires_reauth(value, op)
    }

    /// Get the re-auth threshold amount
    pub fn get_reauth_threshold() -> u128 {
        ReauthChecker::get_reauth_threshold()
    }

    /// Authenticate and create session (validates password and creates session)
    pub async fn authenticate(&self, address: &str, password: &str) -> Result<()> {
        // Check if locked out
        if self.is_locked_out(address).await {
            if let Some(remaining) = self.get_lockout_remaining(address).await {
                return Err(anyhow::anyhow!(
                    "Account locked. Please wait {} seconds before trying again.",
                    remaining
                ));
            }
        }

        // Check rate limit for password attempts
        self.check_rate_limit(address, SensitiveOperation::PasswordAttempt).await?;

        // Try to get key (validates password)
        match self.keystore.get_key(address, password) {
            Ok(_) => {
                // Password correct - reset failed attempts and create session
                self.reset_failed_attempts(address).await;
                self.create_session(address).await;
                info!("Authentication successful for address: {}", address);
                Ok(())
            }
            Err(e) => {
                // Password incorrect - record failed attempt
                let _ = self.record_failed_password_attempt(address).await;
                Err(e)
            }
        }
    }

    /// Verify session or require re-authentication
    pub async fn verify_session_or_reauth(&self, address: &str, password: Option<&str>) -> Result<()> {
        if self.is_session_valid(address).await {
            self.touch_session(address).await;
            return Ok(());
        }

        // Session expired or doesn't exist - need password
        match password {
            Some(pwd) => self.authenticate(address, pwd).await,
            None => Err(anyhow::anyhow!(
                "Session expired. Please re-enter your password to continue."
            )),
        }
    }

    /// Check if this is the first time the wallet is being used
    pub async fn is_first_time_setup(&self) -> bool {
        let accounts = self.accounts.read().await;
        accounts.is_empty()
    }

    /// Perform first-time setup with secure key generation and reward address configuration
    pub async fn perform_first_time_setup(&self, password: &str) -> Result<FirstTimeSetupResult> {
        if !self.is_first_time_setup().await {
            return Err(anyhow::anyhow!("Wallet already has accounts"));
        }

        info!("Performing first-time wallet setup");

        // Create the primary account with credentials to get account and mnemonic
        let (primary_account, _private_key_hex, mnemonic) = self.create_account_with_credentials("Primary Account".to_string(), password).await?;

        // Set as active account
        {
            let mut active_account = self.active_account.write().await;
            *active_account = Some(0);
        }

        let setup_result = FirstTimeSetupResult {
            primary_address: primary_account.address.clone(),
            mnemonic,
            warning_message: "IMPORTANT: Save your recovery phrase securely. This is the ONLY way to recover your wallet if you lose access. Never share it with anyone.".to_string(),
        };

        info!("First-time setup completed. Primary address: {}", primary_account.address);

        Ok(setup_result)
    }

    /// Get the primary account address for reward configuration
    pub async fn get_primary_reward_address(&self) -> Option<String> {
        let accounts = self.accounts.read().await;
        accounts.first().map(|account| account.address.clone())
    }

    /// Validate password before wallet operations
    pub fn validate_password(password: &str) -> Result<()> {
        let strength = validate_password_strength(password);
        if !strength.is_valid {
            return Err(anyhow::anyhow!(
                "Password does not meet security requirements:\n- {}",
                strength.issues.join("\n- ")
            ));
        }
        Ok(())
    }

    /// Check password strength without failing
    pub fn check_password_strength(password: &str) -> PasswordStrength {
        validate_password_strength(password)
    }

    pub async fn create_account(&self, label: String, password: &str) -> Result<Account> {
        // Validate password strength
        Self::validate_password(password)?;

        // Generate 128-bit entropy and build a 12-word mnemonic
        let mut entropy = [0u8; 16];
        OsRng.fill_bytes(&mut entropy);
        let mnemonic = Mnemonic::from_entropy_in(Language::English, &entropy)?;

        // Derive key using BIP44/SLIP-0010 standard
        // Generate seed from mnemonic with empty passphrase
        let seed = mnemonic.to_seed("");

        // Get current account count for derivation index
        let account_index = self.accounts.read().await.len() as u32;

        // Derive Ed25519 key using BIP44 path: m/44'/501'/account'/0'/0'
        let signing_key = derive_bip44_ed25519(&seed, account_index)?;
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
    pub async fn create_account_with_credentials(
        &self,
        label: String,
        password: &str,
    ) -> Result<(Account, String, String)> {
        // Validate password strength
        Self::validate_password(password)?;

        // Generate 128-bit entropy and build a 12-word mnemonic
        let mut entropy = [0u8; 16];
        OsRng.fill_bytes(&mut entropy);
        let mnemonic = Mnemonic::from_entropy_in(Language::English, &entropy)?;

        // Derive key using BIP44/SLIP-0010 standard
        let seed = mnemonic.to_seed("");

        // Get current account count for derivation index
        let account_index = self.accounts.read().await.len() as u32;

        // Derive Ed25519 key using BIP44 path: m/44'/501'/account'/0'/0'
        let signing_key = derive_bip44_ed25519(&seed, account_index)?;
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

        info!("Created account with BIP44 derivation: {}", address);

        Ok((
            account,
            hex::encode(signing_key.to_bytes()),
            mnemonic.to_string(),
        ))
    }

    pub async fn import_account(
        &self,
        private_key: &str,
        label: String,
        password: &str,
    ) -> Result<Account> {
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
        if self
            .accounts
            .read()
            .await
            .iter()
            .any(|a| a.address == address)
        {
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

    pub async fn import_account_from_mnemonic(
        &self,
        mnemonic_phrase: &str,
        label: String,
        password: &str,
    ) -> Result<Account> {
        // Validate password strength
        Self::validate_password(password)?;

        let mnemonic = Mnemonic::parse_in_normalized(Language::English, mnemonic_phrase)
            .map_err(|e| anyhow::anyhow!("Invalid mnemonic: {}", e))?;

        // Use BIP44/SLIP-0010 standard derivation
        let seed = mnemonic.to_seed("");

        // For imports, use account index 0 (primary account)
        // This ensures the same mnemonic always produces the same address
        let signing_key = derive_bip44_ed25519(&seed, 0)?;
        let verifying_key = signing_key.verifying_key();
        let address = self.derive_address(&verifying_key);

        if self
            .accounts
            .read()
            .await
            .iter()
            .any(|a| a.address == address)
        {
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

        info!("Imported account from mnemonic with BIP44 derivation: {}", address);

        Ok(account)
    }

    /// Import account from mnemonic with a specific derivation index
    /// Used for recovering multiple accounts from the same mnemonic
    pub async fn import_account_from_mnemonic_with_index(
        &self,
        mnemonic_phrase: &str,
        label: String,
        password: &str,
        account_index: u32,
    ) -> Result<Account> {
        // Validate password strength
        Self::validate_password(password)?;

        let mnemonic = Mnemonic::parse_in_normalized(Language::English, mnemonic_phrase)
            .map_err(|e| anyhow::anyhow!("Invalid mnemonic: {}", e))?;

        let seed = mnemonic.to_seed("");
        let signing_key = derive_bip44_ed25519(&seed, account_index)?;
        let verifying_key = signing_key.verifying_key();
        let address = self.derive_address(&verifying_key);

        if self
            .accounts
            .read()
            .await
            .iter()
            .any(|a| a.address == address)
        {
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

        info!("Imported account from mnemonic (index {}) with BIP44 derivation: {}", account_index, address);

        Ok(account)
    }

    /// Export private key (ALWAYS requires password - no session caching for exports)
    /// Rate limited and requires re-authentication
    pub async fn export_private_key(&self, address: &str, password: &str) -> Result<String> {
        // Key export ALWAYS requires re-authentication - no session caching
        // Check lockout first
        if self.is_locked_out(address).await {
            if let Some(remaining) = self.get_lockout_remaining(address).await {
                return Err(anyhow::anyhow!(
                    "Account locked due to too many failed attempts. Please wait {} seconds.",
                    remaining
                ));
            }
        }

        // Check rate limit for key export (more strict than regular operations)
        self.check_rate_limit(address, SensitiveOperation::KeyExport).await?;

        // Attempt to get key (validates password)
        match self.keystore.get_key(address, password) {
            Ok(signing_key) => {
                self.reset_failed_attempts(address).await;
                warn!("Private key exported for address: {} - user should be warned about security", address);
                Ok(hex::encode(signing_key.to_bytes()))
            }
            Err(e) => {
                let _ = self.record_failed_password_attempt(address).await;
                Err(e)
            }
        }
    }

    pub async fn get_accounts(&self) -> Vec<Account> {
        self.accounts.read().await.clone()
    }

    pub async fn get_account(&self, address: &str) -> Option<Account> {
        self.accounts
            .read()
            .await
            .iter()
            .find(|a| a.address == address)
            .cloned()
    }

    #[allow(dead_code)]
    pub async fn send_transaction(
        &self,
        request: TransactionRequest,
        password: &str,
    ) -> Result<String> {
        let tx = self.create_signed_transaction(request, password).await?;
        let tx_hash = hex::encode(tx.hash.as_bytes());
        info!("Transaction sent: {}", tx_hash);
        Ok(tx_hash)
    }

    /// Create and sign a transaction, update nonce, and return the full Transaction
    pub async fn create_signed_transaction(
        &self,
        request: TransactionRequest,
        password: &str,
    ) -> Result<Transaction> {
        // Get account
        let account = self
            .get_account(&request.from)
            .await
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
            data: hex::decode(request.data.trim_start_matches("0x")).unwrap_or_default(),
            signature: Signature::new([0u8; 64]),
            tx_type: None,
        };

        // Sign transaction
        self.sign_transaction(&mut tx, &request.from, password)
            .await?;

        // Update nonce
        self.update_nonce(&request.from, account.nonce + 1).await?;
        Ok(tx)
    }

    /// Sign a transaction with rate limiting and session management
    /// High-value transactions require re-authentication
    pub async fn sign_transaction(
        &self,
        tx: &mut Transaction,
        address: &str,
        password: &str,
    ) -> Result<()> {
        // Check lockout first
        if self.is_locked_out(address).await {
            if let Some(remaining) = self.get_lockout_remaining(address).await {
                return Err(anyhow::anyhow!(
                    "Account locked due to too many failed attempts. Please wait {} seconds.",
                    remaining
                ));
            }
        }

        // Check rate limit for signing
        self.check_rate_limit(address, SensitiveOperation::SignTransaction).await?;

        // Check if high-value transaction requires re-authentication
        if Self::requires_reauth(tx.value, SensitiveOperation::SignTransaction) {
            // High-value transactions ALWAYS require password verification
            info!("High-value transaction (>= {} SALT) requires re-authentication",
                  Self::get_reauth_threshold() / 1_000_000_000_000_000_000);
        }

        // Get private key from keystore (validates password)
        let signing_key = match self.keystore.get_key(address, password) {
            Ok(key) => {
                self.reset_failed_attempts(address).await;
                self.touch_session(address).await;
                key
            }
            Err(e) => {
                let _ = self.record_failed_password_attempt(address).await;
                return Err(e);
            }
        };

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

        info!("Transaction signed for address: {}, value: {}", address, tx.value);
        Ok(())
    }

    /// Sign a message with rate limiting
    pub async fn sign_message(
        &self,
        message: &[u8],
        address: &str,
        password: &str,
    ) -> Result<String> {
        // Check lockout first
        if self.is_locked_out(address).await {
            if let Some(remaining) = self.get_lockout_remaining(address).await {
                return Err(anyhow::anyhow!(
                    "Account locked due to too many failed attempts. Please wait {} seconds.",
                    remaining
                ));
            }
        }

        // Check rate limit
        self.check_rate_limit(address, SensitiveOperation::SignMessage).await?;

        // Get key and sign
        let signing_key = match self.keystore.get_key(address, password) {
            Ok(key) => {
                self.reset_failed_attempts(address).await;
                self.touch_session(address).await;
                key
            }
            Err(e) => {
                let _ = self.record_failed_password_attempt(address).await;
                return Err(e);
            }
        };

        let signature = signing_key.sign(message);
        Ok(hex::encode(signature.to_bytes()))
    }

    pub async fn verify_signature(
        &self,
        message: &[u8],
        signature: &str,
        address: &str,
    ) -> Result<bool> {
        let account = self
            .get_account(address)
            .await
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

    /// Delete an account (ALWAYS requires password - no session caching for deletion)
    /// Rate limited and requires re-authentication
    pub async fn delete_account(&self, address: &str, password: &str) -> Result<()> {
        // Account deletion ALWAYS requires re-authentication
        // Check lockout first
        if self.is_locked_out(address).await {
            if let Some(remaining) = self.get_lockout_remaining(address).await {
                return Err(anyhow::anyhow!(
                    "Account locked due to too many failed attempts. Please wait {} seconds.",
                    remaining
                ));
            }
        }

        // Check rate limit for account deletion
        self.check_rate_limit(address, SensitiveOperation::DeleteAccount).await?;

        // Verify password before deletion (critical operation)
        match self.keystore.get_key(address, password) {
            Ok(_) => {
                self.reset_failed_attempts(address).await;
            }
            Err(e) => {
                let _ = self.record_failed_password_attempt(address).await;
                return Err(e);
            }
        }

        // Remove from accounts list
        let mut accounts = self.accounts.write().await;
        accounts.retain(|a| a.address != address);
        drop(accounts);
        self.save_accounts().await?;

        // Delete from keychain
        self.keystore.delete_key(address)?;

        // End any active session for this account
        self.lock_wallet(address).await;

        warn!("Account deleted: {} - this action is irreversible", address);
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
            .join("citrate-core")
            .join("accounts.json")
    }
}

/// Secure key storage with OS keychain and file-based fallback
/// Uses OS keychain when available, falls back to encrypted file storage in dev mode
#[allow(dead_code)]
struct SecureKeyStore {
    entry: Option<Entry>,
    use_file_fallback: bool,
}

impl SecureKeyStore {
    fn new() -> Result<Self> {
        // Try OS keychain first, fall back to file storage if it fails
        match Entry::new(KEYRING_SERVICE, KEYRING_USER) {
            Ok(entry) => {
                // Test if keychain is actually accessible
                let test_result = entry.get_password();
                let use_file_fallback = match test_result {
                    Ok(_) => false,
                    Err(keyring::Error::NoEntry) => false, // Keychain works, just no entry yet
                    Err(_) => {
                        info!("OS keychain not accessible, using encrypted file storage");
                        true
                    }
                };
                Ok(Self { entry: Some(entry), use_file_fallback })
            }
            Err(e) => {
                info!("Failed to initialize OS keychain: {}. Using encrypted file storage.", e);
                Ok(Self { entry: None, use_file_fallback: true })
            }
        }
    }

    fn keys_dir() -> std::path::PathBuf {
        dirs::data_dir()
            .unwrap_or_else(|| std::path::PathBuf::from("."))
            .join("citrate-core")
            .join("keys")
    }

    fn key_file_path(address: &str) -> std::path::PathBuf {
        Self::keys_dir().join(format!("{}.key", address.replace("0x", "")))
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
            salt: &'a str, // PHC salt string
            nonce: String, // base64
            ct: String,    // base64
        }
        let record = StoredKey {
            v: 1,
            salt: salt.as_str(),
            nonce: BASE64.encode(nonce),
            ct: BASE64.encode(&ciphertext),
        };
        let encoded = serde_json::to_string(&record)?;

        // Try OS keychain first, then fall back to file
        if !self.use_file_fallback {
            if let Ok(entry) = Entry::new(KEYRING_SERVICE, &format!("wallet_{}", address)) {
                if entry.set_password(&encoded).is_ok() {
                    return Ok(());
                }
                info!("Keychain store failed, falling back to file storage");
            }
        }

        // File-based fallback with encrypted storage
        let keys_dir = Self::keys_dir();
        std::fs::create_dir_all(&keys_dir)?;
        let key_path = Self::key_file_path(address);
        std::fs::write(&key_path, &encoded)?;
        info!("Stored encrypted key to file for address: {}", address);

        Ok(())
    }

    fn get_key(&self, address: &str, password: &str) -> Result<SigningKey> {
        // Try to retrieve from keychain first
        let stored = if !self.use_file_fallback {
            if let Ok(entry) = Entry::new(KEYRING_SERVICE, &format!("wallet_{}", address)) {
                match entry.get_password() {
                    Ok(s) => Some(s),
                    Err(_) => None,
                }
            } else {
                None
            }
        } else {
            None
        };

        // Fall back to file storage if keychain didn't work
        let stored = match stored {
            Some(s) => s,
            None => {
                let key_path = Self::key_file_path(address);
                if key_path.exists() {
                    std::fs::read_to_string(&key_path)?
                } else {
                    return Err(anyhow::anyhow!("Key not found for address"));
                }
            }
        };

        // Try JSON format first
        #[derive(Deserialize)]
        #[allow(dead_code)]
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
        // Try to delete from keychain
        if !self.use_file_fallback {
            if let Ok(entry) = Entry::new(KEYRING_SERVICE, &format!("wallet_{}", address)) {
                let _ = entry.delete_password(); // Ignore errors, we'll also try file
            }
        }

        // Also delete from file storage if it exists
        let key_path = Self::key_file_path(address);
        if key_path.exists() {
            std::fs::remove_file(&key_path)?;
        }

        info!("Deleted key for address: {}", address);
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Account {
    pub address: String,
    pub label: String,
    pub public_key: String,
    #[serde(
        serialize_with = "serialize_u128",
        deserialize_with = "deserialize_u128"
    )]
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_password_strength_valid() {
        // Valid password with all requirements (no sequential chars)
        let result = validate_password_strength("MyS3cure!W@ll3t");
        assert!(result.is_valid, "Password should be valid. Issues: {:?}", result.issues);
        assert!(result.score >= 75, "Score should be >= 75, got {}", result.score);
        assert!(result.issues.is_empty(), "Should have no issues: {:?}", result.issues);
    }

    #[test]
    fn test_password_strength_too_short() {
        let result = validate_password_strength("Short1!");
        assert!(!result.is_valid);
        assert!(result.issues.iter().any(|i| i.contains("12 characters")));
    }

    #[test]
    fn test_password_strength_no_uppercase() {
        let result = validate_password_strength("myp@ssw0rd123!");
        assert!(!result.is_valid);
        assert!(result.issues.iter().any(|i| i.contains("uppercase")));
    }

    #[test]
    fn test_password_strength_no_lowercase() {
        let result = validate_password_strength("MYP@SSW0RD123!");
        assert!(!result.is_valid);
        assert!(result.issues.iter().any(|i| i.contains("lowercase")));
    }

    #[test]
    fn test_password_strength_no_digit() {
        let result = validate_password_strength("MyP@sswordABC!");
        assert!(!result.is_valid);
        assert!(result.issues.iter().any(|i| i.contains("digit")));
    }

    #[test]
    fn test_password_strength_no_special() {
        let result = validate_password_strength("MyPassw0rd1234");
        assert!(!result.is_valid);
        assert!(result.issues.iter().any(|i| i.contains("special")));
    }

    #[test]
    fn test_password_strength_weak_pattern() {
        let result = validate_password_strength("MyPassword123!");
        assert!(!result.is_valid);
        assert!(result.issues.iter().any(|i| i.contains("weak pattern")));
    }

    #[test]
    fn test_password_strength_sequential() {
        // "abc" is sequential
        let result = validate_password_strength("abcDefGh123!@#");
        assert!(result.issues.iter().any(|i| i.contains("sequential")));
    }

    #[test]
    fn test_bip44_derivation_consistency() {
        // Same seed should always produce the same key
        let test_seed = [42u8; 64]; // Fixed seed for testing

        let key1 = derive_bip44_ed25519(&test_seed, 0).unwrap();
        let key2 = derive_bip44_ed25519(&test_seed, 0).unwrap();

        assert_eq!(key1.to_bytes(), key2.to_bytes());
    }

    #[test]
    fn test_bip44_derivation_different_indices() {
        // Different account indices should produce different keys
        let test_seed = [42u8; 64];

        let key0 = derive_bip44_ed25519(&test_seed, 0).unwrap();
        let key1 = derive_bip44_ed25519(&test_seed, 1).unwrap();
        let key2 = derive_bip44_ed25519(&test_seed, 2).unwrap();

        assert_ne!(key0.to_bytes(), key1.to_bytes());
        assert_ne!(key1.to_bytes(), key2.to_bytes());
        assert_ne!(key0.to_bytes(), key2.to_bytes());
    }

    #[test]
    fn test_bip44_key_is_valid() {
        // Derived key should be a valid Ed25519 signing key
        let test_seed = [0u8; 64];
        let key = derive_bip44_ed25519(&test_seed, 0).unwrap();

        // Should be able to get verifying key
        let _verifying = key.verifying_key();

        // Should be able to sign and verify
        let message = b"test message";
        let signature = key.sign(message);
        assert!(key.verifying_key().verify_strict(message, &signature).is_ok());
    }

    #[test]
    fn test_slip0010_ed25519_derivation() {
        // Test the underlying SLIP-0010 derivation
        let test_seed = [1u8; 64];
        let path = [44, 501, 0, 0, 0]; // BIP44 path

        let key = derive_ed25519_from_seed(&test_seed, &path).unwrap();

        // Should produce a valid 32-byte key
        assert_eq!(key.to_bytes().len(), 32);
    }

    #[test]
    fn test_wallet_password_validation_rejects_weak() {
        // WalletManager::validate_password should reject weak passwords
        let weak_passwords = [
            "short1!A",           // Too short
            "nouppercase123!",    // No uppercase
            "NOLOWERCASE123!",    // No lowercase
            "NoDigitsHere!@#",    // No digits
            "NoSpecial123ABC",    // No special chars
        ];

        for password in &weak_passwords {
            let result = WalletManager::validate_password(password);
            assert!(result.is_err(), "Password '{}' should be rejected", password);
        }
    }

    #[test]
    fn test_wallet_password_validation_accepts_strong() {
        let strong_passwords = [
            "MyStr0ng!Pass@2024",
            "C0mpl3x#Passw0rd!",
            "Secur3_W@llet_2024!",
        ];

        for password in &strong_passwords {
            let result = WalletManager::validate_password(password);
            assert!(result.is_ok(), "Password '{}' should be accepted: {:?}", password, result.err());
        }
    }

    // ========== Rate Limiting Tests ==========

    #[test]
    fn test_rate_limiter_allows_within_limit() {
        let mut limiter = RateLimiter::new();
        let address = "0xtest123";

        // Should allow first MAX_OPERATIONS_PER_WINDOW operations
        for i in 0..MAX_OPERATIONS_PER_WINDOW {
            let result = limiter.check_rate_limit(address, SensitiveOperation::SignTransaction);
            assert!(result.is_ok(), "Operation {} should be allowed", i);
        }
    }

    #[test]
    fn test_rate_limiter_blocks_after_limit() {
        let mut limiter = RateLimiter::new();
        let address = "0xtest123";

        // Use up all operations
        for _ in 0..MAX_OPERATIONS_PER_WINDOW {
            let _ = limiter.check_rate_limit(address, SensitiveOperation::SignTransaction);
        }

        // Next operation should be blocked
        let result = limiter.check_rate_limit(address, SensitiveOperation::SignTransaction);
        assert!(result.is_err(), "Should be rate limited after max operations");
        assert!(result.unwrap_err().contains("Rate limit exceeded"));
    }

    #[test]
    fn test_rate_limiter_separate_operations() {
        let mut limiter = RateLimiter::new();
        let address = "0xtest123";

        // Fill up SignTransaction limit
        for _ in 0..MAX_OPERATIONS_PER_WINDOW {
            let _ = limiter.check_rate_limit(address, SensitiveOperation::SignTransaction);
        }

        // SignMessage should still work (different operation type)
        let result = limiter.check_rate_limit(address, SensitiveOperation::SignMessage);
        assert!(result.is_ok(), "Different operation type should not be rate limited");
    }

    #[test]
    fn test_rate_limiter_separate_addresses() {
        let mut limiter = RateLimiter::new();
        let address1 = "0xtest123";
        let address2 = "0xtest456";

        // Fill up limit for address1
        for _ in 0..MAX_OPERATIONS_PER_WINDOW {
            let _ = limiter.check_rate_limit(address1, SensitiveOperation::SignTransaction);
        }

        // address2 should still work
        let result = limiter.check_rate_limit(address2, SensitiveOperation::SignTransaction);
        assert!(result.is_ok(), "Different address should not be rate limited");
    }

    #[test]
    fn test_failed_attempts_tracking() {
        let mut limiter = RateLimiter::new();
        let address = "0xtest123";

        // Record failed attempts up to limit
        for i in 0..MAX_FAILED_ATTEMPTS {
            let result = limiter.record_failed_attempt(address);
            if i < MAX_FAILED_ATTEMPTS - 1 {
                assert!(result.is_ok(), "Failed attempt {} should be allowed", i);
            } else {
                // Last one triggers lockout
                assert!(result.is_err(), "Should be locked after max failures");
            }
        }

        // Should be locked
        assert!(limiter.is_locked_out(address));
    }

    #[test]
    fn test_reset_failed_attempts() {
        let mut limiter = RateLimiter::new();
        let address = "0xtest123";

        // Record some failed attempts
        let _ = limiter.record_failed_attempt(address);
        let _ = limiter.record_failed_attempt(address);

        // Reset
        limiter.reset_failed_attempts(address);

        // Should be able to start fresh
        assert!(!limiter.is_locked_out(address));

        // Full MAX_FAILED_ATTEMPTS attempts should work again
        for i in 0..MAX_FAILED_ATTEMPTS - 1 {
            let result = limiter.record_failed_attempt(address);
            assert!(result.is_ok(), "Attempt {} after reset should work", i);
        }
    }

    // ========== Session Management Tests ==========

    #[test]
    fn test_session_creation() {
        let mut session_mgr = SessionManager::new();
        let address = "0xtest123";

        session_mgr.create_session(address);
        assert!(session_mgr.is_session_valid(address));
    }

    #[test]
    fn test_session_touch_extends_validity() {
        let mut session_mgr = SessionManager::new();
        let address = "0xtest123";

        session_mgr.create_session(address);

        // Touch the session
        session_mgr.touch_session(address);

        // Should still be valid
        assert!(session_mgr.is_session_valid(address));
    }

    #[test]
    fn test_session_end() {
        let mut session_mgr = SessionManager::new();
        let address = "0xtest123";

        session_mgr.create_session(address);
        assert!(session_mgr.is_session_valid(address));

        session_mgr.end_session(address);
        assert!(!session_mgr.is_session_valid(address));
    }

    #[test]
    fn test_session_remaining_time() {
        let mut session_mgr = SessionManager::new();
        let address = "0xtest123";

        session_mgr.create_session(address);

        // Should have remaining time close to SESSION_TIMEOUT_SECS
        let remaining = session_mgr.get_session_remaining(address);
        assert!(remaining.is_some());
        let secs = remaining.unwrap();
        assert!(secs > SESSION_TIMEOUT_SECS - 5, "Should have most of session time remaining");
        assert!(secs <= SESSION_TIMEOUT_SECS, "Should not exceed session timeout");
    }

    #[test]
    fn test_nonexistent_session_invalid() {
        let session_mgr = SessionManager::new();
        assert!(!session_mgr.is_session_valid("0xnonexistent"));
        assert!(session_mgr.get_session_remaining("0xnonexistent").is_none());
    }

    // ========== Re-auth Checker Tests ==========

    #[test]
    fn test_reauth_required_for_key_export() {
        assert!(ReauthChecker::requires_reauth(0, SensitiveOperation::KeyExport));
        assert!(ReauthChecker::requires_reauth(1000, SensitiveOperation::KeyExport));
    }

    #[test]
    fn test_reauth_required_for_account_deletion() {
        assert!(ReauthChecker::requires_reauth(0, SensitiveOperation::DeleteAccount));
    }

    #[test]
    fn test_reauth_not_required_for_small_tx() {
        let small_value = REAUTH_THRESHOLD_SALT - 1;
        assert!(!ReauthChecker::requires_reauth(small_value, SensitiveOperation::SignTransaction));
    }

    #[test]
    fn test_reauth_required_for_large_tx() {
        let large_value = REAUTH_THRESHOLD_SALT;
        assert!(ReauthChecker::requires_reauth(large_value, SensitiveOperation::SignTransaction));

        let very_large_value = REAUTH_THRESHOLD_SALT * 10;
        assert!(ReauthChecker::requires_reauth(very_large_value, SensitiveOperation::SignTransaction));
    }

    #[test]
    fn test_reauth_threshold_value() {
        // Threshold should be 10 SALT (10 * 10^18 wei)
        assert_eq!(ReauthChecker::get_reauth_threshold(), 10_000_000_000_000_000_000u128);
    }

    #[test]
    fn test_wallet_manager_reauth_threshold() {
        assert_eq!(
            WalletManager::get_reauth_threshold(),
            10_000_000_000_000_000_000u128
        );
    }
}
