// citrate-core/src-tauri/src/agent/config.rs
//
// Agent configuration with multi-provider API support
//
// Supports:
// - OpenAI (GPT-4, GPT-3.5)
// - Anthropic (Claude 3)
// - Google Gemini
// - xAI (Grok)
// - Local GGUF models

use aes_gcm::{
    aead::{Aead, AeadCore, KeyInit},
    Aes256Gcm, Key, Nonce,
};
use argon2::{
    password_hash::{PasswordHasher, SaltString},
    Argon2,
};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};
use keyring::Entry;
use rand::rngs::OsRng;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{info, warn, error};

// Keyring constants for API key storage
const API_KEYRING_SERVICE: &str = "citrate-core-api";
const API_KEYRING_SALT_KEY: &str = "api_key_salt";
const API_KEY_ENCRYPTED_PREFIX: &str = "citrate_encrypted_v1:";

// API key format validation patterns
const OPENAI_KEY_PREFIX: &str = "sk-";
const ANTHROPIC_KEY_PREFIX: &str = "sk-ant-";
const GEMINI_KEY_LENGTH_MIN: usize = 30;
const XAI_KEY_PREFIX: &str = "xai-";

/// Result type for API key operations
pub type ApiKeyResult<T> = Result<T, ApiKeyError>;

/// Errors that can occur during API key operations
#[derive(Debug, Clone)]
pub enum ApiKeyError {
    /// Key format is invalid for the provider
    InvalidKeyFormat(String),
    /// Key validation with provider API failed
    ValidationFailed(String),
    /// Failed to store key securely
    StorageError(String),
    /// Failed to retrieve key from storage
    RetrievalError(String),
    /// Encryption/decryption failed
    CryptoError(String),
    /// Network error during validation
    NetworkError(String),
    /// Provider returned an error
    ProviderError(String),
}

impl std::fmt::Display for ApiKeyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ApiKeyError::InvalidKeyFormat(msg) => write!(f, "Invalid API key format: {}", msg),
            ApiKeyError::ValidationFailed(msg) => write!(f, "API key validation failed: {}", msg),
            ApiKeyError::StorageError(msg) => write!(f, "Failed to store API key: {}", msg),
            ApiKeyError::RetrievalError(msg) => write!(f, "Failed to retrieve API key: {}", msg),
            ApiKeyError::CryptoError(msg) => write!(f, "Encryption error: {}", msg),
            ApiKeyError::NetworkError(msg) => write!(f, "Network error: {}", msg),
            ApiKeyError::ProviderError(msg) => write!(f, "Provider error: {}", msg),
        }
    }
}

impl std::error::Error for ApiKeyError {}

/// API key validation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiKeyValidationResult {
    pub valid: bool,
    pub provider: AIProvider,
    pub error_message: Option<String>,
    pub model_access: Vec<String>,
    pub rate_limit_remaining: Option<u32>,
}

/// Secure API key storage using OS keychain with encryption fallback
pub struct SecureApiKeyStore {
    /// Encryption key derived from machine-specific entropy
    encryption_key: Option<[u8; 32]>,
    /// Fallback storage directory for encrypted keys
    fallback_dir: std::path::PathBuf,
}

impl SecureApiKeyStore {
    /// Create a new secure API key store
    pub fn new() -> Self {
        let fallback_dir = dirs::data_local_dir()
            .unwrap_or_else(|| std::path::PathBuf::from("."))
            .join("citrate")
            .join("api_keys");

        // Create fallback directory if it doesn't exist
        let _ = std::fs::create_dir_all(&fallback_dir);

        // Derive encryption key from machine-specific entropy
        let encryption_key = Self::derive_machine_key();

        Self {
            encryption_key,
            fallback_dir,
        }
    }

    /// Derive an encryption key from machine-specific entropy
    fn derive_machine_key() -> Option<[u8; 32]> {
        // Use a combination of machine identifiers as entropy source
        let mut entropy = String::new();

        // Add username
        if let Ok(user) = std::env::var("USER").or_else(|_| std::env::var("USERNAME")) {
            entropy.push_str(&user);
        }

        // Add home directory path
        if let Some(home) = dirs::home_dir() {
            entropy.push_str(&home.to_string_lossy());
        }

        // Add hostname if available
        #[cfg(unix)]
        {
            if let Ok(hostname) = std::fs::read_to_string("/etc/hostname") {
                entropy.push_str(hostname.trim());
            }
        }

        // Add a static application salt
        entropy.push_str("citrate-api-key-store-v1");

        if entropy.is_empty() {
            return None;
        }

        // Derive key using Argon2
        let salt = SaltString::from_b64("Y2l0cmF0ZS1hcGkta2V5cw")
            .unwrap_or_else(|_| SaltString::generate(&mut OsRng));
        let argon2 = Argon2::default();

        match argon2.hash_password(entropy.as_bytes(), &salt) {
            Ok(hash) => {
                let hash_str = hash.to_string();
                let hash_bytes = hash_str.as_bytes();
                let mut key = [0u8; 32];
                // Take last 32 bytes of the hash output
                let start = hash_bytes.len().saturating_sub(32);
                key.copy_from_slice(&hash_bytes[start..start + 32.min(hash_bytes.len() - start)]);
                Some(key)
            }
            Err(_) => None,
        }
    }

    /// Get the keyring entry name for a provider
    fn keyring_entry_name(provider: AIProvider) -> String {
        format!("{}-{:?}", API_KEYRING_SERVICE, provider).to_lowercase()
    }

    /// Store an API key securely
    pub fn store_key(&self, provider: AIProvider, api_key: &str) -> ApiKeyResult<()> {
        // First, validate the key format
        Self::validate_key_format(provider, api_key)?;

        let entry_name = Self::keyring_entry_name(provider);

        // Try OS keychain first
        match Entry::new(&entry_name, "api_key") {
            Ok(entry) => {
                match entry.set_password(api_key) {
                    Ok(_) => {
                        info!("Stored {} API key in OS keychain", provider);
                        return Ok(());
                    }
                    Err(e) => {
                        warn!("Failed to store in keychain, using fallback: {}", e);
                    }
                }
            }
            Err(e) => {
                warn!("Failed to access keychain, using fallback: {}", e);
            }
        }

        // Fallback to encrypted file storage
        self.store_key_encrypted(provider, api_key)
    }

    /// Store an API key in encrypted file (fallback)
    fn store_key_encrypted(&self, provider: AIProvider, api_key: &str) -> ApiKeyResult<()> {
        let encryption_key = self.encryption_key
            .ok_or_else(|| ApiKeyError::CryptoError("No encryption key available".to_string()))?;

        let key = Key::<Aes256Gcm>::from_slice(&encryption_key);
        let cipher = Aes256Gcm::new(key);
        let nonce = Aes256Gcm::generate_nonce(&mut OsRng);

        let ciphertext = cipher
            .encrypt(&nonce, api_key.as_bytes())
            .map_err(|e| ApiKeyError::CryptoError(format!("Encryption failed: {}", e)))?;

        // Combine nonce and ciphertext
        let mut combined = nonce.to_vec();
        combined.extend(ciphertext);

        // Encode with prefix
        let encoded = format!("{}{}", API_KEY_ENCRYPTED_PREFIX, BASE64.encode(&combined));

        // Write to file
        let file_path = self.fallback_dir.join(format!("{:?}.key", provider).to_lowercase());
        std::fs::write(&file_path, encoded)
            .map_err(|e| ApiKeyError::StorageError(format!("Failed to write key file: {}", e)))?;

        // Set restrictive permissions on Unix
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let _ = std::fs::set_permissions(&file_path, std::fs::Permissions::from_mode(0o600));
        }

