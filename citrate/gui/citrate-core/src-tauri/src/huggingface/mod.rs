//! HuggingFace Integration Module
//!
//! Provides OAuth PKCE authentication and model browsing/download capabilities
//! for HuggingFace Hub integration with Citrate.
//!
//! Features:
//! - OAuth 2.0 PKCE flow for secure desktop authentication
//! - Model search with GGUF filtering
//! - Resumable downloads with progress tracking
//! - Auto-detection of downloaded models
//! - Tauri event emission for real-time progress updates

use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use futures::StreamExt;
use rand::Rng;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncSeekExt, AsyncWriteExt};
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};

/// HuggingFace API base URL
const HF_API_BASE: &str = "https://huggingface.co/api";
const HF_AUTH_URL: &str = "https://huggingface.co/oauth/authorize";
const HF_TOKEN_URL: &str = "https://huggingface.co/oauth/token";

/// PKCE code verifier length (43-128 characters per RFC 7636)
const PKCE_VERIFIER_LENGTH: usize = 64;

/// Recommended GGUF quantization formats (in order of preference)
const RECOMMENDED_QUANTIZATIONS: &[&str] = &[
    "Q4_K_M", "Q4_K_S", "Q5_K_M", "Q5_K_S", "Q6_K", "Q8_0", "Q4_0", "Q5_0",
];

/// HuggingFace model info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HFModelInfo {
    pub id: String,
    #[serde(rename = "modelId")]
    pub model_id: Option<String>,
    pub author: Option<String>,
    pub sha: Option<String>,
    #[serde(rename = "lastModified")]
    pub last_modified: Option<String>,
    pub private: Option<bool>,
    pub disabled: Option<bool>,
    pub gated: Option<String>,
    pub pipeline_tag: Option<String>,
    pub tags: Option<Vec<String>>,
    pub downloads: Option<u64>,
    pub likes: Option<u64>,
    #[serde(rename = "cardData")]
    pub card_data: Option<serde_json::Value>,
    pub siblings: Option<Vec<HFModelFile>>,
}

/// HuggingFace model file info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HFModelFile {
    pub rfilename: String,
    pub size: Option<u64>,
    #[serde(rename = "blobId")]
    pub blob_id: Option<String>,
    pub lfs: Option<HFLfsInfo>,
}

/// LFS file info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HFLfsInfo {
    pub size: u64,
    pub sha256: Option<String>,
    #[serde(rename = "pointerSize")]
    pub pointer_size: Option<u64>,
}

/// HuggingFace user info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HFUserInfo {
    pub id: String,
    pub name: String,
    #[serde(rename = "fullname")]
    pub full_name: Option<String>,
    pub email: Option<String>,
    #[serde(rename = "avatarUrl")]
    pub avatar_url: Option<String>,
    #[serde(rename = "isPro")]
    pub is_pro: Option<bool>,
}

/// OAuth token response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuthToken {
    pub access_token: String,
    pub token_type: String,
    pub expires_in: Option<u64>,
    pub refresh_token: Option<String>,
    pub scope: Option<String>,
}

/// Search parameters for models
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ModelSearchParams {
    pub search: Option<String>,
    pub author: Option<String>,
    pub filter: Option<String>,
    pub sort: Option<String>,
    pub direction: Option<String>,
    pub limit: Option<u32>,
    pub full: Option<bool>,
    pub config: Option<bool>,
}

/// Download progress
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadProgress {
    pub model_id: String,
    pub filename: String,
    pub downloaded: u64,
    pub total: u64,
    pub status: DownloadStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DownloadStatus {
    Pending,
    Downloading,
    Completed,
    Failed,
    Cancelled,
}

/// Authentication state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthState {
    pub authenticated: bool,
    pub user: Option<HFUserInfo>,
    pub token: Option<OAuthToken>,
}

impl Default for AuthState {
    fn default() -> Self {
        Self {
            authenticated: false,
            user: None,
            token: None,
        }
    }
}

/// PKCE challenge for OAuth 2.0 authorization code flow with PKCE
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PkceChallenge {
    /// The code verifier (stored securely, never sent to authorization server)
    pub code_verifier: String,
    /// The code challenge (SHA256 hash of verifier, base64url encoded)
    pub code_challenge: String,
    /// State parameter for CSRF protection
    pub state: String,
    /// Timestamp when challenge was created
    pub created_at: u64,
}

impl PkceChallenge {
    /// Generate a new PKCE challenge
    pub fn generate() -> Self {
        let mut rng = rand::thread_rng();

        // Generate code verifier (RFC 7636: 43-128 characters from unreserved set)
        let code_verifier: String = (0..PKCE_VERIFIER_LENGTH)
            .map(|_| {
                const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-._~";
                let idx = rng.gen_range(0..CHARSET.len());
                CHARSET[idx] as char
            })
            .collect();

        // Generate code challenge (S256 method: base64url(SHA256(verifier)))
        let mut hasher = Sha256::new();
        hasher.update(code_verifier.as_bytes());
        let hash = hasher.finalize();
        let code_challenge = URL_SAFE_NO_PAD.encode(hash);

        // Generate state for CSRF protection
        let state: String = (0..32)
            .map(|_| {
                const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
                let idx = rng.gen_range(0..CHARSET.len());
                CHARSET[idx] as char
            })
            .collect();

        let created_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        Self {
            code_verifier,
            code_challenge,
            state,
            created_at,
        }
    }

