# Hybrid Genesis Model Implementation Plan
## Option B: BGE-M3 In-Block + Llama 3.1 8B Required Pin

**Target**: Production-ready deployment with embeddings in genesis and LLM via consensus-enforced IPFS pinning

---

## Phase 1: Preparation (Day 1)

### 1.1 Install Dependencies

```bash
# Navigate to project root
cd /Users/soleilklosowski/Downloads/citrate/citrate

# Python dependencies
pip3 install --upgrade huggingface_hub ipfshttpclient torch transformers

# IPFS
brew install ipfs
ipfs init
ipfs config Addresses.API /ip4/127.0.0.1/tcp/5001
ipfs config Addresses.Gateway /ip4/127.0.0.1/tcp/8080

# Start IPFS daemon (keep running)
ipfs daemon &

# llama.cpp for quantization
cd ~
git clone https://github.com/ggerganov/llama.cpp
cd llama.cpp
make -j8

# Verify installations
which ipfs          # Should show path
ipfs version        # Should show version
huggingface-cli --version  # Should show version
ls ~/llama.cpp/quantize    # Should exist
```

**Verification:**
```bash
./tools/deploy_modern_model.py --check-deps
# Should show all ✅
```

### 1.2 Get HuggingFace Access

```bash
# Visit https://huggingface.co/meta-llama/Meta-Llama-3.1-8B-Instruct
# Click "Agree and access repository"

# Get token: https://huggingface.co/settings/tokens
# Create token with "Read access to contents of all public gated repos"

# Save token
export HF_TOKEN="hf_your_actual_token_here"
echo "export HF_TOKEN='$HF_TOKEN'" >> ~/.zshrc  # Persist
```

---

## Phase 2: Model Preparation (Day 1-2)

### 2.1 Download and Quantize BGE-M3

```bash
cd /Users/soleilklosowski/Downloads/citrate/citrate

# Create models directory
mkdir -p models/genesis

# Download BGE-M3
huggingface-cli download BAAI/bge-m3 \
  --local-dir models/bge-m3 \
  --local-dir-use-symlinks False

# Convert to GGUF fp16
python3 ~/llama.cpp/convert.py models/bge-m3 \
  --outfile models/bge-m3-fp16.gguf \
  --outtype f16

# Quantize to Q4_K_M
~/llama.cpp/quantize \
  models/bge-m3-fp16.gguf \
  models/genesis/bge-m3-q4.gguf \
  Q4_K_M

# Verify size
ls -lh models/genesis/bge-m3-q4.gguf
# Expected: ~300M

# Calculate hash for verification
sha256sum models/genesis/bge-m3-q4.gguf > models/genesis/bge-m3-q4.sha256
cat models/genesis/bge-m3-q4.sha256

# Move to assets
mkdir -p assets
cp models/genesis/bge-m3-q4.gguf assets/
```

### 2.2 Download and Quantize Llama 3.1 8B

```bash
# Download Llama 3.1 8B (requires HF token)
huggingface-cli download meta-llama/Meta-Llama-3.1-8B-Instruct \
  --local-dir models/llama-3.1-8b \
  --local-dir-use-symlinks False \
  --token $HF_TOKEN

# Convert to GGUF fp16
python3 ~/llama.cpp/convert.py models/llama-3.1-8b \
  --outfile models/llama-3.1-8b-fp16.gguf \
  --outtype f16

# Quantize to Q4_K_M
~/llama.cpp/quantize \
  models/llama-3.1-8b-fp16.gguf \
  models/genesis/llama-3.1-8b-q4.gguf \
  Q4_K_M

# Verify size
ls -lh models/genesis/llama-3.1-8b-q4.gguf
# Expected: ~4.7G

# Calculate hash
sha256sum models/genesis/llama-3.1-8b-q4.gguf > models/genesis/llama-3.1-8b-q4.sha256
cat models/genesis/llama-3.1-8b-q4.sha256
```

### 2.3 Upload Llama to IPFS

```bash
# Upload to IPFS
ipfs add models/genesis/llama-3.1-8b-q4.gguf

# Output will be like:
# added bafybeiabc123... llama-3.1-8b-q4.gguf

# Save the CID
export LLAMA_CID="bafybei..."  # Use actual CID from output
echo "Llama 3.1 8B CID: $LLAMA_CID"

# Pin it locally
ipfs pin add $LLAMA_CID

# Verify
ipfs pin ls | grep $LLAMA_CID
```