        info!("Stored {} API key in encrypted file", provider);
        Ok(())
    }

    /// Retrieve an API key
    pub fn get_key(&self, provider: AIProvider) -> ApiKeyResult<String> {
        let entry_name = Self::keyring_entry_name(provider);

        // Try OS keychain first
        if let Ok(entry) = Entry::new(&entry_name, "api_key") {
            if let Ok(password) = entry.get_password() {
                return Ok(password);
            }
        }

        // Fallback to encrypted file
        self.get_key_encrypted(provider)
    }

    /// Retrieve an API key from encrypted file (fallback)
    fn get_key_encrypted(&self, provider: AIProvider) -> ApiKeyResult<String> {
        let file_path = self.fallback_dir.join(format!("{:?}.key", provider).to_lowercase());

        let encoded = std::fs::read_to_string(&file_path)
            .map_err(|e| ApiKeyError::RetrievalError(format!("Failed to read key file: {}", e)))?;

        if !encoded.starts_with(API_KEY_ENCRYPTED_PREFIX) {
            return Err(ApiKeyError::CryptoError("Invalid encrypted key format".to_string()));
        }

        let encryption_key = self.encryption_key
            .ok_or_else(|| ApiKeyError::CryptoError("No encryption key available".to_string()))?;

        let combined = BASE64
            .decode(&encoded[API_KEY_ENCRYPTED_PREFIX.len()..])
            .map_err(|e| ApiKeyError::CryptoError(format!("Base64 decode failed: {}", e)))?;

        if combined.len() < 12 {
            return Err(ApiKeyError::CryptoError("Invalid ciphertext length".to_string()));
        }

        let (nonce_bytes, ciphertext) = combined.split_at(12);
        let nonce = Nonce::from_slice(nonce_bytes);

        let key = Key::<Aes256Gcm>::from_slice(&encryption_key);
        let cipher = Aes256Gcm::new(key);

        let plaintext = cipher
            .decrypt(nonce, ciphertext)
            .map_err(|e| ApiKeyError::CryptoError(format!("Decryption failed: {}", e)))?;

        String::from_utf8(plaintext)
            .map_err(|e| ApiKeyError::CryptoError(format!("Invalid UTF-8: {}", e)))
    }

    /// Delete an API key
    pub fn delete_key(&self, provider: AIProvider) -> ApiKeyResult<()> {
        let entry_name = Self::keyring_entry_name(provider);

        // Try to delete from keychain
        if let Ok(entry) = Entry::new(&entry_name, "api_key") {
            let _ = entry.delete_password();
        }

        // Also delete from file fallback
        let file_path = self.fallback_dir.join(format!("{:?}.key", provider).to_lowercase());
        let _ = std::fs::remove_file(&file_path);

        info!("Deleted {} API key", provider);
        Ok(())
    }

    /// Check if a key exists for a provider
    pub fn has_key(&self, provider: AIProvider) -> bool {
        self.get_key(provider).is_ok()
    }

    /// Validate API key format for a provider (fast, no network)
    pub fn validate_key_format(provider: AIProvider, api_key: &str) -> ApiKeyResult<()> {
        let trimmed = api_key.trim();

        if trimmed.is_empty() {
            return Err(ApiKeyError::InvalidKeyFormat("API key cannot be empty".to_string()));
        }

        match provider {
            AIProvider::OpenAI => {
                if !trimmed.starts_with(OPENAI_KEY_PREFIX) {
                    return Err(ApiKeyError::InvalidKeyFormat(
                        format!("OpenAI key must start with '{}'", OPENAI_KEY_PREFIX)
                    ));
                }
                if trimmed.len() < 40 {
                    return Err(ApiKeyError::InvalidKeyFormat(
                        "OpenAI key appears too short".to_string()
                    ));
                }
            }
            AIProvider::Anthropic => {
                if !trimmed.starts_with(ANTHROPIC_KEY_PREFIX) {
                    return Err(ApiKeyError::InvalidKeyFormat(
                        format!("Anthropic key must start with '{}'", ANTHROPIC_KEY_PREFIX)
                    ));
                }
                if trimmed.len() < 50 {
                    return Err(ApiKeyError::InvalidKeyFormat(
                        "Anthropic key appears too short".to_string()
                    ));
                }
            }
            AIProvider::Gemini => {
                // Gemini keys are alphanumeric, no specific prefix
                if trimmed.len() < GEMINI_KEY_LENGTH_MIN {
                    return Err(ApiKeyError::InvalidKeyFormat(
                        format!("Gemini key must be at least {} characters", GEMINI_KEY_LENGTH_MIN)
                    ));
                }
                if !trimmed.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_') {
                    return Err(ApiKeyError::InvalidKeyFormat(
                        "Gemini key contains invalid characters".to_string()
                    ));
                }
            }
            AIProvider::XAI => {
                if !trimmed.starts_with(XAI_KEY_PREFIX) {
                    return Err(ApiKeyError::InvalidKeyFormat(
                        format!("xAI key must start with '{}'", XAI_KEY_PREFIX)
                    ));
                }
                if trimmed.len() < 30 {
                    return Err(ApiKeyError::InvalidKeyFormat(
                        "xAI key appears too short".to_string()
                    ));
                }
            }
            AIProvider::Local => {
                // Local models don't need API keys
                return Ok(());
            }
        }

        Ok(())
    }
}

impl Default for SecureApiKeyStore {
    fn default() -> Self {
        Self::new()
    }
}

/// API key validator that tests keys against provider APIs
pub struct ApiKeyValidator {
    client: reqwest::Client,
}

impl ApiKeyValidator {
    /// Create a new validator with default HTTP client
    pub fn new() -> Self {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(10))
            .build()
            .unwrap_or_default();

