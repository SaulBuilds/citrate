# Citrate Python SDK

A comprehensive Python SDK for interacting with the Citrate AI blockchain platform. Deploy AI models, execute inferences, manage encryption, and handle payments with ease.

## Features

- **Model Deployment**: Deploy AI models to Citrate blockchain with encryption and access control
- **Inference Execution**: Run AI inference on deployed models with pay-per-use pricing
- **Encryption & Security**: End-to-end encryption for model weights and inference data
- **Payment Integration**: Built-in payment handling for model access and revenue sharing
- **Multi-format Support**: CoreML, ONNX, TensorFlow, PyTorch model support

## Installation

```bash
pip install citrate-sdk
```

## Quick Start

```python
from citrate_sdk import CitrateClient, ModelConfig

# Connect to Citrate network
client = CitrateClient("https://mainnet.lattice.ai")

# Deploy encrypted model
model = client.deploy_model(
    model_path="./my_model.mlpackage",
    config=ModelConfig(
        encrypted=True,
        access_price=0.01,  # ETH per inference
        access_list=["0x123..."]
    )
)

# Execute inference
result = client.inference(
    model_id=model.id,
    input_data={"text": "Hello world"},
    encrypted=True
)

print(f"Model output: {result.output_data}")
```

## Authentication

### Using Private Key

```python
from citrate_sdk import CitrateClient

# Initialize with private key
client = CitrateClient(
    rpc_url="https://mainnet.lattice.ai",
    private_key="0x1234..."
)
```

### Generate New Key

```python
from citrate_sdk.crypto import KeyManager

# Generate new key pair
key_manager = KeyManager()
print(f"Address: {key_manager.get_address()}")
print(f"Private Key: {key_manager.get_private_key()}")
```

## Model Deployment

### Basic Deployment

```python
from citrate_sdk import ModelConfig, ModelType, AccessType

config = ModelConfig(
    name="My Model",
    description="A powerful AI model",
    model_type=ModelType.COREML,
    access_type=AccessType.PUBLIC
)

deployment = client.deploy_model("./model.mlpackage", config)
print(f"Model ID: {deployment.model_id}")
```

### Encrypted Deployment

```python
from citrate_sdk import EncryptionConfig

config = ModelConfig(
    encrypted=True,
    encryption_config=EncryptionConfig(
        threshold_shares=3,
        total_shares=5
    ),
    access_type=AccessType.PAID,
    access_price=1000000000000000000  # 1 ETH in wei
)

deployment = client.deploy_model("./model.mlpackage", config)
```

## Inference Execution

### Public Model Inference

```python
# Run inference on public model
result = client.inference(
    model_id="model_abc123",
    input_data={
        "text": "Classify this text",
        "image": "base64_encoded_image"
    }
)

print(f"Prediction: {result.output_data}")
print(f"Confidence: {result.confidence}")
print(f"Gas used: {result.gas_used}")
```

### Paid Model Access

```python
# Purchase access to paid model
tx_hash = client.purchase_model_access(
    model_id="model_xyz789",
    payment_amount=1000000000000000000  # 1 ETH
)

# Execute inference after purchase
result = client.inference(
    model_id="model_xyz789",
    input_data={"text": "Premium inference"}
)
```

### Encrypted Inference

```python
# Run encrypted inference
result = client.inference(
    model_id="encrypted_model_456",
    input_data={"sensitive_data": "private input"},
    encrypted=True
)
```

## Model Management

### List Available Models

```python
# List all public models
models = client.list_models(limit=50)

for model in models:
    print(f"{model['name']}: {model['model_id']}")
```

### Get Model Information

```python
info = client.get_model_info("model_abc123")

print(f"Owner: {info['owner']}")
print(f"Price: {info['access_price']} wei")
print(f"Total inferences: {info['total_inferences']}")
```

## Encryption & Security

### Manual Encryption

```python
from citrate_sdk.crypto import KeyManager

key_manager = KeyManager("0x1234...")

# Encrypt arbitrary data
encrypted = key_manager.encrypt_data("sensitive information")

# Decrypt data
decrypted = key_manager.decrypt_data(encrypted)
```