    /// Check if the challenge has expired (10 minute timeout)
    pub fn is_expired(&self) -> bool {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        now - self.created_at > 600
    }
}

/// Model info with GGUF-specific details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GGUFModelInfo {
    /// Model ID on HuggingFace
    pub model_id: String,
    /// Model name (last part of ID)
    pub name: String,
    /// Author/organization
    pub author: String,
    /// Available GGUF files with quantization info
    pub files: Vec<GGUFFileInfo>,
    /// Total downloads
    pub downloads: u64,
    /// Likes count
    pub likes: u64,
    /// Last modified date
    pub last_modified: Option<String>,
    /// Model description from card data
    pub description: Option<String>,
    /// Model tags
    pub tags: Vec<String>,
    /// Whether this model requires authentication
    pub gated: bool,
}

/// GGUF file information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GGUFFileInfo {
    /// Filename
    pub filename: String,
    /// File size in bytes
    pub size: u64,
    /// Quantization type (Q4_K_M, Q5_K_S, etc.)
    pub quantization: Option<String>,
    /// Whether this is a recommended quantization
    pub recommended: bool,
    /// Download URL
    pub download_url: String,
}

/// Local model information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalModelInfo {
    /// Model ID (from HuggingFace or local path)
    pub model_id: String,
    /// Local file path
    pub path: PathBuf,
    /// File size
    pub size: u64,
    /// Quantization type (if detectable from filename)
    pub quantization: Option<String>,
    /// Whether this model is currently loaded
    pub loaded: bool,
}

/// HuggingFace client configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HFConfig {
    /// OAuth client ID (for your HF application)
    pub client_id: Option<String>,
    /// Redirect URI for OAuth callback
    pub redirect_uri: String,
    /// Models download directory
    pub models_dir: PathBuf,
    /// Enable GGUF format filtering
    pub prefer_gguf: bool,
    /// Maximum concurrent downloads
    pub max_concurrent_downloads: u32,
}

impl Default for HFConfig {
    fn default() -> Self {
        let models_dir = dirs::data_local_dir()
            .map(|d| d.join("citrate").join("models"))
            .unwrap_or_else(|| PathBuf::from(".citrate/models"));

        Self {
            client_id: None,
            redirect_uri: "http://localhost:8787/callback".to_string(),
            models_dir,
            prefer_gguf: true,
            max_concurrent_downloads: 2,
        }
    }
}

/// HuggingFace integration manager
pub struct HuggingFaceManager {
    config: Arc<RwLock<HFConfig>>,
    auth_state: Arc<RwLock<AuthState>>,
    http_client: Client,
    downloads: Arc<RwLock<Vec<DownloadProgress>>>,
    /// Active PKCE challenges (keyed by state parameter)
    pkce_challenges: Arc<RwLock<HashMap<String, PkceChallenge>>>,
    /// Active download cancellation tokens
    download_cancellations: Arc<RwLock<HashMap<String, bool>>>,
}

impl HuggingFaceManager {
    /// Create new HuggingFace manager
    pub fn new() -> Self {
        Self::with_config(HFConfig::default())
    }

