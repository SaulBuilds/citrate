# Citrate JavaScript/TypeScript SDK

A comprehensive JavaScript/TypeScript SDK for interacting with the Citrate AI blockchain platform. Deploy AI models, execute inferences, manage encryption, and handle payments with ease.

## Features

- **🚀 Full TypeScript Support**: Complete type definitions for all APIs
- **🔐 End-to-End Encryption**: Secure model weights and inference data
- **💰 Built-in Payments**: Pay-per-use pricing and revenue sharing
- **🌐 Web3 Integration**: MetaMask and WalletConnect support
- **⚡ Real-time Streaming**: WebSocket support for live inference
- **⚛️ React Hooks**: Optional React integration for web apps
- **🔧 Multi-format Models**: CoreML, ONNX, TensorFlow, PyTorch support

## Installation

```bash
npm install lattice-js
# or
yarn add lattice-js
```

## Quick Start

### Basic Usage

```typescript
import { CitrateClient, ModelConfig, ModelType, AccessType } from 'lattice-js';

// Connect to Citrate network
const client = new CitrateClient({
  rpcUrl: 'https://mainnet.lattice.ai',
  privateKey: 'your-private-key' // optional
});

// Deploy a model
const modelData = new Uint8Array(/* your model bytes */);
const config: ModelConfig = {
  name: 'My AI Model',
  modelType: ModelType.COREML,
  accessType: AccessType.PAID,
  accessPrice: 100000000000000000n, // 0.1 ETH in wei
  encrypted: true
};

const deployment = await client.deployModel(modelData, config);
console.log('Model deployed:', deployment.modelId);

// Execute inference
const result = await client.inference({
  modelId: deployment.modelId,
  inputData: { text: 'Hello, AI!' }
});

console.log('AI Response:', result.outputData);
```

### React Integration

```tsx
import React from 'react';
import { useCitrateClient, useInference } from 'lattice-js';

function AIChat() {
  const { client, isConnected } = useCitrateClient({
    rpcUrl: 'https://mainnet.lattice.ai',
    autoConnect: true
  });

  const { execute, result, isExecuting } = useInference(client);

  const handleSubmit = async (input: string) => {
    await execute({
      modelId: 'your-model-id',
      inputData: { prompt: input }
    });
  };

  if (!isConnected) return <div>Connecting...</div>;

  return (
    <div>
      <button
        onClick={() => handleSubmit('Hello')}
        disabled={isExecuting}
      >
        {isExecuting ? 'Processing...' : 'Send Message'}
      </button>

      {result && (
        <div>Response: {result.outputData.text}</div>
      )}
    </div>
  );
}
```

### Real-time Streaming

```typescript
import { WebSocketClient } from 'lattice-js';

const wsClient = new WebSocketClient({
  url: 'wss://mainnet.lattice.ai/ws'
});

await wsClient.connect();

// Start streaming inference
await wsClient.startStreamingInference({
  modelId: 'text-generation-model',
  inputData: { prompt: 'Write a story about...' },
  onPartialResult: (partial) => {
    console.log('Partial:', partial.outputData);
  },
  onComplete: (final) => {
    console.log('Complete:', final.outputData);
  }
});
```

## API Reference

### CitrateClient

#### Constructor

```typescript
new CitrateClient(config: CitrateClientConfig)
```

**Parameters:**
- `config.rpcUrl` - RPC endpoint URL
- `config.privateKey` - Optional private key for transactions
- `config.timeout` - Request timeout in milliseconds (default: 30000)
- `config.headers` - Additional HTTP headers

#### Methods

##### deployModel()

```typescript
deployModel(
  modelData: ArrayBuffer | Uint8Array,
  config: ModelConfig
): Promise<ModelDeployment>
```

Deploy an AI model to the blockchain.

##### inference()

```typescript
inference(request: InferenceRequest): Promise<InferenceResult>
```

Execute inference on a deployed model.

##### batchInference()

```typescript
batchInference(request: BatchInferenceRequest): Promise<BatchInferenceResult>
```

Execute batch inference for multiple inputs.

##### getModelInfo()

```typescript
getModelInfo(modelId: string): Promise<ModelInfo>
```

Get detailed information about a model.

##### listModels()

```typescript
listModels(owner?: string, limit?: number): Promise<ModelInfo[]>
```

List available models in the marketplace.

##### purchaseModelAccess()

```typescript
purchaseModelAccess(modelId: string, paymentAmount: bigint): Promise<string>
```

Purchase access to a paid model.

### Type Definitions

#### ModelConfig

```typescript
interface ModelConfig {
  name: string;
  description?: string;
  modelType: ModelType;
  version?: string;
  accessType: AccessType;
  accessPrice: bigint;
  accessList?: string[];
  encrypted: boolean;
  encryptionConfig?: EncryptionConfig;
  metadata?: Record<string, any>;
  tags?: string[];
  maxBatchSize?: number;
  timeoutSeconds?: number;
  memoryLimitMb?: number;
  revenueShares?: Record<string, number>;
}
```

#### InferenceRequest

```typescript
interface InferenceRequest {
  modelId: string;
  inputData: Record<string, any>;
  encrypted?: boolean;
  batchSize?: number;
  timeout?: number;
  timestamp?: number;
}
```

#### InferenceResult

```typescript
interface InferenceResult {
  modelId: string;
  outputData: Record<string, any>;
  gasUsed: bigint;
  executionTime: number;
  txHash: string;
  confidence?: number;
  metadata?: Record<string, any>;
}
```

### Enums

#### ModelType

```typescript
enum ModelType {
  COREML = 'coreml',
  ONNX = 'onnx',
  TENSORFLOW = 'tensorflow',
  PYTORCH = 'pytorch',
  CUSTOM = 'custom'
}
```

