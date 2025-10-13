# Lattice JavaScript/TypeScript SDK

The official JavaScript/TypeScript SDK for the Lattice AI blockchain platform. This SDK provides a comprehensive interface for interacting with Lattice nodes, deploying AI models, managing accounts, and executing transactions.

## Installation

```bash
npm install @lattice-ai/sdk
```

## Quick Start

```typescript
import { LatticeSDK } from '@lattice-ai/sdk';

// Initialize the SDK
const sdk = new LatticeSDK({
  nodeUrl: 'http://localhost:8545',
  chainId: 1337,
});

// Connect to a wallet
await sdk.account.connectWallet();

// Get account balance
const balance = await sdk.account.getBalance();
console.log(`Balance: ${balance} LAT`);

// Deploy a model
const modelId = await sdk.models.deploy({
  name: 'My AI Model',
  description: 'A simple classification model',
  ipfsHash: 'QmYourModelHash',
  framework: 'pytorch',
  version: '1.0.0',
});

console.log(`Model deployed with ID: ${modelId}`);
```

## Features

### 🔗 Node Connectivity
- Connect to Lattice nodes via RPC/WebSocket
- Real-time event listening
- Automatic reconnection handling

### 🤖 AI Model Management
- Deploy models to the blockchain
- Execute model inference
- Model versioning and metadata
- IPFS integration for model storage

### 💰 Account Management
- Wallet integration (MetaMask, WalletConnect)
- Account creation and import
- Balance queries
- Transaction signing

### 🔄 Transaction Management
- Send transactions
- Smart contract interaction
- Gas estimation
- Transaction status tracking

### 📊 DAG Explorer
- Query block DAG structure
- Get block information
- Explore transaction history
- Real-time chain updates

## API Reference

### LatticeSDK

Main SDK class that provides access to all functionality.

```typescript
const sdk = new LatticeSDK({
  nodeUrl: string,
  chainId?: number,
  timeout?: number,
  retries?: number,
});
```

### Models

Model deployment and management.

```typescript
// Deploy a new model
await sdk.models.deploy({
  name: string,
  description: string,
  ipfsHash: string,
  framework: 'pytorch' | 'tensorflow' | 'onnx',
  version: string,
  accessType: 'public' | 'private',
  price?: string,
});

// Execute model inference
const result = await sdk.models.execute(modelId, {
  inputs: any[],
  outputFormat: 'json' | 'binary',
});

// Get model information
const modelInfo = await sdk.models.getInfo(modelId);
```

### Accounts

Account and wallet management.

```typescript
// Connect wallet
await sdk.account.connectWallet();

// Create new account
const account = await sdk.account.create();

// Get balance
const balance = await sdk.account.getBalance(address?);

// Send transaction
const txHash = await sdk.account.sendTransaction({
  to: string,
  value: string,
  data?: string,
  gasLimit?: number,
});
```

### Contracts

Smart contract interaction.

```typescript
// Deploy contract
const contractAddress = await sdk.contracts.deploy({
  bytecode: string,
  abi: any[],
  constructorArgs?: any[],
});

// Call contract method
const result = await sdk.contracts.call({
  address: string,
  abi: any[],
  method: string,
  args: any[],
});

// Send contract transaction
const txHash = await sdk.contracts.send({
  address: string,
  abi: any[],
  method: string,
  args: any[],
  value?: string,
});
```

## Advanced Usage

### Event Listening

```typescript
// Listen for new blocks
sdk.on('block', (block) => {
  console.log(`New block: ${block.hash}`);
});

// Listen for model deployments
sdk.on('modelDeployed', (event) => {
  console.log(`Model deployed: ${event.modelId}`);
});

// Listen for transactions
sdk.on('transaction', (tx) => {
  console.log(`Transaction: ${tx.hash}`);
});
```

### Custom Providers

```typescript
import { ethers } from 'ethers';

// Use custom provider
const provider = new ethers.providers.WebSocketProvider('ws://localhost:8546');
const sdk = new LatticeSDK({
  provider,
  chainId: 1337,
});
```

### Batch Operations

```typescript
// Batch multiple model executions
const results = await sdk.models.batchExecute([
  { modelId: 'model1', inputs: [1, 2, 3] },
  { modelId: 'model2', inputs: [4, 5, 6] },
]);
```

## Configuration

### Environment Variables

```bash
LATTICE_NODE_URL=http://localhost:8545
LATTICE_CHAIN_ID=1337
LATTICE_TIMEOUT=30000
LATTICE_RETRIES=3
```

### TypeScript Configuration

```json
{
  "compilerOptions": {
    "target": "ES2018",
    "module": "ESNext",
    "moduleResolution": "node",
    "strict": true,
    "esModuleInterop": true,
    "skipLibCheck": true,
    "declaration": true
  }
}
```

## Error Handling

```typescript
import { LatticeError, ModelNotFoundError, InsufficientFundsError } from '@lattice-ai/sdk';

try {
  await sdk.models.execute('invalid-model-id', { inputs: [] });
} catch (error) {
  if (error instanceof ModelNotFoundError) {
    console.log('Model not found');
  } else if (error instanceof InsufficientFundsError) {
    console.log('Insufficient funds for execution');
  } else if (error instanceof LatticeError) {
    console.log('Lattice SDK error:', error.message);
  } else {
    console.log('Unknown error:', error);
  }
}
```

## Testing

```bash
npm test              # Run all tests
npm run test:unit     # Run unit tests
npm run test:integration  # Run integration tests
npm run test:coverage # Run with coverage
```

## Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests for new functionality
5. Run the test suite
6. Submit a pull request

## License

Apache-2.0 License. See [LICENSE](../../LICENSE) for details.

## Support

- 📖 [Documentation](https://docs.lattice.ai)
- 💬 [Discord Community](https://discord.gg/lattice-ai)
- 🐛 [Issue Tracker](https://github.com/lattice-ai/lattice-v3/issues)
- 📧 [Email Support](mailto:support@lattice.ai)

## Changelog

See [CHANGELOG.md](./CHANGELOG.md) for version history and updates.