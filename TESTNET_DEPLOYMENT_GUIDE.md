# Lattice V3 Multi-Node Testnet Deployment Guide

## Overview
This guide provides instructions for deploying and managing a multi-node Lattice V3 testnet for development and testing purposes.

---

## Prerequisites

### System Requirements
- **OS**: Linux or macOS
- **RAM**: 8GB minimum (16GB recommended for 5+ nodes)
- **Disk**: 10GB free space
- **CPU**: 4 cores minimum
- **Network**: Ports 8545-8555, 30303-30313, 8546-8556 available

### Software Requirements
- Rust 1.75+ with cargo
- Git
- curl (for testing)
- Build tools (gcc/clang, make, pkg-config)

---

## Quick Start

### 1. Build the Project
```bash
cd lattice-v3
cargo build --release
```

### 2. Launch Testnet (5 nodes)
```bash
./scripts/launch_testnet.sh
```

### 3. Monitor Testnet
In a new terminal:
```bash
./scripts/monitor_testnet.sh
```

### 4. Send Test Transactions
In another terminal:
```bash
./scripts/test_transactions.sh
```

---

## Detailed Configuration

### Environment Variables

| Variable | Default | Description |
|----------|---------|-------------|
| `NUM_NODES` | 5 | Number of nodes to launch |
| `BASE_DIR` | `~/.lattice-testnet` | Data directory for all nodes |
| `BASE_RPC_PORT` | 8545 | Starting RPC port (increments per node) |
| `BASE_P2P_PORT` | 30303 | Starting P2P port (increments per node) |
| `BASE_WS_PORT` | 8546 | Starting WebSocket port |
| `LOG_LEVEL` | info | Logging level (trace/debug/info/warn/error) |

### Custom Configuration
```bash
# Launch 10 nodes with debug logging
NUM_NODES=10 LOG_LEVEL=debug ./scripts/launch_testnet.sh

# Use custom ports
BASE_RPC_PORT=9000 BASE_P2P_PORT=40000 ./scripts/launch_testnet.sh
```

---

## Network Architecture

### Node Configuration
Each node is configured with:
- **Unique ports**: RPC, P2P, WebSocket
- **Data directory**: `~/.lattice-testnet/node{N}`
- **Genesis file**: Shared across all nodes
- **Bootstrap**: Node 0 acts as bootstrap for others

### Port Allocation
```
Node 0: RPC=8545, P2P=30303, WS=8546
Node 1: RPC=8546, P2P=30304, WS=8547
Node 2: RPC=8547, P2P=30305, WS=8548
...
```

### Consensus Parameters
- **Algorithm**: GhostDAG
- **K-parameter**: 18
- **Blue score threshold**: 100
- **Finality depth**: 100 blocks
- **Block time**: ~2 seconds

---

## Interacting with the Testnet

### Using curl

#### Get Node Info
```bash
# Peer count
curl -X POST http://127.0.0.1:8545 \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","method":"net_peerCount","params":[],"id":1}'

# Block number
curl -X POST http://127.0.0.1:8545 \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","method":"eth_blockNumber","params":[],"id":1}'

# Get latest block
curl -X POST http://127.0.0.1:8545 \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","method":"eth_getBlockByNumber","params":["latest",false],"id":1}'
```

### Using lattice-wallet
```bash
# Check balance
cargo run --bin lattice-wallet -- \
  --rpc-url http://127.0.0.1:8545 \
  balance

# Send transaction
cargo run --bin lattice-wallet -- \
  --rpc-url http://127.0.0.1:8545 \
  transfer \
  --to 0x2345678901234567890123456789012345678901 \
  --value 1000000000000000000
```

### Pre-funded Accounts
The genesis configuration includes these pre-funded accounts:
```
0x1234567890123456789012345678901234567890: 1,000,000 tokens
0x2345678901234567890123456789012345678901: 1,000,000 tokens
0x3456789012345678901234567890123456789012: 1,000,000 tokens
0x4567890123456789012345678901234567890123: 1,000,000 tokens
0x5678901234567890123456789012345678901234: 1,000,000 tokens
```

---

## Monitoring and Debugging

### Live Monitoring
The monitor script shows:
- Node status (online/offline)
- Peer connections
- Block height
- Mempool size
- Blue score
- Network health indicators

```bash
./scripts/monitor_testnet.sh
```

### Log Files
Each node has its own log file:
```bash
# View logs for node 0
tail -f ~/.lattice-testnet/node0/node.log

# Check for errors
grep ERROR ~/.lattice-testnet/node*/node.log
```

### Health Checks
```bash
# Check if nodes are running
ps aux | grep lattice-node

# Check port usage
lsof -i :8545-8555

# Check peer connections for all nodes
for i in {0..4}; do
  echo "Node $i peers:"
  curl -s -X POST http://127.0.0.1:$((8545+i)) \
    -H "Content-Type: application/json" \
    -d '{"jsonrpc":"2.0","method":"net_peerCount","params":[],"id":1}'
  echo
done
```

