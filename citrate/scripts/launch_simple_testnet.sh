#!/bin/bash

# Simple Citrate V3 Testnet Launcher
# Uses the built-in devnet mode

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${GREEN}Citrate V3 Simple Testnet Launcher${NC}"
echo "========================================="
echo ""

# Function to kill the node
cleanup() {
    echo -e "\n${YELLOW}Stopping node...${NC}"
    pkill -f "lattice devnet" || true
    echo -e "${GREEN}Node stopped${NC}"
}

# Set up cleanup on exit
trap cleanup EXIT

# Build if needed
if [ ! -f "target/release/lattice" ]; then
    echo -e "${YELLOW}Building lattice in release mode...${NC}"
    cargo build --release --bin lattice
fi

# Initialize chain if needed
DATA_DIR="${DATA_DIR:-$HOME/.citrate-devnet}"

if [ ! -d "$DATA_DIR" ]; then
    echo -e "${GREEN}Initializing new chain...${NC}"
    ./target/release/lattice init --data-dir "$DATA_DIR"
fi

# Launch devnet
echo -e "${GREEN}Launching Citrate devnet...${NC}"
echo "Data directory: $DATA_DIR"
echo ""
echo "Starting node..."

# Run in foreground so we can see output
./target/release/lattice devnet --data-dir "$DATA_DIR" --mine

echo -e "${GREEN}Devnet stopped${NC}"