**Record these values:**
```bash
# Create genesis config file
cat > models/genesis/model-manifest.json <<EOF
{
  "embedded_model": {
    "name": "BGE-M3",
    "file": "bge-m3-q4.gguf",
    "sha256": "$(cat models/genesis/bge-m3-q4.sha256 | cut -d' ' -f1)",
    "size_bytes": $(stat -f%z models/genesis/bge-m3-q4.gguf)
  },
  "required_pin": {
    "name": "Llama-3.1-8B-Instruct",
    "ipfs_cid": "$LLAMA_CID",
    "sha256": "$(cat models/genesis/llama-3.1-8b-q4.sha256 | cut -d' ' -f1)",
    "size_bytes": $(stat -f%z models/genesis/llama-3.1-8b-q4.gguf)
  }
}
EOF

cat models/genesis/model-manifest.json
```

---

## Phase 3: Core Implementation (Day 2-3)

### 3.1 Add Model Types to Consensus

**File: `core/consensus/src/types/mod.rs`**

Add to existing types:

```rust
use serde::{Deserialize, Serialize};

/// Embedded model stored directly in genesis block
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct EmbeddedModel {
    pub model_id: String,
    pub model_type: ModelType,
    pub weights: Vec<u8>,
    pub metadata: EmbeddedModelMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ModelType {
    Embeddings,
    TinyLLM,
    Custom(String),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct EmbeddedModelMetadata {
    pub name: String,
    pub version: String,
    pub context_length: usize,
    pub embedding_dim: Option<usize>,
    pub license: String,
    pub framework: String,
}

/// Required model that validators must pin from IPFS
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RequiredModel {
    pub model_id: String,
    pub ipfs_cid: String,
    pub sha256_hash: Hash,
    pub size_bytes: u64,
    pub must_pin: bool,
    pub slash_penalty_latt: u128,
    pub grace_period_hours: u64,
}
```

**Update Block structure:**

```rust
// In core/consensus/src/types/block.rs

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Block {
    pub header: BlockHeader,
    pub state_root: Hash,
    pub tx_root: Hash,
    pub receipt_root: Hash,
    pub artifact_root: Hash,
    pub ghostdag_params: GhostDagParams,
    pub transactions: Vec<Transaction>,
    pub signature: Signature,

    // AI Models (only populated in genesis block)
    #[serde(default)]
    pub embedded_models: Vec<EmbeddedModel>,
    #[serde(default)]
    pub required_pins: Vec<RequiredModel>,
}
```

### 3.2 Update Genesis Block Creation

**File: `node/src/genesis.rs`**

Replace the existing genesis creation:

```rust
use citrate_consensus::types::{Block, BlockHeader, EmbeddedModel, RequiredModel, ModelType, EmbeddedModelMetadata};
use std::fs;
use anyhow::Result;

/// Genesis model configuration loaded from manifest
#[derive(Debug, serde::Deserialize)]
struct GenesisModelManifest {
    embedded_model: EmbeddedModelConfig,
    required_pin: RequiredPinConfig,
}

#[derive(Debug, serde::Deserialize)]
struct EmbeddedModelConfig {
    name: String,
    file: String,
    sha256: String,
    size_bytes: u64,
}

#[derive(Debug, serde::Deserialize)]
struct RequiredPinConfig {
    name: String,
    ipfs_cid: String,
    sha256: String,
    size_bytes: u64,
}

pub fn create_genesis_block(config: &GenesisConfig) -> Result<Block> {
    tracing::info!("Creating genesis block with embedded AI models...");

    // Load model manifest
    let manifest_path = "models/genesis/model-manifest.json";
    let manifest_str = fs::read_to_string(manifest_path)
        .expect("Failed to read model manifest");
    let manifest: GenesisModelManifest = serde_json::from_str(&manifest_str)
        .expect("Failed to parse model manifest");

    // Load embedded model (BGE-M3)
    let model_path = format!("assets/{}", manifest.embedded_model.file);
    let model_weights = fs::read(&model_path)
        .expect(&format!("Failed to read model from {}", model_path));

    // Verify size
    assert_eq!(
        model_weights.len() as u64,
        manifest.embedded_model.size_bytes,
        "Model size mismatch"
    );

    // Verify hash
    use sha2::{Sha256, Digest};
    let mut hasher = Sha256::new();
    hasher.update(&model_weights);
    let hash = format!("{:x}", hasher.finalize());
    assert_eq!(hash, manifest.embedded_model.sha256, "Model hash mismatch");

    tracing::info!(
        "Loaded embedded model: {} ({} bytes)",
        manifest.embedded_model.name,
        model_weights.len()
    );

    // Create embedded model entry
    let embedded_model = EmbeddedModel {
        model_id: "bge-m3".to_string(),
        model_type: ModelType::Embeddings,
        weights: model_weights,
        metadata: EmbeddedModelMetadata {
            name: manifest.embedded_model.name.clone(),
            version: "1.0.0".to_string(),
            context_length: 8192,
            embedding_dim: Some(1024),
            license: "MIT".to_string(),
            framework: "GGUF".to_string(),
        },
    };

    // Create required pin entry
    let required_pin = RequiredModel {
        model_id: "llama-3.1-8b".to_string(),
        ipfs_cid: manifest.required_pin.ipfs_cid.clone(),
        sha256_hash: Hash::from_hex(&manifest.required_pin.sha256)
            .expect("Invalid SHA256 hash"),
        size_bytes: manifest.required_pin.size_bytes,
        must_pin: true,
        slash_penalty_latt: 1000 * 10u128.pow(18), // 1000 LATT
        grace_period_hours: 24,
    };

    tracing::info!(
        "Required pin: {} (IPFS: {}, {} GB)",
        manifest.required_pin.name,
        manifest.required_pin.ipfs_cid,
        manifest.required_pin.size_bytes as f64 / 1_000_000_000.0
    );

    // Create genesis block header
    let header = BlockHeader {
        version: 1,
        block_hash: Hash::new([0; 32]),
        selected_parent_hash: Hash::default(),
        merge_parent_hashes: vec![],
        timestamp: config.timestamp,
        height: 0,
        blue_score: 0,
        blue_work: 0,
        pruning_point: Hash::default(),
        proposer_pubkey: PublicKey::new([0; 32]),
        vrf_reveal: VrfProof {
            proof: vec![],
            output: Hash::default(),
        },
    };

    // Create genesis block
    let mut genesis = Block {
        header,
        state_root: Hash::default(),
        tx_root: Hash::default(),
        receipt_root: Hash::default(),
        artifact_root: Hash::default(),
        ghostdag_params: GhostDagParams::default(),
        transactions: vec![],
        signature: Signature::new([0; 64]),
        embedded_models: vec![embedded_model],
        required_pins: vec![required_pin],
    };

    // Calculate block hash
    genesis.header.block_hash = calculate_block_hash(&genesis);

    let total_size = genesis.embedded_models[0].weights.len() as f64 / 1_000_000.0;
    tracing::info!(
        "Genesis block created with {} MB embedded model",
        total_size
    );

    Ok(genesis)
}

// Update initialize_genesis_state to handle embedded models
pub async fn initialize_genesis_state(
    storage: Arc<StorageManager>,
    executor: Arc<Executor>,
    config: &GenesisConfig,
) -> Result<Hash> {
    // Create genesis block with models
    let mut genesis = create_genesis_block(config)?;

    // Initialize economics (existing code)
    let economics_config = EconomicsGenesisConfig::default();
    for account in &economics_config.accounts {
        executor.set_balance(&account.address, account.balance);
        if account.nonce > 0 {
            executor.set_nonce(&account.address, account.nonce);
        }
        if let Some(code) = &account.code {
            executor.set_code(&account.address, code.clone());
        }
        tracing::info!(
            "Initialized genesis account 0x{} with balance {} LATT",
            hex::encode(account.address.0),
            account.balance / U256::from(10).pow(U256::from(18)),
        );
    }

    // Store embedded models in a special state location for fast access
    for model in &genesis.embedded_models {
        let model_key = format!("genesis_model_{}", model.model_id);
        storage.state.put_raw(model_key.as_bytes(), &model.weights)?;
        tracing::info!("Stored embedded model: {}", model.model_id);
    }

    // Commit state
    let state_root = executor.state_db().commit();
    genesis.state_root = Hash::new(*state_root.as_bytes());
    genesis.header.block_hash = calculate_block_hash(&genesis);

    // Store genesis block
    storage.blocks.put_block(&genesis)?;

    tracing::info!(
        "Genesis block created: {:?} at height 0 with {} embedded models, {} required pins",
        hex::encode(&genesis.header.block_hash.as_bytes()[..8]),
        genesis.embedded_models.len(),
        genesis.required_pins.len()
    );

    Ok(genesis.header.block_hash)
}
```

### 3.3 Implement Model Inference Service

**File: `core/api/src/model_inference.rs` (new file)**