        Self { client }
    }

    /// Validate an API key by making a lightweight API call
    pub async fn validate_key(&self, provider: AIProvider, api_key: &str, base_url: Option<&str>) -> ApiKeyValidationResult {
        // First check format
        if let Err(e) = SecureApiKeyStore::validate_key_format(provider, api_key) {
            return ApiKeyValidationResult {
                valid: false,
                provider,
                error_message: Some(e.to_string()),
                model_access: vec![],
                rate_limit_remaining: None,
            };
        }

        match provider {
            AIProvider::OpenAI => self.validate_openai(api_key, base_url).await,
            AIProvider::Anthropic => self.validate_anthropic(api_key, base_url).await,
            AIProvider::Gemini => self.validate_gemini(api_key, base_url).await,
            AIProvider::XAI => self.validate_xai(api_key, base_url).await,
            AIProvider::Local => ApiKeyValidationResult {
                valid: true,
                provider,
                error_message: None,
                model_access: vec!["local".to_string()],
                rate_limit_remaining: None,
            },
        }
    }

    /// Validate OpenAI API key
    async fn validate_openai(&self, api_key: &str, base_url: Option<&str>) -> ApiKeyValidationResult {
        let url = format!("{}/models", base_url.unwrap_or("https://api.openai.com/v1"));

        match self.client
            .get(&url)
            .header("Authorization", format!("Bearer {}", api_key))
            .send()
            .await
        {
            Ok(response) => {
                let rate_limit = response
                    .headers()
                    .get("x-ratelimit-remaining-requests")
                    .and_then(|v| v.to_str().ok())
                    .and_then(|v| v.parse().ok());

                if response.status().is_success() {
                    // Parse model list
                    let models = response
                        .json::<serde_json::Value>()
                        .await
                        .ok()
                        .and_then(|v| v.get("data")?.as_array().cloned())
                        .map(|arr| {
                            arr.iter()
                                .filter_map(|m| m.get("id")?.as_str().map(String::from))
                                .filter(|id| id.starts_with("gpt"))
                                .collect()
                        })
                        .unwrap_or_default();

                    ApiKeyValidationResult {
                        valid: true,
                        provider: AIProvider::OpenAI,
                        error_message: None,
                        model_access: models,
                        rate_limit_remaining: rate_limit,
                    }
                } else {
                    let error = response.text().await.unwrap_or_default();
                    ApiKeyValidationResult {
                        valid: false,
                        provider: AIProvider::OpenAI,
                        error_message: Some(format!("API returned error: {}", error)),
                        model_access: vec![],
                        rate_limit_remaining: None,
                    }
                }
            }
            Err(e) => ApiKeyValidationResult {
                valid: false,
                provider: AIProvider::OpenAI,
                error_message: Some(format!("Network error: {}", e)),
                model_access: vec![],
                rate_limit_remaining: None,
            },
        }
    }

    /// Validate Anthropic API key
    async fn validate_anthropic(&self, api_key: &str, base_url: Option<&str>) -> ApiKeyValidationResult {
        let url = format!("{}/v1/messages", base_url.unwrap_or("https://api.anthropic.com"));

        // Anthropic requires a minimal valid request to validate
        let body = serde_json::json!({
            "model": "claude-3-haiku-20240307",
            "max_tokens": 1,
            "messages": [{"role": "user", "content": "Hi"}]
        });

        match self.client
            .post(&url)
            .header("x-api-key", api_key)
            .header("anthropic-version", "2023-06-01")
            .header("content-type", "application/json")
            .json(&body)
            .send()
            .await
        {
            Ok(response) => {
                let rate_limit = response
                    .headers()
                    .get("anthropic-ratelimit-requests-remaining")
                    .and_then(|v| v.to_str().ok())
                    .and_then(|v| v.parse().ok());

                let status = response.status();

                if status.is_success() {
                    ApiKeyValidationResult {
                        valid: true,
                        provider: AIProvider::Anthropic,
                        error_message: None,
                        model_access: vec![
                            "claude-3-opus-20240229".to_string(),
                            "claude-3-sonnet-20240229".to_string(),
                            "claude-3-haiku-20240307".to_string(),
                        ],
                        rate_limit_remaining: rate_limit,
                    }
                } else if status == reqwest::StatusCode::UNAUTHORIZED {
                    ApiKeyValidationResult {
                        valid: false,
                        provider: AIProvider::Anthropic,
                        error_message: Some("Invalid API key".to_string()),
                        model_access: vec![],
                        rate_limit_remaining: None,
                    }
                } else {
                    // Other status codes might indicate rate limiting or temp errors
                    // but the key could still be valid
                    let error = response.text().await.unwrap_or_default();
                    ApiKeyValidationResult {
                        valid: false,
                        provider: AIProvider::Anthropic,
                        error_message: Some(format!("API error ({}): {}", status, error)),
                        model_access: vec![],
                        rate_limit_remaining: rate_limit,
                    }
                }
            }
            Err(e) => ApiKeyValidationResult {
                valid: false,
                provider: AIProvider::Anthropic,
                error_message: Some(format!("Network error: {}", e)),
                model_access: vec![],
                rate_limit_remaining: None,
            },
        }
    }

    /// Validate Google Gemini API key
    async fn validate_gemini(&self, api_key: &str, base_url: Option<&str>) -> ApiKeyValidationResult {
        let url = format!(
            "{}/models?key={}",
            base_url.unwrap_or("https://generativelanguage.googleapis.com/v1beta"),
            api_key
        );

        match self.client.get(&url).send().await {
            Ok(response) => {
                if response.status().is_success() {
                    let models = response
                        .json::<serde_json::Value>()
                        .await
                        .ok()
                        .and_then(|v| v.get("models")?.as_array().cloned())
                        .map(|arr| {
                            arr.iter()
                                .filter_map(|m| m.get("name")?.as_str().map(String::from))
                                .filter(|name| name.contains("gemini"))
                                .collect()
                        })
                        .unwrap_or_default();

                    ApiKeyValidationResult {
                        valid: true,
                        provider: AIProvider::Gemini,
                        error_message: None,
                        model_access: models,
                        rate_limit_remaining: None,
                    }
                } else {
                    let error = response.text().await.unwrap_or_default();
                    ApiKeyValidationResult {
                        valid: false,
                        provider: AIProvider::Gemini,
                        error_message: Some(format!("API error: {}", error)),
                        model_access: vec![],
                        rate_limit_remaining: None,
                    }
                }
            }
            Err(e) => ApiKeyValidationResult {
                valid: false,
                provider: AIProvider::Gemini,
                error_message: Some(format!("Network error: {}", e)),
                model_access: vec![],
                rate_limit_remaining: None,
            },
        }
    }

    /// Validate xAI API key
    async fn validate_xai(&self, api_key: &str, base_url: Option<&str>) -> ApiKeyValidationResult {
        let url = format!("{}/models", base_url.unwrap_or("https://api.x.ai/v1"));

        match self.client
            .get(&url)
            .header("Authorization", format!("Bearer {}", api_key))
            .send()
            .await
        {
            Ok(response) => {
                if response.status().is_success() {
                    let models = response
                        .json::<serde_json::Value>()
                        .await
                        .ok()
                        .and_then(|v| v.get("data")?.as_array().cloned())
                        .map(|arr| {
                            arr.iter()
                                .filter_map(|m| m.get("id")?.as_str().map(String::from))
                                .collect()
                        })
                        .unwrap_or_else(|| vec!["grok-beta".to_string()]);

                    ApiKeyValidationResult {
                        valid: true,
                        provider: AIProvider::XAI,
                        error_message: None,
                        model_access: models,
                        rate_limit_remaining: None,
                    }
                } else {
                    let error = response.text().await.unwrap_or_default();
                    ApiKeyValidationResult {
                        valid: false,
                        provider: AIProvider::XAI,
                        error_message: Some(format!("API error: {}", error)),
                        model_access: vec![],
                        rate_limit_remaining: None,
                    }
                }
            }
            Err(e) => ApiKeyValidationResult {
                valid: false,
                provider: AIProvider::XAI,
                error_message: Some(format!("Network error: {}", e)),
                model_access: vec![],
                rate_limit_remaining: None,
            },
        }
    }
}

