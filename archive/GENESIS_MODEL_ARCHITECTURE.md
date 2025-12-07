# Genesis Model Architecture: Hybrid Approach

## Overview

Citrate uses a **hybrid architecture** for genesis models:
- **Embeddings in-block** (300 MB) - For semantic search and discovery
- **LLM via required pins** (4.7 GB) - For full inference

This balances on-chain guarantees with practical performance.

## Architecture

### 1. Genesis Block Structure

```rust
pub struct GenesisBlock {
    // Standard fields
    header: BlockHeader,
    state_root: Hash,

    // AI Models section
    embedded_models: Vec<EmbeddedModel>,
    required_pins: Vec<RequiredModel>,
}

pub struct EmbeddedModel {
    model_id: ModelId,
    model_type: ModelType,  // Embeddings, Tiny-LLM, etc.
    weights: Vec<u8>,       // Actual model weights
    metadata: ModelMetadata,
}

pub struct RequiredModel {
    model_id: ModelId,
    ipfs_cid: String,
    sha256_hash: Hash,
    size_bytes: u64,
    must_pin: bool,
    slash_penalty: u128,  // LATT tokens
}
```

### 2. Embedded Model: BGE-M3 (300 MB)

**File:** `assets/bge-m3-q4.gguf`

**Why BGE-M3?**
- Text embeddings (1024-dim vectors)
- Powers semantic search across models
- Enables RAG (Retrieval-Augmented Generation)
- Small enough for in-block storage

**Quantization:**
```
Original (fp16): 1.1 GB
Quantized (Q8): 600 MB
Quantized (Q4): 300 MB ← Use this
```

**Storage in Genesis:**
```rust
const BGE_M3_Q4: &[u8] = include_bytes!("../assets/bge-m3-q4.gguf");

fn embed_genesis_model() -> EmbeddedModel {
    EmbeddedModel {
        model_id: ModelId::from_name("bge-m3"),
        model_type: ModelType::Embeddings,
        weights: BGE_M3_Q4.to_vec(),
        metadata: ModelMetadata {
            name: "BGE-M3".to_string(),
            version: "1.0.0".to_string(),
            context_length: 8192,
            embedding_dim: 1024,
            license: "MIT".to_string(),
        },
    }
}
```

**Access Pattern:**
```rust
// Inference directly from genesis state
pub fn get_embedding(text: &str) -> Result<Vec<f32>> {
    let genesis = get_genesis_block();
    let model = genesis.embedded_models
        .iter()
        .find(|m| m.model_id == ModelId::from_name("bge-m3"))
        .ok_or("Model not found")?;

    // Load model from weights
    let embedder = load_gguf_model(&model.weights)?;
    let embedding = embedder.embed(text)?;

    Ok(embedding)
}
```

### 3. Required Pin: Llama 3.1 8B (4.7 GB)

**IPFS CID:** `bafybei...` (determined at genesis creation)

**Validator Requirements:**
1. Download model from IPFS within 24 hours
2. Verify SHA256 hash
3. Keep pinned (checked periodically)
4. Serve to peers on request

**Enforcement:**
```rust
pub struct ValidatorPinCheck {
    model_cid: String,
    last_check: Timestamp,
    status: PinStatus,
}

pub enum PinStatus {
    Pinned,
    Unpinned,
    Unverified,
}

// Consensus rule
fn validate_block(block: &Block, validator: &Validator) -> Result<()> {
    // Check if validator has required models pinned
    for required in &genesis.required_pins {
        let status = check_pin_status(validator, &required.ipfs_cid)?;

        if status != PinStatus::Pinned {
            // Slash validator
            slash_validator(validator, required.slash_penalty);
            return Err("Validator missing required model");
        }
    }

    Ok(())
}
```

**Pin Verification Protocol:**
```
Proposer → Validator: CHALLENGE(model_cid, random_offset)
Validator → Proposer: RESPONSE(data_chunk, merkle_proof)
Proposer: Verify(data_chunk, offset, merkle_root)
```

### 4. On-Chain Inference API

**Embeddings (In-Block):**
```rust
#[rpc]
pub async fn get_text_embedding(text: String) -> Result<Vec<f32>> {
    // Use embedded BGE-M3
    let embedding = genesis_embedder.embed(&text)?;
    Ok(embedding)
}

#[rpc]
pub async fn semantic_search(
    query: String,
    candidates: Vec<String>,
    top_k: usize,
) -> Result<Vec<(String, f32)>> {
    let query_emb = genesis_embedder.embed(&query)?;

    let mut scores = vec![];
    for candidate in candidates {
        let cand_emb = genesis_embedder.embed(&candidate)?;
        let similarity = cosine_similarity(&query_emb, &cand_emb);
        scores.push((candidate, similarity));
    }

    scores.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
    Ok(scores.into_iter().take(top_k).collect())
}
```