    /// Create with custom config
    pub fn with_config(config: HFConfig) -> Self {
        let http_client = Client::builder()
            .user_agent("Citrate-GUI/1.0")
            .timeout(std::time::Duration::from_secs(60))
            .connect_timeout(std::time::Duration::from_secs(10))
            .build()
            .unwrap_or_else(|_| Client::new());

        Self {
            config: Arc::new(RwLock::new(config)),
            auth_state: Arc::new(RwLock::new(AuthState::default())),
            http_client,
            downloads: Arc::new(RwLock::new(Vec::new())),
            pkce_challenges: Arc::new(RwLock::new(HashMap::new())),
            download_cancellations: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Get current config
    pub async fn get_config(&self) -> HFConfig {
        self.config.read().await.clone()
    }

    /// Update config
    pub async fn update_config(&self, config: HFConfig) {
        *self.config.write().await = config;
    }

    /// Get authentication state
    pub async fn get_auth_state(&self) -> AuthState {
        self.auth_state.read().await.clone()
    }

    /// Generate OAuth authorization URL with PKCE (RFC 7636)
    /// Returns (authorization_url, state) - the state should be used to verify the callback
    pub async fn start_auth_flow(&self) -> Result<(String, String), String> {
        let config = self.config.read().await;

        let client_id = config.client_id.as_ref()
            .ok_or_else(|| "HuggingFace client_id not configured. Please set your HuggingFace OAuth application client ID.".to_string())?;

        // Generate PKCE challenge
        let challenge = PkceChallenge::generate();
        let state = challenge.state.clone();

        // Store challenge for later verification
        {
            let mut challenges = self.pkce_challenges.write().await;
            // Clean up expired challenges
            challenges.retain(|_, c| !c.is_expired());
            challenges.insert(state.clone(), challenge.clone());
        }

        // Build authorization URL with PKCE
        let url = format!(
            "{}?client_id={}&redirect_uri={}&response_type=code&scope=read%20repo&state={}&code_challenge={}&code_challenge_method=S256",
            HF_AUTH_URL,
            urlencoding::encode(client_id),
            urlencoding::encode(&config.redirect_uri),
            urlencoding::encode(&state),
            urlencoding::encode(&challenge.code_challenge)
        );

        info!("Generated OAuth authorization URL with PKCE");
        Ok((url, state))
    }

    /// Legacy method for backwards compatibility
    pub async fn get_auth_url(&self, _state: &str) -> Result<String, String> {
        let (url, _) = self.start_auth_flow().await?;
        Ok(url)
    }

    /// Exchange authorization code for token using PKCE verifier
    pub async fn exchange_code_with_pkce(&self, code: &str, state: &str) -> Result<OAuthToken, String> {
        let config = self.config.read().await;

        let client_id = config.client_id.as_ref()
            .ok_or_else(|| "HuggingFace client_id not configured".to_string())?;

        // Retrieve and validate PKCE challenge
        let challenge = {
            let mut challenges = self.pkce_challenges.write().await;
            challenges.remove(state)
                .ok_or_else(|| "Invalid or expired authorization state. Please try again.".to_string())?
        };

        if challenge.is_expired() {
            return Err("Authorization session expired. Please try again.".to_string());
        }

        // Exchange code for token with PKCE verifier
        let params = [
            ("grant_type", "authorization_code"),
            ("code", code),
            ("redirect_uri", config.redirect_uri.as_str()),
            ("client_id", client_id.as_str()),
            ("code_verifier", challenge.code_verifier.as_str()),
        ];

        let response = self.http_client
            .post(HF_TOKEN_URL)
            .form(&params)
            .send()
            .await
            .map_err(|e| format!("Token exchange failed: {}", e))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            error!("Token exchange failed: {} - {}", status, error_text);
            return Err(format!("Token exchange failed ({}): {}", status, error_text));
        }

        let token: OAuthToken = response.json().await
            .map_err(|e| format!("Failed to parse token response: {}", e))?;

        // Store token and fetch user info
        self.set_token(token.clone()).await?;

        info!("HuggingFace OAuth PKCE flow completed successfully");
        Ok(token)
    }

    /// Exchange authorization code for token (legacy - uses PKCE internally)
    pub async fn exchange_code(&self, code: &str) -> Result<OAuthToken, String> {
        // For legacy callers, try to find any valid challenge
        let state = {
            let challenges = self.pkce_challenges.read().await;
            challenges.keys().next().cloned()
        };

        match state {
            Some(s) => self.exchange_code_with_pkce(code, &s).await,
            None => {
                // Fallback to non-PKCE flow for API tokens
                warn!("No PKCE challenge found, attempting direct token exchange");
                self.exchange_code_direct(code).await
            }
        }
    }

    /// Direct code exchange without PKCE (for fallback scenarios)
    async fn exchange_code_direct(&self, code: &str) -> Result<OAuthToken, String> {
        let config = self.config.read().await;

        let client_id = config.client_id.as_ref()
            .ok_or_else(|| "HuggingFace client_id not configured".to_string())?;

        let params = [
            ("grant_type", "authorization_code"),
            ("code", code),
            ("redirect_uri", &config.redirect_uri),
            ("client_id", client_id),
        ];

        let response = self.http_client
            .post(HF_TOKEN_URL)
            .form(&params)
            .send()
            .await
            .map_err(|e| format!("Token exchange failed: {}", e))?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(format!("Token exchange failed: {}", error_text));
        }

        let token: OAuthToken = response.json().await
            .map_err(|e| format!("Failed to parse token: {}", e))?;

        self.set_token(token.clone()).await?;
        Ok(token)
    }

    /// Set access token (from stored credentials or manual entry)
    pub async fn set_token(&self, token: OAuthToken) -> Result<(), String> {
        // Fetch user info with the token
        let user = self.fetch_user_info(&token.access_token).await?;

        let mut state = self.auth_state.write().await;
        state.authenticated = true;
        state.token = Some(token);
        state.user = Some(user);

        info!("HuggingFace authentication successful");
        Ok(())
    }

    /// Set API token directly (for users with HF tokens)
    pub async fn set_api_token(&self, token: &str) -> Result<(), String> {
        let oauth_token = OAuthToken {
            access_token: token.to_string(),
            token_type: "Bearer".to_string(),
            expires_in: None,
            refresh_token: None,
            scope: Some("read".to_string()),
        };

        self.set_token(oauth_token).await
    }

