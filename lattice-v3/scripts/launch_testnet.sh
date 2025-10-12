#!/bin/bash

# Lattice V3 Multi-Node Testnet Launcher
# This script launches multiple Lattice nodes for local testnet testing

set -e

# Configuration
NUM_NODES=${NUM_NODES:-5}
BASE_DIR=${BASE_DIR:-"$HOME/.lattice-testnet"}
BASE_RPC_PORT=${BASE_RPC_PORT:-8545}
BASE_P2P_PORT=${BASE_P2P_PORT:-30303}
BASE_WS_PORT=${BASE_WS_PORT:-8546}
LOG_LEVEL=${LOG_LEVEL:-"info"}

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${GREEN}Lattice V3 Multi-Node Testnet Launcher${NC}"
echo "========================================="
echo "Number of nodes: $NUM_NODES"
echo "Base directory: $BASE_DIR"
echo "RPC ports: $BASE_RPC_PORT-$((BASE_RPC_PORT + NUM_NODES - 1))"
echo "P2P ports: $BASE_P2P_PORT-$((BASE_P2P_PORT + NUM_NODES - 1))"
echo "WS ports: $BASE_WS_PORT-$((BASE_WS_PORT + NUM_NODES - 1))"
echo ""

# Function to kill all nodes
cleanup() {
    echo -e "\n${YELLOW}Stopping all nodes...${NC}"
    pkill -f "lattice" || true
    echo -e "${GREEN}All nodes stopped${NC}"
}

# Set up cleanup on exit
trap cleanup EXIT

# Check if binary exists
if [ ! -f "target/release/lattice" ]; then
    echo -e "${YELLOW}Building lattice in release mode...${NC}"
    cargo build --release --bin lattice
fi

# Clean up previous testnet data
if [ -d "$BASE_DIR" ]; then
    echo -e "${YELLOW}Cleaning up previous testnet data...${NC}"
    rm -rf "$BASE_DIR"
fi

# Create base directory
mkdir -p "$BASE_DIR"

# Generate node keys and configs
echo -e "${GREEN}Generating node configurations...${NC}"

# Create a shared genesis configuration
cat > "$BASE_DIR/genesis.json" << EOF
{
  "version": 1,
  "chain_id": 12345,
  "genesis_time": $(date +%s),
  "initial_difficulty": 1000,
  "initial_blue_score": 0,
  "treasury_address": "0x1234567890123456789012345678901234567890",
  "initial_balances": {
    "0x1234567890123456789012345678901234567890": "1000000000000000000000000",
    "0x2345678901234567890123456789012345678901": "1000000000000000000000000",
    "0x3456789012345678901234567890123456789012": "1000000000000000000000000",
    "0x4567890123456789012345678901234567890123": "1000000000000000000000000",
    "0x5678901234567890123456789012345678901234": "1000000000000000000000000"
  }
}
EOF

# Array to store node multiaddresses for bootstrapping
declare -a NODE_ADDRS

# Launch nodes
for ((i=0; i<$NUM_NODES; i++)); do
    NODE_DIR="$BASE_DIR/node$i"
    RPC_PORT=$((BASE_RPC_PORT + i))
    P2P_PORT=$((BASE_P2P_PORT + i))
    WS_PORT=$((BASE_WS_PORT + i))

    echo -e "${GREEN}Launching Node $i...${NC}"
    echo "  Data dir: $NODE_DIR"
    echo "  RPC port: $RPC_PORT"
    echo "  P2P port: $P2P_PORT"
    echo "  WS port: $WS_PORT"

    # Create node directory
    mkdir -p "$NODE_DIR"

    # Copy genesis to node directory
    cp "$BASE_DIR/genesis.json" "$NODE_DIR/genesis.json"

    # Build bootstrap nodes argument (connect to previous nodes)
    BOOTSTRAP_NODES=""
    if [ $i -gt 0 ]; then
        # Connect to first node as bootstrap
        BOOTSTRAP_NODES="--bootstrap-nodes /ip4/127.0.0.1/tcp/$BASE_P2P_PORT/p2p/node0"
    fi

    # Create node config file
    cat > "$NODE_DIR/config.toml" << EOF
[node]
name = "testnet-node-$i"
data_dir = "$NODE_DIR"
log_level = "$LOG_LEVEL"

[network]
listen_addr = "/ip4/0.0.0.0/tcp/$P2P_PORT"
max_peers = 50
max_inbound = 25
max_outbound = 25

[rpc]
enabled = true
host = "127.0.0.1"
port = $RPC_PORT
cors = ["*"]
max_connections = 100

[ws]
enabled = true
host = "127.0.0.1"
port = $WS_PORT

[consensus]
k_parameter = 18
blue_score_threshold = 100
finality_depth = 100

[mempool]
max_size = 10000
tx_expiry_secs = 600
min_gas_price = 1000000000