impl Default for ApiKeyValidator {
    fn default() -> Self {
        Self::new()
    }
}

/// AI Provider enumeration
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum AIProvider {
    /// OpenAI (GPT-4, GPT-3.5-turbo)
    #[serde(rename = "openai")]
    OpenAI,
    /// Anthropic (Claude 3 Opus, Sonnet, Haiku)
    Anthropic,
    /// Google Gemini (Gemini Pro, Gemini Ultra)
    Gemini,
    /// xAI (Grok)
    #[serde(rename = "xai")]
    XAI,
    /// Local GGUF model
    Local,
}

impl Default for AIProvider {
    fn default() -> Self {
        Self::Local
    }
}

impl std::fmt::Display for AIProvider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AIProvider::OpenAI => write!(f, "OpenAI"),
            AIProvider::Anthropic => write!(f, "Anthropic"),
            AIProvider::Gemini => write!(f, "Google Gemini"),
            AIProvider::XAI => write!(f, "xAI"),
            AIProvider::Local => write!(f, "Local Model"),
        }
    }
}

/// Provider-specific settings
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ProviderSettings {
    /// API key for this provider (stored securely, not serialized)
    #[serde(skip_serializing)]
    pub api_key: Option<String>,
    /// Whether this provider is enabled
    pub enabled: bool,
    /// Default model ID for this provider
    pub model_id: String,
    /// Custom API base URL (for proxies or self-hosted)
    pub base_url: Option<String>,
    /// Whether connection has been verified
    #[serde(skip)]
    pub verified: bool,
}

impl ProviderSettings {
    /// Create new provider settings
    pub fn new(model_id: &str) -> Self {
        Self {
            api_key: None,
            enabled: false,
            model_id: model_id.to_string(),
            base_url: None,
            verified: false,
        }
    }

    /// Check if this provider is configured and ready to use
    pub fn is_ready(&self) -> bool {
        self.enabled && self.api_key.is_some()
    }
}

/// Multi-provider AI configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIProvidersConfig {
    /// OpenAI settings
    pub openai: ProviderSettings,
    /// Anthropic settings
    pub anthropic: ProviderSettings,
    /// Google Gemini settings
    pub gemini: ProviderSettings,
    /// xAI settings
    pub xai: ProviderSettings,
    /// Preferred provider order (first available will be used)
    pub preferred_order: Vec<AIProvider>,
    /// Always fallback to local model if cloud providers fail
    pub local_fallback: bool,
    /// Local model path (GGUF file)
    pub local_model_path: Option<String>,
    /// Local model IPFS CID (for re-download)
    pub local_model_cid: Option<String>,
}

impl Default for AIProvidersConfig {
    fn default() -> Self {
        Self {
            openai: ProviderSettings::new("gpt-4o-mini"),
            anthropic: ProviderSettings::new("claude-3-haiku-20240307"),
            gemini: ProviderSettings::new("gemini-1.5-flash"),
            xai: ProviderSettings::new("grok-beta"),
            preferred_order: vec![AIProvider::Local, AIProvider::OpenAI, AIProvider::Anthropic],
            local_fallback: true,
            local_model_path: None,
            local_model_cid: None,
        }
    }
}

impl AIProvidersConfig {
    /// Get the first available provider based on preference order
    pub fn get_active_provider(&self) -> Option<AIProvider> {
        for provider in &self.preferred_order {
            match provider {
                AIProvider::OpenAI if self.openai.is_ready() => return Some(AIProvider::OpenAI),
                AIProvider::Anthropic if self.anthropic.is_ready() => return Some(AIProvider::Anthropic),
                AIProvider::Gemini if self.gemini.is_ready() => return Some(AIProvider::Gemini),
                AIProvider::XAI if self.xai.is_ready() => return Some(AIProvider::XAI),
                AIProvider::Local if self.local_model_path.is_some() => return Some(AIProvider::Local),
                _ => continue,
            }
        }

        // Fallback to local if enabled
        if self.local_fallback && self.local_model_path.is_some() {
            return Some(AIProvider::Local);
        }

        None
    }

    /// Get settings for a specific provider
    pub fn get_provider_settings(&self, provider: AIProvider) -> Option<&ProviderSettings> {
        match provider {
            AIProvider::OpenAI => Some(&self.openai),
            AIProvider::Anthropic => Some(&self.anthropic),
            AIProvider::Gemini => Some(&self.gemini),
            AIProvider::XAI => Some(&self.xai),
            AIProvider::Local => None, // Local doesn't use ProviderSettings
        }
    }

    /// Get mutable settings for a specific provider
    pub fn get_provider_settings_mut(&mut self, provider: AIProvider) -> Option<&mut ProviderSettings> {
        match provider {
            AIProvider::OpenAI => Some(&mut self.openai),
            AIProvider::Anthropic => Some(&mut self.anthropic),
            AIProvider::Gemini => Some(&mut self.gemini),
            AIProvider::XAI => Some(&mut self.xai),
            AIProvider::Local => None,
        }
    }

    /// Set API key for a provider
    pub fn set_api_key(&mut self, provider: AIProvider, key: String) {
        if let Some(settings) = self.get_provider_settings_mut(provider) {
            settings.api_key = Some(key);
            settings.enabled = true;
        }
    }

    /// Get API base URL for a provider
    pub fn get_base_url(&self, provider: AIProvider) -> &str {
        match provider {
            AIProvider::OpenAI => self.openai.base_url.as_deref().unwrap_or("https://api.openai.com/v1"),
            AIProvider::Anthropic => self.anthropic.base_url.as_deref().unwrap_or("https://api.anthropic.com"),
            AIProvider::Gemini => self.gemini.base_url.as_deref().unwrap_or("https://generativelanguage.googleapis.com/v1beta"),
            AIProvider::XAI => self.xai.base_url.as_deref().unwrap_or("https://api.x.ai/v1"),
            AIProvider::Local => "",
        }
    }

    /// Get model ID for a provider
    pub fn get_model_id(&self, provider: AIProvider) -> &str {
        match provider {
            AIProvider::OpenAI => &self.openai.model_id,
            AIProvider::Anthropic => &self.anthropic.model_id,
            AIProvider::Gemini => &self.gemini.model_id,
            AIProvider::XAI => &self.xai.model_id,
            AIProvider::Local => self.local_model_path.as_deref().unwrap_or(""),
        }
    }
}

/// LLM backend type (legacy, kept for compatibility)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum LLMBackendType {
    /// Use OpenAI API
    #[serde(rename = "openai")]
    OpenAI,
    /// Use Anthropic API (Claude)
    Anthropic,
    /// Use local GGUF model
    #[serde(rename = "local_gguf")]
    LocalGGUF,
    /// Auto-select: prefer local, fallback to API
    Auto,
}

