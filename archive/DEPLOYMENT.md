# Lattice V3 Deployment Guide

## Table of Contents
1. [Prerequisites](#prerequisites)
2. [Quick Start](#quick-start)
3. [Deployment Options](#deployment-options)
4. [Configuration](#configuration)
5. [API Documentation](#api-documentation)
6. [Monitoring](#monitoring)
7. [Troubleshooting](#troubleshooting)

## Prerequisites

### System Requirements
- **CPU**: 4+ cores recommended
- **RAM**: 8GB minimum, 16GB recommended
- **Storage**: 100GB+ SSD for mainnet
- **Network**: Stable internet connection with open ports

### Software Requirements
- Rust 1.75 or later
- Docker (optional, for containerized deployment)
- Git

## Quick Start

### 1. Clone and Build
```bash
git clone https://github.com/lattice-network/lattice-v3.git
cd lattice-v3
cargo build --release
```

### 2. Run Development Network
```bash
./target/release/lattice devnet
```

This starts a local development network with:
- JSON-RPC on http://localhost:8545
- WebSocket on ws://localhost:8546
- REST API on http://localhost:3000

## Deployment Options

### Option 1: Direct Binary Deployment

Use the provided deployment script:
```bash
chmod +x scripts/deploy.sh
./scripts/deploy.sh [network] [data_dir] [rpc_port] [ws_port] [rest_port] [p2p_port]

# Examples:
./scripts/deploy.sh devnet      # Development network
./scripts/deploy.sh testnet     # Test network
./scripts/deploy.sh mainnet     # Main network
```

### Option 2: Docker Deployment

Build and run with Docker:
```bash
# Build the image
docker build -t lattice:v3 .

# Run the container
docker run -d \
  --name lattice-node \
  -p 8545:8545 \
  -p 8546:8546 \
  -p 3000:3000 \
  -p 30303:30303 \
  -v lattice-data:/data \
  lattice:v3
```

### Option 3: Docker Compose

Full stack deployment with monitoring:
```bash
docker-compose up -d
```

This deploys:
- Lattice node
- Block explorer (port 8080)
- Prometheus metrics (port 9090)
- Grafana dashboard (port 3001)
- PostgreSQL database

### Option 4: Systemd Service

For production servers:
```bash
# Install as systemd service
sudo cp scripts/lattice.service /etc/systemd/system/
sudo systemctl daemon-reload
sudo systemctl enable lattice
sudo systemctl start lattice

# Check status
sudo systemctl status lattice
```

## Configuration

### Configuration File

Create `config.toml`:

```toml
[network]
id = "mainnet"
listen_addr = "0.0.0.0:30303"
max_peers = 50
enable_discovery = true
bootnodes = [
    "enode://abc123@1.2.3.4:30303",
    "enode://def456@5.6.7.8:30303"
]

[consensus]
type = "ghostdag"
k_parameter = 16
block_time_seconds = 2
finality_depth = 100

[api]
rpc_enabled = true
rpc_port = 8545
rpc_host = "127.0.0.1"
rpc_cors = ["*"]
ws_enabled = true
ws_port = 8546
ws_host = "127.0.0.1"
rest_enabled = true
rest_port = 3000
rest_host = "127.0.0.1"

[storage]
db_path = "./data/chain"
state_path = "./data/state"
model_storage = "./data/models"
cache_size_mb = 1024
prune_blocks = true
prune_threshold = 100000

[ai]
enable_inference = true
enable_training = true
max_model_size_mb = 5000
inference_timeout_seconds = 30
cache_inference_results = true
model_validation = true

[mining]
enabled = false
coinbase = "0x0000000000000000000000000000000000000000"
threads = 4
target_block_time = 2

[logging]
level = "info"
file = "./logs/lattice.log"
rotate_size_mb = 100
max_backups = 10
```

### Environment Variables

Override configuration with environment variables:
```bash
export LATTICE_NETWORK=mainnet
export LATTICE_RPC_PORT=8545
export LATTICE_DATA_DIR=/data/lattice
export RUST_LOG=info,lattice_api=debug
```

## API Documentation

### JSON-RPC API (Port 8545)

#### Standard Ethereum Methods
- `eth_blockNumber`
- `eth_getBlockByHash`
- `eth_getBlockByNumber`
- `eth_sendRawTransaction`
- `eth_getTransactionReceipt`
- `eth_getBalance`
- `eth_call`
- `eth_estimateGas`
- `eth_gasPrice`
- `eth_chainId`

#### Lattice AI Methods
- `lattice_deployModel` - Deploy new AI model
- `lattice_getModel` - Get model information
- `lattice_listModels` - List available models
- `lattice_requestInference` - Request model inference
- `lattice_getInferenceResult` - Get inference result
- `lattice_createTrainingJob` - Create training job
- `lattice_getTrainingJob` - Get training job status
- `lattice_submitGradient` - Submit training gradient

### WebSocket API (Port 8546)

Subscribe to real-time events:
```javascript
// Connect to WebSocket
const ws = new WebSocket('ws://localhost:8546');

// Subscribe to new blocks
ws.send(JSON.stringify({
  jsonrpc: '2.0',
  method: 'eth_subscribe',
  params: ['newHeads'],
  id: 1
}));

// Subscribe to inference results
ws.send(JSON.stringify({
  jsonrpc: '2.0',
  method: 'lattice_subscribe',
  params: ['inferenceResults', {modelId: '0x...'}],
  id: 2
}));
```

### REST API (Port 3000)

#### OpenAI Compatible Endpoints
```bash
# Chat completion
curl -X POST http://localhost:3000/v1/chat/completions \
  -H "Content-Type: application/json" \
  -d '{
    "model": "lattice-gpt",
    "messages": [{"role": "user", "content": "Hello"}],
    "stream": false
  }'

# Embeddings
curl -X POST http://localhost:3000/v1/embeddings \
  -H "Content-Type: application/json" \
  -d '{
    "model": "lattice-embed",
    "input": ["Hello world"]
  }'
```

#### Lattice Specific Endpoints
- `GET /lattice/models` - List all models
- `GET /lattice/models/{id}` - Get model details
- `POST /lattice/models` - Deploy new model
- `POST /lattice/inference` - Request inference
- `GET /lattice/training/jobs` - List training jobs
- `POST /lattice/training/jobs` - Create training job

## Monitoring

### Metrics Endpoint
Access Prometheus metrics at:
```
http://localhost:9090/metrics
```

### Grafana Dashboard
Access at http://localhost:3001 (default login: admin/admin)

Pre-configured dashboards:
- Node Overview
- Transaction Pool
- Network Peers
- AI Model Activity
- System Resources

### Health Check
```bash
curl http://localhost:8545/health
```

Response:
```json
{
  "status": "healthy",
  "block_height": 12345,
  "peers": 25,
  "syncing": false,
  "version": "3.0.0"
}
```

## Troubleshooting

### Common Issues

#### Port Already in Use
```bash
# Find process using port
lsof -i :8545
# Kill process
kill -9 <PID>
```

#### Database Corruption
```bash
# Reset blockchain data
rm -rf ~/.lattice/chain
./lattice --resync
```

#### Out of Memory
Increase heap size:
```bash
export RUST_MIN_STACK=8388608
```

#### Sync Issues
Check peers:
```bash
curl -X POST http://localhost:8545 \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","method":"net_peerCount","id":1}'
```

### Logs

Check logs for errors:
```bash
# System logs
tail -f ~/.lattice/logs/lattice.log

# Docker logs
docker logs -f lattice-node

# Systemd logs
journalctl -u lattice -f
```

### Support

- GitHub Issues: https://github.com/lattice-network/lattice-v3/issues
- Discord: https://discord.gg/lattice
- Documentation: https://docs.lattice.network

## Security Considerations

### Firewall Configuration
```bash
# Allow P2P
sudo ufw allow 30303/tcp

# Allow RPC (restrict to localhost in production)
sudo ufw allow from 127.0.0.1 to any port 8545

# Allow WebSocket (restrict to localhost in production)
sudo ufw allow from 127.0.0.1 to any port 8546
```

### SSL/TLS Setup
For production, use nginx as reverse proxy:
```nginx
server {
    listen 443 ssl;
    server_name api.lattice.network;
    
    ssl_certificate /etc/ssl/certs/lattice.crt;
    ssl_certificate_key /etc/ssl/private/lattice.key;
    
    location / {
        proxy_pass http://localhost:8545;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
    }
}
```

## Performance Tuning

### System Optimization
```bash
# Increase file descriptors
ulimit -n 65536

# Tune network stack
sudo sysctl -w net.core.rmem_max=134217728
sudo sysctl -w net.core.wmem_max=134217728
sudo sysctl -w net.ipv4.tcp_rmem="4096 87380 134217728"
sudo sysctl -w net.ipv4.tcp_wmem="4096 65536 134217728"
```

### Database Optimization
```toml
[storage]
cache_size_mb = 4096
write_buffer_size_mb = 256
max_open_files = 10000
compression = "lz4"
```

## Backup and Recovery

### Backup
```bash
# Stop node
systemctl stop lattice

# Backup data
tar -czf lattice-backup-$(date +%Y%m%d).tar.gz ~/.lattice

# Restart node
systemctl start lattice
```

### Recovery
```bash
# Stop node
systemctl stop lattice

# Restore backup
tar -xzf lattice-backup-20240101.tar.gz -C ~/

# Restart node
systemctl start lattice
```

## Conclusion

Lattice V3 is now ready for deployment! Choose the deployment option that best fits your needs and follow the configuration guidelines for optimal performance.

For updates and announcements, follow:
- Twitter: @LatticeNetwork
- Blog: https://blog.lattice.network
- GitHub: https://github.com/lattice-network