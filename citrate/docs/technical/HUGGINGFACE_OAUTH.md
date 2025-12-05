# HuggingFace OAuth Integration Technical Specification

**Version**: 1.0
**Date**: 2025-12-03
**Status**: Planning

---

## Overview

Citrate integrates with HuggingFace to provide:
1. Model discovery and search
2. One-click model downloads
3. Access to gated models (with user authentication)
4. Model metadata and compatibility information

---

## OAuth 2.0 Flow

### Architecture

```
┌─────────────────────────────────────────────────────────────────────────┐
│                         OAuth 2.0 PKCE Flow                              │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                          │
│  ┌──────────┐     1. Auth Request      ┌────────────────────┐           │
│  │ Citrate  │─────────────────────────▶│  HuggingFace       │           │
│  │   GUI    │     (with PKCE)          │  OAuth Server      │           │
│  └────┬─────┘                          └─────────┬──────────┘           │
│       │                                          │                       │
│       │  4. Exchange code                        │ 2. User Login         │
│       │     for token                            │    & Consent          │
│       │                                          ▼                       │
│       │                                ┌────────────────────┐           │
│       │                                │  User's Browser    │           │
│       │                                └─────────┬──────────┘           │
│       │                                          │                       │
│       │◀─────────────────────────────────────────┘                       │
│       │         3. Redirect with auth code                               │
│       │                                                                  │
│       ▼                                                                  │
│  ┌──────────┐     5. API Requests      ┌────────────────────┐           │
│  │ Citrate  │─────────────────────────▶│  HuggingFace       │           │
│  │ Backend  │◀─────────────────────────│  API               │           │
│  └──────────┘     6. Model Data        └────────────────────┘           │
│                                                                          │
└─────────────────────────────────────────────────────────────────────────┘
```

### PKCE (Proof Key for Code Exchange)

Required for public clients (desktop apps):

```rust
// src-tauri/src/huggingface/oauth.rs

use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use sha2::{Sha256, Digest};
use rand::Rng;

pub struct PKCEChallenge {
    pub verifier: String,
    pub challenge: String,
}

impl PKCEChallenge {
    pub fn generate() -> Self {
        // Generate 32 random bytes
        let mut rng = rand::thread_rng();
        let random_bytes: Vec<u8> = (0..32).map(|_| rng.gen()).collect();

        // Base64url encode as verifier
        let verifier = URL_SAFE_NO_PAD.encode(&random_bytes);

        // SHA256 hash and base64url encode as challenge
        let mut hasher = Sha256::new();
        hasher.update(verifier.as_bytes());
        let hash = hasher.finalize();
        let challenge = URL_SAFE_NO_PAD.encode(&hash);

        Self { verifier, challenge }
    }
}
```

---

## Configuration

### HuggingFace OAuth App Setup

1. Register OAuth application at: https://huggingface.co/settings/applications/new
2. Configure redirect URI: `citrate://oauth/callback` (custom scheme)
3. Request scopes: `read-repos`, `manage-repos` (for private models)

### Application Configuration

```toml
# citrate.toml
[huggingface]
client_id = "your-client-id"  # From HF OAuth app registration
redirect_uri = "citrate://oauth/callback"
scopes = ["read-repos"]  # Minimal required scope

# Optional: for gated models
request_gated_access = true
```

---

## Implementation

### Phase 1: OAuth Flow

