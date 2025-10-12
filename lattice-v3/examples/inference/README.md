# Lattice AI Inference Examples

This directory contains practical examples demonstrating AI model deployment and inference on Lattice blockchain using Apple Silicon optimization.

## Prerequisites

### System Requirements
- macOS 13+ (Ventura or newer)
- Apple Silicon Mac (M1/M2/M3)
- Python 3.9+
- 8GB+ RAM

### Setup
```bash
# Install Python dependencies
pip install torch transformers coremltools
pip install web3 ipfshttpclient
pip install numpy pillow requests

# Start required services
ipfs daemon &
./target/release/lattice devnet &
```

## Available Examples

### 1. Text Classification (`text_classification.py`)
Demonstrates sentiment analysis using DistilBERT optimized for Neural Engine.

**Features:**
- HuggingFace model deployment
- Real-time sentiment analysis
- CoreML optimization
- Neural Engine acceleration

**Usage:**
```bash
python3 examples/inference/text_classification.py
```

**Sample Output:**
```
📝 Text: "The new MacBook Pro with M3 chip delivers incredible..."
   😊 Sentiment: POSITIVE (confidence: 99.8%)
```

### 2. Image Classification (`image_classification.py`)
Shows vision model inference using ResNet-50 with Metal GPU acceleration.

**Features:**
- Vision model deployment
- Multi-class image classification
- Top-k predictions
- Metal GPU optimization

**Usage:**
```bash
python3 examples/inference/image_classification.py
```

**Sample Output:**
```
🖼️  Image: cat.jpg
   Predictions:
   1. tabby cat (92.3%)
   2. tiger cat (5.1%)
   3. Egyptian cat (1.8%)
```

### 3. Batch Inference (`batch_inference.py`)
Demonstrates high-throughput batch processing with performance metrics.

**Features:**
- Parallel processing with thread pools
- Performance benchmarking
- Latency percentiles (P50, P95, P99)
- Throughput measurement

**Usage:**
```bash
python3 examples/inference/batch_inference.py
```

**Sample Output:**
```
⚡ Performance Metrics:
  • Throughput: 42.3 items/second
  • Average latency: 23.6ms
  • P95 latency: 35.2ms
  • P99 latency: 48.1ms
```

## Model Formats

### CoreML (Recommended)
- Native Apple format
- Best Neural Engine utilization
- Smallest model size
- Fastest inference

### MLX (Alternative)
- Apple's new ML framework
- Good for large language models
- Supports quantization
- Unified memory optimization

### ONNX (Compatibility)
- Cross-platform support
- Metal Performance Shaders
- Broader model compatibility

## Performance Optimization

### Neural Engine (ANE)
Best for models < 500MB:
- 4-bit quantization support
- Ultra-low latency (~5ms)
- High energy efficiency
- Automatic batch optimization

### Metal GPU
Best for larger models:
- Full precision support
- Large batch sizes
- Complex architectures
- Generation tasks

### Memory Management
- Unified memory architecture
- Zero-copy tensor sharing
- Automatic memory pooling
- Smart caching

## Supported Models

### Text Models
- **DistilBERT**: Fast text classification
- **BERT**: General NLP tasks
- **GPT-2**: Text generation
- **DeBERTa**: High-accuracy understanding

### Vision Models
- **ResNet-50**: Classic CNN
- **Vision Transformer**: Modern architecture
- **EfficientNet**: Mobile-optimized
- **MobileNet**: Edge deployment

### Multimodal
- **CLIP**: Image-text matching
- **ALIGN**: Visual-language tasks

## Deployment Workflow

1. **Import from HuggingFace**
   ```python
   python tools/import_model.py huggingface bert-base-uncased
   ```

2. **Convert to CoreML**
   - Automatic during import
   - Optimizes for target hardware
   - Applies quantization if beneficial

3. **Upload to IPFS**
   - Distributed storage
   - Content-addressed
   - Pinning incentives

4. **Register On-chain**
   - Immutable registry
   - Access control
   - Revenue tracking

5. **Run Inference**
   - CLI or Python SDK
   - Metal acceleration
   - Proof generation

## Benchmarks

### M2 MacBook Pro
| Model | Size | Format | Latency | Throughput |
|-------|------|--------|---------|------------|
| DistilBERT | 265MB | CoreML | 5ms | 200 req/s |
| BERT-base | 440MB | CoreML | 8ms | 125 req/s |
| ResNet-50 | 100MB | CoreML | 3ms | 330 req/s |
| GPT-2 | 550MB | MLX | 20ms | 50 req/s |

### M3 Max
| Model | Size | Format | Latency | Throughput |
|-------|------|--------|---------|------------|
| DistilBERT | 265MB | CoreML | 3ms | 330 req/s |
| BERT-base | 440MB | CoreML | 5ms | 200 req/s |
| ResNet-50 | 100MB | CoreML | 2ms | 500 req/s |
| GPT-2 | 550MB | MLX | 12ms | 83 req/s |

## Troubleshooting

### IPFS Connection Error
```bash
# Ensure IPFS daemon is running
ipfs daemon

# Check IPFS status
ipfs swarm peers
```

### Model Deployment Failed
```bash
# Check Lattice node
curl http://localhost:8545

# View logs
tail -f lattice.log
```

### Slow Inference
- Ensure using optimized model format
- Check if Neural Engine is engaged
- Verify batch size is appropriate
- Monitor memory pressure

## Advanced Topics

### Custom Models
See `tools/convert_to_coreml.py` for converting your own models.

### Distributed Inference
Multiple nodes can serve the same model for load balancing.

### Proof Generation
All inferences generate cryptographic proofs for verification.

### Revenue Sharing
Model owners earn tokens for each inference request.

## Resources

- [CoreML Documentation](https://developer.apple.com/documentation/coreml)
- [Metal Performance Shaders](https://developer.apple.com/documentation/metalperformanceshaders)
- [Lattice Documentation](https://lattice.xyz/docs)
- [Model Registry Contract](../../contracts/src/ModelRegistry.sol)

## License

MIT - See LICENSE file in repository root.