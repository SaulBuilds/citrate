#!/bin/bash

# Lattice v3 Testnet Startup Script
# This script ensures complete reset and proper testnet configuration

set -e

SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

# Configuration
CHAIN_ID=42069  # Testnet chain ID (0xa455 in hex)
RPC_PORT=8545
WS_PORT=8546
P2P_PORT=30303
DATA_DIR="$PROJECT_ROOT/.lattice-testnet"

echo "========================================="
echo "  Lattice v3 Testnet Launcher"
echo "========================================="
echo ""
echo "Chain ID: $CHAIN_ID (0x$(printf '%x' $CHAIN_ID))"
echo "RPC Port: $RPC_PORT"
echo "WS Port:  $WS_PORT"
echo "P2P Port: $P2P_PORT"
echo ""

# Step 1: Clean existing state
echo "🧹 Cleaning existing state..."
rm -rf "$DATA_DIR"
rm -rf "$PROJECT_ROOT/.lattice"
rm -rf "$PROJECT_ROOT/.lattice-devnet"
rm -rf "$PROJECT_ROOT/gui/lattice-core/src-tauri/gui-data"
rm -rf "$PROJECT_ROOT/gui/lattice-core/.lattice-gui-temp"

# Kill any existing processes
echo "⚡ Stopping any running nodes..."
pkill -f "lattice" 2>/dev/null || true
lsof -i :$RPC_PORT 2>/dev/null | grep LISTEN | awk '{print $2}' | xargs kill -9 2>/dev/null || true
lsof -i :$WS_PORT 2>/dev/null | grep LISTEN | awk '{print $2}' | xargs kill -9 2>/dev/null || true
lsof -i :$P2P_PORT 2>/dev/null | grep LISTEN | awk '{print $2}' | xargs kill -9 2>/dev/null || true

# Step 2: Create testnet config if it doesn't exist
CONFIG_FILE="$PROJECT_ROOT/node/config/testnet.toml"
if [ ! -f "$CONFIG_FILE" ]; then
    echo "📝 Creating testnet configuration..."
    mkdir -p "$PROJECT_ROOT/node/config"
    cat > "$CONFIG_FILE" << EOF
# Lattice v3 Testnet Configuration

[chain]
chain_id = $CHAIN_ID
genesis_hash = ""
block_time = 2
ghostdag_k = 18

[network]
listen_addr = "0.0.0.0:$P2P_PORT"
bootstrap_nodes = []
max_peers = 50

[rpc]
enabled = true
listen_addr = "0.0.0.0:$RPC_PORT"
ws_addr = "0.0.0.0:$WS_PORT"

[storage]
data_dir = "$DATA_DIR"
pruning = false
keep_blocks = 100000

[mining]
enabled = true
coinbase = "0000000000000000000000000000000000000000000000000000000000000000"
target_block_time = 2
min_gas_price = 1000000000
EOF
fi

# Step 3: Build if needed
if [ ! -f "$PROJECT_ROOT/target/release/lattice" ]; then
    echo "🔨 Building node..."
    cd "$PROJECT_ROOT"
    cargo build --release -p lattice-node
fi

# Step 4: Start the node
echo ""
echo "🚀 Starting Lattice testnet node..."
echo "========================================="
echo ""
echo "Node will start with:"
echo "  - Fresh genesis block"
echo "  - Chain ID: $CHAIN_ID"
echo "  - RPC: http://localhost:$RPC_PORT"
echo "  - WebSocket: ws://localhost:$WS_PORT"
echo ""
echo "To connect:"
echo "  - MetaMask: Add network with Chain ID $CHAIN_ID"
echo "  - CLI: ./target/release/lattice-cli --rpc http://localhost:$RPC_PORT"
echo "  - GUI: Set RPC endpoint to http://localhost:$RPC_PORT"
echo ""
echo "Press Ctrl+C to stop the node"
echo "========================================="
echo ""

# Start the node
exec "$PROJECT_ROOT/target/release/lattice" \
    --config "$CONFIG_FILE" \
    --data-dir "$DATA_DIR" \
    --mine