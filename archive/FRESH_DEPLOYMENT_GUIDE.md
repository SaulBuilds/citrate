# Citrate Fresh Deployment Guide
## Production-Ready AI Models with LoRA Training (2024-2025)

**Last Updated**: 2025-01-06

---

## 🎯 Quick Start (TL;DR)

```bash
# 1. Clean everything
cd /Users/soleilklosowski/Downloads/citrate/citrate
./scripts/clean_all.sh

# 2. Check dependencies
./tools/deploy_modern_model.py --check-deps

# 3. Deploy Llama 3.1 8B (recommended)
./tools/deploy_modern_model.py llama-3.1-8b-instruct \
  --formats safetensors gguf_q4 mlx \
  --hf-token YOUR_HF_TOKEN

# 4. Start node
cargo run --release --bin citrate-node devnet

# 5. Start GUI
cd gui/citrate-core && npm run tauri dev
```

---

## 📊 Model Selection Summary

### ✅ RECOMMENDED: Llama 3.1 8B Instruct

**Why This Model?**
- ✅ **8.03B parameters** - Meets your 5-10B requirement
- ✅ **128K context** - Handles very long documents
- ✅ **Full LoRA support** - PEFT, Axolotl, Unsloth compatible
- ✅ **RLHF ready** - DPO, PPO, RLAIF all supported
- ✅ **Cross-platform** - PyTorch → Metal + CUDA + CPU
- ✅ **Best performance** - State-of-the-art for size
- ✅ **Active ecosystem** - Huge community, many LoRA adapters available

**Storage**:
- SafeTensors (fp16): 16GB - For training/fine-tuning
- GGUF Q4_K_M: 4.7GB - For fast inference (CPU/Metal)
- GGUF Q8_0: 8.5GB - For higher quality inference
- MLX 4-bit: 4.5GB - Apple Silicon optimized

**Training Requirements**:
- **LoRA**: 16GB+ RAM (M2 Pro or better, or A100 40GB)
- **Full fine-tune**: Not recommended (use LoRA instead)
- **Inference**: 6GB+ RAM (quantized), 18GB+ (fp16)

### Alternative Options

#### Mistral 7B v0.3
**Use if**: You need Apache 2.0 license (more permissive than Llama)
- Same capabilities as Llama
- Slightly smaller (7.24B params)
- No usage restrictions

#### Phi-3 Medium 14B
**Use if**: You want maximum quality and have 32GB+ RAM
- 14B parameters
- Excellent reasoning
- MIT license
- Requires more memory

#### Qwen 2.5 Coder 7B
**Use if**: Primary use case is code generation
- Specialized for code
- 7.61B parameters
- Apache 2.0 license

---

## 🛠️ Step-by-Step Deployment

### Prerequisites

#### 1. Install Dependencies

```bash
# Python packages
pip3 install huggingface_hub ipfshttpclient

# IPFS (macOS)
brew install ipfs
ipfs init
ipfs daemon &  # Run in background

# llama.cpp (for GGUF quantization)
cd ~
git clone https://github.com/ggerganov/llama.cpp
cd llama.cpp
make

# MLX (for Apple Silicon - optional)
pip3 install mlx mlx-lm

# Rust (if not installed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

#### 2. Get HuggingFace Token

Llama 3.1 requires accepting terms and using an access token:

1. Go to https://huggingface.co/meta-llama/Meta-Llama-3.1-8B-Instruct
2. Click "Agree and access repository"
3. Get your token: https://huggingface.co/settings/tokens
4. Create token with "Read access to contents of all public gated repos"

```bash
# Save token
export HF_TOKEN="hf_your_token_here"
```

### Step 1: Clean Deployment

```bash
cd /Users/soleilklosowski/Downloads/citrate/citrate

# Option A: Clean everything (recommended)
./scripts/clean_all.sh

# Option B: Keep builds, just clean data
rm -rf .citrate-*
rm -rf ~/Library/Application\ Support/citrate-*
```

### Step 2: Check Your Setup

```bash
./tools/deploy_modern_model.py --check-deps
```

**Expected output**:
```
🔍 Checking dependencies...

   ✅ huggingface-cli found
   ✅ ipfs found

Optional tools:
   ✅ llama.cpp found
   ✅ mlx found

✅ All required dependencies found!
```

### Step 3: Choose Your Model Strategy

#### Strategy A: Minimal Genesis + Fast Deploy (RECOMMENDED)

**Timeline**: 30 min setup + 2-4 hours model download/conversion

```bash
# 1. Deploy Llama 3.1 8B (all formats)
./tools/deploy_modern_model.py llama-3.1-8b-instruct \
  --formats safetensors gguf_q4 gguf_q8 mlx \
  --hf-token $HF_TOKEN