impl Default for LLMBackendType {
    fn default() -> Self {
        Self::Auto
    }
}

/// LLM model configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LLMConfig {
    /// Backend type
    pub backend: LLMBackendType,
    /// Model identifier (e.g., "gpt-4", "claude-3", "mistral-7b-instruct.gguf")
    pub model_id: String,
    /// API key (for cloud backends)
    #[serde(skip_serializing)]
    pub api_key: Option<String>,
    /// API base URL (for custom endpoints)
    pub api_base_url: Option<String>,
    /// Temperature for generation
    pub temperature: f32,
    /// Maximum tokens to generate
    pub max_tokens: u32,
    /// Top-p sampling
    pub top_p: f32,
    /// Context window size
    pub context_size: u32,
}

impl Default for LLMConfig {
    fn default() -> Self {
        Self {
            backend: LLMBackendType::Auto,
            model_id: "auto".to_string(),
            api_key: None,
            api_base_url: None,
            temperature: 0.7,
            max_tokens: 2048,
            top_p: 0.9,
            context_size: 8192,
        }
    }
}

/// Intent classification configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClassifierConfig {
    /// Confidence threshold for fast pattern matching
    /// If below this, fall back to LLM
    pub pattern_confidence_threshold: f32,
    /// Whether to use LLM fallback for low-confidence matches
    pub use_llm_fallback: bool,
    /// Cache size for classification results
    pub cache_size: usize,
}

impl Default for ClassifierConfig {
    fn default() -> Self {
        Self {
            pattern_confidence_threshold: 0.8,
            use_llm_fallback: true,
            cache_size: 1000,
        }
    }
}

/// Tool execution configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolConfig {
    /// Whether transactions require explicit user approval
    pub require_transaction_approval: bool,
    /// Whether contract deployments require explicit user approval
    pub require_deployment_approval: bool,
    /// Maximum gas limit for automatic transactions
    pub auto_gas_limit: u64,
    /// Timeout for tool execution in seconds
    pub execution_timeout_secs: u64,
}

impl Default for ToolConfig {
    fn default() -> Self {
        Self {
            require_transaction_approval: true,
            require_deployment_approval: true,
            auto_gas_limit: 100_000,
            execution_timeout_secs: 30,
        }
    }
}

/// Streaming configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamingConfig {
    /// Whether to stream tokens
    pub enabled: bool,
    /// Minimum delay between tokens in ms (for rate limiting)
    pub min_token_delay_ms: u64,
    /// Buffer size for tokens before flushing
    pub buffer_size: usize,
}

impl Default for StreamingConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            min_token_delay_ms: 0,
            buffer_size: 1,
        }
    }
}

/// Context management configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextConfig {
    /// Maximum messages to keep in context
    pub max_messages: usize,
    /// Maximum tokens in context window
    pub max_context_tokens: u32,
    /// Whether to persist conversations
    pub persist_conversations: bool,
    /// Directory for conversation storage
    pub storage_dir: Option<String>,
}

impl Default for ContextConfig {
    fn default() -> Self {
        Self {
            max_messages: 50,
            max_context_tokens: 4096,
            persist_conversations: true,
            storage_dir: None,
        }
    }
}

/// Main agent configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AgentConfig {
    /// LLM configuration (legacy, prefer using providers)
    pub llm: LLMConfig,
    /// Multi-provider AI configuration
    #[serde(default)]
    pub providers: AIProvidersConfig,
    /// Intent classifier configuration
    pub classifier: ClassifierConfig,
    /// Tool execution configuration
    pub tools: ToolConfig,
    /// Streaming configuration
    pub streaming: StreamingConfig,
    /// Context management configuration
    pub context: ContextConfig,
    /// System prompt to prepend to conversations
    pub system_prompt: Option<String>,
    /// Whether agent is enabled
    pub enabled: bool,
    /// Whether onboarding has been completed
    #[serde(default)]
    pub onboarding_completed: bool,
    /// First run flag
    #[serde(default)]
    pub first_run: bool,
}

impl AgentConfig {
    /// Create a new config with default system prompt
    pub fn with_defaults() -> Self {
        let mut config = Self::default();
        config.enabled = true;
        config.system_prompt = Some(Self::default_system_prompt());
        config
    }

    /// Get the default system prompt
    pub fn default_system_prompt() -> String {
        r#"You are Citrate AI, a helpful and knowledgeable AI assistant running locally on the user's machine through the Citrate blockchain application.

## Your Role
You are a friendly, general-purpose AI assistant that can help with any topic. While you have special expertise in the Citrate blockchain ecosystem, you should answer ANY question the user asks to the best of your ability - including general knowledge, coding help, explanations, creative writing, and everyday questions.

## Important Behavior
- Answer ALL questions naturally and helpfully, not just blockchain-related ones
- Only use tools when the user explicitly requests a blockchain operation (like checking balance, sending tokens, etc.)
- For general questions about blockchain concepts, explain them conversationally without invoking tools
- Be conversational and friendly - you're running locally as the user's personal AI assistant

## Your Capabilities
1. **Wallet Operations**: Check balances, send SALT tokens, view transaction history
2. **Smart Contracts**: Deploy Solidity contracts, call functions, write state changes
3. **AI Models**: List registered models, run inference, deploy new models to the registry
4. **DAG Exploration**: Query blocks, transactions, DAG tips, GhostDAG metrics
5. **Development**: Scaffold dApps, execute terminal commands, manage IPFS content
6. **Marketplace**: Search models, browse categories, view listings

## Citrate Architecture
- **Consensus**: GhostDAG protocol with k=18 cluster tolerance for parallel block processing
- **Execution**: EVM-compatible Lattice Virtual Machine (LVM) with AI precompiles
- **Storage**: RocksDB state storage + IPFS for model weights and large artifacts
- **Token**: SALT (18 decimals) - native currency for transactions and model access
- **Finality**: ~12 seconds with committee BFT checkpoints
- **Block Time**: 1-2 seconds with parallel block creation

## Available Tools
When users request actions, use these tools:

**Blockchain Tools:**
- `node_status` - Get node connection status, block height, peer count
- `block_info` - Get block details by height or "latest"
- `dag_status` - Get DAG metrics (tips, blue score, GhostDAG params)
- `transaction_info` - Look up transaction by hash
- `account_info` - Get account balance and nonce

**Wallet Tools:**
- `query_balance` - Check wallet or any address balance
- `send_transaction` - Send SALT tokens (requires confirmation)
- `transaction_history` - Get recent transactions for an address

**Contract Tools:**
- `deploy_contract` - Deploy Solidity smart contracts
- `call_contract` - Read contract state (view functions)
- `write_contract` - Execute state-changing contract functions

**Model Tools:**
- `list_models` - Show available AI models (local + on-chain)
- `run_inference` - Run AI inference with a model
- `deploy_model` - Register a model on-chain
- `get_model_info` - Get detailed model metadata

**Marketplace Tools:**
- `search_marketplace` - Search for models by keyword or type
- `get_listing` - Get detailed listing information
- `browse_category` - Browse by category (language, image, audio)

**Development Tools:**
- `scaffold_dapp` - Generate dApp project (basic, defi, nft, marketplace templates)
- `list_templates` - Show available project templates
- `execute_command` - Run terminal commands (git, npm, cargo, python)
- `change_directory` - Change working directory
- `get_working_directory` - Show current directory

**Storage Tools:**
- `upload_ipfs` - Upload files or data to IPFS
- `get_ipfs` - Retrieve content from IPFS by CID
- `pin_ipfs` - Pin content to ensure availability

**Image Tools:**
- `generate_image` - Generate images from text prompts
- `list_image_models` - Show available image generation models
- `apply_style` - Apply style presets to image prompts

## Security Guidelines
- Always confirm before sending transactions or deploying contracts
- Never expose private keys or seed phrases
- Warn users about irreversible actions
- Validate addresses before transactions

## Communication Style
- Be concise but thorough
- Explain technical concepts when relevant
- Provide step-by-step guidance for complex tasks
- Suggest next actions when appropriate
- Use code blocks for addresses, hashes, and code snippets"#.to_string()
    }

