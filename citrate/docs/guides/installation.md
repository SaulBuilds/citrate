# Citrate V3 Node Installation Guide

## Table of Contents
1. [System Requirements](#system-requirements)
2. [Quick Start](#quick-start)
3. [Building from Source](#building-from-source)
4. [Running a Node](#running-a-node)
5. [Joining the Testnet](#joining-the-testnet)
6. [Configuration](#configuration)
7. [Monitoring](#monitoring)
8. [Troubleshooting](#troubleshooting)

---

## System Requirements

### Minimum Requirements
- **CPU**: 4 cores @ 2.0GHz
- **RAM**: 8GB
- **Storage**: 100GB SSD
- **Network**: 100 Mbps internet connection
- **OS**: Ubuntu 20.04+, macOS 12+, or Windows 10+ with WSL2

### Recommended Requirements
- **CPU**: 8 cores @ 3.0GHz
- **RAM**: 16GB
- **Storage**: 500GB NVMe SSD
- **Network**: 1 Gbps internet connection

### GPU Requirements (for AI compute - Phase 2)
- **NVIDIA GPU**: 8GB+ VRAM (RTX 3070 or better)
- **CUDA**: Version 11.8 or higher
- **Driver**: 515.0 or higher

---

## Quick Start

### 1. Download Pre-built Binary (Coming Soon)
```bash
# Linux/macOS
curl -L https://github.com/lattice/releases/latest/download/lattice-linux -o lattice
chmod +x lattice

# Verify installation
./citrate-node --version
```

### 2. Run a Node
```bash
# Initialize the chain
./citrate-node init --chain-id 1337

# Start node with mining
./citrate-node --mine --rpc-addr 0.0.0.0:8545
```

---

## Building from Source

### Prerequisites

#### Install Rust
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
rustup default stable
rustup update
```

#### Install Build Dependencies

**Ubuntu/Debian:**
```bash
sudo apt update
sudo apt install -y build-essential pkg-config libssl-dev git clang
```

**macOS:**
```bash
# Install Xcode Command Line Tools
xcode-select --install

# Install Homebrew if not already installed
/bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"

# Install dependencies
brew install pkg-config openssl git
```

**Windows (WSL2):**
```bash
# Inside WSL2 Ubuntu
sudo apt update
sudo apt install -y build-essential pkg-config libssl-dev git clang
```

### Clone and Build

```bash
# Clone the repository
git clone https://github.com/lattice/citrate.git
cd citrate

# Build in release mode
cargo build --release

# Binary will be at: target/release/lattice
```

---

## Running a Node

### Single Node (Development)

```bash
# Quick devnet mode
./target/release/lattice devnet

# Custom configuration
./target/release/lattice \
    --data-dir ~/.citrate-data \
    --chain-id 1337 \
    --mine \
    --coinbase 0xYOUR_ADDRESS \
    --rpc-addr 0.0.0.0:8545
```

### Bootstrap Node

```bash
./target/release/lattice \
    --bootstrap \
    --p2p-addr 0.0.0.0:30303 \
    --rpc-addr 0.0.0.0:8545 \
    --data-dir ~/.citrate-bootstrap \
    --mine
```

### Worker Node

```bash
./target/release/lattice \
    --bootstrap-nodes "bootstrap.citrate.network:30303" \
    --p2p-addr 0.0.0.0:30303 \
    --rpc-addr 127.0.0.1:8545 \
    --data-dir ~/.citrate-node \
    --mine \
    --coinbase 0xYOUR_ADDRESS
```

---

## Joining the Testnet

### Testnet Information
- **Network Name**: Citrate Testnet
- **Chain ID**: 1337
- **Bootstrap Nodes**: 
  - `testnet-1.citrate.network:30303`
  - `testnet-2.citrate.network:30303`
- **Block Explorer**: https://explorer.lattice.network (Coming Soon)
- **Faucet**: https://faucet.lattice.network (Coming Soon)

### Connect to Testnet

```bash
./citrate-node \
    --bootstrap-nodes "testnet-1.citrate.network:30303,testnet-2.citrate.network:30303" \
    --chain-id 1337 \
    --p2p-addr 0.0.0.0:30303 \
    --rpc-addr 0.0.0.0:8545 \
    --data-dir ~/.citrate-testnet \
    --max-peers 50
```

### Enable Mining on Testnet

```bash
# Generate a new keypair
./citrate-node keygen
# Save the public key as your coinbase address

# Start mining
./citrate-node \
    --bootstrap-nodes "testnet-1.citrate.network:30303" \
    --mine \
    --coinbase 0xYOUR_GENERATED_ADDRESS \
    --data-dir ~/.citrate-testnet
```

---

## Configuration

### Using Configuration File

Create `config.toml`:

```toml
[chain]
chain_id = 1337
network = "testnet"
block_time = 2
ghostdag_k = 18

[network]
listen_addr = "0.0.0.0:30303"
bootstrap_nodes = [
    "testnet-1.citrate.network:30303",
    "testnet-2.citrate.network:30303"
]
max_peers = 50

[rpc]
enabled = true
listen_addr = "0.0.0.0:8545"
ws_addr = "0.0.0.0:8546"

[storage]
data_dir = "~/.citrate-data"
pruning = false
keep_blocks = 100000

[mining]
enabled = true
coinbase = "0xYOUR_ADDRESS"
target_block_time = 2
min_gas_price = 1000000000
```

Run with config:
```bash
./citrate-node --config config.toml
```

### Environment Variables

```bash
# Enable metrics
export CITRATE_METRICS=true
export CITRATE_METRICS_ADDR=0.0.0.0:9100

# Set log level
export RUST_LOG=info,lattice=debug

# Custom IPFS gateway (for Phase 2)
export CITRATE_IPFS_API=http://localhost:5001

# Signature verification (devnet only)
export CITRATE_REQUIRE_VALID_SIGNATURE=false
```

---

## Monitoring

### Prometheus Metrics

Metrics are exposed at `http://localhost:9100/metrics` when enabled.

```bash
# Enable metrics
CITRATE_METRICS=true ./citrate-node --mine
```

### Grafana Dashboard

1. Start monitoring stack:
```bash
cd monitoring
docker-compose up -d
```

2. Access Grafana at http://localhost:3000
   - Username: admin
   - Password: lattice123

### Log Monitoring

```bash
# Follow logs
tail -f ~/.citrate-data/node.log

# Filter for errors
grep ERROR ~/.citrate-data/node.log

# Watch block production
tail -f ~/.citrate-data/node.log | grep "Produced block"
```

---

## Troubleshooting

### Common Issues

#### Node Won't Start
```bash
# Check if port is in use
lsof -i :30303
lsof -i :8545

# Kill existing processes
pkill -f citrate

# Clear data directory
rm -rf ~/.citrate-data
./citrate-node init
```

#### Can't Connect to Peers
```bash
# Check firewall settings
sudo ufw allow 30303/tcp
sudo ufw allow 8545/tcp

# Test connectivity to bootstrap
telnet testnet-1.citrate.network 30303

# Check NAT/router settings
# Ensure port 30303 is forwarded to your machine
```

#### RPC Not Responding
```bash
# Check if RPC is enabled
grep "rpc.enabled" config.toml

# Test RPC
curl -X POST http://localhost:8545 \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","method":"net_version","params":[],"id":1}'
```

#### High Memory Usage
```bash
# Reduce cache size
export CITRATE_CACHE_SIZE=1000

# Enable pruning in config
pruning = true
keep_blocks = 10000
```

#### Sync Issues
```bash
# Check current block
curl -s http://localhost:8545 -X POST \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","method":"eth_blockNumber","params":[],"id":1}'

# Check peer count
curl -s http://localhost:8545 -X POST \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","method":"net_peerCount","params":[],"id":1}'

# Reset and resync
rm -rf ~/.citrate-data/blocks
./citrate-node --bootstrap-nodes "testnet-1.citrate.network:30303"
```

### Getting Help

- **Discord**: https://discord.gg/lattice
- **Documentation**: https://docs.lattice.network
- **GitHub Issues**: https://github.com/lattice/citrate/issues
- **Email**: support@lattice.network

---

## Security Considerations

### Firewall Configuration

```bash
# Ubuntu/Debian
sudo ufw allow 30303/tcp  # P2P
sudo ufw allow 8545/tcp   # RPC (only if external access needed)
sudo ufw allow 8546/tcp   # WebSocket (only if needed)
sudo ufw allow 9100/tcp   # Metrics (only from monitoring server)
```

### RPC Security

**Never expose RPC to the internet without protection:**

```nginx
# nginx reverse proxy with rate limiting
limit_req_zone $binary_remote_addr zone=rpc:10m rate=10r/s;

server {
    listen 443 ssl;
    location /rpc {
        limit_req zone=rpc burst=20;
        proxy_pass http://127.0.0.1:8545;
    }
}
```

### Key Management

- Never share your private keys
- Use hardware wallets for significant funds
- Keep coinbase address separate from personal wallet
- Regular backups of keystore files

---

## Advanced Topics

### Running with Docker

```dockerfile
# Dockerfile (coming soon)
FROM rust:1.75 as builder
# ... build steps ...

FROM ubuntu:22.04
# ... runtime setup ...
```

```bash
# Run with Docker
docker run -d \
  -p 30303:30303 \
  -p 8545:8545 \
  -v lattice-data:/data \
  lattice/node:latest
```

### Kubernetes Deployment

```yaml
# citrate-node.yaml (coming soon)
apiVersion: apps/v1
kind: StatefulSet
metadata:
  name: citrate-node
spec:
  replicas: 3
  # ... full config ...
```

### Performance Tuning

```bash
# Increase file descriptor limits
ulimit -n 65536

# Optimize network buffers
sudo sysctl -w net.core.rmem_max=134217728
sudo sysctl -w net.core.wmem_max=134217728

# CPU affinity for mining
taskset -c 0-3 ./citrate-node --mine
```

---

## Next Steps

1. **Join the Network**: Follow the testnet connection guide
2. **Start Mining**: Earn rewards by validating blocks
3. **Run a Validator**: Participate in consensus
4. **Deploy Smart Contracts**: Build on Citrate
5. **Contribute**: Join our open-source community

---

*Last Updated: October 2024*
*Version: 1.0.0*