---

## Testing Scenarios

### 1. Basic Connectivity Test
```bash
# Launch testnet
./scripts/launch_testnet.sh

# Wait 10 seconds for connections
sleep 10

# Check all nodes have peers
for i in {0..4}; do
  echo -n "Node $i: "
  curl -s http://127.0.0.1:$((8545+i)) \
    -X POST -H "Content-Type: application/json" \
    -d '{"jsonrpc":"2.0","method":"net_peerCount","params":[],"id":1}' | \
    grep -o '"result":"[^"]*"'
done
```

### 2. Transaction Throughput Test
```bash
# Send 100 transactions
NUM_TXS=100 ./scripts/test_transactions.sh

# Monitor block production
watch -n 1 'curl -s http://127.0.0.1:8545 \
  -X POST -H "Content-Type: application/json" \
  -d "{\"jsonrpc\":\"2.0\",\"method\":\"eth_blockNumber\",\"params\":[],\"id\":1}"'
```

### 3. Consensus Test
```bash
# Stop one node
kill $(cat ~/.lattice-testnet/node0/node.pid)

# Check if network continues (should work with 4/5 nodes)
./scripts/monitor_testnet.sh
```

### 4. Fork Resolution Test
```bash
# Partition network (requires iptables)
# Block communication between nodes 0-1 and nodes 2-4
sudo iptables -A INPUT -p tcp --sport 30303:30304 --dport 30305:30307 -j DROP
sudo iptables -A INPUT -p tcp --sport 30305:30307 --dport 30303:30304 -j DROP

# Wait for fork
sleep 30

# Restore network
sudo iptables -F

# Monitor convergence
./scripts/monitor_testnet.sh
```

---

## Troubleshooting

### Common Issues

#### Nodes Not Starting
```bash
# Check if ports are in use
lsof -i :8545
lsof -i :30303

# Kill any existing nodes
pkill -f lattice-node

# Clean data directory
rm -rf ~/.lattice-testnet
```

#### Nodes Not Connecting
```bash
# Check firewall settings
sudo iptables -L

# Verify localhost resolution
ping 127.0.0.1

# Check node logs for errors
grep -i "error\|fail" ~/.lattice-testnet/node*/node.log
```

#### Low Transaction Throughput
```bash
# Increase mempool size in config
# Edit scripts/launch_testnet.sh
# Change: max_size = 10000 to max_size = 50000

# Reduce block time (if supported)
# Change consensus parameters in config
```

#### High Memory Usage
```bash
# Reduce number of nodes
NUM_NODES=3 ./scripts/launch_testnet.sh

# Limit cache sizes in config
# Reduce execution.cache_size
```

---

## Advanced Configuration

### Custom Genesis File
Create `custom_genesis.json`:
```json
{
  "version": 1,
  "chain_id": 99999,
  "initial_difficulty": 500,
  "treasury_address": "0xYOUR_ADDRESS",
  "initial_balances": {
    "0xADDRESS1": "1000000000000000000000",
    "0xADDRESS2": "1000000000000000000000"
  }
}
```

### Multi-Machine Deployment
```bash
# On machine 1 (bootstrap node)
./scripts/launch_testnet.sh
# Note the P2P address: /ip4/MACHINE1_IP/tcp/30303/p2p/node0

# On machine 2
BOOTSTRAP_NODES="/ip4/MACHINE1_IP/tcp/30303/p2p/node0" \
  ./scripts/launch_testnet.sh
```

### Performance Tuning
```toml
# In config.toml
[consensus]
k_parameter = 10  # Lower for faster consensus
blue_score_threshold = 50  # Lower for quicker finality

[mempool]
max_size = 50000  # Increase for higher throughput
min_gas_price = 100000000  # Lower for more transactions

[network]
max_peers = 100  # Increase for better connectivity
```

---

## Cleanup

### Stop All Nodes
```bash
# Graceful shutdown
pkill -SIGTERM lattice-node

# Force stop
pkill -9 lattice-node
```

### Remove Data
```bash
# Remove all testnet data
rm -rf ~/.lattice-testnet

# Keep logs for debugging
mv ~/.lattice-testnet/node*/node.log /tmp/
rm -rf ~/.lattice-testnet
```

---

## Next Steps

1. **Deploy Public Testnet**: Use cloud providers for multi-region deployment
2. **Stress Testing**: Run load tests with thousands of transactions
3. **Security Audit**: Test attack vectors and edge cases
4. **Performance Benchmarking**: Measure TPS, latency, finality
5. **Documentation**: Create user guides and API documentation

---

## Support

For issues or questions:
- Check logs: `~/.lattice-testnet/node*/node.log`
- Run diagnostics: `./scripts/monitor_testnet.sh`
- Review documentation: `/docs/`
- File issues: Create GitHub issue with logs and configuration

---

*Last Updated: October 2025*
*Version: 1.0.0*