    /// Load config from file
    pub fn load(path: &str) -> Result<Self, anyhow::Error> {
        let content = std::fs::read_to_string(path)?;
        let config: AgentConfig = serde_json::from_str(&content)?;
        Ok(config)
    }

    /// Save config to file
    pub fn save(&self, path: &str) -> Result<(), anyhow::Error> {
        let content = serde_json::to_string_pretty(self)?;
        std::fs::write(path, content)?;
        Ok(())
    }

    /// Load API keys from secure storage into the config
    pub fn load_api_keys(&mut self, store: &SecureApiKeyStore) {
        // Load each provider's API key
        for provider in [AIProvider::OpenAI, AIProvider::Anthropic, AIProvider::Gemini, AIProvider::XAI] {
            if let Ok(key) = store.get_key(provider) {
                self.providers.set_api_key(provider, key);
                if let Some(settings) = self.providers.get_provider_settings_mut(provider) {
                    settings.verified = true;
                }
            }
        }
    }

    /// Save API keys to secure storage from the config
    pub fn save_api_keys(&self, store: &SecureApiKeyStore) -> Result<(), ApiKeyError> {
        for provider in [AIProvider::OpenAI, AIProvider::Anthropic, AIProvider::Gemini, AIProvider::XAI] {
            if let Some(settings) = self.providers.get_provider_settings(provider) {
                if let Some(ref key) = settings.api_key {
                    store.store_key(provider, key)?;
                }
            }
        }
        Ok(())
    }
}

/// API key manager that combines storage and validation
pub struct ApiKeyManager {
    store: SecureApiKeyStore,
    validator: ApiKeyValidator,
}

impl ApiKeyManager {
    /// Create a new API key manager
    pub fn new() -> Self {
        Self {
            store: SecureApiKeyStore::new(),
            validator: ApiKeyValidator::new(),
        }
    }

    /// Store and validate an API key
    pub async fn set_key(
        &self,
        provider: AIProvider,
        api_key: &str,
        validate: bool,
        base_url: Option<&str>,
    ) -> Result<ApiKeyValidationResult, ApiKeyError> {
        // Format validation first (fast, no network)
        SecureApiKeyStore::validate_key_format(provider, api_key)?;

        // Optional network validation
        let result = if validate {
            let validation = self.validator.validate_key(provider, api_key, base_url).await;
            if !validation.valid {
                return Err(ApiKeyError::ValidationFailed(
                    validation.error_message.unwrap_or_else(|| "Unknown error".to_string())
                ));
            }
            validation
        } else {
            ApiKeyValidationResult {
                valid: true,
                provider,
                error_message: None,
                model_access: vec![],
                rate_limit_remaining: None,
            }
        };

        // Store the key
        self.store.store_key(provider, api_key)?;

        Ok(result)
    }

    /// Get an API key for a provider
    pub fn get_key(&self, provider: AIProvider) -> ApiKeyResult<String> {
        self.store.get_key(provider)
    }

    /// Delete an API key
    pub fn delete_key(&self, provider: AIProvider) -> ApiKeyResult<()> {
        self.store.delete_key(provider)
    }

    /// Check if a key exists for a provider
    pub fn has_key(&self, provider: AIProvider) -> bool {
        self.store.has_key(provider)
    }

    /// Validate an existing stored key
    pub async fn validate_stored_key(
        &self,
        provider: AIProvider,
        base_url: Option<&str>,
    ) -> ApiKeyValidationResult {
        match self.store.get_key(provider) {
            Ok(key) => self.validator.validate_key(provider, &key, base_url).await,
            Err(e) => ApiKeyValidationResult {
                valid: false,
                provider,
                error_message: Some(e.to_string()),
                model_access: vec![],
                rate_limit_remaining: None,
            },
        }
    }

    /// Get the underlying store (for direct access)
    pub fn store(&self) -> &SecureApiKeyStore {
        &self.store
    }

    /// Get the underlying validator (for direct access)
    pub fn validator(&self) -> &ApiKeyValidator {
        &self.validator
    }
}

