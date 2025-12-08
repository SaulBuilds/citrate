# Citrate Quick Start Guide

Get started with Citrate in under 10 minutes!

## Prerequisites

- **macOS**: 10.15 or later (Intel or Apple Silicon)
- **Windows**: Windows 10/11 (64-bit)
- **Linux**: Ubuntu 20.04+ or compatible distro

## Installation

### Option 1: Download Pre-built Binaries (Recommended)

Download the latest release for your platform:

| Platform | Download |
|----------|----------|
| macOS (Intel) | [Citrate-x64.dmg](https://github.com/citrate-ai/citrate/releases/latest) |
| macOS (Apple Silicon) | [Citrate-arm64.dmg](https://github.com/citrate-ai/citrate/releases/latest) |
| Windows | [Citrate-Setup.exe](https://github.com/citrate-ai/citrate/releases/latest) |
| Linux (AppImage) | [Citrate.AppImage](https://github.com/citrate-ai/citrate/releases/latest) |
| Linux (Debian) | [citrate.deb](https://github.com/citrate-ai/citrate/releases/latest) |

### Option 2: Build from Source

```bash
# Clone the repository
git clone https://github.com/citrate-ai/citrate.git
cd citrate/citrate

# Install Rust (if not installed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Build the node
cargo build --release

# Build the GUI (requires Node.js 18+)
cd gui/citrate-core
npm install
npm run tauri build
```

## First Run

### 1. Launch the Application

**macOS**: Open `Citrate.app` from Applications
**Windows**: Run `Citrate` from Start Menu
**Linux**: Execute `./Citrate.AppImage` or run `citrate` if installed via .deb

### 2. Welcome & Onboarding

When you first launch Citrate, the AI assistant will guide you through:

1. **Skill Assessment** - Quick questions to personalize your experience
2. **Wallet Setup** - Create or import a wallet
3. **Network Connection** - Connect to testnet
4. **First Transaction** - Get test tokens and send your first transaction

### 3. Download AI Models (First Time Only)

For full AI functionality, run the model download script:

```bash
# Download required AI models (~4.5GB total)
./scripts/download-models.sh
```

This downloads:
- **Mistral 7B Instruct** (~4.1GB) - General purpose model
- **Qwen2 0.5B** (~400MB) - Fast inference model

Models are stored in `~/.citrate/models/` by default.

## Key Features

### Wallet

- Create new wallets with secure key generation
- Import existing wallets (private key or mnemonic)
- View balances and transaction history
- Send CTR tokens

### AI Chat

- Ask questions about blockchain and Citrate
- Get help with transactions and contracts
- Run AI model inference
- Natural language interface for all features

### DAG Explorer

- Visualize the block DAG structure
- Explore blocks and transactions
- Understand GhostDAG consensus

### Model Marketplace

- Browse deployed AI models
- Run inference on any listed model
- Deploy your own models

### Smart Contracts

- Deploy Solidity contracts (EVM-compatible)
- Interact with deployed contracts
- View contract state and events

## Command Line Interface

For advanced users, the CLI provides full node control:

```bash
# Start a devnet node
citrate-node devnet

# Start with custom config
citrate-node --config testnet.toml

# Run the wallet CLI
citrate-wallet --rpc-url http://localhost:8545
```

## Configuration

### Network Settings

| Network | RPC URL | Chain ID |
|---------|---------|----------|
| Devnet | `http://localhost:8545` | 1337 |
| Testnet | `https://testnet.citrate.ai` | 42069 |
| Mainnet | `https://api.citrate.ai` | 1 |

### Config Files

- **Node**: `~/.citrate/config.toml`
- **Wallet**: `~/.citrate/wallet.json`
- **Models**: `~/.citrate/models/`

## Getting Help

### In-App

Ask the AI assistant! Just type your question in the chat:

- "How do I send tokens?"
- "What's my wallet balance?"
- "Help me deploy a contract"
- "Explain how the DAG works"

### Documentation

- [Full Documentation](https://docs.citrate.ai)
- [API Reference](https://docs.citrate.ai/api)
- [SDK Guide](https://docs.citrate.ai/sdk)

### Community

- [GitHub Issues](https://github.com/citrate-ai/citrate/issues)
- [Discord](https://discord.gg/citrate)
- [Twitter](https://twitter.com/citrate_ai)

## Troubleshooting

### Node Won't Start

```bash
# Check if port is in use
lsof -i :8545

# Kill existing process
pkill citrate-node

# Start fresh
rm -rf ~/.citrate-devnet
citrate-node devnet
```

### AI Models Not Loading

```bash
# Verify models are downloaded
ls -la ~/.citrate/models/

# Re-download if needed
./scripts/download-models.sh
```

### Connection Issues

1. Check your internet connection
2. Verify RPC endpoint is reachable
3. Try a different network (testnet vs devnet)

## Next Steps

Now that you're set up, try:

1. **Get Test Tokens**: Visit the [Faucet](https://faucet.citrate.ai)
2. **Deploy a Contract**: Try deploying a simple token contract
3. **Run AI Inference**: Browse the marketplace and run a model
4. **Explore the SDK**: Build your own dApp with our [JavaScript SDK](https://docs.citrate.ai/sdk/javascript)

---

*Welcome to Citrate - Where AI meets blockchain!*