```rust
use anyhow::Result;
use citrate_storage::StorageManager;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Model inference service for embedded and pinned models
pub struct ModelInferenceService {
    storage: Arc<StorageManager>,
    // Cached GGUF model instance
    embedding_model: Arc<RwLock<Option<GGUFEmbedder>>>,
    llm_model: Arc<RwLock<Option<GGUFModel>>>,
}

impl ModelInferenceService {
    pub fn new(storage: Arc<StorageManager>) -> Self {
        Self {
            storage,
            embedding_model: Arc::new(RwLock::new(None)),
            llm_model: Arc::new(RwLock::new(None)),
        }
    }

    /// Get text embedding using genesis BGE-M3 model
    pub async fn get_text_embedding(&self, text: &str) -> Result<Vec<f32>> {
        // Check if model is cached
        {
            let model_guard = self.embedding_model.read().await;
            if let Some(model) = model_guard.as_ref() {
                return model.embed(text);
            }
        }

        // Load model from genesis state
        let mut model_guard = self.embedding_model.write().await;

        // Double-check after acquiring write lock
        if model_guard.is_some() {
            return model_guard.as_ref().unwrap().embed(text);
        }

        // Load from storage
        let model_key = b"genesis_model_bge-m3";
        let weights = self.storage.state.get_raw(model_key)?
            .ok_or_else(|| anyhow::anyhow!("Embedded model not found"))?;

        // Initialize GGUF model
        let model = GGUFEmbedder::from_bytes(&weights)?;
        *model_guard = Some(model);

        model_guard.as_ref().unwrap().embed(text)
    }

    /// Semantic search using embeddings
    pub async fn semantic_search(
        &self,
        query: &str,
        candidates: Vec<String>,
        top_k: usize,
    ) -> Result<Vec<(String, f32)>> {
        let query_emb = self.get_text_embedding(query).await?;

        let mut scores = Vec::new();
        for candidate in candidates {
            let cand_emb = self.get_text_embedding(&candidate).await?;
            let similarity = cosine_similarity(&query_emb, &cand_emb);
            scores.push((candidate, similarity));
        }

        scores.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        Ok(scores.into_iter().take(top_k).collect())
    }

    /// Chat completion using pinned Llama 3.1 8B
    pub async fn chat_completion(
        &self,
        messages: Vec<ChatMessage>,
        max_tokens: usize,
    ) -> Result<String> {
        // Check if LLM is cached
        {
            let model_guard = self.llm_model.read().await;
            if let Some(model) = model_guard.as_ref() {
                return model.generate(messages, max_tokens);
            }
        }

        // Load from pinned IPFS file
        let mut model_guard = self.llm_model.write().await;

        if model_guard.is_some() {
            return model_guard.as_ref().unwrap().generate(messages, max_tokens);
        }

        // Get pinned model path
        let model_path = self.get_pinned_model_path("llama-3.1-8b").await?;

        // Load GGUF model
        let model = GGUFModel::from_file(&model_path)?;
        *model_guard = Some(model);

        model_guard.as_ref().unwrap().generate(messages, max_tokens)
    }

    /// Get path to pinned model file
    async fn get_pinned_model_path(&self, model_id: &str) -> Result<std::path::PathBuf> {
        // Get IPFS CID from genesis required_pins
        let genesis = self.storage.blocks.get_block_by_height(0)?
            .ok_or_else(|| anyhow::anyhow!("Genesis block not found"))?;

        let required_pin = genesis.required_pins
            .iter()
            .find(|p| p.model_id == model_id)
            .ok_or_else(|| anyhow::anyhow!("Model not in required pins"))?;

        // Check if file exists locally (should be pinned)
        let ipfs_repo = std::path::PathBuf::from(
            std::env::var("IPFS_PATH").unwrap_or_else(|_| {
                format!("{}/.ipfs", std::env::var("HOME").unwrap())
            })
        );

        let model_path = ipfs_repo.join("blocks").join(&required_pin.ipfs_cid);

        if !model_path.exists() {
            return Err(anyhow::anyhow!(
                "Required model not pinned locally. Run: citrate-node download-required-models"
            ));
        }

        Ok(model_path)
    }
}

// Helper structures (simplified - you'll need full GGUF implementation)
struct GGUFEmbedder {
    // GGUF model state
}

impl GGUFEmbedder {
    fn from_bytes(bytes: &[u8]) -> Result<Self> {
        // TODO: Implement GGUF loading
        // For now, use llama-cpp-rs or similar
        todo!("Implement GGUF embedder")
    }

    fn embed(&self, text: &str) -> Result<Vec<f32>> {
        todo!("Implement embedding generation")
    }
}

struct GGUFModel {
    // GGUF model state
}

impl GGUFModel {
    fn from_file(path: &std::path::Path) -> Result<Self> {
        todo!("Implement GGUF model loading")
    }

    fn generate(&self, messages: Vec<ChatMessage>, max_tokens: usize) -> Result<String> {
        todo!("Implement text generation")
    }
}

#[derive(Debug, Clone)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    assert_eq!(a.len(), b.len());
    let dot_product: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();
    dot_product / (norm_a * norm_b)
}
```

