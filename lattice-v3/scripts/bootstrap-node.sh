#!/bin/bash

# Lattice v3 Bootstrap Node Management Script
# This script helps launch and manage bootstrap nodes for the network

set -e

SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

# Configuration
NETWORK="${1:-testnet}"
NODE_INDEX="${2:-1}"
BASE_P2P_PORT=30303
BASE_RPC_PORT=8545
BASE_WS_PORT=8546

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

show_usage() {
    echo "Usage: $0 [network] [node-index]"
    echo ""
    echo "Networks: mainnet, testnet, devnet"
    echo "Node Index: 1-10 (for running multiple bootstrap nodes)"
    echo ""
    echo "Examples:"
    echo "  $0 testnet 1    # Start first testnet bootstrap node"
    echo "  $0 testnet 2    # Start second testnet bootstrap node"
    echo "  $0 devnet 1     # Start devnet bootstrap node"
}

# Parse network type
case "$NETWORK" in
    mainnet)
        CHAIN_ID=1
        DATA_DIR="$PROJECT_ROOT/.lattice-mainnet-boot-$NODE_INDEX"
        ;;
    testnet)
        CHAIN_ID=42069
        DATA_DIR="$PROJECT_ROOT/.lattice-testnet-boot-$NODE_INDEX"
        ;;
    devnet)
        CHAIN_ID=1337
        DATA_DIR="$PROJECT_ROOT/.lattice-devnet-boot-$NODE_INDEX"
        ;;
    *)
        echo -e "${RED}Invalid network: $NETWORK${NC}"
        show_usage
        exit 1
        ;;
esac

# Calculate ports based on node index
P2P_PORT=$((BASE_P2P_PORT + NODE_INDEX - 1))
RPC_PORT=$((BASE_RPC_PORT + (NODE_INDEX - 1) * 10))
WS_PORT=$((BASE_WS_PORT + (NODE_INDEX - 1) * 10))

echo "========================================="
echo "  Lattice v3 Bootstrap Node"
echo "========================================="
echo ""
echo "Network:    $NETWORK"
echo "Chain ID:   $CHAIN_ID"
echo "Node Index: $NODE_INDEX"
echo "Data Dir:   $DATA_DIR"
echo "P2P Port:   $P2P_PORT"
echo "RPC Port:   $RPC_PORT"
echo "WS Port:    $WS_PORT"
echo ""

# Check if binary exists
if [ ! -f "$PROJECT_ROOT/target/release/lattice" ]; then
    echo -e "${YELLOW}Building node binary...${NC}"
    cd "$PROJECT_ROOT"
    cargo build --release -p lattice-node
fi

# Clean existing data if requested
if [ "$3" == "--clean" ]; then
    echo -e "${YELLOW}Cleaning existing data...${NC}"
    rm -rf "$DATA_DIR"
fi

# Create bootstrap node config
CONFIG_FILE="$DATA_DIR/config.toml"
mkdir -p "$DATA_DIR"

if [ ! -f "$CONFIG_FILE" ]; then
    echo -e "${GREEN}Creating bootstrap node configuration...${NC}"
    cat > "$CONFIG_FILE" << EOF
# Lattice v3 Bootstrap Node Configuration
# Network: $NETWORK
# Node Index: $NODE_INDEX

# Chain configuration
chain_id = $CHAIN_ID
network_id = $CHAIN_ID
chain_name = "lattice-$NETWORK"

# Network configuration
[network]
listen_addr = "/ip4/0.0.0.0/tcp/$P2P_PORT"
external_addr = ""  # Set this to your public IP if running on a server
bootstrap_nodes = []  # Bootstrap nodes don't connect to other bootstraps
max_peers = 500  # Higher limit for bootstrap nodes
enable_discovery = true

# RPC configuration
[rpc]
enabled = true
listen_addr = "0.0.0.0:$RPC_PORT"
ws_addr = "0.0.0.0:$WS_PORT"
cors_origins = ["*"]
max_connections = 200

# Storage configuration
[storage]
data_dir = "$DATA_DIR"
cache_size = 2147483648  # 2GB for bootstrap nodes

# Consensus configuration
[consensus]
k = 18
block_time_target = 2000
max_block_size = 5242880

# Mining configuration (bootstrap nodes typically don't mine)
[mining]
enabled = false
coinbase_address = "0x0000000000000000000000000000000000000000"
min_gas_price = 1000000000

# Mempool configuration
[mempool]
max_size = 20000  # Larger mempool for bootstrap nodes
max_tx_size = 131072
min_gas_price = 1000000000

# Genesis configuration
[genesis]
timestamp = 1700000000
difficulty = 1
gas_limit = 30000000

# Bootstrap node specific settings
[bootstrap]
is_bootstrap = true
announce_interval = 300  # Announce presence every 5 minutes
peer_exchange = true     # Share peer lists with connected nodes
EOF
fi

# Generate or load node key
KEY_FILE="$DATA_DIR/node.key"
if [ ! -f "$KEY_FILE" ]; then
    echo -e "${GREEN}Generating node keypair...${NC}"
    "$PROJECT_ROOT/target/release/lattice" keygen > "$KEY_FILE" 2>&1
    echo -e "${GREEN}Node key saved to $KEY_FILE${NC}"
fi

# Extract peer ID from key file (if we have a tool for this)
# For now, we'll just show the key file location
echo ""
echo -e "${GREEN}Node key location: $KEY_FILE${NC}"
echo "To get the peer ID for this bootstrap node, check the logs after startup"
echo ""

# Start the bootstrap node
echo -e "${GREEN}Starting bootstrap node...${NC}"
echo "========================================="
echo ""
echo "Bootstrap node will be available at:"
echo "  P2P:       /ip4/<YOUR_IP>/tcp/$P2P_PORT/p2p/<PEER_ID>"
echo "  RPC:       http://localhost:$RPC_PORT"
echo "  WebSocket: ws://localhost:$WS_PORT"
echo ""
echo "Add this to other nodes' bootstrap list once you see the peer ID in logs"
echo ""
echo "Press Ctrl+C to stop the node"
echo "========================================="
echo ""

# Start the node with bootstrap configuration
exec "$PROJECT_ROOT/target/release/lattice" \
    --config "$CONFIG_FILE" \
    --data-dir "$DATA_DIR"