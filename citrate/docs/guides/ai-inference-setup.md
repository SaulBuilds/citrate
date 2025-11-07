# AI Inference Setup Guide

This guide explains how to set up AI model inference on Citrate using llama.cpp for real LLM inference.

## Overview

Citrate supports two types of AI inference:
1. **Embeddings** (BGE-M3) - Embedded in genesis block, instant access
2. **LLM Inference** (Mistral 7B) - Requires llama.cpp installation

## Prerequisites

- macOS 13+ (for Metal GPU acceleration)
- Apple Silicon (M1/M2/M3/M4) - Highly recommended
- At least 8GB RAM (16GB+ recommended)
- 10GB free disk space for models and llama.cpp

## Quick Start

### 1. Install llama.cpp

```bash
# Clone llama.cpp
cd ~
git clone https://github.com/ggerganov/llama.cpp.git
cd llama.cpp

# Build with Metal GPU support (Apple Silicon)
rm -rf build && mkdir build && cd build
cmake .. -DCMAKE_BUILD_TYPE=Release -DLLAMA_METAL=ON
make -j8 llama-cli llama-embedding

# Verify installation
~/llama.cpp/build/bin/llama-cli --version
```

**Expected build time:** 2-5 minutes on Apple Silicon

### 2. Download AI Models

Citrate requires two models:

#### BGE-M3 Embeddings (437 MB)
```bash
# Already embedded in genesis block - no action needed
# Available immediately when node starts
```

#### Mistral 7B Instruct v0.3 (4.07 GB)
```bash
# Create models directory
mkdir -p ~/Downloads/citrate/citrate/models

# Download from Hugging Face (using wget or curl)
cd ~/Downloads/citrate/citrate/models

# Option 1: Using wget
wget https://huggingface.co/TheBloke/Mistral-7B-Instruct-v0.3-GGUF/resolve/main/Mistral-7B-Instruct-v0.3-Q4_K_M.gguf

# Option 2: Using curl
curl -L -o Mistral-7B-Instruct-v0.3-Q4_K_M.gguf \
  https://huggingface.co/TheBloke/Mistral-7B-Instruct-v0.3-GGUF/resolve/main/Mistral-7B-Instruct-v0.3-Q4_K_M.gguf

# Verify download (should show ~4.07 GB)
ls -lh Mistral-7B-Instruct-v0.3-Q4_K_M.gguf
```

**Download time:** 5-15 minutes depending on connection speed

### 3. Configure Environment

```bash
# Set llama.cpp path (add to ~/.zshrc or ~/.bashrc)
export LLAMA_CPP_PATH="$HOME/llama.cpp"

# Verify environment
echo $LLAMA_CPP_PATH
```

### 4. Start Citrate Node

```bash
cd ~/Downloads/citrate/citrate

# Start devnet node
./target/release/citrate --data-dir .citrate-devnet devnet

# Node will automatically detect llama.cpp and models
```

## Testing Inference

### Test LLM Inference via RPC

```bash
# Simple math question
curl -X POST http://localhost:8545 \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "method": "citrate_chatCompletion",
    "params": [{
      "model": "mistral-7b-instruct-v0.3",
      "messages": [
        {"role": "system", "content": "You are a helpful assistant"},
        {"role": "user", "content": "What is 2 + 2?"}
      ],
      "max_tokens": 50,
      "temperature": 0.3
    }],
    "id": 1
  }' | jq -r '.result.choices[0].message.content'
```

**Expected output:**
```
The sum of 2 plus 2 is 4.
```

### Test Embeddings (BGE-M3)

```bash
# Generate embeddings
curl -X POST http://localhost:8545 \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "method": "citrate_getTextEmbedding",
    "params": [["artificial intelligence", "blockchain technology"]],
    "id": 1
  }' | jq -r '.result | length'
```

**Expected output:**
```
2
```
(Returns 2 embedding vectors, each with 1024 dimensions)

### Run Comprehensive Benchmark

```bash
# Create test script
cat > /tmp/test-ai.sh << 'EOF'
#!/bin/bash
echo "Testing Math Reasoning..."
curl -s -X POST http://localhost:8545 \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "method": "citrate_chatCompletion",
    "params": [{
      "model": "mistral-7b-instruct-v0.3",
      "messages": [
        {"role": "system", "content": "You are a math tutor"},
        {"role": "user", "content": "If a train travels 120 km in 2 hours, what is its average speed?"}
      ],
      "max_tokens": 150,
      "temperature": 0.3
    }],
    "id": 1
  }' | jq -r '.result.choices[0].message.content'
echo ""
echo "✓ Math test complete"
EOF

chmod +x /tmp/test-ai.sh
/tmp/test-ai.sh
```

## Performance Benchmarks

### Apple M2 Max (32GB RAM)

| Task | Model | Latency | Tokens/sec |
|------|-------|---------|------------|
| Math reasoning | Mistral 7B | 2.3s | ~65 tok/s |
| Code generation | Mistral 7B | 4.1s | ~50 tok/s |
| Historical knowledge | Mistral 7B | 4.9s | ~55 tok/s |
| Embeddings (batch 3) | BGE-M3 | 0.012s | Instant |

### GPU Acceleration

llama.cpp automatically uses Metal GPU acceleration on Apple Silicon:
- **Metal Performance Shaders** for matrix operations
- **Neural Engine** for quantized inference
- **Unified Memory** for efficient data transfer