### Shared Key Derivation

```python
# Generate ECDH shared key with another party
peer_public_key = "0x5678..."
shared_key = key_manager.derive_shared_key(peer_public_key)
```

## Error Handling

```python
from citrate_sdk.errors import (
    ModelNotFoundError,
    InsufficientFundsError,
    InferenceError
)

try:
    result = client.inference("invalid_model", {"data": "test"})
except ModelNotFoundError:
    print("Model doesn't exist")
except InsufficientFundsError:
    print("Not enough funds for inference")
except InferenceError as e:
    print(f"Inference failed: {e}")
```

## Configuration

### Custom RPC Endpoint

```python
# Connect to local development node
client = CitrateClient("http://localhost:8545")

# Connect to testnet
client = CitrateClient("https://testnet.lattice.ai")
```

### Advanced Configuration

```python
from citrate_sdk import CitrateClient

client = CitrateClient(
    rpc_url="https://mainnet.lattice.ai",
    private_key="0x1234...",
)

# Customize timeouts and gas limits
result = client.inference(
    model_id="model_123",
    input_data={"text": "test"},
    max_gas=2000000  # Higher gas limit
)
```

## Examples

### Image Classification

```python
import base64
from citrate_sdk import CitrateClient, ModelConfig, ModelType

client = CitrateClient(private_key="0x1234...")

# Deploy image classifier
config = ModelConfig(
    name="Image Classifier",
    model_type=ModelType.COREML,
    access_type=AccessType.PAID,
    access_price=100000000000000000  # 0.1 ETH
)

model = client.deploy_model("./classifier.mlpackage", config)

# Classify image
with open("image.jpg", "rb") as f:
    image_data = base64.b64encode(f.read()).decode()

result = client.inference(
    model_id=model.model_id,
    input_data={"image": image_data}
)

print(f"Classification: {result.output_data['label']}")
print(f"Confidence: {result.output_data['confidence']}")
```

### Text Generation

```python
# Deploy text generation model
config = ModelConfig(
    name="Text Generator",
    model_type=ModelType.PYTORCH,
    access_type=AccessType.PUBLIC
)

model = client.deploy_model("./text_gen.pt", config)

# Generate text
result = client.inference(
    model_id=model.model_id,
    input_data={
        "prompt": "The future of AI is",
        "max_tokens": 100,
        "temperature": 0.7
    }
)

print(f"Generated: {result.output_data['text']}")
```

## API Reference

### CitrateClient

#### Methods

- `deploy_model(model_path, config)` - Deploy AI model
- `inference(model_id, input_data, **kwargs)` - Execute inference
- `get_model_info(model_id)` - Get model information
- `list_models(owner=None, limit=100)` - List available models
- `purchase_model_access(model_id, amount)` - Purchase model access

### ModelConfig

#### Parameters

- `name` - Model name
- `description` - Model description
- `model_type` - ModelType enum (COREML, ONNX, etc.)
- `access_type` - AccessType enum (PUBLIC, PRIVATE, PAID)
- `encrypted` - Enable encryption (bool)
- `access_price` - Price per inference in wei (int)

### KeyManager

#### Methods

- `get_address()` - Get Ethereum address
- `encrypt_data(data)` - Encrypt string data
- `decrypt_data(encrypted)` - Decrypt string data
- `derive_shared_key(peer_pubkey)` - ECDH key derivation

## Development

### Testing

```bash
# Install dev dependencies
pip install -e ".[dev]"

# Run tests
pytest tests/

# Run with coverage
pytest --cov=citrate_sdk tests/
```

### Code Formatting

```bash
# Format code
black citrate_sdk/

# Check style
flake8 citrate_sdk/

# Type checking
mypy citrate_sdk/
```

## Support

- **Documentation**: https://docs.lattice.ai
- **GitHub**: https://github.com/lattice-ai/citrate
- **Discord**: https://discord.gg/lattice-ai
- **Issues**: https://github.com/lattice-ai/citrate/issues

## License

Apache License 2.0 - see [LICENSE](LICENSE) file for details.