    /// Fetch user info from API
    async fn fetch_user_info(&self, token: &str) -> Result<HFUserInfo, String> {
        let response = self.http_client
            .get(format!("{}/whoami-v2", HF_API_BASE))
            .bearer_auth(token)
            .send()
            .await
            .map_err(|e| format!("Failed to fetch user info: {}", e))?;

        if !response.status().is_success() {
            return Err("Invalid token or unauthorized".to_string());
        }

        response.json().await
            .map_err(|e| format!("Failed to parse user info: {}", e))
    }

    /// Logout
    pub async fn logout(&self) {
        let mut state = self.auth_state.write().await;
        *state = AuthState::default();
        info!("HuggingFace logged out");
    }

    /// Check if authenticated
    pub async fn is_authenticated(&self) -> bool {
        self.auth_state.read().await.authenticated
    }

    /// Search for models
    pub async fn search_models(&self, params: ModelSearchParams) -> Result<Vec<HFModelInfo>, String> {
        let mut url = format!("{}/models", HF_API_BASE);
        let mut query_params = Vec::new();

        if let Some(ref search) = params.search {
            query_params.push(format!("search={}", urlencoding::encode(search)));
        }
        if let Some(ref author) = params.author {
            query_params.push(format!("author={}", urlencoding::encode(author)));
        }
        if let Some(ref filter) = params.filter {
            query_params.push(format!("filter={}", urlencoding::encode(filter)));
        }
        if let Some(ref sort) = params.sort {
            query_params.push(format!("sort={}", sort));
        }
        if let Some(ref direction) = params.direction {
            query_params.push(format!("direction={}", direction));
        }
        if let Some(limit) = params.limit {
            query_params.push(format!("limit={}", limit));
        }
        if params.full.unwrap_or(false) {
            query_params.push("full=true".to_string());
        }

        if !query_params.is_empty() {
            url = format!("{}?{}", url, query_params.join("&"));
        }

        let mut request = self.http_client.get(&url);

        // Add auth if available
        if let Some(ref token) = self.auth_state.read().await.token {
            request = request.bearer_auth(&token.access_token);
        }

        let response = request.send().await
            .map_err(|e| format!("Search failed: {}", e))?;

        if !response.status().is_success() {
            return Err(format!("Search failed: {}", response.status()));
        }

        response.json().await
            .map_err(|e| format!("Failed to parse models: {}", e))
    }

    /// Get model info
    pub async fn get_model(&self, model_id: &str) -> Result<HFModelInfo, String> {
        let url = format!("{}/models/{}", HF_API_BASE, model_id);

        let mut request = self.http_client.get(&url);

        if let Some(ref token) = self.auth_state.read().await.token {
            request = request.bearer_auth(&token.access_token);
        }

        let response = request.send().await
            .map_err(|e| format!("Failed to get model: {}", e))?;

        if !response.status().is_success() {
            return Err(format!("Model not found: {}", response.status()));
        }

        response.json().await
            .map_err(|e| format!("Failed to parse model: {}", e))
    }

    /// List GGUF files for a model
    pub async fn list_gguf_files(&self, model_id: &str) -> Result<Vec<HFModelFile>, String> {
        let model = self.get_model(model_id).await?;

        let files = model.siblings.unwrap_or_default()
            .into_iter()
            .filter(|f| f.rfilename.ends_with(".gguf"))
            .collect();

        Ok(files)
    }

    /// Download a model file
    pub async fn download_file(
        &self,
        model_id: &str,
        filename: &str,
    ) -> Result<PathBuf, String> {
        let config = self.config.read().await;

        // Create model directory
        let model_dir = config.models_dir.join(model_id.replace('/', "__"));
        std::fs::create_dir_all(&model_dir)
            .map_err(|e| format!("Failed to create model directory: {}", e))?;

        let file_path = model_dir.join(filename);

        // Check if already downloaded
        if file_path.exists() {
            info!("Model file already exists: {:?}", file_path);
            return Ok(file_path);
        }

        let url = format!(
            "https://huggingface.co/{}/resolve/main/{}",
            model_id, filename
        );

        info!("Downloading {} to {:?}", url, file_path);

        // Initialize progress tracking
        let progress = DownloadProgress {
            model_id: model_id.to_string(),
            filename: filename.to_string(),
            downloaded: 0,
            total: 0,
            status: DownloadStatus::Pending,
        };

        self.downloads.write().await.push(progress);

        // Start download
        let mut request = self.http_client.get(&url);

        if let Some(ref token) = self.auth_state.read().await.token {
            request = request.bearer_auth(&token.access_token);
        }

        let response = request.send().await
            .map_err(|e| format!("Download failed: {}", e))?;

        if !response.status().is_success() {
            self.update_download_status(model_id, filename, DownloadStatus::Failed).await;
            return Err(format!("Download failed: {}", response.status()));
        }

        let total_size = response.content_length().unwrap_or(0);
        self.update_download_total(model_id, filename, total_size).await;
        self.update_download_status(model_id, filename, DownloadStatus::Downloading).await;

        // Stream to file
        let mut file = tokio::fs::File::create(&file_path).await
            .map_err(|e| format!("Failed to create file: {}", e))?;

        let mut downloaded: u64 = 0;
        let mut stream = response.bytes_stream();

        while let Some(chunk) = stream.next().await {
            let chunk = chunk.map_err(|e| format!("Download error: {}", e))?;
            file.write_all(&chunk).await
                .map_err(|e| format!("Write error: {}", e))?;

            downloaded += chunk.len() as u64;
            self.update_download_progress(model_id, filename, downloaded).await;
        }

        file.flush().await
            .map_err(|e| format!("Flush error: {}", e))?;

        self.update_download_status(model_id, filename, DownloadStatus::Completed).await;
        info!("Download complete: {:?}", file_path);

        Ok(file_path)
    }