## Troubleshooting

### Model Not Found Error

```
Error: Model file not found: ./models/Mistral-7B-Instruct-v0.3-Q4_K_M.gguf
```

**Solution:**
```bash
# Verify model exists
ls -lh ./models/Mistral-7B-Instruct-v0.3-Q4_K_M.gguf

# If missing, download again (see step 2)
```

### llama.cpp Not Found

```
Error: Failed to initialize GGUF engine: llama.cpp binary not found
```

**Solution:**
```bash
# Set environment variable
export LLAMA_CPP_PATH="$HOME/llama.cpp"

# Verify binary exists
ls -lh $LLAMA_CPP_PATH/build/bin/llama-cli

# If missing, rebuild llama.cpp (see step 1)
```

### Slow Inference Performance

**Possible causes:**
1. Running on Intel Mac (no Metal support)
2. Low RAM causing swap usage
3. Model file on slow disk (network drive)

**Solutions:**
```bash
# Check system RAM usage
top -l 1 | grep PhysMem

# Move models to SSD if needed
mv ./models ~/.citrate/models

# Reduce thread count for lower RAM usage
export GGUF_THREADS=2
```

### Out of Memory

```
Error: Failed to load model: insufficient memory
```

**Solution:**
```bash
# Use smaller quantized model
# Q4_K_M (4.07 GB) → Q3_K_M (3.28 GB)

# Or close other applications to free RAM
```

## Advanced Configuration

### Custom Model Path

```bash
# Models directory location
export CITRATE_MODELS_DIR="$HOME/.citrate/models"

# llama.cpp binary location
export LLAMA_CPP_PATH="/opt/llama.cpp"
```

### Inference Parameters

Modify parameters in RPC request:

```json
{
  "model": "mistral-7b-instruct-v0.3",
  "max_tokens": 512,        // Max response length
  "temperature": 0.7,       // Creativity (0.0-1.0)
  "top_p": 0.9,            // Nucleus sampling
  "stop": ["</s>"]         // Stop sequences
}
```

### GPU Memory Management

```bash
# Check Metal GPU usage
sudo powermetrics --samplers gpu_power -i 1000 -n 1

# Adjust context window to reduce VRAM
# Default: 4096 tokens, adjust in GGUFEngineConfig
```

## Model Registry

### Supported Models

| Model | Size | Type | Status |
|-------|------|------|--------|
| Mistral 7B Instruct v0.3 | 4.07 GB | LLM | ✅ Working |
| BGE-M3 Embeddings | 437 MB | Embeddings | ✅ Working |
| Llama 3.1 8B (future) | 4.7 GB | LLM | 🚧 Planned |

### Adding Custom Models

```bash
# Place GGUF model in models directory
cp my-model.gguf ./models/

# Model will be automatically detected by name
# Use model name without .gguf extension in RPC calls
```

## API Reference

### citrate_chatCompletion

OpenAI-compatible chat completion endpoint.

**Request:**
```json
{
  "jsonrpc": "2.0",
  "method": "citrate_chatCompletion",
  "params": [{
    "model": "mistral-7b-instruct-v0.3",
    "messages": [
      {"role": "system", "content": "You are a helpful assistant"},
      {"role": "user", "content": "Hello!"}
    ],
    "max_tokens": 150,
    "temperature": 0.7
  }],
  "id": 1
}
```

**Response:**
```json
{
  "jsonrpc": "2.0",
  "result": {
    "id": "chatcmpl-1730932997000",
    "object": "chat.completion",
    "created": 1730932997,
    "model": "mistral-7b-instruct-v0.3",
    "choices": [{
      "index": 0,
      "message": {
        "role": "assistant",
        "content": "Hello! How can I help you today?"
      },
      "finish_reason": "stop"
    }],
    "usage": {
      "prompt_tokens": 20,
      "completion_tokens": 12,
      "total_tokens": 32
    }
  },
  "id": 1
}
```

### citrate_getTextEmbedding

Generate semantic embeddings using BGE-M3.

**Request:**
```json
{
  "jsonrpc": "2.0",
  "method": "citrate_getTextEmbedding",
  "params": [["text to embed", "another text"]],
  "id": 1
}
```

**Response:**
```json
{
  "jsonrpc": "2.0",
  "result": [
    [0.123, 0.456, ...],  // 1024 dimensions
    [0.789, 0.012, ...]   // 1024 dimensions
  ],
  "id": 1
}
```

## Production Deployment

### Requirements

- **Disk:** 50GB+ SSD for models and data
- **RAM:** 16GB+ for production workloads
- **CPU:** Apple M2 Pro or better
- **Network:** 1 Gbps+ for validator nodes

### Monitoring

```bash
# Check inference latency
curl -w "@curl-format.txt" -o /dev/null -s http://localhost:8545/...

# Monitor GPU usage
sudo powermetrics --samplers gpu_power

# Watch node logs
tail -f devnet.log | grep -i "inference"
```

## Support

- **Documentation:** [docs.citrate.ai](https://docs.citrate.ai)
- **GitHub Issues:** [github.com/citrate-ai/citrate/issues](https://github.com/citrate-ai/citrate/issues)
- **Discord:** [discord.gg/citrate](https://discord.gg/citrate)

## Next Steps

- [Deploy Smart Contracts](./deployment.md)
- [Build DApps with SDKs](../../sdk/javascript/README.md)
- [Run a Validator Node](./validator-guide.md)
