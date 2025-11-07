# Citrate AI Model Strategy (2024-2025)

## Overview
This document outlines Citrate's model deployment strategy for cross-platform AI inference with training capabilities.

## Core Philosophy
- **Cross-Platform First**: All models must work on Apple Silicon (Metal) AND NVIDIA (CUDA)
- **Training-Ready**: All models support LoRA fine-tuning and RLHF
- **Multi-Format**: Store models in multiple formats for optimal performance per platform
- **Modern Models**: Use 2024+ models (7B+ parameters for general tasks)

## Storage Architecture

### On-Chain (Smart Contract)
- Model metadata (name, version, creator, license)
- IPFS CID for each format variant
- Hash verification (SHA256 of model weights)
- Access control and pricing

### Off-Chain (IPFS)
Store each model in multiple formats:

1. **SafeTensors** (primary, cross-platform)
   - Modern, safe format
   - Faster loading than pickle
   - Works with HuggingFace transformers
   - Size: Original (e.g., 8B model ≈ 16GB fp16)

2. **GGUF** (optimized inference)
   - Quantized formats (Q4_K_M, Q5_K_M, Q8_0)
   - CPU-friendly, memory efficient
   - Compatible with llama.cpp
   - Size: 4-8GB for 8B model (quantized)

3. **MLX** (Apple Silicon optimized)
   - Native Metal Performance Shaders
   - Optimized for M1/M2/M3
   - Quantization support
   - Size: Similar to GGUF

4. **TensorRT-LLM** (NVIDIA optimized)
   - Pre-compiled for specific GPUs
   - Maximum performance on NVIDIA
   - Requires compilation per GPU architecture
   - Size: Similar to original

### Format Selection Logic
```python
def select_model_format(device_type, memory_available_gb, task_type):
    """
    Auto-select optimal format based on device and requirements.
    """
    if device_type == "apple_silicon":
        if task_type == "inference_only":
            return "mlx"  # Fastest
        else:
            return "safetensors"  # For fine-tuning

    elif device_type == "nvidia":
        if task_type == "inference_only":
            return "tensorrt_llm"  # Fastest
        else:
            return "safetensors"  # For fine-tuning

    elif device_type == "cpu":
        if memory_available_gb < 16:
            return "gguf_q4"  # Quantized
        else:
            return "gguf_q8"  # Higher quality

    return "safetensors"  # Fallback
```

## Tier 1: Primary Models (Genesis Block)

### 1. Llama 3.1 8B Instruct
**Purpose**: General-purpose text generation, instruction following, chat

**Specifications**:
- Parameters: 8.03B
- Context: 128K tokens
- License: Llama 3.1 Community License
- Quantization: 4-bit, 8-bit, 16-bit available

**Use Cases**:
- User-facing chatbot
- Agent reasoning and planning
- Code assistance
- Document analysis

**LoRA Training**:
- Supported: ✅ Yes
- Tools: PEFT, Axolotl, Unsloth
- Typical LoRA rank: 8-64
- Training hardware: Single A100 40GB or M2 Ultra

**Storage Breakdown**:
- SafeTensors (fp16): 16GB
- GGUF (Q4_K_M): 4.7GB
- GGUF (Q8_0): 8.5GB
- MLX (4-bit): 4.5GB
- **Total IPFS**: ~34GB (all formats)

**IPFS CIDs** (example):
```json
{
  "model_id": "0x...",
  "name": "Llama-3.1-8B-Instruct",
  "version": "1.0.0",
  "formats": {
    "safetensors": "bafybei...",  // 16GB
    "gguf_q4": "bafybei...",       // 4.7GB
    "gguf_q8": "bafybei...",       // 8.5GB
    "mlx": "bafybei..."            // 4.5GB
  },
  "metadata": {
    "license": "Llama-3.1-Community",
    "context_length": 131072,
    "architecture": "LlamaForCausalLM"
  }
}
```

### 2. Mistral 7B v0.3 (Alternative/Fallback)
**Purpose**: Efficient general-purpose model with permissive license

**Specifications**:
- Parameters: 7.24B
- Context: 32K tokens (extendable)
- License: Apache 2.0
- Quantization: All formats supported

**Advantages over Llama**:
- More permissive license (no restrictions)
- Slightly more efficient (fewer parameters)
- Better for commercial deployments

**Storage**: ~30GB (all formats)

### 3. Qwen 2.5 Coder 7B
**Purpose**: Code generation, completion, debugging

**Specifications**:
- Parameters: 7.61B
- Context: 128K tokens
- License: Apache 2.0
- Specialized: Code understanding

**Use Cases**:
- Smart contract generation
- Code review and analysis
- Developer tools
- Automated testing