    /// Get download progress
    pub async fn get_downloads(&self) -> Vec<DownloadProgress> {
        self.downloads.read().await.clone()
    }

    /// Cancel download
    pub async fn cancel_download(&self, model_id: &str, filename: &str) {
        self.update_download_status(model_id, filename, DownloadStatus::Cancelled).await;
    }

    // Helper methods for download progress
    async fn update_download_status(&self, model_id: &str, filename: &str, status: DownloadStatus) {
        let mut downloads = self.downloads.write().await;
        if let Some(d) = downloads.iter_mut().find(|d| d.model_id == model_id && d.filename == filename) {
            d.status = status;
        }
    }

    async fn update_download_total(&self, model_id: &str, filename: &str, total: u64) {
        let mut downloads = self.downloads.write().await;
        if let Some(d) = downloads.iter_mut().find(|d| d.model_id == model_id && d.filename == filename) {
            d.total = total;
        }
    }

    async fn update_download_progress(&self, model_id: &str, filename: &str, downloaded: u64) {
        let mut downloads = self.downloads.write().await;
        if let Some(d) = downloads.iter_mut().find(|d| d.model_id == model_id && d.filename == filename) {
            d.downloaded = downloaded;
        }
    }

    /// Get local models directory
    pub async fn get_models_dir(&self) -> PathBuf {
        self.config.read().await.models_dir.clone()
    }

    /// List downloaded models
    pub async fn list_local_models(&self) -> Result<Vec<String>, String> {
        let config = self.config.read().await;

        if !config.models_dir.exists() {
            return Ok(Vec::new());
        }

        let mut models = Vec::new();

        let entries = std::fs::read_dir(&config.models_dir)
            .map_err(|e| format!("Failed to read models dir: {}", e))?;

        for entry in entries.flatten() {
            if entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
                if let Some(name) = entry.file_name().to_str() {
                    // Convert back from filesystem-safe name
                    let model_id = name.replace("__", "/");
                    models.push(model_id);
                }
            }
        }

