//! HuggingFace Integration Module
//!
//! Provides OAuth authentication and model browsing/download capabilities
//! for HuggingFace Hub integration with Citrate.

use futures::StreamExt;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::io::AsyncWriteExt;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

/// HuggingFace API base URL
const HF_API_BASE: &str = "https://huggingface.co/api";
const HF_AUTH_URL: &str = "https://huggingface.co/oauth/authorize";
const HF_TOKEN_URL: &str = "https://huggingface.co/oauth/token";

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
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .unwrap_or_else(|_| Client::new());

        Self {
            config: Arc::new(RwLock::new(config)),
            auth_state: Arc::new(RwLock::new(AuthState::default())),
            http_client,
            downloads: Arc::new(RwLock::new(Vec::new())),
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

    /// Generate OAuth authorization URL
    pub async fn get_auth_url(&self, state: &str) -> Result<String, String> {
        let config = self.config.read().await;

        let client_id = config.client_id.as_ref()
            .ok_or_else(|| "HuggingFace client_id not configured".to_string())?;

        let url = format!(
            "{}?client_id={}&redirect_uri={}&response_type=code&scope=read&state={}",
            HF_AUTH_URL,
            urlencoding::encode(client_id),
            urlencoding::encode(&config.redirect_uri),
            urlencoding::encode(state)
        );

        Ok(url)
    }

    /// Exchange authorization code for token
    pub async fn exchange_code(&self, code: &str) -> Result<OAuthToken, String> {
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

        // Store token and fetch user info
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
}

impl Default for HuggingFaceManager {
    fn default() -> Self {
        Self::new()
    }
}