**Storage**: ~32GB (all formats)

### 4. BGE-M3
**Purpose**: Text embeddings, semantic search, RAG

**Specifications**:
- Parameters: 568M
- Max length: 8192 tokens
- Output: 1024-dim embeddings
- License: MIT

**Use Cases**:
- Model discovery (semantic search)
- Documentation RAG
- Similarity matching

**Storage**: ~2GB (all formats)

## Tier 2: Specialized Models (Post-Genesis)

### LLaVA 1.6 Mistral 7B
**Purpose**: Vision-language understanding

**Specifications**:
- Parameters: 7B (LLM) + 336M (vision)
- Input: Images + text
- License: Apache 2.0

**Use Cases**:
- NFT image analysis
- Visual model documentation
- Multimodal agents

**Storage**: ~15GB

### Stable Diffusion XL
**Purpose**: Image generation

**Specifications**:
- Parameters: 3.5B
- Resolution: Up to 1024x1024
- License: OpenRAIL++-M

**Use Cases**:
- Model visualization
- NFT generation
- Marketing materials

**Storage**: ~7GB

## Genesis Block Model Inclusion Strategy

### Recommended: Minimal Genesis + Fast Deploy

**Genesis Block** (keep tiny):
- Placeholder ONNX (147 bytes)
- Metadata contracts only

**Immediate Post-Genesis Deployment** (automated):
1. Llama 3.1 8B (GGUF Q4 only) - 5GB
2. BGE-M3 embeddings - 1GB
3. Total initial download: **6GB**

**Rationale**:
- Fast genesis block creation
- Quick node sync for new validators
- Users download only formats they need
- Can update models without re-genesis

### Alternative: Rich Genesis (Not Recommended)

If you insist on genesis inclusion:
1. Llama 3.1 8B (GGUF Q4) - 5GB
2. BGE-M3 - 1GB
3. Total genesis size: ~6GB + blockchain overhead

**Issues**:
- Slow initial sync
- Hard to update
- New validators download all models even if unused

## LoRA Training Architecture

### On-Chain LoRA Registry
```solidity
contract LoRARegistry {
    struct LoRAAdapter {
        address creator;
        bytes32 baseModelHash;  // Parent model
        string ipfsCid;         // LoRA weights
        uint256 rank;           // LoRA rank (8, 16, 32, 64)
        uint256 alpha;          // Scaling factor
        string targetModules;   // Which layers adapted
        uint256 trainingSteps;
        bytes32 datasetHash;    // Training data provenance
    }

    mapping(bytes32 => LoRAAdapter) public adapters;

    function registerLoRA(...) external returns (bytes32 adapterId);
    function mergeLoRA(bytes32 baseModel, bytes32 lora) external;
}
```

### Training Workflow
1. **Select Base Model**: Llama 3.1 8B from registry
2. **Download Format**: SafeTensors (for training)
3. **Prepare Dataset**: Upload to IPFS, register hash
4. **Train LoRA**:
   - Use PEFT/Unsloth
   - Rank 8-32 (small), 64+ (complex tasks)
   - 1000-10000 steps typical
5. **Upload LoRA**: To IPFS (typically 50-200MB)
6. **Register On-Chain**: Link to base model
7. **Verification**: Reproducible training proof

### RLHF/DPO Support
```python
# Example: Train with DPO (Direct Preference Optimization)
from trl import DPOTrainer
from peft import LoraConfig, get_peft_model

# Load base model
model = AutoModelForCausalLM.from_pretrained("llama-3.1-8b")

# Add LoRA
lora_config = LoraConfig(
    r=64,
    lora_alpha=16,
    target_modules=["q_proj", "v_proj", "k_proj", "o_proj"],
    lora_dropout=0.05,
    bias="none",
)
model = get_peft_model(model, lora_config)

# Train with human preferences
dpo_trainer = DPOTrainer(
    model=model,
    ref_model=ref_model,
    train_dataset=preference_dataset,
    beta=0.1,  # KL penalty
)
dpo_trainer.train()

# Upload LoRA adapter to IPFS
adapter_path = "lora_adapter"
model.save_pretrained(adapter_path)
ipfs_cid = upload_to_ipfs(adapter_path)

# Register on-chain
register_lora(base_model_hash, ipfs_cid, lora_config)
```

## Implementation Roadmap

### Phase 1: Foundation (Week 1-2)
- [ ] Implement multi-format model storage (SafeTensors, GGUF, MLX)
- [ ] Create IPFS upload/download tools
- [ ] Deploy smart contracts for model registry
- [ ] Download and quantize Llama 3.1 8B
- [ ] Deploy BGE-M3 embeddings

