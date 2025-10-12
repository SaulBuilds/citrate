#!/bin/bash

# Lattice V3 Multi-Node Devnet Launcher
# Launches multiple devnet instances on different ports

set -e

# Configuration
NUM_NODES=${NUM_NODES:-3}
BASE_DIR=${BASE_DIR:-"$HOME/.lattice-multi-devnet"}

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${GREEN}Lattice V3 Multi-Node Devnet Launcher${NC}"
echo "========================================="
echo "Number of nodes: $NUM_NODES"
echo "Base directory: $BASE_DIR"
echo ""

# Function to kill all nodes
cleanup() {
    echo -e "\n${YELLOW}Stopping all nodes...${NC}"
    # Kill all lattice processes
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

# Clean up previous devnet data
if [ -d "$BASE_DIR" ]; then
    echo -e "${YELLOW}Cleaning up previous devnet data...${NC}"
    rm -rf "$BASE_DIR"
fi

# Create base directory
mkdir -p "$BASE_DIR"

echo -e "${GREEN}Note: Current lattice binary doesn't support multi-node networking via CLI.${NC}"
echo -e "${YELLOW}This script will launch independent nodes for development.${NC}"
echo ""

# Launch nodes
for ((i=0; i<$NUM_NODES; i++)); do
    NODE_DIR="$BASE_DIR/node$i"

    echo -e "${GREEN}Launching Node $i...${NC}"
    echo "  Data dir: $NODE_DIR"

    # Create node directory
    mkdir -p "$NODE_DIR"

    # Create a simple config for each node
    cat > "$NODE_DIR/config.toml" << EOF
[chain]
chain_id = 1337
network = "devnet"

[storage]
data_dir = "$NODE_DIR/data"

[network]
listen_addr = "/ip4/127.0.0.1/tcp/$((30303 + i))"
max_peers = 50

[rpc]
enabled = true
host = "127.0.0.1"
port = $((8545 + i))
cors = ["*"]

[mempool]
max_size = 10000
min_gas_price = 1000000000
EOF

    # Launch the node in background
    LOG_FILE="$NODE_DIR/node.log"

    # Start each node with its own data directory
    RUST_LOG=info nohup target/release/lattice \
        --config "$NODE_DIR/config.toml" \
        --data-dir "$NODE_DIR/data" \
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
        echo "Last 20 lines of log:"
        tail -20 "$LOG_FILE"

        # Try the simpler devnet mode instead
        echo -e "${YELLOW}Trying devnet mode for node $i...${NC}"

        cd "$NODE_DIR"
        RUST_LOG=info nohup ../../target/release/lattice devnet > "$LOG_FILE" 2>&1 &
        NODE_PID=$!
        cd - > /dev/null

        echo $NODE_PID > "$NODE_DIR/node.pid"
        sleep 2

        if ! kill -0 $NODE_PID 2>/dev/null; then
            echo -e "${RED}Devnet mode also failed${NC}"
            continue
        fi
    fi

    echo -e "${GREEN}Node $i started${NC}"
    echo ""
done

echo -e "${GREEN}Devnet nodes launched!${NC}"
echo ""

# Show status
echo "Node Status:"
echo "============"
for ((i=0; i<$NUM_NODES; i++)); do
    NODE_DIR="$BASE_DIR/node$i"
    if [ -f "$NODE_DIR/node.pid" ]; then
        PID=$(cat "$NODE_DIR/node.pid")
        if kill -0 $PID 2>/dev/null; then
            echo "  Node $i: ✅ Running (PID: $PID)"
        else
            echo "  Node $i: ❌ Stopped"
        fi
    else
        echo "  Node $i: ❌ No PID file"
    fi
done

echo ""
echo "Logs available at:"
for ((i=0; i<$NUM_NODES; i++)); do
    echo "  Node $i: $BASE_DIR/node$i/node.log"
done

echo ""
echo -e "${YELLOW}Press Ctrl+C to stop all nodes${NC}"
echo ""

# For now, just run a simple devnet since the binary doesn't support all the features yet
echo -e "${GREEN}Alternatively, run a single devnet node with:${NC}"
echo "  ./target/release/lattice devnet"
echo ""
echo -e "${GREEN}Or initialize and run with mining:${NC}"
echo "  ./target/release/lattice init"
echo "  ./target/release/lattice --mine"

# Keep script running and show logs
tail -f "$BASE_DIR"/node*/node.log