```rust
// src-tauri/src/huggingface/oauth.rs

use tauri::AppHandle;
use tokio::sync::oneshot;

const HF_AUTH_URL: &str = "https://huggingface.co/oauth/authorize";
const HF_TOKEN_URL: &str = "https://huggingface.co/oauth/token";

pub struct HuggingFaceOAuth {
    client_id: String,
    redirect_uri: String,
    pkce: Option<PKCEChallenge>,
    access_token: Option<String>,
    refresh_token: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TokenResponse {
    access_token: String,
    token_type: String,
    expires_in: u64,
    refresh_token: Option<String>,
    scope: String,
}

impl HuggingFaceOAuth {
    pub fn new(client_id: String, redirect_uri: String) -> Self {
        Self {
            client_id,
            redirect_uri,
            pkce: None,
            access_token: None,
            refresh_token: None,
        }
    }

    /// Start OAuth flow - returns URL to open in browser
    pub fn start_auth_flow(&mut self) -> String {
        let pkce = PKCEChallenge::generate();

        let auth_url = format!(
            "{}?client_id={}&redirect_uri={}&response_type=code&scope={}&code_challenge={}&code_challenge_method=S256",
            HF_AUTH_URL,
            urlencoding::encode(&self.client_id),
            urlencoding::encode(&self.redirect_uri),
            urlencoding::encode("read-repos"),
            urlencoding::encode(&pkce.challenge),
        );

        self.pkce = Some(pkce);
        auth_url
    }

    /// Exchange authorization code for tokens
    pub async fn exchange_code(&mut self, code: &str) -> Result<TokenResponse, HFError> {
        let pkce = self.pkce.take().ok_or(HFError::NoPKCEChallenge)?;

        let client = reqwest::Client::new();
        let response = client
            .post(HF_TOKEN_URL)
            .form(&[
                ("client_id", &self.client_id),
                ("code", &code.to_string()),
                ("redirect_uri", &self.redirect_uri),
                ("grant_type", &"authorization_code".to_string()),
                ("code_verifier", &pkce.verifier),
            ])
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(HFError::TokenExchangeFailed(error_text));
        }

        let tokens: TokenResponse = response.json().await?;
        self.access_token = Some(tokens.access_token.clone());
        self.refresh_token = tokens.refresh_token.clone();

        Ok(tokens)
    }

    /// Check if we have a valid token
    pub fn is_authenticated(&self) -> bool {
        self.access_token.is_some()
    }

    /// Get the current access token
    pub fn get_token(&self) -> Option<&str> {
        self.access_token.as_deref()
    }
}
```

### Phase 2: Deep Link Handler

```rust
// src-tauri/src/main.rs

use tauri::Manager;

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_deep_link::init())
        .setup(|app| {
            #[cfg(desktop)]
            {
                let handle = app.handle().clone();
                app.listen("deep-link://new-url", move |event| {
                    let url = event.payload();
                    if url.starts_with("citrate://oauth/callback") {
                        handle_oauth_callback(&handle, url);
                    }
                });
            }
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

fn handle_oauth_callback(app: &AppHandle, url: &str) {
    // Parse the callback URL
    if let Some(code) = extract_auth_code(url) {
        // Emit event to frontend
        app.emit("oauth-callback", code).unwrap();
    }
}

fn extract_auth_code(url: &str) -> Option<String> {
    url::Url::parse(url)
        .ok()?
        .query_pairs()
        .find(|(key, _)| key == "code")
        .map(|(_, value)| value.into_owned())
}
```

### Phase 3: API Client