### Phase 2: Training Infrastructure (Week 3-4)
- [ ] Implement LoRA registry contract
- [ ] Create training workflow scripts (PEFT/Axolotl)
- [ ] Build dataset upload/verification
- [ ] Add on-chain LoRA adapter tracking
- [ ] Create merge tooling (base + LoRA → full model)

### Phase 3: Optimization (Week 5-6)
- [ ] Add TensorRT-LLM compilation for NVIDIA
- [ ] Optimize MLX quantization for Apple Silicon
- [ ] Implement automatic format selection
- [ ] Add caching layer (local + IPFS pinning)
- [ ] Performance benchmarking

### Phase 4: Advanced Training (Week 7-8)
- [ ] Implement RLHF workflow
- [ ] Add DPO (Direct Preference Optimization)
- [ ] Create distributed training support
- [ ] Build federated learning framework
- [ ] Add model merging (SLERP, TIES)

## Cost Analysis

### Storage Costs (IPFS + Filecoin)
- Llama 3.1 8B (all formats): ~34GB × $0.00001/GB/month = $0.34/month
- 10 models: ~$3.40/month
- LoRA adapters (50MB each): Negligible

### Training Costs
- **Cloud GPU** (A100 40GB):
  - LoRA fine-tune: 2-6 hours = $20-60
  - Full fine-tune: Not recommended (use LoRA)

- **Apple Silicon** (M2 Ultra):
  - LoRA fine-tune: 4-12 hours = Free (local)
  - Memory: 64GB+ recommended

### Inference Costs
- **Per 1000 tokens**:
  - Cloud (A100): $0.001
  - Apple Silicon (M2): Free (local)
  - CPU (quantized): Free (local)

## Security Considerations

### Model Verification
1. **Hash Verification**: SHA256 of weights
2. **Signature**: Model creator cryptographic signature
3. **Reproducibility**: Training config + dataset hash
4. **Sandboxing**: Run inference in isolated environment

### LoRA Safety
1. **Base Model Lock**: LoRA must reference approved base
2. **Size Limits**: Max 500MB per LoRA adapter
3. **Rate Limiting**: Max 10 LoRAs per user per day
4. **Audit Trail**: All training parameters on-chain

### Malicious Model Detection
1. **Behavioral Analysis**: Monitor outputs for harmful content
2. **Community Reporting**: Stake-weighted voting
3. **Automatic Flagging**: Keyword/pattern detection
4. **Quarantine**: Pause serving while under review

## Monitoring & Analytics

### Model Performance Metrics
- Inference latency (p50, p95, p99)
- Throughput (tokens/second)
- Memory usage
- Error rates
- User satisfaction scores

### Training Metrics
- LoRA adapters created per day
- Training dataset sizes
- Fine-tuning success rates
- Model improvement deltas

### Economic Metrics
- Storage costs (IPFS/Filecoin)
- Inference revenue
- Training fees
- Popular models (downloads/usage)

## Future Considerations

### Quantization Research
- **QLoRA**: 4-bit quantized LoRA training
- **GPTQ**: Better quantization than GGUF
- **AWQ**: Activation-aware quantization

### Emerging Models (2025+)
- Llama 4 (expected Q2 2025)
- Mistral Large 2
- Gemini Nano (on-device)
- Phi-4 (Microsoft)

### Advanced Techniques
- **Mixture of Experts (MoE)**: Efficient large models
- **Speculative Decoding**: Faster inference
- **Model Merging**: Combine multiple LoRAs
- **Continuous Pre-training**: Incremental updates

## License Compliance

### Llama 3.1 Community License
- ✅ Commercial use allowed
- ✅ Fine-tuning allowed
- ✅ Distribution allowed
- ❌ Cannot use to train competing models >700M params
- ❌ Acceptable use policy applies

### Apache 2.0 (Mistral, Qwen)
- ✅ Fully permissive
- ✅ No usage restrictions
- ✅ Patent grant included

### Best Practice
- Display license info in UI
- Link to full license text
- Attribute model creators
- Track license per model version

## References

- [Llama 3.1 Model Card](https://huggingface.co/meta-llama/Meta-Llama-3.1-8B-Instruct)
- [Mistral AI Documentation](https://docs.mistral.ai/)
- [LoRA Paper (Hu et al., 2021)](https://arxiv.org/abs/2106.09685)
- [GGUF Specification](https://github.com/ggerganov/llama.cpp/blob/master/gguf-py/README.md)
- [MLX Documentation](https://ml-explore.github.io/mlx/build/html/index.html)
- [TensorRT-LLM](https://github.com/NVIDIA/TensorRT-LLM)