        Ok(models)
    }

    /// Search for GGUF-compatible models on HuggingFace
    /// Filters for models with GGUF files and returns enhanced metadata
    pub async fn search_gguf_models(&self, query: &str, limit: u32) -> Result<Vec<GGUFModelInfo>, String> {
        // Search with GGUF tag filter
        let params = ModelSearchParams {
            search: Some(query.to_string()),
            filter: Some("gguf".to_string()),
            sort: Some("downloads".to_string()),
            direction: Some("-1".to_string()),
            limit: Some(limit),
            full: Some(true),
            ..Default::default()
        };

        let models = self.search_models(params).await?;
        let mut gguf_models = Vec::new();

        for model in models {
            if let Some(gguf_info) = self.convert_to_gguf_info(&model).await {
                gguf_models.push(gguf_info);
            }
        }

        Ok(gguf_models)
    }

    /// Get detailed GGUF model information including all available files
    pub async fn get_gguf_model(&self, model_id: &str) -> Result<GGUFModelInfo, String> {
        let model = self.get_model(model_id).await?;
        self.convert_to_gguf_info(&model).await
            .ok_or_else(|| format!("Model {} has no GGUF files available", model_id))
    }

    /// Convert HFModelInfo to GGUFModelInfo with file analysis
    async fn convert_to_gguf_info(&self, model: &HFModelInfo) -> Option<GGUFModelInfo> {
        let siblings = model.siblings.as_ref()?;

        // Find GGUF files
        let gguf_files: Vec<GGUFFileInfo> = siblings
            .iter()
            .filter(|f| f.rfilename.ends_with(".gguf"))
            .map(|f| {
                let quantization = extract_quantization(&f.rfilename);
                let recommended = quantization.as_ref()
                    .map(|q| RECOMMENDED_QUANTIZATIONS.iter().any(|r| q.contains(r)))
                    .unwrap_or(false);

                let size = f.lfs.as_ref().map(|l| l.size).or(f.size).unwrap_or(0);

                GGUFFileInfo {
                    filename: f.rfilename.clone(),
                    size,
                    quantization,
                    recommended,
                    download_url: format!(
                        "https://huggingface.co/{}/resolve/main/{}",
                        model.id, f.rfilename
                    ),
                }
            })
            .collect();

        if gguf_files.is_empty() {
            return None;
        }

        // Parse model ID into author and name
        let parts: Vec<&str> = model.id.split('/').collect();
        let (author, name) = if parts.len() >= 2 {
            (parts[0].to_string(), parts[1].to_string())
        } else {
            ("unknown".to_string(), model.id.clone())
        };

        // Extract description from card data
        let description = model.card_data.as_ref()
            .and_then(|c| c.get("model_summary"))
            .and_then(|s| s.as_str())
            .map(|s| s.to_string());

        Some(GGUFModelInfo {
            model_id: model.id.clone(),
            name,
            author,
            files: gguf_files,
            downloads: model.downloads.unwrap_or(0),
            likes: model.likes.unwrap_or(0),
            last_modified: model.last_modified.clone(),
            description,
            tags: model.tags.clone().unwrap_or_default(),
            gated: model.gated.is_some(),
        })
    }

    /// Scan local models directory and return detailed info about downloaded models
    pub async fn scan_local_models(&self) -> Result<Vec<LocalModelInfo>, String> {
        let config = self.config.read().await;
        let mut local_models = Vec::new();

        if !config.models_dir.exists() {
            return Ok(local_models);
        }

        // Scan both model subdirectories and root directory for GGUF files
        let entries = std::fs::read_dir(&config.models_dir)
            .map_err(|e| format!("Failed to read models dir: {}", e))?;

        for entry in entries.flatten() {
            let path = entry.path();

            if path.is_dir() {
                // Check subdirectory for GGUF files
                if let Ok(sub_entries) = std::fs::read_dir(&path) {
                    for sub_entry in sub_entries.flatten() {
                        let file_path = sub_entry.path();
                        if file_path.extension().map(|e| e == "gguf").unwrap_or(false) {
                            if let Some(info) = self.create_local_model_info(&file_path, &config.models_dir) {
                                local_models.push(info);
                            }
                        }
                    }
                }
            } else if path.extension().map(|e| e == "gguf").unwrap_or(false) {
                // GGUF file directly in models dir
                if let Some(info) = self.create_local_model_info(&path, &config.models_dir) {
                    local_models.push(info);
                }
            }
        }

        // Sort by size (larger models first)
        local_models.sort_by(|a, b| b.size.cmp(&a.size));

        Ok(local_models)
    }

    /// Create LocalModelInfo from a file path
    fn create_local_model_info(&self, path: &PathBuf, models_dir: &PathBuf) -> Option<LocalModelInfo> {
        let metadata = std::fs::metadata(path).ok()?;
        let filename = path.file_name()?.to_str()?;

        // Try to extract model_id from path
        let model_id = if let Ok(relative) = path.strip_prefix(models_dir) {
            let parts: Vec<&str> = relative.iter()
                .filter_map(|s| s.to_str())
                .collect();
            if parts.len() >= 2 {
                // Directory structure: models_dir/author__model/file.gguf
                parts[0].replace("__", "/")
            } else {
                filename.to_string()
            }
        } else {
            filename.to_string()
        };

        let quantization = extract_quantization(filename);

        Some(LocalModelInfo {
            model_id,
            path: path.clone(),
            size: metadata.len(),
            quantization,
            loaded: false, // Will be updated by model manager
        })
    }

    /// Auto-detect and return the best available local model for inference
    pub async fn auto_select_model(&self) -> Option<LocalModelInfo> {
        let models = self.scan_local_models().await.ok()?;

        if models.is_empty() {
            return None;
        }

        // Prefer models with recommended quantizations
        for quant in RECOMMENDED_QUANTIZATIONS {
            if let Some(model) = models.iter().find(|m| {
                m.quantization.as_ref().map(|q| q.contains(quant)).unwrap_or(false)
            }) {
                return Some(model.clone());
            }
        }

        // Fall back to first available model
        models.into_iter().next()
    }

    /// Download model file with resume support
    pub async fn download_file_resumable(
        &self,
        model_id: &str,
        filename: &str,
    ) -> Result<PathBuf, String> {
        let config = self.config.read().await;

        // Create model directory
        let model_dir = config.models_dir.join(model_id.replace('/', "__"));
        tokio::fs::create_dir_all(&model_dir).await
            .map_err(|e| format!("Failed to create model directory: {}", e))?;

        let file_path = model_dir.join(filename);
        let partial_path = model_dir.join(format!("{}.partial", filename));

        // Check if already completed
        if file_path.exists() {
            info!("Model file already exists: {:?}", file_path);
            return Ok(file_path);
        }

        let download_key = format!("{}:{}", model_id, filename);

        // Register for cancellation tracking
        self.download_cancellations.write().await.insert(download_key.clone(), false);

        // Check for partial download
        let existing_size = if partial_path.exists() {
            tokio::fs::metadata(&partial_path).await.ok().map(|m| m.len()).unwrap_or(0)
        } else {
            0
        };

        let url = format!(
            "https://huggingface.co/{}/resolve/main/{}",
            model_id, filename
        );

        info!("Downloading {} to {:?} (resume from {})", url, file_path, existing_size);

        // Initialize progress tracking
        let progress = DownloadProgress {
            model_id: model_id.to_string(),
            filename: filename.to_string(),
            downloaded: existing_size,
            total: 0,
            status: DownloadStatus::Pending,
        };
        self.downloads.write().await.push(progress);

        // Build request with Range header for resume
        let mut request = self.http_client.get(&url);

        if existing_size > 0 {
            request = request.header("Range", format!("bytes={}-", existing_size));
        }

        if let Some(ref token) = self.auth_state.read().await.token {
            request = request.bearer_auth(&token.access_token);
        }

        let response = request.send().await
            .map_err(|e| format!("Download failed: {}", e))?;

        let status = response.status();
        if !status.is_success() && status.as_u16() != 206 {
            self.update_download_status(model_id, filename, DownloadStatus::Failed).await;
            self.download_cancellations.write().await.remove(&download_key);
            return Err(format!("Download failed: {}", status));
        }

        // Get total size
        let content_length = response.content_length().unwrap_or(0);
        let total_size = if status.as_u16() == 206 {
            // Partial content - need to parse Content-Range header
            response.headers()
                .get("content-range")
                .and_then(|v| v.to_str().ok())
                .and_then(|s| s.split('/').last())
                .and_then(|s| s.parse::<u64>().ok())
                .unwrap_or(existing_size + content_length)
        } else {
            content_length
        };

        self.update_download_total(model_id, filename, total_size).await;
        self.update_download_status(model_id, filename, DownloadStatus::Downloading).await;

        // Open file for append
        let mut file = tokio::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&partial_path)
            .await
            .map_err(|e| format!("Failed to open file: {}", e))?;

        let mut downloaded = existing_size;
        let mut stream = response.bytes_stream();
        let mut last_progress_update = std::time::Instant::now();

        while let Some(chunk_result) = stream.next().await {
            // Check for cancellation
            if *self.download_cancellations.read().await.get(&download_key).unwrap_or(&false) {
                self.update_download_status(model_id, filename, DownloadStatus::Cancelled).await;
                self.download_cancellations.write().await.remove(&download_key);
                return Err("Download cancelled by user".to_string());
            }

            let chunk = chunk_result.map_err(|e| format!("Download error: {}", e))?;
            file.write_all(&chunk).await
                .map_err(|e| format!("Write error: {}", e))?;

            downloaded += chunk.len() as u64;

            // Throttle progress updates to avoid overwhelming the UI
            if last_progress_update.elapsed().as_millis() >= 100 {
                self.update_download_progress(model_id, filename, downloaded).await;
                last_progress_update = std::time::Instant::now();
            }
        }

        file.flush().await.map_err(|e| format!("Flush error: {}", e))?;
        drop(file);

        // Rename partial file to final file
        tokio::fs::rename(&partial_path, &file_path).await
            .map_err(|e| format!("Failed to finalize download: {}", e))?;

        self.update_download_status(model_id, filename, DownloadStatus::Completed).await;
        self.update_download_progress(model_id, filename, total_size).await;
        self.download_cancellations.write().await.remove(&download_key);

        info!("Download complete: {:?}", file_path);
        Ok(file_path)
    }

    /// Cancel an active download
    pub async fn cancel_download_resumable(&self, model_id: &str, filename: &str) {
        let key = format!("{}:{}", model_id, filename);
        self.download_cancellations.write().await.insert(key, true);
        self.update_download_status(model_id, filename, DownloadStatus::Cancelled).await;
    }

    /// Delete a local model file
    pub async fn delete_local_model(&self, path: &PathBuf) -> Result<(), String> {
        if !path.exists() {
            return Err("Model file not found".to_string());
        }

        tokio::fs::remove_file(path).await
            .map_err(|e| format!("Failed to delete model: {}", e))?;

        info!("Deleted model file: {:?}", path);
        Ok(())
    }

    /// Get recommended models for first-time users
    pub async fn get_recommended_models(&self) -> Vec<(&'static str, &'static str, &'static str)> {
        vec![
            ("TheBloke/TinyLlama-1.1B-Chat-v1.0-GGUF", "TinyLlama 1.1B", "Small, fast, good for testing"),
            ("TheBloke/Mistral-7B-Instruct-v0.2-GGUF", "Mistral 7B Instruct", "Excellent quality/speed balance"),
            ("TheBloke/Llama-2-7B-Chat-GGUF", "Llama 2 7B Chat", "Meta's Llama 2 chat model"),
            ("TheBloke/CodeLlama-7B-Instruct-GGUF", "CodeLlama 7B", "Optimized for code generation"),
            ("TheBloke/zephyr-7B-beta-GGUF", "Zephyr 7B", "Strong general purpose model"),
        ]
    }

    /// Get download statistics
    pub async fn get_download_stats(&self) -> (usize, usize, u64) {
        let downloads = self.downloads.read().await;
        let active = downloads.iter().filter(|d| d.status == DownloadStatus::Downloading).count();
        let completed = downloads.iter().filter(|d| d.status == DownloadStatus::Completed).count();
        let total_downloaded: u64 = downloads.iter().map(|d| d.downloaded).sum();
        (active, completed, total_downloaded)
    }
}