**LLM Inference (IPFS-Backed):**
```rust
#[rpc]
pub async fn chat_completion(
    messages: Vec<ChatMessage>,
    model: String,  // "llama-3.1-8b"
) -> Result<String> {
    // Load from IPFS (cached locally)
    let model_path = get_pinned_model(&model).await?;
    let llm = load_gguf_model(&model_path)?;

    // Run inference
    let response = llm.generate(messages)?;
    Ok(response)
}
```

## Genesis Creation Process

### Step 1: Prepare Embedded Model

```bash
# Download BGE-M3
huggingface-cli download BAAI/bge-m3 --local-dir bge-m3

# Quantize to Q4 using llama.cpp
python3 ~/llama.cpp/convert.py bge-m3 --outfile bge-m3-fp16.gguf --outtype f16
~/llama.cpp/quantize bge-m3-fp16.gguf bge-m3-q4.gguf Q4_K_M

# Move to assets
mv bge-m3-q4.gguf citrate/assets/
```

**Verify size:**
```bash
ls -lh citrate/assets/bge-m3-q4.gguf
# Expected: ~300M
```

### Step 2: Prepare Required Pin Model

```bash
# Download Llama 3.1 8B
huggingface-cli download meta-llama/Meta-Llama-3.1-8B-Instruct \
  --local-dir llama-3.1-8b \
  --token YOUR_HF_TOKEN

# Convert to GGUF Q4
python3 ~/llama.cpp/convert.py llama-3.1-8b \
  --outfile llama-3.1-8b-fp16.gguf \
  --outtype f16

~/llama.cpp/quantize \
  llama-3.1-8b-fp16.gguf \
  llama-3.1-8b-q4.gguf \
  Q4_K_M

# Calculate hash
sha256sum llama-3.1-8b-q4.gguf
# Output: abc123... llama-3.1-8b-q4.gguf

# Upload to IPFS
ipfs add llama-3.1-8b-q4.gguf
# Output: added bafybei... llama-3.1-8b-q4.gguf
```

### Step 3: Create Genesis Configuration

```rust
// citrate/node/src/genesis.rs

pub fn create_ai_genesis() -> GenesisBlock {
    // 1. Embed BGE-M3
    let bge_m3 = EmbeddedModel {
        model_id: ModelId::from_name("bge-m3"),
        model_type: ModelType::Embeddings,
        weights: include_bytes!("../../assets/bge-m3-q4.gguf").to_vec(),
        metadata: ModelMetadata {
            name: "BGE-M3 Embeddings".to_string(),
            version: "1.0.0".to_string(),
            context_length: 8192,
            embedding_dim: 1024,
            license: "MIT".to_string(),
            framework: "GGUF".to_string(),
        },
    };

    // 2. Require Llama 3.1 8B pin
    let llama_3_1 = RequiredModel {
        model_id: ModelId::from_name("llama-3.1-8b"),
        ipfs_cid: "bafybei...".to_string(),  // From Step 2
        sha256_hash: Hash::from_hex("abc123..."),  // From Step 2
        size_bytes: 4_700_000_000,  // 4.7 GB
        must_pin: true,
        slash_penalty: latt_to_wei(1000),  // 1000 LATT
        grace_period_hours: 24,
    };

    GenesisBlock {
        embedded_models: vec![bge_m3],
        required_pins: vec![llama_3_1],
        // ... other genesis fields
    }
}
```

### Step 4: Test Genesis Size

```bash
cargo build --release
./target/release/citrate-node genesis --output genesis.bin

ls -lh genesis.bin
# Expected: ~320M (300M model + 20M blockchain data)
```

## Validator Onboarding

### New Validator Joins Network

```bash
# 1. Download and verify genesis
citrate-node init --genesis genesis.bin
# Downloads 320 MB genesis block

# 2. Sync blockchain
citrate-node sync
# Fast sync (no models yet)

# 3. Download required models
citrate-node download-required-models
# Downloads Llama 3.1 8B from IPFS (4.7 GB)
# Verifies SHA256
# Pins locally

# 4. Start validating
citrate-node start
```

**Timeline:**
- Genesis download: 3-5 minutes (320 MB)
- Required models: 20-30 minutes (4.7 GB)
- **Total**: ~30 minutes (vs 60+ for in-block)

## Storage Breakdown

### Per Validator

```
Genesis block: 320 MB
  ├─ BGE-M3 (embedded): 300 MB
  └─ Blockchain data: 20 MB

Required pins: 4.7 GB
  └─ Llama 3.1 8B: 4.7 GB

Optional models: 0-∞ GB
  └─ Additional IPFS pins

Total minimum: ~5 GB
```

