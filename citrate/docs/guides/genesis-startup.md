# Citrate v3 Genesis Startup Guide

This guide provides step-by-step instructions to start Citrate from genesis (block 0) and keep everything properly synced between the core node and GUI.

## Prerequisites

- Rust installed (latest stable)
- Node.js installed (v18+)
- All dependencies installed

## Step 1: Complete Clean State

First, ensure you're starting completely fresh by removing ALL existing data:

```bash
# Navigate to citrate directory
cd /Users/soleilklosowski/Downloads/citrate/citrate

# Kill any running processes
pkill -f "citrate" || true
pkill -f "tauri" || true

# Clean ALL state directories
rm -rf .citrate*
rm -rf target/
rm -rf gui/citrate-core/src-tauri/gui-data
rm -rf gui/citrate-core/.citrate-gui-temp
rm -rf node_modules/
rm -rf gui/citrate-core/node_modules/

# Clean build artifacts
cargo clean
```

## Step 2: Build Everything Fresh

```bash
# Build the core node
cargo build --release -p citrate-node

# Build the CLI wallet
cargo build --release -p citrate-wallet

# Install GUI dependencies and build
cd gui/citrate-core
npm install
npm run tauri:build
cd ../..
```

## Step 3: Start the Core Testnet Node

This will be your main node that starts from genesis:

```bash
# From citrate directory
./scripts/start_testnet.sh
```

You should see:
```
🚀 Starting Citrate testnet node...
Node will start with:
  - Fresh genesis block
  - Chain ID: 42069
  - RPC: http://localhost:8545
  - WebSocket: ws://localhost:8546
```

**IMPORTANT**: Keep this terminal open and running!

## Step 4: Verify Genesis Block

In a new terminal, verify the node started from genesis:

```bash
# Check block number (should be 0)
curl -X POST http://localhost:8545 \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","method":"eth_blockNumber","params":[],"id":1}'

# Check chain ID (should be 0xa455 = 42069)
curl -X POST http://localhost:8545 \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","method":"eth_chainId","params":[],"id":1}'
```

## Step 5: Start the GUI in Testnet Mode

Now start the GUI and configure it to connect to the testnet:

```bash
# From citrate directory
cd gui/citrate-core

# For development mode (recommended for testing):
npm run tauri dev

# OR for production build:
open src-tauri/target/release/bundle/macos/citrate-core.app
```

## Step 6: Configure GUI for Testnet Sync

Once the GUI opens:

1. **Go to Settings Page** (gear icon)
2. **Network Configuration**:
   - Find the "Network" dropdown
   - Change from "devnet" to "testnet"
   - The GUI will automatically restart with:
     - Chain ID: 42069
     - RPC Port: 18545 (GUI's embedded node)
     - P2P Port: 30304 (different from core node's 30303)
     - Bootstrap: 127.0.0.1:30303 (connects to core node)

3. **Wait for Sync Message**:
   - You should see: "Switched to testnet - Chain ID: 42069"
   - The GUI node will now sync with the core node

## Step 7: Connect the Nodes

The GUI node needs to connect to the core node for syncing:

1. **In GUI Settings**, scroll to "Network" section
2. **Click "Connect Bootnodes"**
   - This connects to the core node at 127.0.0.1:30303
3. **Verify Connection**:
   - Check "Connected Peers" section
   - You should see at least 1 peer connected

## Step 8: Verify Sync Status

### Check Core Node Status:
```bash
# Get latest block on core node
curl -X POST http://localhost:8545 \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","method":"eth_getBlockByNumber","params":["latest", false],"id":1}'
```

### Check GUI Node Sync:
- The GUI node will sync blocks via P2P from the core node
- Check the DAG Explorer tab to see if blocks are appearing
- The block count should match the core node once synced
- Note: GUI RPC server (port 18545) is currently disabled

## Step 9: Monitor DAG Explorer

1. **In GUI**, click on "DAG Explorer" tab
2. You should see blocks appearing as they're produced
3. The DAG should show the GhostDAG structure with:
   - Selected parent connections (solid lines)
   - Merge parent connections (dashed lines)
   - Blue/red coloring based on GhostDAG consensus

## Step 10: Test Transaction Flow

### From CLI:
```bash
# Create a test transaction
./target/release/citrate-wallet \
  --rpc-url http://localhost:8545 \
  send --to 0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb9 \
  --amount 1.0
```

### From GUI:
1. Go to "Wallet" tab
2. Click "Send"
3. Enter recipient and amount
4. Click "Send Transaction"

## Troubleshooting

### If nodes aren't syncing:

1. **Check P2P connectivity**:
```bash
# Check if ports are open
lsof -i :30303  # Core node P2P
lsof -i :30304  # GUI node P2P
```

2. **Manually add peer in GUI**:
   - In Settings → Network
   - Enter: `127.0.0.1:30303`
   - Click "Connect"

3. **Check logs**:
   - Core node logs appear in terminal
   - GUI logs: Open Developer Tools (Cmd+Opt+I on Mac)

### If seeing "lock file" errors:

```bash
# Stop everything
pkill -f "citrate"

# Remove lock files
find . -name "LOCK" -delete

# Restart from Step 3
```

### If blocks aren't appearing in DAG Explorer:

1. **Verify core node is producing blocks**:
```bash
# Check latest block number
curl -X POST http://localhost:8545 \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","method":"eth_blockNumber","params":[],"id":1}'
```

2. **Ensure P2P connection is established**:
   - In GUI Settings → Network section
   - Check "Connected Peers" shows at least 1
   - If not, manually connect to `127.0.0.1:30303`

3. **Force refresh DAG data**:
   - Click the refresh button in DAG Explorer
   - Or restart the GUI node via Settings

4. **Check sync is working**:
   - New blocks from core node should appear in GUI storage
   - DAG Explorer reads from GUI's local storage
   - If sync works, blocks will appear automatically

## Network Architecture

```
┌─────────────────────────┐
│   Core Testnet Node     │
│   Chain ID: 42069       │
│   RPC: localhost:8545   │
│   P2P: localhost:30303  │
│   (Started from genesis)│
└───────────┬─────────────┘
            │ P2P Sync
            │
┌───────────▼─────────────┐
│   GUI Embedded Node     │
│   Chain ID: 42069       │
│   RPC: localhost:18545  │
│   P2P: localhost:30304  │
│   (Syncs from core)     │
└─────────────────────────┘
```

## Important Notes

1. **Always start the core node first** - This establishes genesis
2. **GUI connects as a peer** - It syncs the DAG from the core node
3. **Different ports prevent conflicts** - Each node has its own RPC/P2P ports
4. **Chain ID must match** - Both use 42069 for testnet
5. **Clean state is critical** - Any old data will cause sync issues

## Quick Restart Sequence

If you need to restart everything:

```bash
# 1. Stop everything
pkill -f "citrate"

# 2. Clean state (optional for full reset)
rm -rf .citrate*
rm -rf gui/citrate-core/src-tauri/gui-data

# 3. Start core node
./scripts/start_testnet.sh

# 4. Start GUI
cd gui/citrate-core && npm run tauri dev

# 5. In GUI: Settings → Network → Select "testnet"
# 6. In GUI: Settings → Network → Click "Connect Bootnodes"
```

## Success Indicators

You know everything is working when:

✅ Core node shows "Started block producer"  
✅ GUI shows "Switched to testnet - Chain ID: 42069"  
✅ GUI shows at least 1 connected peer  
✅ Both nodes report the same block height  
✅ DAG Explorer shows blocks appearing  
✅ Transactions can be sent and appear in both nodes  

## Support

If you encounter issues not covered here:
- Check the [CLAUDE.md](./CLAUDE.md) file for technical details
- Review recent git commits for any system changes
- Ensure all dependencies are up to date