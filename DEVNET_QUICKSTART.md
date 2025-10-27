# Citrate V3 Devnet Quick Start Guide

## 🚀 Quick Start (Single Node)

### 1. Build the Project
```bash
cargo build --release
```

### 2. Start the Devnet
```bash
./scripts/start_devnet.sh
```

This will start a single-node development network with:
- **RPC Port**: 8545
- **P2P Port**: 30303
- **Chain ID**: 1337
- **Block Time**: ~2 seconds
- **Pre-funded Treasury**: 1,000,000 tokens

### 3. Test the Network
In a new terminal:
```bash
./scripts/test_devnet.sh
```

---

## 📊 Monitoring

### Watch Blocks Being Produced
```bash
watch -n 1 'curl -s http://127.0.0.1:8545 -X POST -H "Content-Type: application/json" \
  -d "{\"jsonrpc\":\"2.0\",\"method\":\"eth_blockNumber\",\"params\":[],\"id\":1}" | \
  jq -r ".result" | xargs printf "Block: %d\n"'
```

### Check Node Status
```bash
# Get peer count
curl -X POST http://127.0.0.1:8545 \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","method":"net_peerCount","params":[],"id":1}'

# Get latest block
curl -X POST http://127.0.0.1:8545 \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","method":"eth_getBlockByNumber","params":["latest",false],"id":1}'
```

---

## 🔧 Advanced Usage

### Initialize Custom Chain
```bash
# Initialize with custom chain ID
./target/release/lattice init --chain-id 9999

# Run with mining enabled
./target/release/lattice --mine --data-dir ~/.my-lattice
```

### Use Custom Configuration
Create `config.toml`:
```toml
[chain]
chain_id = 1337
network = "devnet"

[storage]
data_dir = ".lattice-devnet"

[network]
listen_addr = "/ip4/127.0.0.1/tcp/30303"
max_peers = 50

[rpc]
enabled = true
host = "127.0.0.1"
port = 8545
cors = ["*"]

[mempool]
max_size = 10000
min_gas_price = 1000000000
```

Run with config:
```bash
./target/release/lattice --config config.toml
```

---

## 📝 Available Scripts

| Script | Description |
|--------|-------------|
| `start_devnet.sh` | Start single-node devnet (recommended) |
| `test_devnet.sh` | Test connectivity and status |
| `launch_simple_testnet.sh` | Alternative simple launcher |
| `launch_multi_devnet.sh` | Experimental multi-node setup |

---

## 🛠 Troubleshooting

### Node Won't Start
```bash
# Kill any existing processes
pkill -f lattice

# Clean data directory
rm -rf .lattice-devnet

# Restart
./scripts/start_devnet.sh
```

### Can't Connect to RPC
```bash
# Check if node is running
ps aux | grep lattice

# Check if port is in use
lsof -i :8545

# Check logs
tail -f .lattice-devnet/node.log
```

### Low Block Production
- The devnet produces blocks every 2 seconds by default
- Blocks are only produced when there are transactions or on the timer
- This is normal behavior for development

---

## 🎯 Next Steps

1. **Send Transactions**: Use the wallet CLI to send transactions
2. **Deploy Contracts**: Use standard Ethereum tools (Hardhat, Foundry)
3. **Connect MetaMask**: Add custom RPC `http://127.0.0.1:8545`
4. **Run Tests**: Execute integration tests against the devnet

---

## 📚 Key Information

### Pre-funded Account
- **Address**: `0x0f572e5295c57f15886f9b263e2f6d2d6c7b5ec6`
- **Balance**: 1,000,000 tokens
- **Purpose**: Treasury account for testing

### Network Details
- **Chain ID**: 1337
- **Network Name**: Citrate Devnet
- **RPC URL**: `http://127.0.0.1:8545`
- **Currency Symbol**: LATTICE
- **Block Explorer**: None (local only)

### Consensus
- **Algorithm**: GhostDAG
- **K-parameter**: 18
- **Block Time**: ~2 seconds
- **Finality**: Optimistic (instant for devnet)

---

## ⚠️ Important Notes

1. **Development Only**: This devnet is for development and testing only
2. **No Persistence**: Data is stored in `.lattice-devnet` and can be deleted
3. **Single Node**: Current implementation runs as a single node
4. **Mining Rewards**: Validator receives rewards for each block

---

## 🤝 Support

For issues:
1. Check the logs in `.lattice-devnet/`
2. Run `./scripts/test_devnet.sh` to diagnose
3. Ensure all dependencies are installed
4. Try rebuilding with `cargo build --release`

---

*Last Updated: October 2024*
*Version: 1.0.0*