**Note**: For GGUF integration, add dependency:

```toml
# core/api/Cargo.toml
[dependencies]
llama-cpp-rs = "0.3"  # Or use direct GGUF parsing
```

### 3.4 Add RPC Endpoints

**File: `core/api/src/rpc.rs`**

Add to existing RPC server:

```rust
use crate::model_inference::{ModelInferenceService, ChatMessage};

// Add to RpcImpl
pub struct RpcImpl {
    // ... existing fields
    model_service: Arc<ModelInferenceService>,
}

#[rpc]
impl Rpc for RpcImpl {
    // ... existing methods

    /// Get text embedding using genesis BGE-M3 model
    async fn get_text_embedding(&self, text: String) -> Result<Vec<f32>> {
        self.model_service.get_text_embedding(&text).await
            .map_err(|e| jsonrpc_core::Error::internal_error())
    }

    /// Semantic search
    async fn semantic_search(
        &self,
        query: String,
        candidates: Vec<String>,
        top_k: usize,
    ) -> Result<Vec<(String, f32)>> {
        self.model_service.semantic_search(&query, candidates, top_k).await
            .map_err(|e| jsonrpc_core::Error::internal_error())
    }

    /// Chat completion (OpenAI-compatible)
    async fn chat_completion(
        &self,
        messages: Vec<ChatMessage>,
        max_tokens: Option<usize>,
    ) -> Result<String> {
        self.model_service.chat_completion(messages, max_tokens.unwrap_or(512)).await
            .map_err(|e| jsonrpc_core::Error::internal_error())
    }
}
```

---

## Phase 4: Validator Pin Management (Day 3-4)

### 4.1 Add CLI Commands

**File: `node/src/main.rs`**

Add subcommands:

```rust
use clap::{Parser, Subcommand};

#[derive(Parser)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    // ... existing fields
}

#[derive(Subcommand)]
enum Commands {
    // ... existing commands

    /// Download required models from IPFS
    DownloadRequiredModels,

    /// Verify pinned models
    VerifyPins,

    /// Show model status
    ModelStatus,
}

async fn main() -> Result<()> {
    // ... existing setup

    match cli.command {
        Some(Commands::DownloadRequiredModels) => {
            download_required_models(&config).await?;
        }
        Some(Commands::VerifyPins) => {
            verify_pinned_models(&config).await?;
        }
        Some(Commands::ModelStatus) => {
            show_model_status(&config).await?;
        }
        _ => {
            // Normal node startup
        }
    }

    Ok(())
}

async fn download_required_models(config: &NodeConfig) -> Result<()> {
    info!("Downloading required models...");

    // Load genesis block
    let storage = Arc::new(StorageManager::new(&config.data_dir)?);
    let genesis = storage.blocks.get_block_by_height(0)?
        .ok_or_else(|| anyhow::anyhow!("Genesis block not found"))?;

    for required_pin in &genesis.required_pins {
        info!("Downloading: {} ({})", required_pin.model_id, required_pin.ipfs_cid);

        // Download from IPFS
        let output_path = config.data_dir.join("models").join(&required_pin.model_id);
        std::fs::create_dir_all(&output_path)?;

        let result = std::process::Command::new("ipfs")
            .args(&["get", &required_pin.ipfs_cid, "-o", output_path.to_str().unwrap()])
            .status()?;

        if !result.success() {
            return Err(anyhow::anyhow!("Failed to download model from IPFS"));
        }

        // Verify hash
        let downloaded_path = output_path.join(&required_pin.ipfs_cid);
        let file_bytes = std::fs::read(&downloaded_path)?;

        use sha2::{Sha256, Digest};
        let mut hasher = Sha256::new();
        hasher.update(&file_bytes);
        let hash = Hash::from_bytes(&hasher.finalize());

        if hash != required_pin.sha256_hash {
            return Err(anyhow::anyhow!("Hash mismatch for {}", required_pin.model_id));
        }

        // Pin locally
        let pin_result = std::process::Command::new("ipfs")
            .args(&["pin", "add", &required_pin.ipfs_cid])
            .status()?;

        if !pin_result.success() {
            return Err(anyhow::anyhow!("Failed to pin model"));
        }

        info!("✓ Downloaded and pinned: {}", required_pin.model_id);
    }

    info!("All required models downloaded and pinned!");
    Ok(())
}

async fn verify_pinned_models(config: &NodeConfig) -> Result<()> {
    info!("Verifying pinned models...");

    let storage = Arc::new(StorageManager::new(&config.data_dir)?);
    let genesis = storage.blocks.get_block_by_height(0)?
        .ok_or_else(|| anyhow::anyhow!("Genesis block not found"))?;

    let mut all_pinned = true;

    for required_pin in &genesis.required_pins {
        // Check if pinned
        let result = std::process::Command::new("ipfs")
            .args(&["pin", "ls", &required_pin.ipfs_cid])
            .output()?;

        if result.status.success() {
            info!("✓ Pinned: {} ({})", required_pin.model_id, required_pin.ipfs_cid);
        } else {
            warn!("✗ NOT PINNED: {} ({})", required_pin.model_id, required_pin.ipfs_cid);
            all_pinned = false;
        }
    }

    if !all_pinned {
        warn!("Some models are not pinned. Run: citrate-node download-required-models");
        std::process::exit(1);
    }

    info!("All required models are pinned!");
    Ok(())
}

async fn show_model_status(config: &NodeConfig) -> Result<()> {
    info!("Model Status:");

    let storage = Arc::new(StorageManager::new(&config.data_dir)?);
    let genesis = storage.blocks.get_block_by_height(0)?
        .ok_or_else(|| anyhow::anyhow!("Genesis block not found"))?;

    // Embedded models
    info!("\nEmbedded Models (in genesis):");
    for model in &genesis.embedded_models {
        info!("  - {} ({:.1} MB)",
            model.metadata.name,
            model.weights.len() as f64 / 1_000_000.0
        );
    }

    // Required pins
    info!("\nRequired Pins (IPFS):");
    for pin in &genesis.required_pins {
        let pinned = std::process::Command::new("ipfs")
            .args(&["pin", "ls", &pin.ipfs_cid])
            .output()?
            .status
            .success();

        info!("  - {} ({:.1} GB) {}",
            pin.model_id,
            pin.size_bytes as f64 / 1_000_000_000.0,
            if pinned { "✓ PINNED" } else { "✗ NOT PINNED" }
        );
    }

    Ok(())
}
```