/// Extract quantization type from GGUF filename
fn extract_quantization(filename: &str) -> Option<String> {
    // Common patterns: model.Q4_K_M.gguf, model-q4_k_m.gguf, etc.
    let upper = filename.to_uppercase();

    let quant_patterns = [
        "Q2_K", "Q3_K_S", "Q3_K_M", "Q3_K_L",
        "Q4_0", "Q4_1", "Q4_K_S", "Q4_K_M",
        "Q5_0", "Q5_1", "Q5_K_S", "Q5_K_M",
        "Q6_K", "Q8_0", "F16", "F32",
        "IQ1_S", "IQ1_M", "IQ2_XXS", "IQ2_XS", "IQ2_S", "IQ2_M",
        "IQ3_XXS", "IQ3_XS", "IQ3_S", "IQ3_M", "IQ4_NL", "IQ4_XS",
    ];

    for pattern in quant_patterns {
        if upper.contains(pattern) {
            return Some(pattern.to_string());
        }
    }

    None
}

impl Default for HuggingFaceManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pkce_challenge_generation() {
        let challenge = PkceChallenge::generate();

        // Verify verifier length
        assert_eq!(challenge.code_verifier.len(), PKCE_VERIFIER_LENGTH);

        // Verify challenge is different from verifier
        assert_ne!(challenge.code_verifier, challenge.code_challenge);

