# Citrate JavaScript/TypeScript SDK

The official JavaScript/TypeScript SDK for the Citrate AI blockchain platform. This SDK provides a comprehensive interface for interacting with Citrate nodes, deploying AI models, managing accounts, and executing transactions.

## Installation

```bash
npm install @citrate-ai/sdk
```

## Quick Start

```typescript
import { CitrateSDK } from '@citrate-ai/sdk';

// Initialize the SDK (JSON-RPC endpoint)
const sdk = new CitrateSDK({
  rpcEndpoint: 'http://localhost:8545',
  chainId: 1337,
});

// Import an account (enables raw-tx signing)
const address = sdk.accounts.importAccount('<PRIVATE_KEY_HEX>');

// Get balance
const balance = await sdk.accounts.getBalance(address);
console.log(`Balance (wei): ${balance}`);

// List models and run inference
const ids: string[] = await sdk.models.listModels();
if (ids.length) {
  const info = await sdk.models.getModel(ids[0]);
  console.log('Model:', info);
  const result = await sdk.models.runInference(ids[0], { text: 'hello lattice' });
  console.log('Inference:', result);
}
```

## Features

### 🔗 Node Connectivity
- Connect to Citrate nodes via RPC/WebSocket
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

### CitrateSDK

Main SDK class that provides access to all functionality.

```typescript
const sdk = new CitrateSDK({
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
const sdk = new CitrateSDK({
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
CITRATE_NODE_URL=http://localhost:8545
CITRATE_CHAIN_ID=1337
CITRATE_TIMEOUT=30000
CITRATE_RETRIES=3
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
try {
  const ids = await sdk.models.listModels();
  if (!ids.length) throw new Error('No models registered');
  const result = await sdk.models.runInference(ids[0], { text: 'hello' });
  console.log(result.output);
} catch (e) {
  console.error('SDK/RPC error:', e);
}
```

## Signing Best Practices

- Prefer client-side signing (raw tx) for public/testnet RPCs. Import a private key or mnemonic via `sdk.accounts.importAccount(...)`.
- In local development, the node can accept `eth_sendTransaction` without a valid signature only if started with `CITRATE_REQUIRE_VALID_SIGNATURE=false`.

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
- 🐛 [Issue Tracker](https://github.com/lattice-ai/citrate/issues)
- 📧 [Email Support](mailto:support@lattice.ai)

## Changelog

See [CHANGELOG.md](./CHANGELOG.md) for version history and updates.
