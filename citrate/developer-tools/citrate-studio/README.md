# Citrate Studio - Visual IDE for AI Blockchain Development

Citrate Studio is a comprehensive, web-based integrated development environment (IDE) specifically designed for developing, deploying, and managing AI models on the Citrate blockchain.

## Features

### 🎨 **Visual Interface**
- Modern, dark-themed UI optimized for blockchain development
- Real-time connection status monitoring
- Intuitive navigation with dedicated workspaces

### 💻 **Code Editor**
- Monaco Editor (VS Code engine) with syntax highlighting
- Multi-language support (Python, JavaScript, Solidity)
- Built-in templates for common Citrate development patterns
- Real-time code execution and testing
- Integrated terminal output

### 🚀 **Model Management**
- Deploy AI models directly to Citrate blockchain
- Model registry with version control
- Real-time inference testing
- Revenue and usage analytics
- Model encryption and access control

### 🌐 **Network Monitoring**
- Live blockchain statistics and metrics
- Network topology visualization
- Peer connection monitoring
- Block and transaction inspection

### 🔧 **Debug Tools**
- Transaction debugger with trace analysis
- Smart contract inspection
- Gas optimization suggestions
- Error tracking and logging

### 📊 **Dashboard**
- Real-time network statistics
- Model performance metrics
- Recent activity tracking
- Quick action shortcuts

## Quick Start

### Prerequisites
- Node.js 16+
- Citrate node running on localhost:8545
- Modern web browser

### Installation

```bash
# Navigate to Citrate Studio directory
cd developer-tools/lattice-studio

# Install dependencies
npm install

# Start development server
npm start
```

The IDE will be available at `http://localhost:3000`

### Production Build

```bash
# Build for production
npm run build

# Serve production build
npm run serve
```

## Usage

### 1. **Getting Started**
- Ensure your Citrate node is running
- Open Citrate Studio in your browser
- Check connection status in the top bar
- Navigate through different workspaces using the sidebar

### 2. **Developing Models**
- Go to **Code Editor** workspace
- Use provided templates or create new files
- Write your AI model code in Python
- Test your code using the integrated runner

### 3. **Deploying Models**
- Navigate to **Model Manager**
- Click "Deploy Model"
- Fill in model metadata and pricing
- Upload your model file
- Deploy to the blockchain

### 4. **Running Inference**
- Select a deployed model
- Click "Run Inference"
- Provide input data in JSON format
- View results and execution metrics

### 5. **Monitoring Network**
- Use **Network** workspace to view topology
- Monitor peer connections and status
- Track blockchain activity in real-time

### 6. **Debugging**
- Use **Debugger** workspace for transaction analysis
- Enter transaction hash to inspect details
- View execution traces and gas usage

## File Templates

Citrate Studio comes with built-in templates for common development patterns:

### **model.py** - AI Model Template
```python
class CitrateModel:
    def predict(self, input_data):
        # Your model logic here
        return {"prediction": "result"}
```

### **deploy.js** - Deployment Script
```javascript
const { CitrateClient } = require('citrate-js');
// Automated deployment script
```

### **contract.sol** - Smart Contract
```solidity
contract CitrateModelContract {
    // Model access control and payments
}
```

### **test.js** - Testing Suite
```javascript
// Comprehensive testing framework
```

## Configuration

### Environment Variables
Create a `.env` file in the root directory:

```env
REACT_APP_CITRATE_RPC=http://localhost:8545
REACT_APP_IPFS_URL=http://localhost:5001
```

### Settings
Customize your development environment:
- **Network Settings**: Configure RPC endpoints
- **Auto Refresh**: Set update intervals
- **Debug Mode**: Enable detailed logging
- **Notifications**: Control alert preferences

## Architecture

### Component Structure
```
src/
├── components/          # React components
│   ├── Dashboard.js     # Main dashboard
│   ├── CodeEditor.js    # Monaco editor integration
│   ├── ModelManager.js  # Model deployment & management
│   ├── NetworkViewer.js # Network topology
│   ├── Debugger.js      # Transaction debugging
│   ├── Deployment.js    # Deployment workflows
│   └── Settings.js      # Configuration
├── services/
│   └── CitrateService.js # Blockchain interaction layer
└── App.js               # Main application
```

### Service Layer
The `CitrateService` class provides a unified interface for:
- RPC calls to Citrate node
- Model deployment and inference
- Network topology queries
- Transaction debugging
- Real-time updates via WebSocket

## Integration

### Citrate SDK
Citrate Studio integrates seamlessly with the Citrate JavaScript SDK:

```javascript
import { CitrateClient } from 'citrate-js';

const client = new CitrateClient('http://localhost:8545');
const result = await client.deployModel(modelData);
```

### External Tools
- **IPFS**: For decentralized model storage
- **Web3**: For blockchain interactions
- **Monaco Editor**: For code editing
- **Material-UI**: For UI components

## Development

### Adding New Features
1. Create component in `src/components/`
2. Add route in `App.js`
3. Update navigation in sidebar
4. Add service methods in `CitrateService.js`

### Extending Templates
Add new file templates to `FILE_TEMPLATES` object in `CodeEditor.js`

### Custom Themes
Modify theme configuration in `src/index.js`

## Production Deployment

### Docker
```dockerfile
FROM node:18-alpine
WORKDIR /app
COPY package*.json ./
RUN npm ci --only=production
COPY build ./build
EXPOSE 3001
CMD ["npm", "run", "serve"]
```

### Environment Setup
- Configure reverse proxy (nginx)
- Set up SSL certificates
- Configure CORS for Citrate node
- Set up monitoring and logging

## Troubleshooting

### Common Issues

**Connection Failed**
- Verify Citrate node is running
- Check RPC URL in settings
- Ensure CORS is enabled on node

**Model Deployment Fails**
- Check account has sufficient balance
- Verify model file format
- Check gas settings

**Code Editor Not Loading**
- Clear browser cache
- Check browser console for errors
- Verify Monaco editor resources

### Debug Mode
Enable debug mode in settings to see:
- Detailed RPC call logs
- Component render information
- Error stack traces

## Contributing

1. Fork the repository
2. Create feature branch
3. Implement changes with tests
4. Submit pull request

## License

MIT License - see LICENSE file for details

## Support

- Documentation: [Citrate Docs](https://docs.citrate.ai)
- Discord: [Citrate Community](https://discord.gg/lattice)
- GitHub Issues: [Report Bugs](https://github.com/lattice/issues)

---

Built with ❤️ for the Citrate AI blockchain ecosystem.