#### AccessType

```typescript
enum AccessType {
  PUBLIC = 'public',
  PRIVATE = 'private',
  PAID = 'paid',
  WHITELIST = 'whitelist'
}
```

## Examples

### Image Classification

```typescript
// Deploy image classifier
const imageModel = await client.deployModel(modelBytes, {
  name: 'Image Classifier',
  modelType: ModelType.COREML,
  accessType: AccessType.PAID,
  accessPrice: 50000000000000000n, // 0.05 ETH
  encrypted: true
});

// Classify image
const imageBytes = new Uint8Array(/* image data */);
const result = await client.inference({
  modelId: imageModel.modelId,
  inputData: {
    image: Array.from(imageBytes),
    format: 'jpg'
  }
});

console.log('Classification:', result.outputData.label);
console.log('Confidence:', result.outputData.confidence);
```

### Text Generation

```typescript
// Deploy text generation model
const textModel = await client.deployModel(modelBytes, {
  name: 'GPT Model',
  modelType: ModelType.PYTORCH,
  accessType: AccessType.PUBLIC,
  encrypted: false
});

// Generate text
const result = await client.inference({
  modelId: textModel.modelId,
  inputData: {
    prompt: 'The future of AI is',
    maxTokens: 100,
    temperature: 0.7
  }
});

console.log('Generated text:', result.outputData.text);
```

### Encrypted Private Model

```typescript
import { KeyManager } from 'lattice-js';

const keyManager = new KeyManager();

// Deploy encrypted model
const encryptedModel = await client.deployModel(modelBytes, {
  name: 'Private Medical Model',
  modelType: ModelType.ONNX,
  accessType: AccessType.WHITELIST,
  accessList: ['0x123...', '0x456...'],
  encrypted: true,
  encryptionConfig: {
    algorithm: 'AES-256-GCM',
    keyDerivation: 'HKDF-SHA256',
    accessControl: true,
    thresholdShares: 3,
    totalShares: 5
  }
});

// Execute encrypted inference
const sensitiveResult = await client.inference({
  modelId: encryptedModel.modelId,
  inputData: {
    patientData: { /* sensitive medical data */ }
  },
  encrypted: true
});
```

### Revenue Sharing

```typescript
// Deploy model with revenue sharing
const sharedModel = await client.deployModel(modelBytes, {
  name: 'Collaborative Model',
  modelType: ModelType.TENSORFLOW,
  accessType: AccessType.PAID,
  accessPrice: 200000000000000000n, // 0.2 ETH
  revenueShares: {
    '0xModel-Creator-Address': 0.60,     // 60% to model creator
    '0xData-Provider-Address': 0.30,     // 30% to data provider
    '0xPlatform-Address': 0.10           // 10% to platform
  }
});
```

## React Hooks

### useCitrateClient

```typescript
const {
  client,
  isConnected,
  isConnecting,
  error,
  connect,
  disconnect,
  chainId,
  address,
  balance
} = useCitrateClient({
  rpcUrl: 'https://mainnet.lattice.ai',
  autoConnect: true
});
```

### useModelDeployment

```typescript
const {
  deploy,
  deployment,
  isDeploying,
  error
} = useModelDeployment(client);
```

### useInference

```typescript
const {
  execute,
  result,
  isExecuting,
  error
} = useInference(client);
```

### useModelInfo

```typescript
const {
  modelInfo,
  isLoading,
  error,
  refetch
} = useModelInfo(client, modelId);
```

### useModelList

```typescript
const {
  models,
  isLoading,
  error,
  refetch
} = useModelList(client, ownerAddress, 50);
```

## Error Handling

```typescript
import {
  CitrateError,
  ModelNotFoundError,
  InsufficientFundsError,
  ValidationError
} from 'lattice-js';

try {
  const result = await client.inference({
    modelId: 'invalid-model',
    inputData: { test: 'data' }
  });
} catch (error) {
  if (error instanceof ModelNotFoundError) {
    console.error('Model does not exist');
  } else if (error instanceof InsufficientFundsError) {
    console.error('Not enough funds for inference');
  } else if (error instanceof ValidationError) {
    console.error('Invalid input:', error.message);
  } else if (error instanceof CitrateError) {
    console.error('Citrate error:', error.message, error.code);
  }
}
```

## Configuration

### Network Configuration

```typescript
import { CHAIN_IDS, DEFAULT_RPC_URLS } from 'lattice-js';

const client = new CitrateClient({
  rpcUrl: DEFAULT_RPC_URLS[CHAIN_IDS.MAINNET],
  // or for testnet:
  // rpcUrl: DEFAULT_RPC_URLS[CHAIN_IDS.TESTNET],
  privateKey: process.env.PRIVATE_KEY
});
```

### Custom Timeouts

```typescript
const client = new CitrateClient({
  rpcUrl: 'https://mainnet.lattice.ai',
  timeout: 60000, // 60 seconds
  retries: 3
});
```

## Development

### Building

```bash
npm run build
```

### Testing

```bash
npm test
```

### Linting

```bash
npm run lint
```

### Formatting

```bash
npm run format
```

## Browser Support

The SDK works in all modern browsers and supports:

- **ES2020+** JavaScript environments
- **WebAssembly** for cryptographic operations
- **WebSockets** for real-time features
- **Web3** wallet integration

## Node.js Support

Requires Node.js 16.0.0 or higher.

## Contributing

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## License

Apache License 2.0 - see [LICENSE](LICENSE) file for details.

## Support

- **Documentation**: https://docs.lattice.ai
- **GitHub**: https://github.com/lattice-ai/citrate
- **Discord**: https://discord.gg/lattice-ai
- **Issues**: https://github.com/lattice-ai/citrate/issues