```rust
// src-tauri/src/huggingface/api.rs

const HF_API_BASE: &str = "https://huggingface.co/api";

pub struct HuggingFaceAPI {
    client: reqwest::Client,
    token: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ModelInfo {
    pub id: String,
    pub model_id: String,
    pub author: String,
    pub sha: String,
    pub last_modified: String,
    pub private: bool,
    pub gated: Option<String>,
    pub disabled: bool,
    pub downloads: u64,
    pub likes: u64,
    pub tags: Vec<String>,
    pub pipeline_tag: Option<String>,
    pub library_name: Option<String>,
    pub siblings: Vec<ModelFile>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ModelFile {
    pub rfilename: String,
    pub size: Option<u64>,
    pub blob_id: Option<String>,
    pub lfs: Option<LfsInfo>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LfsInfo {
    pub size: u64,
    pub sha256: String,
    pub pointer_size: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ModelSearchResult {
    pub models: Vec<ModelInfo>,
    pub num_items_per_page: u32,
}

impl HuggingFaceAPI {
    pub fn new(token: Option<String>) -> Self {
        Self {
            client: reqwest::Client::new(),
            token,
        }
    }

    /// Search for models
    pub async fn search_models(
        &self,
        query: &str,
        filters: &ModelSearchFilters,
    ) -> Result<Vec<ModelInfo>, HFError> {
        let mut url = format!("{}/models?search={}", HF_API_BASE, urlencoding::encode(query));

        // Add filters
        if let Some(ref library) = filters.library {
            url.push_str(&format!("&library={}", library));
        }
        if let Some(ref pipeline) = filters.pipeline_tag {
            url.push_str(&format!("&pipeline_tag={}", pipeline));
        }
        if filters.gguf_only {
            url.push_str("&filter=gguf");
        }
        if let Some(limit) = filters.limit {
            url.push_str(&format!("&limit={}", limit));
        }

        let mut request = self.client.get(&url);
        if let Some(ref token) = self.token {
            request = request.bearer_auth(token);
        }

        let response = request.send().await?;
        let models: Vec<ModelInfo> = response.json().await?;

        Ok(models)
    }

    /// Get detailed model info
    pub async fn get_model(&self, model_id: &str) -> Result<ModelInfo, HFError> {
        let url = format!("{}/models/{}", HF_API_BASE, model_id);

        let mut request = self.client.get(&url);
        if let Some(ref token) = self.token {
            request = request.bearer_auth(token);
        }

        let response = request.send().await?;
        let model: ModelInfo = response.json().await?;

        Ok(model)
    }

    /// Get download URL for a model file
    pub async fn get_file_url(
        &self,
        model_id: &str,
        filename: &str,
    ) -> Result<String, HFError> {
        // For public models, direct URL works
        let url = format!(
            "https://huggingface.co/{}/resolve/main/{}",
            model_id, filename
        );

        // For gated models, we need to include auth
        if let Some(ref token) = self.token {
            Ok(format!("{}?token={}", url, token))
        } else {
            Ok(url)
        }
    }

    /// List GGUF files for a model
    pub async fn list_gguf_files(&self, model_id: &str) -> Result<Vec<ModelFile>, HFError> {
        let model = self.get_model(model_id).await?;

        let gguf_files: Vec<ModelFile> = model
            .siblings
            .into_iter()
            .filter(|f| f.rfilename.ends_with(".gguf"))
            .collect();

        Ok(gguf_files)
    }
}

#[derive(Debug, Default)]
pub struct ModelSearchFilters {
    pub library: Option<String>,
    pub pipeline_tag: Option<String>,
    pub gguf_only: bool,
    pub limit: Option<u32>,
    pub min_downloads: Option<u64>,
}
```

---

## Frontend Integration

### React Components

```typescript
// src/components/models/HuggingFaceLogin.tsx

import { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { open } from '@tauri-apps/plugin-shell';

interface HFUser {
  username: string;
  avatarUrl: string;
  fullName: string;
}

export function HuggingFaceLogin() {
  const [isAuthenticated, setIsAuthenticated] = useState(false);
  const [user, setUser] = useState<HFUser | null>(null);
  const [loading, setLoading] = useState(false);

  useEffect(() => {
    // Check existing auth status
    checkAuthStatus();

    // Listen for OAuth callback
    const unlisten = listen<string>('oauth-callback', async (event) => {
      const code = event.payload;
      await handleOAuthCallback(code);
    });

    return () => {
      unlisten.then(fn => fn());
    };
  }, []);

  const checkAuthStatus = async () => {
    try {
      const authenticated = await invoke<boolean>('hf_check_auth');
      setIsAuthenticated(authenticated);
      if (authenticated) {
        const userInfo = await invoke<HFUser>('hf_get_user');
        setUser(userInfo);
      }
    } catch (err) {
      console.error('Failed to check auth status:', err);
    }
  };

  const startLogin = async () => {
    setLoading(true);
    try {
      const authUrl = await invoke<string>('hf_start_auth');
      await open(authUrl);
    } catch (err) {
      console.error('Failed to start auth:', err);
      setLoading(false);
    }
  };

  const handleOAuthCallback = async (code: string) => {
    try {
      await invoke('hf_exchange_code', { code });
      await checkAuthStatus();
    } catch (err) {
      console.error('Failed to exchange code:', err);
    } finally {
      setLoading(false);
    }
  };

  const logout = async () => {
    await invoke('hf_logout');
    setIsAuthenticated(false);
    setUser(null);
  };

  if (isAuthenticated && user) {
    return (
      <div className="hf-user-info">
        <img src={user.avatarUrl} alt={user.username} className="avatar" />
        <span>{user.fullName || user.username}</span>
        <button onClick={logout} className="logout-btn">
          Logout
        </button>
      </div>
    );
  }

  return (
    <button
      onClick={startLogin}
      disabled={loading}
      className="hf-login-btn"
    >
      {loading ? 'Connecting...' : 'Connect HuggingFace'}
    </button>
  );
}
```

