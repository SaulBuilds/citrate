# Citrate v1.0.0 Release Notes

**Release Date:** December 2025

We're excited to announce the first stable release of Citrate - an AI-native Layer-1 BlockDAG blockchain. This release marks a major milestone in bringing together blockchain technology and AI inference in a seamless, production-ready platform.

## Highlights

### AI-Native Blockchain
- **On-chain AI models** as first-class assets with verifiable provenance
- **Native inference** via llama.cpp with Metal GPU acceleration
- **Model marketplace** for discovering, deploying, and monetizing AI models
- **IPFS-backed storage** for efficient model weight distribution

### High-Performance Consensus
- **GhostDAG consensus** enabling 10,000+ TPS
- **Sub-12 second finality** with BFT committee checkpoints
- **Parallel block processing** supporting 100+ concurrent blocks
- **Optimistic confirmation** in ~2.5 blocks

### EVM Compatibility
- Full **Solidity/EVM support** - deploy existing contracts unchanged
- **EIP-1559** and **EIP-2930** transaction types
- Standard JSON-RPC interface compatible with MetaMask and web3 tools
- AI-specific precompiles for on-chain inference

### Cross-Platform GUI
- **Tauri-based desktop application** for macOS, Windows, and Linux
- **AI onboarding assistant** with personalized skill-based guidance
- **Integrated wallet** with transaction history and model marketplace
- **DAG visualization** for exploring block structure

## What's New in v1.0

### Core Infrastructure
- Production-ready node with validator enforcement
- Mainnet and testnet configuration templates
- External RPC access via ngrok with full documentation
- Comprehensive model download script with IPFS gateway fallback

### AI Features
- **Qwen2 0.5B** model for fast inference (~400MB)
- **Mistral 7B Instruct** for complex reasoning tasks (~4.1GB)
- **BGE-M3** embeddings for semantic search (embedded)
- Multi-gateway IPFS model distribution

### Onboarding Experience
- Skill assessment to personalize user experience
- Three learning paths: Beginner, Intermediate, Advanced
- Step-by-step guidance through wallet setup and first transaction
- Context-aware AI assistant

### Developer Tools
- JavaScript SDK (`@citrate-ai/sdk`)
- Python SDK (`citrate-sdk`)
- Foundry integration for smart contracts
- Comprehensive API documentation

## Installation

### Pre-built Binaries

| Platform | Download |
|----------|----------|
| macOS (Apple Silicon) | `Citrate_arm64.dmg` |
| macOS (Intel) | `Citrate_x64.dmg` |
| Windows | `Citrate_Setup.exe` |
| Linux (AppImage) | `Citrate.AppImage` |
| Linux (Debian) | `citrate_1.0.0_amd64.deb` |

### From Source

```bash
git clone https://github.com/citrate-ai/citrate.git
cd citrate/citrate
cargo build --release
cd gui/citrate-core && npm run tauri build
```

### AI Models

```bash
# Download required models (~4.5GB)
./scripts/download-models.sh
```

## Model CIDs

| Model | IPFS CID | Size |
|-------|----------|------|
| Mistral 7B Instruct v0.3 | `QmUsYyxg71bV8USRQ6Ccm3SdMqeWgEEVnCYkgNDaxvBTZB` | 4.1 GB |
| Qwen2 0.5B Instruct Q4 | `QmZj4ZaG9v6nXKnT5yqwi8YaH5bm48zooNdh9ff4CHGTY4` | 379 MB |
| BGE-M3 Q4 | Embedded in genesis | 417 MB |

## Network Configuration

| Network | Chain ID | RPC URL | Status |
|---------|----------|---------|--------|
| Devnet | 1337 | `http://localhost:8545` | Local |
| Testnet | 42069 | `https://testnet.citrate.ai` | Public |
| Mainnet | 1 | `https://api.citrate.ai` | Coming Soon |

## Breaking Changes

This is the initial stable release. Future versions will maintain backwards compatibility.

## Known Issues

1. **Large model downloads** may timeout on slow connections - use the download script which supports resume
2. **Windows ARM** not yet supported - use x86_64 emulation
3. **Linux ARM** packages pending - build from source for now

## Upgrade Path

For users running pre-release versions:

```bash
# Backup your wallet
cp -r ~/.citrate/wallet ~/.citrate-wallet-backup

# Clean old data (optional)
rm -rf ~/.citrate-devnet

# Install new version
./scripts/download-models.sh  # Re-download models if needed
```

## Documentation

- **Quick Start**: `docs/QUICK_START.md`
- **External RPC Access**: `docs/guides/external-rpc-access.md`
- **AI Inference Setup**: `docs/guides/ai-inference-setup.md`
- **Smart Contracts**: `docs/guides/deployment.md`
- **Full Documentation**: [docs.citrate.ai](https://docs.citrate.ai)

## Contributing

We welcome contributions! See [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

### Key Areas for Contribution
- AI model integrations
- SDK improvements
- Documentation and tutorials
- Bug fixes and optimizations

## Security

Report security vulnerabilities to: security@citrate.ai

## Acknowledgments

Thanks to all contributors, testers, and community members who made this release possible:

- The GhostDAG team for consensus research
- Hugging Face for model hosting
- IPFS community for distributed storage
- llama.cpp team for inference runtime
- All early adopters and beta testers

## What's Next

### v1.1 (Q1 2025)
- WebGPU inference support
- Mobile SDK (iOS/Android)
- Enhanced model marketplace UI

### v1.2 (Q2 2025)
- On-chain model fine-tuning
- Cross-chain bridges
- Advanced ZKP integration

## Links

- **Website**: [citrate.ai](https://citrate.ai)
- **GitHub**: [github.com/citrate-ai/citrate](https://github.com/citrate-ai/citrate)
- **Documentation**: [docs.citrate.ai](https://docs.citrate.ai)
- **Discord**: [discord.gg/citrate](https://discord.gg/citrate)
- **Twitter**: [@citrate_ai](https://twitter.com/citrate_ai)

---

*Thank you for being part of the Citrate journey! Together, we're building the future of AI-native blockchain.*