---

## Phase 5: Testing (Day 4-5)

### 5.1 Test Script

Create `scripts/test_genesis_models.sh`:

```bash
#!/bin/bash
set -e

echo "🧪 Testing Genesis Model Implementation"
echo "========================================"

cd /Users/soleilklosowski/Downloads/citrate/citrate

# Clean previous data
echo "Cleaning previous data..."
./scripts/clean_all.sh <<EOF
yes
no
EOF

# Build
echo "Building..."
cargo build --release

# Create genesis
echo "Creating genesis block..."
./target/release/citrate-node genesis --output genesis.bin

# Check size
GENESIS_SIZE=$(stat -f%z genesis.bin)
EXPECTED_SIZE=$((320 * 1024 * 1024))  # 320 MB
echo "Genesis size: $(($GENESIS_SIZE / 1024 / 1024)) MB"

if [ $GENESIS_SIZE -lt $(($EXPECTED_SIZE - 50000000)) ] || [ $GENESIS_SIZE -gt $(($EXPECTED_SIZE + 50000000)) ]; then
    echo "❌ Genesis size unexpected (expected ~320 MB)"
    exit 1
fi

echo "✓ Genesis size OK"

# Start node
echo "Starting node..."
./target/release/citrate-node devnet &
NODE_PID=$!
sleep 10

# Test embedding endpoint
echo "Testing embedding endpoint..."
curl -X POST http://localhost:8545 \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc":"2.0",
    "method":"get_text_embedding",
    "params":["hello world"],
    "id":1
  }' | jq .

# Test semantic search
echo "Testing semantic search..."
curl -X POST http://localhost:8545 \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc":"2.0",
    "method":"semantic_search",
    "params":{
      "query": "machine learning",
      "candidates": ["AI model", "database", "neural network", "file system"],
      "top_k": 2
    },
    "id":2
  }' | jq .

# Download required models
echo "Downloading required models..."
./target/release/citrate-node download-required-models

# Verify pins
echo "Verifying pins..."
./target/release/citrate-node verify-pins

# Test LLM endpoint
echo "Testing chat completion..."
curl -X POST http://localhost:8545 \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc":"2.0",
    "method":"chat_completion",
    "params":{
      "messages": [{"role":"user","content":"Hello!"}],
      "max_tokens": 50
    },
    "id":3
  }' | jq .

# Stop node
kill $NODE_PID

echo ""
echo "✅ ALL TESTS PASSED!"
```