        // Verify state is generated
        assert_eq!(challenge.state.len(), 32);

        // Verify not expired
        assert!(!challenge.is_expired());
    }

    #[test]
    fn test_pkce_challenge_expiry() {
        let mut challenge = PkceChallenge::generate();

        // Not expired initially
        assert!(!challenge.is_expired());

        // Simulate expired challenge
        challenge.created_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() - 700; // More than 10 minutes ago

        assert!(challenge.is_expired());
    }

    #[test]
    fn test_extract_quantization() {
        assert_eq!(extract_quantization("model.Q4_K_M.gguf"), Some("Q4_K_M".to_string()));
        assert_eq!(extract_quantization("model-q5_k_s.gguf"), Some("Q5_K_S".to_string()));
        assert_eq!(extract_quantization("llama-7b-Q8_0.gguf"), Some("Q8_0".to_string()));
        assert_eq!(extract_quantization("model.f16.gguf"), Some("F16".to_string()));
        assert_eq!(extract_quantization("model.gguf"), None);
    }

    #[test]
    fn test_quantization_recommendation() {
        for quant in RECOMMENDED_QUANTIZATIONS {
            assert!(["Q4_K_M", "Q4_K_S", "Q5_K_M", "Q5_K_S", "Q6_K", "Q8_0", "Q4_0", "Q5_0"].contains(quant));
        }
    }

    #[tokio::test]
    async fn test_huggingface_manager_creation() {
        let manager = HuggingFaceManager::new();
        let config = manager.get_config().await;

        assert!(config.client_id.is_none());
        assert!(config.prefer_gguf);
        assert_eq!(config.max_concurrent_downloads, 2);
    }

    #[tokio::test]
    async fn test_auth_state_default() {
        let manager = HuggingFaceManager::new();
        let state = manager.get_auth_state().await;

        assert!(!state.authenticated);
        assert!(state.user.is_none());
        assert!(state.token.is_none());
    }

    #[tokio::test]
    async fn test_download_progress_tracking() {
        let manager = HuggingFaceManager::new();

        // Initially empty
        let downloads = manager.get_downloads().await;
        assert!(downloads.is_empty());
    }

    #[tokio::test]
    async fn test_recommended_models() {
        let manager = HuggingFaceManager::new();
        let recommended = manager.get_recommended_models().await;

        assert!(!recommended.is_empty());
        assert!(recommended.iter().any(|(id, _, _)| id.contains("TinyLlama")));
    }

    #[test]
    fn test_local_model_info() {
        let info = LocalModelInfo {
            model_id: "test/model".to_string(),
            path: PathBuf::from("/tmp/test.gguf"),
            size: 1000000,
            quantization: Some("Q4_K_M".to_string()),
            loaded: false,
        };

        assert_eq!(info.model_id, "test/model");
        assert_eq!(info.quantization, Some("Q4_K_M".to_string()));
    }

    #[test]
    fn test_gguf_file_info() {
        let info = GGUFFileInfo {
            filename: "model.Q4_K_M.gguf".to_string(),
            size: 4_000_000_000,
            quantization: Some("Q4_K_M".to_string()),
            recommended: true,
            download_url: "https://example.com/model.gguf".to_string(),
        };

        assert!(info.recommended);
        assert_eq!(info.size, 4_000_000_000);
    }
}
