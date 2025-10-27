#!/bin/bash

# Citrate V3 Devnet Starter
# Simple script to start a local development network

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

clear

echo -e "${GREEN}╔══════════════════════════════════════════════════════╗${NC}"
echo -e "${GREEN}║         Citrate V3 Development Network               ║${NC}"
echo -e "${GREEN}╚══════════════════════════════════════════════════════╝${NC}"
echo ""

# Function to kill the node
cleanup() {
    echo -e "\n${YELLOW}Stopping devnet...${NC}"
    pkill -f "lattice devnet" || true
    echo -e "${GREEN}Devnet stopped${NC}"
}

# Set up cleanup on exit
trap cleanup EXIT

# Build if needed
if [ ! -f "target/release/lattice" ]; then
    echo -e "${YELLOW}Building lattice in release mode...${NC}"
    cargo build --release --bin lattice
    echo ""
fi

# Show info
echo -e "${BLUE}Network Information:${NC}"
echo "  Chain ID:    1337"
echo "  RPC Port:    8545"
echo "  P2P Port:    30303"
echo "  Data Dir:    .citrate-devnet"
echo ""

echo -e "${BLUE}Pre-funded Treasury:${NC}"
echo "  Address:     0x0f572e5295c57f15886f9b263e2f6d2d6c7b5ec6"
echo "  Balance:     1,000,000 tokens"
echo ""

echo -e "${GREEN}Starting devnet...${NC}"
echo "  Block production: Every 2 seconds"
echo "  Consensus:        GhostDAG"
echo "  Mining rewards:   Enabled"
echo ""

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

# Run devnet
RUST_LOG=info ./target/release/lattice devnet