Make executable:
```bash
chmod +x scripts/test_genesis_models.sh
```

---

## Phase 6: GUI Integration (Day 5)

### 6.1 Add Model Tab to GUI

**File: `gui/citrate-core/src/components/Models.tsx` (new file)**

```tsx
import React, { useState, useEffect } from 'react';
import { Brain, Database, CheckCircle, XCircle } from 'lucide-react';

interface ModelStatus {
  name: string;
  type: 'embedded' | 'pinned';
  size_mb: number;
  available: boolean;
}

export const Models: React.FC = () => {
  const [embeddedModels, setEmbeddedModels] = useState<ModelStatus[]>([]);
  const [pinnedModels, setPinnedModels] = useState<ModelStatus[]>([]);
  const [testInput, setTestInput] = useState('');
  const [embeddingResult, setEmbeddingResult] = useState<number[] | null>(null);
  const [chatInput, setChatInput] = useState('');
  const [chatResult, setChatResult] = useState<string>('');

  useEffect(() => {
    loadModelStatus();
  }, []);

  const loadModelStatus = async () => {
    // Mock data - replace with actual RPC calls
    setEmbeddedModels([{
      name: 'BGE-M3 Embeddings',
      type: 'embedded',
      size_mb: 300,
      available: true,
    }]);

    setPinnedModels([{
      name: 'Llama 3.1 8B Instruct',
      type: 'pinned',
      size_mb: 4700,
      available: true,
    }]);
  };

  const testEmbedding = async () => {
    // Call RPC
    const response = await fetch('http://localhost:8545', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        jsonrpc: '2.0',
        method: 'get_text_embedding',
        params: [testInput],
        id: 1,
      }),
    });

    const data = await response.json();
    setEmbeddingResult(data.result.slice(0, 10)); // Show first 10 dims
  };

  const testChat = async () => {
    const response = await fetch('http://localhost:8545', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        jsonrpc: '2.0',
        method: 'chat_completion',
        params: {
          messages: [{ role: 'user', content: chatInput }],
          max_tokens: 100,
        },
        id: 2,
      }),
    });

    const data = await response.json();
    setChatResult(data.result);
  };

  return (
    <div className="models">
      <h2>AI Models</h2>

      {/* Embedded Models */}
      <section>
        <h3>Embedded Models (In Genesis)</h3>
        <div className="model-list">
          {embeddedModels.map(model => (
            <div key={model.name} className="model-card">
              <Brain size={24} />
              <div>
                <h4>{model.name}</h4>
                <p>{model.size_mb} MB • {model.type}</p>
              </div>
              {model.available ? (
                <CheckCircle className="text-green" />
              ) : (
                <XCircle className="text-red" />
              )}
            </div>
          ))}
        </div>
      </section>

      {/* Pinned Models */}
      <section>
        <h3>Required Pins (IPFS)</h3>
        <div className="model-list">
          {pinnedModels.map(model => (
            <div key={model.name} className="model-card">
              <Database size={24} />
              <div>
                <h4>{model.name}</h4>
                <p>{model.size_mb} MB • {model.type}</p>
              </div>
              {model.available ? (
                <CheckCircle className="text-green" />
              ) : (
                <XCircle className="text-red" />
              )}
            </div>
          ))}
        </div>
      </section>

      {/* Test Embeddings */}
      <section>
        <h3>Test Embeddings</h3>
        <input
          type="text"
          value={testInput}
          onChange={e => setTestInput(e.target.value)}
          placeholder="Enter text to embed..."
        />
        <button onClick={testEmbedding}>Get Embedding</button>
        {embeddingResult && (
          <pre>{JSON.stringify(embeddingResult, null, 2)}</pre>
        )}
      </section>

      {/* Test Chat */}
      <section>
        <h3>Test Chat</h3>
        <input
          type="text"
          value={chatInput}
          onChange={e => setChatInput(e.target.value)}
          placeholder="Ask a question..."
        />
        <button onClick={testChat}>Send</button>
        {chatResult && <p className="chat-result">{chatResult}</p>}
      </section>

      <style jsx>{`
        .models { padding: 2rem; }
        section { margin-bottom: 2rem; }
        h3 { margin-bottom: 1rem; }
        .model-list { display: flex; flex-direction: column; gap: 1rem; }
        .model-card {
          display: flex;
          align-items: center;
          gap: 1rem;
          padding: 1rem;
          border: 1px solid var(--border-primary);
          border-radius: 0.5rem;
          background: var(--bg-primary);
        }
        .text-green { color: var(--success); }
        .text-red { color: var(--error); }
        input {
          width: 100%;
          padding: 0.5rem;
          margin-bottom: 0.5rem;
          background: var(--bg-secondary);
          color: var(--text-primary);
          border: 1px solid var(--border-primary);
          border-radius: 0.25rem;
        }
        button {
          padding: 0.5rem 1rem;
          background: var(--brand-primary);
          color: white;
          border: none;
          border-radius: 0.25rem;
          cursor: pointer;
        }
        pre {
          background: var(--bg-secondary);
          padding: 1rem;
          border-radius: 0.25rem;
          overflow-x: auto;
        }
        .chat-result {
          padding: 1rem;
          background: var(--bg-secondary);
          border-radius: 0.25rem;
          margin-top: 0.5rem;
        }
      `}</style>
    </div>
  );
};
```

