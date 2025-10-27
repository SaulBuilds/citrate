# Citrate AI Blockchain - VS Code Extension

The official VS Code extension for Citrate AI blockchain development. Provides comprehensive tooling for developing, deploying, and managing AI models on the Citrate blockchain.

## Features

### 🔗 **Blockchain Integration**
- Direct connection to Citrate nodes
- Real-time network status monitoring
- Account and balance management
- Transaction tracking and debugging

### 🤖 **AI Model Development**
- Specialized syntax highlighting for Citrate models
- Code snippets for common patterns
- Model validation and testing tools
- Integrated deployment workflows

### 📁 **Project Management**
- Project templates for different AI model types
- Automated project scaffolding
- Dependency management
- Configuration validation

### 🚀 **Deployment & Testing**
- One-click model deployment
- Inference testing within VS Code
- Gas optimization suggestions
- Smart contract interaction

### 🔍 **Debugging & Monitoring**
- Transaction debugger with trace analysis
- Model performance analytics
- Network topology visualization
- Error tracking and diagnostics

## Installation

### From VS Code Marketplace
1. Open VS Code
2. Go to Extensions (Ctrl+Shift+X)
3. Search for "Citrate AI Blockchain"
4. Click Install

### Manual Installation
1. Download the `.vsix` file from releases
2. Open VS Code
3. Run `Extensions: Install from VSIX...` command
4. Select the downloaded file

## Quick Start

### 1. Setup
1. Install the extension
2. Ensure Citrate node is running on `localhost:8545`
3. Open Command Palette (Ctrl+Shift+P)
4. Run `Lattice: Connect to Citrate Node`

### 2. Create Project
1. Run `Lattice: Create Citrate Project`
2. Choose a template (Image Classification, NLP, etc.)
3. Select project location
4. Extension will scaffold the project

### 3. Develop Model
```python
# Use snippets: type 'lattice-model' and press Tab
class CitrateModel:
    def predict(self, input_data):
        # Your AI model logic here
        return {"prediction": "result"}
```

### 4. Deploy Model
1. Right-click on your Python file
2. Select `Lattice: Deploy Model`
3. Configure deployment options
4. Deploy to blockchain

### 5. Test Inference
1. Run `Lattice: Run Inference`
2. Select your deployed model
3. Provide test input
4. View results in VS Code

## Commands

| Command | Description | Shortcut |
|---------|-------------|----------|
| `Lattice: Connect to Citrate Node` | Connect to blockchain | - |
| `Lattice: Deploy Model` | Deploy AI model | - |
| `Lattice: Run Inference` | Test model inference | - |
| `Lattice: Create Citrate Project` | Create new project | - |
| `Lattice: View Models` | Show model explorer | - |
| `Lattice: Run Citrate Tests` | Execute test suite | - |

## Code Snippets

### Python Snippets
- `lattice-model` - Complete model class template
- `lattice-deploy` - Deployment script
- `lattice-inference` - Inference script
- `lattice-test` - Test case template

### JavaScript Snippets
- `citrate-client` - Client initialization
- `lattice-contract` - Smart contract interaction
- `lattice-deploy-js` - JavaScript deployment

### Solidity Snippets
- `lattice-contract` - Model contract template
- `lattice-payment` - Payment handling
- `lattice-access` - Access control

## Extension Settings

Configure the extension through VS Code settings:

```json
{
    "lattice.rpcUrl": "http://localhost:8545",
    "lattice.autoConnect": true,
    "lattice.defaultGasLimit": 1000000,
    "lattice.enableCodeLens": true,
    "lattice.showNotifications": true
}
```

### Settings Details

| Setting | Description | Default |
|---------|-------------|---------|
| `lattice.rpcUrl` | Citrate node RPC URL | `http://localhost:8545` |
| `lattice.autoConnect` | Auto-connect on startup | `true` |
| `lattice.defaultGasLimit` | Default gas limit | `1000000` |
| `lattice.enableCodeLens` | Show CodeLens actions | `true` |
| `lattice.showNotifications` | Show status notifications | `true` |

## Project Templates

### Image Classification
```
my-image-classifier/
├── model.py              # Main model class
├── train.py             # Training script
├── deploy.py            # Deployment script
├── test_model.py        # Test cases
├── requirements.txt     # Dependencies
└── lattice.config.json  # Project config
```

### NLP Model
```
my-nlp-model/
├── model.py              # NLP model implementation
├── preprocessing.py      # Data preprocessing
├── deploy.py            # Deployment script
├── test_inference.py    # Inference tests
├── requirements.txt     # Dependencies
└── lattice.config.json  # Project config
```

### Custom Model
```
my-custom-model/
├── model.py              # Custom model
├── utils.py             # Utility functions
├── deploy.py            # Deployment script
├── tests/               # Test directory
│   └── test_model.py    # Model tests
├── requirements.txt     # Dependencies
└── lattice.config.json  # Project config
```

## Debugging

### Transaction Debugging
1. Open Command Palette
2. Run `Lattice: Debug Transaction`
3. Enter transaction hash
4. View detailed trace and gas analysis

### Model Debugging
- Set breakpoints in your model code
- Use `Debug: Start Debugging` with Citrate configuration
- Extension provides specialized debugging for blockchain deployment

### Common Issues

**Connection Failed**
```
Error: Failed to connect to Citrate node
```
- Check if node is running on correct port
- Verify RPC URL in settings
- Ensure CORS is enabled on node

**Deployment Failed**
```
Error: Insufficient funds for deployment
```
- Check account balance
- Verify gas settings
- Ensure account is unlocked

**Model Not Found**
```
Error: Model not found on blockchain
```
- Verify model ID is correct
- Check if model deployment completed
- Refresh model list

## API Reference

### CitrateClient

```typescript
class CitrateClient {
    // Connection management
    async connect(): Promise<void>
    disconnect(): void
    get isConnected(): boolean

    // Model operations
    async deployModel(modelData: Buffer, config: DeploymentConfig): Promise<DeploymentResult>
    async runInference(modelId: string, inputData: any): Promise<InferenceResult>
    async getModels(): Promise<LatticeModel[]>

    // Network operations
    async getNetworkStatus(): Promise<NetworkStatus>
    async getBlock(blockNumber?: string | number): Promise<any>
    async getTransaction(txHash: string): Promise<any>
}
```

### Deployment Configuration

```typescript
interface DeploymentConfig {
    encrypted?: boolean;        // Encrypt model data
    price?: string;            // Price in wei
    metadata?: {
        name: string;          // Model name
        description: string;   // Description
        version: string;       // Version
        tags?: string[];       // Category tags
    };
}
```

## Contributing

### Development Setup
1. Clone the repository
2. Install dependencies: `npm install`
3. Open in VS Code
4. Press F5 to launch Extension Development Host

### Building
```bash
npm run compile          # Compile TypeScript
npm run watch           # Watch for changes
npm run package         # Package for distribution
```

### Testing
```bash
npm run test           # Run all tests
npm run lint          # Run linter
```

## License

MIT License - see LICENSE file for details.

## Support

- 📚 [Documentation](https://docs.citrate.ai/vscode)
- 💬 [Discord Community](https://discord.gg/lattice)
- 🐛 [Report Issues](https://github.com/lattice/vscode-extension/issues)
- 📧 [Email Support](mailto:support@citrate.ai)

## Changelog

### v1.0.0
- Initial release
- Citrate node integration
- Model deployment and inference
- Project templates
- Debugging tools
- Code snippets and syntax highlighting

---

Built with ❤️ for the Citrate AI blockchain developer community.