```typescript
// src/components/models/ModelBrowser.tsx

import { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';

interface Model {
  id: string;
  modelId: string;
  author: string;
  downloads: number;
  likes: number;
  tags: string[];
  pipelineTag?: string;
  files: ModelFile[];
}

interface ModelFile {
  filename: string;
  size: number;
}

interface SearchFilters {
  query: string;
  ggufOnly: boolean;
  minSize?: number;
  maxSize?: number;
  quantization?: string;
}

export function ModelBrowser() {
  const [models, setModels] = useState<Model[]>([]);
  const [filters, setFilters] = useState<SearchFilters>({
    query: 'qwen coder',
    ggufOnly: true,
  });
  const [loading, setLoading] = useState(false);
  const [selectedModel, setSelectedModel] = useState<Model | null>(null);

  const searchModels = async () => {
    setLoading(true);
    try {
      const results = await invoke<Model[]>('hf_search_models', {
        query: filters.query,
        ggufOnly: filters.ggufOnly,
      });
      setModels(results);
    } catch (err) {
      console.error('Search failed:', err);
    } finally {
      setLoading(false);
    }
  };

  const downloadModel = async (model: Model, file: ModelFile) => {
    try {
      await invoke('hf_download_model', {
        modelId: model.id,
        filename: file.filename,
      });
    } catch (err) {
      console.error('Download failed:', err);
    }
  };

  return (
    <div className="model-browser">
      <div className="search-bar">
        <input
          type="text"
          value={filters.query}
          onChange={(e) => setFilters({ ...filters, query: e.target.value })}
          placeholder="Search models..."
        />
        <label>
          <input
            type="checkbox"
            checked={filters.ggufOnly}
            onChange={(e) => setFilters({ ...filters, ggufOnly: e.target.checked })}
          />
          GGUF only
        </label>
        <button onClick={searchModels} disabled={loading}>
          {loading ? 'Searching...' : 'Search'}
        </button>
      </div>

      <div className="model-list">
        {models.map((model) => (
          <div
            key={model.id}
            className="model-card"
            onClick={() => setSelectedModel(model)}
          >
            <h3>{model.modelId}</h3>
            <p>by {model.author}</p>
            <div className="stats">
              <span>Downloads: {model.downloads.toLocaleString()}</span>
              <span>Likes: {model.likes}</span>
            </div>
            <div className="tags">
              {model.tags.slice(0, 5).map((tag) => (
                <span key={tag} className="tag">{tag}</span>
              ))}
            </div>
          </div>
        ))}
      </div>

      {selectedModel && (
        <ModelDetails
          model={selectedModel}
          onDownload={downloadModel}
          onClose={() => setSelectedModel(null)}
        />
      )}
    </div>
  );
}

function ModelDetails({
  model,
  onDownload,
  onClose
}: {
  model: Model;
  onDownload: (model: Model, file: ModelFile) => void;
  onClose: () => void;
}) {
  return (
    <div className="model-details-modal">
      <button className="close-btn" onClick={onClose}>×</button>
      <h2>{model.modelId}</h2>
      <p>Author: {model.author}</p>

      <h3>Available Files (GGUF)</h3>
      <ul className="file-list">
        {model.files
          .filter((f) => f.filename.endsWith('.gguf'))
          .map((file) => (
            <li key={file.filename}>
              <span>{file.filename}</span>
              <span>{formatSize(file.size)}</span>
              <button onClick={() => onDownload(model, file)}>
                Download
              </button>
            </li>
          ))}
      </ul>
    </div>
  );
}

function formatSize(bytes: number): string {
  const gb = bytes / (1024 * 1024 * 1024);
  if (gb >= 1) return `${gb.toFixed(2)} GB`;
  const mb = bytes / (1024 * 1024);
  return `${mb.toFixed(2)} MB`;
}
```