### Network-Wide (100 validators)

```
Genesis (embedded):
  100 validators × 300 MB = 30 GB

Required pins (IPFS):
  10 pins × 4.7 GB = 47 GB
  (Redundancy: 10x, not 100x)

Total: 77 GB
vs Full in-block: 500 GB
Savings: 85%
```

## Performance Characteristics

### Embeddings (In-Block)

**Cold start:**
```
Load from RocksDB: 100-200 ms
Initialize model: 50-100 ms
First embedding: 150-300 ms
```

**Warm (cached):**
```
Embedding: 5-20 ms
Batch (100 texts): 200-500 ms
```

**Bottleneck:** RocksDB read latency

**Optimization:**
```rust
// Cache model in memory
lazy_static! {
    static ref GENESIS_EMBEDDER: Model = {
        let genesis = get_genesis_block();
        let model = &genesis.embedded_models[0];
        load_gguf_model(&model.weights).unwrap()
    };
}
```

### LLM Inference (IPFS-Backed)

**Cold start:**
```
Download from IPFS: 0 ms (already pinned)
Memory map file: 10-50 ms
Load model: 100-500 ms
First token: 150-600 ms
```

**Warm:**
```
First token: 50-200 ms
Subsequent tokens: 20-100 ms (depends on hardware)
```

**Bottleneck:** GPU/CPU inference speed

## Upgrade Path

### Embedded Models (Hard to Upgrade)

**Option 1: Fork**
- Create new chain with updated genesis
- Migrate state
- Not ideal

**Option 2: Soft Fork**
- Add new embedded model via consensus
- Old model still in genesis (wasted space)
- New model in state

**Option 3: Accept Immutability**
- BGE-M3 is stable (2023 model)
- Embeddings don't need frequent updates
- LoRA can adapt if needed

**Recommendation:** Accept immutability for genesis model

### Required Pins (Easy to Upgrade)

**Via Governance:**
```solidity
contract ModelGovernance {
    function proposeModelUpdate(
        bytes32 oldModelId,
        string memory newIpfsCid,
        bytes32 newHash
    ) external onlyGovernance {
        // Vote on upgrade
        // If passed, update required_pins
    }
}
```

**Migration:**
1. Governance proposes new model
2. Community votes
3. If approved, update required pins list
4. Validators download new model
5. Old model can be unpinned

## Security Considerations

### Embedded Model

**Attack: Malicious Genesis**
- Mitigation: Community verifies genesis hash
- Public download of BGE-M3 weights
- Reproducible build from HuggingFace

**Attack: Model Exploitation**
- Mitigation: Sandboxed inference
- Resource limits
- Rate limiting

### Required Pins

**Attack: Model Unavailability**
- Mitigation: Multiple IPFS gateways
- Fallback CIDs
- Slash validators who don't pin

**Attack: Model Substitution**
- Mitigation: SHA256 verification
- Merkle proofs for chunks
- Challenge-response protocol

## Cost Analysis

### Storage Costs

**Embedded (320 MB):**
- Per validator: Free (required)
- Network: 32 GB (100 validators)

**Required pins (4.7 GB):**
- Per validator: ~$0.10/month (IPFS)
- Network: 47 GB (10 redundancy)

**Total:** ~$0.10/month per validator

### Inference Costs

**Embeddings:**
- Free (on-chain)
- Gas cost: ~100k gas per embedding

**LLM:**
- Free (local inference)
- Gas cost: Only for storage/retrieval

## Comparison to Original Proposal

| Aspect | Full In-Block | Hybrid (This Doc) |
|--------|---------------|-------------------|
| Genesis size | 5+ GB | 320 MB |
| Validator storage | 5 GB | 5 GB |
| Initial sync | 30-60 min | 5-10 min |
| Inference speed | Slow (RocksDB) | Fast (memory-mapped) |
| Upgradability | None | Pins via governance |
| Network storage | 500 GB | 77 GB |
| IPFS dependency | None | For LLM only |
| On-chain guarantees | Full | Embeddings only |

## Conclusion

The **hybrid approach** provides:
- ✅ On-chain semantic search (embedded BGE-M3)
- ✅ Fast LLM inference (IPFS pins)
- ✅ Reasonable genesis size (320 MB)
- ✅ Validator-enforced availability
- ✅ Upgrade path for LLM
- ✅ 85% storage savings vs full in-block

**Tradeoff:**
- ❌ LLM not fully on-chain (but practically required)

**Verdict:** Best balance of on-chain guarantees and practical performance.
