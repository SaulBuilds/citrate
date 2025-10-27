# Citrate AI Model Tools

Tools for importing, converting, and deploying AI models on Citrate blockchain with Apple Silicon optimization.

## Prerequisites

### Python Dependencies
```bash
pip install torch transformers coremltools
pip install web3 ipfshttpclient
pip install numpy pillow
```

### System Requirements
- macOS 13+ (Ventura or newer)
- Apple Silicon (M1/M2/M3) recommended
- Python 3.9+
- IPFS daemon running
- Citrate node running

## Available Tools

### 1. convert_to_coreml.py
Converts HuggingFace models to CoreML format optimized for Apple Silicon.

```bash
# Convert a model
python convert_to_coreml.py bert-base-uncased

# Optimize for Neural Engine (4-bit quantization)
python convert_to_coreml.py bert-base-uncased --optimize-neural-engine

# List supported models
python convert_to_coreml.py --list-supported
```

### 2. import_model.py
Imports models from HuggingFace and deploys them to Citrate.

```bash
# Import from HuggingFace (auto-converts to CoreML)
python import_model.py huggingface bert-base-uncased

# Deploy existing model file
python import_model.py deploy model.mlpackage

# List recommended models
python import_model.py list
```

## Quick Start

### 1. Start Required Services
```bash
# Start IPFS
ipfs daemon &

# Start Citrate node
./target/release/citrate-node devnet
```

### 2. Deploy Your First Model
```bash
# Import BERT from HuggingFace
python tools/import_model.py huggingface bert-base-uncased

# This will:
# 1. Download model from HuggingFace
# 2. Convert to CoreML format
# 3. Upload to IPFS
# 4. Register on Citrate blockchain
```

### 3. Run Inference
```bash
# Use the Citrate CLI
citrate-cli model inference \
  --model-id <model_hash> \
  --input input.json
```

## Supported Models

### Text Models (Neural Engine Optimized)
- `bert-base-uncased` - General NLP tasks
- `distilbert-base-uncased` - Lightweight BERT
- `microsoft/deberta-v3-base` - Enhanced BERT

### Generation Models (GPU Optimized)
- `gpt2` - Text generation
- `distilgpt2` - Lightweight GPT-2
- `microsoft/phi-2` - Small but powerful LLM

### Vision Models (Neural Engine Optimized)
- `google/vit-base-patch16-224` - Vision Transformer
- `microsoft/resnet-50` - Classic CNN

### Multimodal Models
- `openai/clip-vit-base-patch32` - Image-text matching

## Model Optimization Guide

### Neural Engine vs GPU

**Use Neural Engine for:**
- Models < 500MB
- Classification tasks
- Real-time inference
- Battery-efficient operation

**Use GPU for:**
- Large language models
- Generation tasks
- Models > 500MB
- Maximum performance

### Optimization Tips

1. **Quantization**: Use 4-bit quantization for Neural Engine
   ```bash
   python convert_to_coreml.py model --optimize-neural-engine
   ```

2. **Batch Size**: Keep batch size small (1-4) for Neural Engine

3. **Input Size**: Use standard sizes (512 for text, 224x224 for images)

## Troubleshooting

### IPFS Connection Error
```bash
# Make sure IPFS is running
ipfs daemon

# Check IPFS status
ipfs id
```

### Citrate Connection Error
```bash
# Make sure Citrate node is running
./target/release/citrate-node devnet

# Check node status
curl -X POST http://localhost:8545 \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","method":"eth_blockNumber","params":[],"id":1}'
```

### Model Conversion Error
```bash
# Update dependencies
pip install --upgrade torch transformers coremltools

# Try with CPU-only torch if GPU issues
pip install torch --index-url https://download.pytorch.org/whl/cpu
```

## Examples

### Deploy a Text Classifier
```python
# 1. Import and convert
python import_model.py huggingface distilbert-base-uncased --optimize

# 2. Use in your app
from citrate_client import CitrateClient

client = CitrateClient()
result = client.inference(
    model_id="0x...",
    input={"text": "This movie is amazing!"}
)
print(result["sentiment"])  # "positive"
```

### Deploy an Image Classifier
```python
# 1. Import vision model
python import_model.py huggingface google/vit-base-patch16-224

# 2. Run inference
result = client.inference(
    model_id="0x...",
    input={"image_path": "cat.jpg"}
)
print(result["class"])  # "cat"
```

## Performance Benchmarks

| Model | Size | Neural Engine | GPU | CPU |
|-------|------|---------------|-----|-----|
| DistilBERT | 265MB | 5ms | 8ms | 25ms |
| BERT-base | 440MB | 8ms | 12ms | 40ms |
| GPT-2 | 550MB | N/A | 20ms | 80ms |
| ResNet-50 | 100MB | 3ms | 5ms | 15ms |
| ViT-base | 350MB | 10ms | 15ms | 50ms |

*Benchmarks on M2 MacBook Pro*

## Contributing

To add support for new models:

1. Add model config to `SUPPORTED_MODELS` in `convert_to_coreml.py`
2. Test conversion and inference
3. Add to recommended models in `import_model.py`
4. Submit PR with benchmarks

## License

MIT License - See LICENSE file for details.