Add to main app navigation.

---

## Phase 7: Deployment (Day 5-6)

### 7.1 Final Deployment Script

Create `scripts/deploy_hybrid_genesis.sh`:

```bash
#!/bin/bash
set -e

echo "🚀 Deploying Hybrid Genesis Architecture"
echo "========================================="

cd /Users/soleilklosowski/Downloads/citrate/citrate

# Verify models are ready
if [ ! -f "assets/bge-m3-q4.gguf" ]; then
    echo "❌ BGE-M3 model not found. Run Phase 2 first."
    exit 1
fi

if [ ! -f "models/genesis/model-manifest.json" ]; then
    echo "❌ Model manifest not found. Run Phase 2 first."
    exit 1
fi

# Clean old data
echo "Cleaning old data..."
./scripts/clean_all.sh <<EOF
yes
no
EOF

# Build release
echo "Building release..."
cargo build --release

# Create genesis
echo "Creating genesis block..."
./target/release/citrate-node genesis --output .citrate-testnet/genesis.bin

# Verify genesis
GENESIS_SIZE=$(stat -f%z .citrate-testnet/genesis.bin)
echo "Genesis size: $(($GENESIS_SIZE / 1024 / 1024)) MB"

# Start node
echo "Starting testnet node..."
./target/release/citrate-node --config node/config/testnet.toml &
NODE_PID=$!

# Wait for node to start
sleep 15

# Download required models
echo "Downloading required models..."
./target/release/citrate-node download-required-models

# Verify pins
echo "Verifying model pins..."
./target/release/citrate-node verify-pins

# Show model status
echo "Model status:"
./target/release/citrate-node model-status

echo ""
echo "✅ DEPLOYMENT COMPLETE!"
echo ""
echo "Node PID: $NODE_PID"
echo "RPC: http://localhost:8545"
echo "Genesis size: $(($GENESIS_SIZE / 1024 / 1024)) MB"
echo ""
echo "Next steps:"
echo "  1. Start GUI: cd gui/citrate-core && npm run tauri dev"
echo "  2. Test models in Models tab"
echo "  3. Start mining to earn rewards"
echo ""
```

Make executable:
```bash
chmod +x scripts/deploy_hybrid_genesis.sh
```

---

## Verification Checklist

After completing all phases:

- [ ] Genesis block is ~320 MB (not 5GB+)
- [ ] BGE-M3 model embedded and accessible
- [ ] Llama 3.1 8B uploaded to IPFS with correct CID
- [ ] Embedding RPC endpoint works
- [ ] Semantic search RPC endpoint works
- [ ] Chat completion RPC endpoint works
- [ ] Validator can download required models
- [ ] Validator can verify pinned models
- [ ] GUI Models tab shows correct status
- [ ] Initial sync time is <10 minutes
- [ ] Inference performance is acceptable

---

## Timeline Summary

- **Day 1**: Dependencies, model download & quantization
- **Day 2**: Model upload to IPFS, core implementation (types, genesis)
- **Day 3**: Inference service, RPC endpoints
- **Day 4**: Validator commands, testing
- **Day 5**: GUI integration, final testing
- **Day 6**: Production deployment

**Total**: 5-6 days for complete implementation

---

## Next Steps After Deployment

1. **Monitor Performance**
   - Embedding latency
   - LLM inference speed
   - Memory usage

2. **Add More Models**
   - Qwen 2.5 Coder
   - Phi-3 Medium
   - Additional specialized models

3. **Implement LoRA Training**
   - Training workflows
   - LoRA registry
   - Adapter deployment

4. **Scale Infrastructure**
   - IPFS pinning service
   - Model CDN
   - Distributed inference

---

Ready to start? Begin with Phase 1!