# 2. Deploy BGE-M3 embeddings (for semantic search)
./tools/deploy_modern_model.py bge-m3 \
  --formats safetensors gguf_q8
```

**What you get**:
- ✅ Primary LLM for all tasks
- ✅ Embeddings for semantic search
- ✅ All format variants (choose based on device)
- ✅ LoRA training ready

**Total storage**: ~50GB (all formats combined)

#### Strategy B: Core Suite (Multiple Models)

**Timeline**: 6-12 hours

```bash
# Primary LLM
./tools/deploy_modern_model.py llama-3.1-8b-instruct \
  --formats safetensors gguf_q4 mlx \
  --hf-token $HF_TOKEN

# Code specialist
./tools/deploy_modern_model.py qwen-2.5-coder-7b \
  --formats safetensors gguf_q4

# Embeddings
./tools/deploy_modern_model.py bge-m3 \
  --formats safetensors gguf_q8

# High-quality alternative (14B)
./tools/deploy_modern_model.py phi-3-medium-14b \
  --formats gguf_q4  # Smaller format only
```

**Total storage**: ~120GB

### Step 4: Build and Start Node

```bash
# Build node (release mode)
cargo build --release --bin citrate-node

# Start devnet (single node, local testing)
./target/release/citrate-node devnet

# OR start testnet (multi-node, public)
./target/release/citrate-node --config node/config/testnet.toml
```

**Verify node is running**:
```bash
curl -X POST http://localhost:8545 \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","method":"eth_blockNumber","params":[],"id":1}'

# Expected: {"jsonrpc":"2.0","id":1,"result":"0x0"}  (or higher)
```

### Step 5: Register Models On-Chain

The deployment script creates `metadata.json` for each model. You need to register these on-chain.

**Option A: Using GUI (when available)**
1. Open Models tab
2. Click "Register Model"
3. Upload metadata.json
4. Confirm transaction

**Option B: Using CLI (manual)**

```bash
# Example: Register Llama 3.1 8B
curl -X POST http://localhost:8545 \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc":"2.0",
    "method":"citrate_registerModel",
    "params":[{
      "name": "Llama-3.1-8B-Instruct",
      "version": "1.0.0",
      "formats": {
        "safetensors": "bafybei...",
        "gguf_q4": "bafybei...",
        "mlx": "bafybei..."
      },
      "metadata": { ... }
    }],
    "id":1
  }'
```

### Step 6: Start GUI and Create Wallet

```bash
cd gui/citrate-core

# Install dependencies (first time only)
npm install

# Run GUI in development mode
npm run tauri dev
```

**On first launch**:
1. You'll be prompted to create a wallet
2. **Set a NEW password** (write it down!)
3. **Save your 12-word mnemonic** (critical for recovery)
4. Your first account becomes the mining reward address

### Step 7: Start Mining

1. In GUI, go to Dashboard
2. Click "Start Node" (if not already running)
3. Verify "Mining Status" shows "⛏️ Active"
4. Check "Reward Address" matches your wallet
5. Watch balance increase as blocks are mined!

---

## 🧪 Testing LoRA Training

Once models are deployed, test LoRA fine-tuning:

### Prerequisites

```bash
pip3 install torch transformers peft accelerate datasets
```

### Quick LoRA Training Example

```python
from transformers import AutoModelForCausalLM, AutoTokenizer
from peft import LoraConfig, get_peft_model, TaskType
import torch

# 1. Load base model
model_name = "meta-llama/Meta-Llama-3.1-8B-Instruct"
model = AutoModelForCausalLM.from_pretrained(
    model_name,
    torch_dtype=torch.float16,
    device_map="auto",
)
tokenizer = AutoTokenizer.from_pretrained(model_name)

# 2. Configure LoRA
lora_config = LoraConfig(
    task_type=TaskType.CAUSAL_LM,
    r=32,  # Rank (higher = more expressive, but slower)
    lora_alpha=64,
    lora_dropout=0.05,
    target_modules=["q_proj", "v_proj", "k_proj", "o_proj"],  # Which layers to adapt
    bias="none",
)

# 3. Apply LoRA
model = get_peft_model(model, lora_config)
model.print_trainable_parameters()
# Output: trainable params: 83M || all params: 8B || trainable%: 1.03%

# 4. Train (example - you'd use a Trainer here)
# ... training code ...

# 5. Save LoRA adapter (only 50-200MB!)
model.save_pretrained("./lora_adapter")

# 6. Upload to IPFS
import subprocess
result = subprocess.run(
    ["ipfs", "add", "-r", "-Q", "./lora_adapter"],
    capture_output=True,
    text=True
)
lora_cid = result.stdout.strip()
print(f"LoRA adapter CID: {lora_cid}")