[execution]
enable_precompiles = true
cache_size = 1000
EOF

    # Launch the node
    LOG_FILE="$NODE_DIR/node.log"

    # Start the node in background
    nohup target/release/lattice \
        --config "$NODE_DIR/config.toml" \
        --data-dir "$NODE_DIR" \
        --rpc-port $RPC_PORT \
        --p2p-port $P2P_PORT \
        --ws-port $WS_PORT \
        --log-level $LOG_LEVEL \
        $BOOTSTRAP_NODES \
        > "$LOG_FILE" 2>&1 &

    NODE_PID=$!
    echo "  PID: $NODE_PID"
    echo "  Log: $LOG_FILE"

    # Save PID for later
    echo $NODE_PID > "$NODE_DIR/node.pid"

    # Give node time to start
    sleep 2

    # Check if node started successfully
    if ! kill -0 $NODE_PID 2>/dev/null; then
        echo -e "${RED}Failed to start node $i${NC}"
        echo "Check log file: $LOG_FILE"
        tail -20 "$LOG_FILE"
        exit 1
    fi

    echo -e "${GREEN}Node $i started successfully${NC}"
    echo ""
done

echo -e "${GREEN}All nodes launched!${NC}"
echo ""

# Wait for nodes to connect
echo -e "${YELLOW}Waiting for nodes to establish connections...${NC}"
sleep 5

# Function to check node status
check_node_status() {
    local port=$1
    local node_num=$2

    echo -e "Node $node_num (RPC port $port):"

    # Check if RPC is responding
    if curl -s -X POST http://127.0.0.1:$port \
        -H "Content-Type: application/json" \
        -d '{"jsonrpc":"2.0","method":"net_peerCount","params":[],"id":1}' > /dev/null 2>&1; then

        # Get peer count
        PEER_COUNT=$(curl -s -X POST http://127.0.0.1:$port \
            -H "Content-Type: application/json" \
            -d '{"jsonrpc":"2.0","method":"net_peerCount","params":[],"id":1}' | \
            grep -o '"result":"0x[0-9a-f]*"' | cut -d'"' -f4 | xargs printf "%d\n" 2>/dev/null || echo "0")

        # Get block height
        BLOCK_HEIGHT=$(curl -s -X POST http://127.0.0.1:$port \
            -H "Content-Type: application/json" \
            -d '{"jsonrpc":"2.0","method":"eth_blockNumber","params":[],"id":1}' | \
            grep -o '"result":"0x[0-9a-f]*"' | cut -d'"' -f4 | xargs printf "%d\n" 2>/dev/null || echo "0")

        echo "  Status: ✅ Online"
        echo "  Peers: $PEER_COUNT"
        echo "  Block Height: $BLOCK_HEIGHT"
    else
        echo "  Status: ❌ Offline or not responding"
    fi
}

# Check status of all nodes
echo ""
echo -e "${GREEN}Node Status:${NC}"
echo "============"
for ((i=0; i<$NUM_NODES; i++)); do
    RPC_PORT=$((BASE_RPC_PORT + i))
    check_node_status $RPC_PORT $i
    echo ""
done

# Show how to interact with the testnet
echo -e "${GREEN}Testnet is running!${NC}"
echo ""
echo "To interact with the nodes:"
echo "  RPC endpoints: http://127.0.0.1:$BASE_RPC_PORT to http://127.0.0.1:$((BASE_RPC_PORT + NUM_NODES - 1))"
echo "  WS endpoints: ws://127.0.0.1:$BASE_WS_PORT to ws://127.0.0.1:$((BASE_WS_PORT + NUM_NODES - 1))"
echo ""
echo "Example commands:"
echo "  # Get peer count for node 0"
echo "  curl -X POST http://127.0.0.1:$BASE_RPC_PORT -H \"Content-Type: application/json\" \\"
echo "    -d '{\"jsonrpc\":\"2.0\",\"method\":\"net_peerCount\",\"params\":[],\"id\":1}'"
echo ""
echo "  # Get latest block"
echo "  curl -X POST http://127.0.0.1:$BASE_RPC_PORT -H \"Content-Type: application/json\" \\"
echo "    -d '{\"jsonrpc\":\"2.0\",\"method\":\"eth_getBlockByNumber\",\"params\":[\"latest\", false],\"id\":1}'"
echo ""
echo "  # Send transaction (using lattice-wallet)"
echo "  cargo run --bin lattice-wallet -- --rpc-url http://127.0.0.1:$BASE_RPC_PORT transfer \\"
echo "    --to 0x2345678901234567890123456789012345678901 --value 1000000000000000000"
echo ""
echo "Logs are available at:"
for ((i=0; i<$NUM_NODES; i++)); do
    echo "  Node $i: $BASE_DIR/node$i/node.log"
done
echo ""
echo -e "${YELLOW}Press Ctrl+C to stop all nodes${NC}"

# Keep script running
while true; do
    sleep 60

    # Periodically check node health
    echo -e "\n${GREEN}[$(date '+%Y-%m-%d %H:%M:%S')] Health Check${NC}"
    for ((i=0; i<$NUM_NODES; i++)); do
        NODE_DIR="$BASE_DIR/node$i"
        PID=$(cat "$NODE_DIR/node.pid" 2>/dev/null)
        if [ -n "$PID" ] && kill -0 $PID 2>/dev/null; then
            echo -n "  Node $i: ✅ "
        else
            echo -n "  Node $i: ❌ "
        fi
    done
    echo ""
done