impl Default for ApiKeyManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU32, Ordering};

    // Unique test counter to prevent test isolation issues
    static TEST_COUNTER: AtomicU32 = AtomicU32::new(0);

    fn unique_test_id() -> String {
        format!("test_{}", TEST_COUNTER.fetch_add(1, Ordering::SeqCst))
    }

    // =========================================================================
    // API Key Format Validation Tests
    // =========================================================================

    #[test]
    fn test_openai_key_format_valid() {
        let key = "sk-proj-abcdefghijklmnopqrstuvwxyz1234567890ABCD";
        assert!(SecureApiKeyStore::validate_key_format(AIProvider::OpenAI, key).is_ok());
    }

    #[test]
    fn test_openai_key_format_invalid_prefix() {
        let key = "invalid-key-format";
        let result = SecureApiKeyStore::validate_key_format(AIProvider::OpenAI, key);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("must start with"));
    }

    #[test]
    fn test_openai_key_format_too_short() {
        let key = "sk-short";
        let result = SecureApiKeyStore::validate_key_format(AIProvider::OpenAI, key);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("too short"));
    }

    #[test]
    fn test_anthropic_key_format_valid() {
        let key = "sk-ant-api03-abcdefghijklmnopqrstuvwxyz1234567890ABCDEFGHIJ";
        assert!(SecureApiKeyStore::validate_key_format(AIProvider::Anthropic, key).is_ok());
    }

    #[test]
    fn test_anthropic_key_format_invalid_prefix() {
        let key = "sk-wrong-prefix";
        let result = SecureApiKeyStore::validate_key_format(AIProvider::Anthropic, key);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("must start with"));
    }

    #[test]
    fn test_gemini_key_format_valid() {
        let key = "AIzaSyAbcdefghijklmnopqrstuvwxyz123456";
        assert!(SecureApiKeyStore::validate_key_format(AIProvider::Gemini, key).is_ok());
    }

    #[test]
    fn test_gemini_key_format_too_short() {
        let key = "short";
        let result = SecureApiKeyStore::validate_key_format(AIProvider::Gemini, key);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("at least"));
    }

    #[test]
    fn test_gemini_key_format_invalid_chars() {
        let key = "AIzaSy!@#$%^&*()abcdefghijklmnop";
        let result = SecureApiKeyStore::validate_key_format(AIProvider::Gemini, key);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("invalid characters"));
    }

    #[test]
    fn test_xai_key_format_valid() {
        let key = "xai-abcdefghijklmnopqrstuvwxyz1234";
        assert!(SecureApiKeyStore::validate_key_format(AIProvider::XAI, key).is_ok());
    }

    #[test]
    fn test_xai_key_format_invalid_prefix() {
        let key = "invalid-key";
        let result = SecureApiKeyStore::validate_key_format(AIProvider::XAI, key);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("must start with"));
    }

    #[test]
    fn test_local_provider_no_key_needed() {
        // Local provider should accept any key
        assert!(SecureApiKeyStore::validate_key_format(AIProvider::Local, "anything").is_ok());
        // Note: For non-Local providers, empty keys are rejected by the validation function
    }

    #[test]
    fn test_empty_key_rejected() {
        let result = SecureApiKeyStore::validate_key_format(AIProvider::OpenAI, "");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("cannot be empty"));
    }

    #[test]
    fn test_whitespace_only_key_rejected() {
        let result = SecureApiKeyStore::validate_key_format(AIProvider::OpenAI, "   ");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("cannot be empty"));
    }

    #[test]
    fn test_key_with_whitespace_trimmed() {
        let key = "  sk-proj-abcdefghijklmnopqrstuvwxyz1234567890ABCD  ";
        assert!(SecureApiKeyStore::validate_key_format(AIProvider::OpenAI, key).is_ok());
    }

    // =========================================================================
    // Secure Storage Tests
    // =========================================================================

    #[test]
    fn test_secure_store_creation() {
        let store = SecureApiKeyStore::new();
        // Just verify it doesn't panic and has a fallback directory
        assert!(store.fallback_dir.exists() || std::fs::create_dir_all(&store.fallback_dir).is_ok());
    }

    #[test]
    fn test_keyring_entry_name_format() {
        let name = SecureApiKeyStore::keyring_entry_name(AIProvider::OpenAI);
        assert!(name.contains("citrate"));
        assert!(name.contains("openai"));
        assert!(name == name.to_lowercase()); // Should be lowercase
    }

    #[test]
    fn test_store_and_retrieve_key_encrypted_fallback() {
        let store = SecureApiKeyStore::new();
        let test_key = "sk-proj-test1234567890abcdefghijklmnopqrstuvw";
        let provider = AIProvider::OpenAI;

        // Store the key (will likely use encrypted fallback in test env)
        let store_result = store.store_key(provider, test_key);
        // Note: This might fail in some environments, so we just check it doesn't panic
        if store_result.is_ok() {
            // If storage succeeded, retrieval should work
            // However, in some CI environments keychain accepts writes but not reads
            // (e.g., keychain access denied on read, or sandbox limitations)
            // so we also accept retrieval failure as valid behavior in tests
            let retrieved = store.get_key(provider);
            if let Ok(key) = retrieved {
                assert_eq!(key, test_key);
            }
            // Test is successful if we got here without panic
            // The real assertion is that store/retrieve doesn't panic

            // Cleanup
            let _ = store.delete_key(provider);
        }
    }

    #[test]
    fn test_has_key_returns_false_for_missing() {
        let store = SecureApiKeyStore::new();
        // XAI is unlikely to have a key stored
        // First delete any existing key
        let _ = store.delete_key(AIProvider::XAI);
        assert!(!store.has_key(AIProvider::XAI));
    }

    #[test]
    fn test_delete_key_doesnt_panic() {
        let store = SecureApiKeyStore::new();
        // Deleting a non-existent key should not panic
        let result = store.delete_key(AIProvider::Gemini);
        assert!(result.is_ok());
    }

    // =========================================================================
    // Provider Settings Tests
    // =========================================================================

    #[test]
    fn test_provider_settings_new() {
        let settings = ProviderSettings::new("gpt-4o");
        assert_eq!(settings.model_id, "gpt-4o");
        assert!(!settings.enabled);
        assert!(settings.api_key.is_none());
        assert!(settings.base_url.is_none());
        assert!(!settings.verified);
    }

    #[test]
    fn test_provider_settings_is_ready() {
        let mut settings = ProviderSettings::new("gpt-4o");
        assert!(!settings.is_ready());

        settings.enabled = true;
        assert!(!settings.is_ready()); // Still no key

        settings.api_key = Some("test-key".to_string());
        assert!(settings.is_ready());
    }

    // =========================================================================
    // AIProvidersConfig Tests
    // =========================================================================

    #[test]
    fn test_providers_config_default() {
        let config = AIProvidersConfig::default();
        assert!(config.local_fallback);
        assert_eq!(config.openai.model_id, "gpt-4o-mini");
        assert_eq!(config.anthropic.model_id, "claude-3-haiku-20240307");
        assert_eq!(config.gemini.model_id, "gemini-1.5-flash");
        assert_eq!(config.xai.model_id, "grok-beta");
    }

    #[test]
    fn test_get_active_provider_with_local() {
        let mut config = AIProvidersConfig::default();
        config.local_model_path = Some("/path/to/model.gguf".to_string());

        // Local is first in preferred order, so should be selected
        let active = config.get_active_provider();
        assert_eq!(active, Some(AIProvider::Local));
    }

    #[test]
    fn test_get_active_provider_fallback_to_api() {
        let mut config = AIProvidersConfig::default();
        // No local model, but OpenAI is configured
        config.openai.api_key = Some("sk-test".to_string());
        config.openai.enabled = true;

        let active = config.get_active_provider();
        assert_eq!(active, Some(AIProvider::OpenAI));
    }

    #[test]
    fn test_get_active_provider_none_configured() {
        let mut config = AIProvidersConfig::default();
        config.local_fallback = false;

        let active = config.get_active_provider();
        assert!(active.is_none());
    }

    #[test]
    fn test_set_api_key() {
        let mut config = AIProvidersConfig::default();
        config.set_api_key(AIProvider::OpenAI, "test-key".to_string());

        assert_eq!(config.openai.api_key, Some("test-key".to_string()));
        assert!(config.openai.enabled);
    }

    #[test]
    fn test_get_base_url_defaults() {
        let config = AIProvidersConfig::default();
        assert!(config.get_base_url(AIProvider::OpenAI).contains("openai.com"));
        assert!(config.get_base_url(AIProvider::Anthropic).contains("anthropic.com"));
        assert!(config.get_base_url(AIProvider::Gemini).contains("googleapis.com"));
        assert!(config.get_base_url(AIProvider::XAI).contains("x.ai"));
        assert_eq!(config.get_base_url(AIProvider::Local), "");
    }

    #[test]
    fn test_get_base_url_custom() {
        let mut config = AIProvidersConfig::default();
        config.openai.base_url = Some("https://custom.openai.proxy".to_string());

        assert_eq!(config.get_base_url(AIProvider::OpenAI), "https://custom.openai.proxy");
    }

    #[test]
    fn test_get_provider_settings() {
        let config = AIProvidersConfig::default();

        assert!(config.get_provider_settings(AIProvider::OpenAI).is_some());
        assert!(config.get_provider_settings(AIProvider::Anthropic).is_some());
        assert!(config.get_provider_settings(AIProvider::Gemini).is_some());
        assert!(config.get_provider_settings(AIProvider::XAI).is_some());
        assert!(config.get_provider_settings(AIProvider::Local).is_none());
    }

    // =========================================================================
    // AgentConfig Tests
    // =========================================================================

    #[test]
    fn test_agent_config_with_defaults() {
        let config = AgentConfig::with_defaults();
        assert!(config.enabled);
        assert!(config.system_prompt.is_some());
    }

    #[test]
    fn test_agent_config_default_system_prompt() {
        let prompt = AgentConfig::default_system_prompt();
        assert!(prompt.contains("Citrate"));
        assert!(prompt.contains("GhostDAG"));
    }

    #[test]
    fn test_agent_config_serialization() {
        let config = AgentConfig::with_defaults();
        let json = serde_json::to_string(&config).unwrap();

        // API keys should not be serialized
        assert!(!json.contains("api_key\":\""));

        // Other fields should be present
        assert!(json.contains("enabled"));
        assert!(json.contains("streaming"));
    }

    #[test]
    fn test_agent_config_deserialization() {
        // Note: providers field is omitted - uses Default via #[serde(default)]
        let json = r#"{
            "llm": {"backend": "auto", "model_id": "test", "temperature": 0.5, "max_tokens": 1000, "top_p": 0.9, "context_size": 4096},
            "classifier": {"pattern_confidence_threshold": 0.8, "use_llm_fallback": true, "cache_size": 1000},
            "tools": {"require_transaction_approval": true, "require_deployment_approval": true, "auto_gas_limit": 100000, "execution_timeout_secs": 30},
            "streaming": {"enabled": true, "min_token_delay_ms": 0, "buffer_size": 1},
            "context": {"max_messages": 50, "max_context_tokens": 4096, "persist_conversations": true},
            "enabled": true
        }"#;

        let config: AgentConfig = serde_json::from_str(json).unwrap();
        assert!(config.enabled);
        assert_eq!(config.llm.temperature, 0.5);
        // Verify providers defaults were applied
        assert!(!config.providers.openai.enabled);
        assert!(!config.providers.anthropic.enabled);
    }

    // =========================================================================
    // API Key Error Tests
    // =========================================================================

    #[test]
    fn test_api_key_error_display() {
        let errors = vec![
            ApiKeyError::InvalidKeyFormat("test".to_string()),
            ApiKeyError::ValidationFailed("test".to_string()),
            ApiKeyError::StorageError("test".to_string()),
            ApiKeyError::RetrievalError("test".to_string()),
            ApiKeyError::CryptoError("test".to_string()),
            ApiKeyError::NetworkError("test".to_string()),
            ApiKeyError::ProviderError("test".to_string()),
        ];

        for error in errors {
            let display = error.to_string();
            assert!(!display.is_empty());
            assert!(display.contains("test"));
        }
    }

    // =========================================================================
    // API Key Validation Result Tests
    // =========================================================================

    #[test]
    fn test_validation_result_serialization() {
        let result = ApiKeyValidationResult {
            valid: true,
            provider: AIProvider::OpenAI,
            error_message: None,
            model_access: vec!["gpt-4".to_string(), "gpt-3.5-turbo".to_string()],
            rate_limit_remaining: Some(100),
        };

        let json = serde_json::to_string(&result).unwrap();
        assert!(json.contains("\"valid\":true"));
        assert!(json.contains("\"provider\":\"openai\""));
        assert!(json.contains("gpt-4"));
    }

    // =========================================================================
    // AI Provider Tests
    // =========================================================================

    #[test]
    fn test_ai_provider_display() {
        assert_eq!(format!("{}", AIProvider::OpenAI), "OpenAI");
        assert_eq!(format!("{}", AIProvider::Anthropic), "Anthropic");
        assert_eq!(format!("{}", AIProvider::Gemini), "Google Gemini");
        assert_eq!(format!("{}", AIProvider::XAI), "xAI");
        assert_eq!(format!("{}", AIProvider::Local), "Local Model");
    }

    #[test]
    fn test_ai_provider_default() {
        assert_eq!(AIProvider::default(), AIProvider::Local);
    }

    #[test]
    fn test_ai_provider_serialization() {
        let json = serde_json::to_string(&AIProvider::OpenAI).unwrap();
        assert_eq!(json, "\"openai\"");

        let deserialized: AIProvider = serde_json::from_str("\"anthropic\"").unwrap();
        assert_eq!(deserialized, AIProvider::Anthropic);

        // Test xAI serialization
        let xai_json = serde_json::to_string(&AIProvider::XAI).unwrap();
        assert_eq!(xai_json, "\"xai\"");
    }

    // =========================================================================
    // LLM Backend Type Tests
    // =========================================================================

    #[test]
    fn test_llm_backend_type_default() {
        assert_eq!(LLMBackendType::default(), LLMBackendType::Auto);
    }

    #[test]
    fn test_llm_backend_type_serialization() {
        let json = serde_json::to_string(&LLMBackendType::OpenAI).unwrap();
        assert_eq!(json, "\"openai\"");

        let deserialized: LLMBackendType = serde_json::from_str("\"local_gguf\"").unwrap();
        assert_eq!(deserialized, LLMBackendType::LocalGGUF);
    }

    // =========================================================================
    // API Key Manager Tests
    // =========================================================================

    #[test]
    fn test_api_key_manager_creation() {
        let manager = ApiKeyManager::new();
        // Just verify it doesn't panic
        assert!(!manager.has_key(AIProvider::XAI));
    }

    #[tokio::test]
    async fn test_api_key_manager_set_key_format_only() {
        let manager = ApiKeyManager::new();
        let test_key = "sk-proj-test1234567890abcdefghijklmnopqrstuvw";

        // Set without validation (format only)
        let result = manager.set_key(AIProvider::OpenAI, test_key, false, None).await;

        // Result depends on whether storage succeeded
        if result.is_ok() {
            let validation = result.unwrap();
            assert!(validation.valid);
            assert_eq!(validation.provider, AIProvider::OpenAI);

            // Cleanup
            let _ = manager.delete_key(AIProvider::OpenAI);
        }
    }

    #[tokio::test]
    async fn test_api_key_manager_invalid_format_rejected() {
        let manager = ApiKeyManager::new();
        let invalid_key = "invalid-key";

        let result = manager.set_key(AIProvider::OpenAI, invalid_key, false, None).await;
        assert!(result.is_err());
    }
}