---

## Tauri Commands

```rust
// src-tauri/src/huggingface/commands.rs

use tauri::State;
use std::sync::Mutex;

pub struct HuggingFaceState {
    oauth: Mutex<HuggingFaceOAuth>,
    api: Mutex<Option<HuggingFaceAPI>>,
}

#[tauri::command]
pub async fn hf_check_auth(state: State<'_, HuggingFaceState>) -> Result<bool, String> {
    let oauth = state.oauth.lock().map_err(|e| e.to_string())?;
    Ok(oauth.is_authenticated())
}

#[tauri::command]
pub async fn hf_start_auth(state: State<'_, HuggingFaceState>) -> Result<String, String> {
    let mut oauth = state.oauth.lock().map_err(|e| e.to_string())?;
    Ok(oauth.start_auth_flow())
}

#[tauri::command]
pub async fn hf_exchange_code(
    code: String,
    state: State<'_, HuggingFaceState>,
) -> Result<(), String> {
    let mut oauth = state.oauth.lock().map_err(|e| e.to_string())?;
    let tokens = oauth.exchange_code(&code).await.map_err(|e| e.to_string())?;

    // Store token securely
    store_token_securely(&tokens.access_token)?;

    // Initialize API with token
    let mut api = state.api.lock().map_err(|e| e.to_string())?;
    *api = Some(HuggingFaceAPI::new(Some(tokens.access_token)));

    Ok(())
}

#[tauri::command]
pub async fn hf_search_models(
    query: String,
    gguf_only: bool,
    state: State<'_, HuggingFaceState>,
) -> Result<Vec<ModelInfo>, String> {
    let api = state.api.lock().map_err(|e| e.to_string())?;
    let api = api.as_ref().ok_or("Not authenticated")?;

    let filters = ModelSearchFilters {
        gguf_only,
        limit: Some(20),
        ..Default::default()
    };

    api.search_models(&query, &filters)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn hf_download_model(
    model_id: String,
    filename: String,
    state: State<'_, HuggingFaceState>,
) -> Result<String, String> {
    let api = state.api.lock().map_err(|e| e.to_string())?;
    let api = api.as_ref().ok_or("Not authenticated")?;

    // Get download URL
    let url = api.get_file_url(&model_id, &filename)
        .await
        .map_err(|e| e.to_string())?;

    // Hand off to IPFS downloader
    // This will download and pin to local IPFS
    let cid = download_and_pin_to_ipfs(&url, &filename).await?;

    Ok(cid)
}

#[tauri::command]
pub async fn hf_logout(state: State<'_, HuggingFaceState>) -> Result<(), String> {
    let mut oauth = state.oauth.lock().map_err(|e| e.to_string())?;
    *oauth = HuggingFaceOAuth::new(
        std::env::var("HF_CLIENT_ID").unwrap_or_default(),
        "citrate://oauth/callback".to_string(),
    );

    let mut api = state.api.lock().map_err(|e| e.to_string())?;
    *api = None;

    // Clear stored token
    clear_stored_token()?;

    Ok(())
}
```