# 7. Register on-chain (pseudo-code)
# citrate_registerLoRA(base_model_hash, lora_cid, lora_config)
```

**Hardware Requirements**:
- **LoRA (8B model)**: 16GB+ RAM (M2 Pro, A100 40GB)
- **QLoRA (4-bit)**: 10GB RAM (M1 Pro, RTX 3080)
- **Training time**: 2-12 hours depending on dataset

---

## 🔐 Wallet Password Recovery

Since you're starting fresh, you'll create a NEW wallet with a NEW password.

**Critical: Save These Immediately**
1. **Password** - Write it down physically
2. **12-word mnemonic** - Store in password manager AND physical backup
3. **Private key** (optional) - Export after creation

**To export private key** (after wallet creation):
1. Go to Wallet tab
2. Select your account
3. Click "Export Private Key"
4. Enter password
5. Save the hex string securely

---

## 📊 Model Comparison Chart

| Model | Params | Context | License | LoRA | RLHF | Best For |
|-------|--------|---------|---------|------|------|----------|
| **Llama 3.1 8B** | 8.03B | 128K | Llama 3.1 | ✅ | ✅ | General purpose (BEST) |
| Mistral 7B | 7.24B | 32K | Apache 2.0 | ✅ | ✅ | Permissive license |
| Qwen 2.5 Coder 7B | 7.61B | 128K | Apache 2.0 | ✅ | ✅ | Code generation |
| Phi-3 Medium | 14B | 128K | MIT | ✅ | ✅ | High quality (more RAM) |
| BGE-M3 | 568M | 8K | MIT | ✅ | ❌ | Embeddings |

**Recommendation**: Start with **Llama 3.1 8B** + **BGE-M3**

---

## 🎓 Learning Resources

### LoRA Training
- [PEFT Documentation](https://huggingface.co/docs/peft)
- [Axolotl (advanced training)](https://github.com/OpenAccess-AI-Collective/axolotl)
- [Unsloth (2-5x faster training)](https://github.com/unslothai/unsloth)

### RLHF/DPO
- [TRL (Transformer Reinforcement Learning)](https://huggingface.co/docs/trl)
- [DPO Paper](https://arxiv.org/abs/2305.18290)
- [RLHF Tutorial](https://huggingface.co/blog/rlhf)

### Model Optimization
- [llama.cpp Quantization](https://github.com/ggerganov/llama.cpp)
- [MLX Examples](https://github.com/ml-explore/mlx-examples)
- [TensorRT-LLM Guide](https://github.com/NVIDIA/TensorRT-LLM)

---

## ❓ Troubleshooting

### "HuggingFace authentication required"
```bash
# Make sure you accepted the license and have a token
export HF_TOKEN="hf_your_token_here"
./tools/deploy_modern_model.py llama-3.1-8b-instruct --hf-token $HF_TOKEN
```

### "IPFS connection failed"
```bash
# Make sure IPFS is running
ipfs daemon &

# Check status
ipfs id
```

### "Out of memory during conversion"
- Use fewer formats at once
- Close other applications
- For GGUF: run conversion on a machine with 32GB+ RAM
- For MLX: requires Apple Silicon (M1/M2/M3)

### "Model download is slow"
- HuggingFace servers can be slow for large models
- Llama 3.1 8B is 16GB - expect 30-60 min on fast connection
- Use `--formats gguf_q4` only to skip SafeTensors download

### "Wallet password reset"
**You can't reset the password!** But you can:
1. Delete the account (loses access forever if no backup)
2. Re-import using mnemonic or private key
3. Set a NEW password

---

## 🚀 Next Steps After Deployment

1. **Test Inference**
   ```bash
   curl -X POST http://localhost:8545/v1/chat/completions \
     -H "Content-Type: application/json" \
     -d '{
       "model": "llama-3.1-8b-instruct",
       "messages": [{"role": "user", "content": "Hello!"}]
     }'
   ```

2. **Create Your First LoRA**
   - Choose a domain (customer support, coding, etc.)
   - Prepare dataset (100-10,000 examples)
   - Train LoRA adapter
   - Upload to IPFS
   - Register on-chain

3. **Set Up Model Marketplace**
   - List your LoRA adapters
   - Set pricing
   - Enable discovery

4. **Monitor Performance**
   - Track inference latency
   - Monitor IPFS availability
   - Watch memory usage

---

## 📞 Support

- **Documentation**: `/Users/soleilklosowski/Downloads/citrate/citrate/MODEL_STRATEGY.md`
- **Model Tool**: `./tools/deploy_modern_model.py --help`
- **Model Catalog**: `./tools/deploy_modern_model.py --list`

---

**Ready to deploy? Start here:**
```bash
./tools/deploy_modern_model.py --check-deps
```