---

## Token Storage

### Secure Token Persistence

```rust
// src-tauri/src/huggingface/token_store.rs

use keyring::Entry;

const SERVICE_NAME: &str = "citrate";
const HF_TOKEN_KEY: &str = "huggingface_token";

pub fn store_token_securely(token: &str) -> Result<(), String> {
    let entry = Entry::new(SERVICE_NAME, HF_TOKEN_KEY)
        .map_err(|e| format!("Keyring error: {}", e))?;

    entry.set_password(token)
        .map_err(|e| format!("Failed to store token: {}", e))
}

pub fn get_stored_token() -> Result<Option<String>, String> {
    let entry = Entry::new(SERVICE_NAME, HF_TOKEN_KEY)
        .map_err(|e| format!("Keyring error: {}", e))?;

    match entry.get_password() {
        Ok(token) => Ok(Some(token)),
        Err(keyring::Error::NoEntry) => Ok(None),
        Err(e) => Err(format!("Failed to get token: {}", e)),
    }
}

pub fn clear_stored_token() -> Result<(), String> {
    let entry = Entry::new(SERVICE_NAME, HF_TOKEN_KEY)
        .map_err(|e| format!("Keyring error: {}", e))?;

    match entry.delete_credential() {
        Ok(()) => Ok(()),
        Err(keyring::Error::NoEntry) => Ok(()), // Already cleared
        Err(e) => Err(format!("Failed to clear token: {}", e)),
    }
}
```

---

## Recommended Models

### Qwen 2.5 Coder Models

| Model | Size | Quantization | Download Size | RAM Required |
|-------|------|--------------|---------------|--------------|
| Qwen2.5-Coder-7B-Instruct | 7B | Q4_K_M | ~5 GB | ~7 GB |
| Qwen2.5-Coder-7B-Instruct | 7B | Q5_K_M | ~5.5 GB | ~8 GB |
| Qwen2.5-Coder-14B-Instruct | 14B | Q4_K_M | ~9 GB | ~12 GB |
| Qwen2.5-Coder-14B-Instruct | 14B | Q5_K_M | ~10 GB | ~14 GB |

### Search Queries

```typescript
// Recommended search queries for coding models
const CODING_MODEL_QUERIES = [
  'qwen2.5 coder gguf',
  'deepseek coder gguf',
  'codellama gguf',
  'starcoder gguf',
  'wizardcoder gguf',
];
```

---

## Security Considerations

1. **PKCE Required**: Always use PKCE for OAuth flow (public client)
2. **Secure Token Storage**: Use system keychain (not localStorage)
3. **Token Expiration**: Handle token refresh before expiration
4. **Scope Minimization**: Request only necessary scopes
5. **Redirect Validation**: Validate redirect URI matches exactly
6. **State Parameter**: Include state parameter to prevent CSRF

---

## Error Handling

```rust
#[derive(Debug, thiserror::Error)]
pub enum HFError {
    #[error("OAuth flow not started")]
    NoPKCEChallenge,

    #[error("Token exchange failed: {0}")]
    TokenExchangeFailed(String),

    #[error("API request failed: {0}")]
    ApiError(String),

    #[error("Model not found: {0}")]
    ModelNotFound(String),

    #[error("Download failed: {0}")]
    DownloadFailed(String),

    #[error("Authentication required")]
    NotAuthenticated,

    #[error("Gated model - access not granted")]
    GatedModelAccessDenied,

    #[error("Network error: {0}")]
    NetworkError(#[from] reqwest::Error),
}
```

---

## Testing

```bash
# Test HuggingFace integration
cargo test -p citrate-core --test huggingface_integration

# Test OAuth flow (requires manual interaction)
cargo test -p citrate-core --test oauth_flow -- --ignored

# Test API without auth (public models only)
cargo test -p citrate-core --test hf_api_public
```

---

*Document maintained by Citrate development team*
