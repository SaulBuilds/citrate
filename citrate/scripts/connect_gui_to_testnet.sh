#!/bin/bash

# Connect GUI to running testnet node
# This configures the GUI to sync blocks from the testnet

set -e

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
MAGENTA='\033[0;35m'
NC='\033[0m'

echo -e "${CYAN}╔══════════════════════════════════════════════════════════════════════╗${NC}"
echo -e "${CYAN}║           CONNECT GUI TO TESTNET NODE                               ║${NC}"
echo -e "${CYAN}╚══════════════════════════════════════════════════════════════════════╝${NC}"

# Step 1: Check if testnet is running
echo -e "\n${BLUE}[1/4] Checking testnet node...${NC}"
if ! curl -s http://localhost:8545 -X POST -H "Content-Type: application/json" \
    -d '{"jsonrpc":"2.0","method":"eth_chainId","params":[],"id":1}' 2>/dev/null | grep -q "0xa455"; then
    echo -e "${RED}[✗] Testnet node not running on port 8545${NC}"
    echo -e "${YELLOW}Please start the testnet first:${NC}"
    echo -e "  ./scripts/start_fresh_testnet.sh"
    exit 1
fi

HEIGHT=$(curl -s -X POST http://localhost:8545 \
    -H "Content-Type: application/json" \
    -d '{"jsonrpc":"2.0","method":"eth_blockNumber","params":[],"id":1}' \
    | grep -o '"result":"[^"]*"' | cut -d'"' -f4 | xargs printf "%d\n" 2>/dev/null || echo "0")

echo -e "${GREEN}[✓]${NC} Testnet running at block #$HEIGHT"

# Step 2: Configure GUI for testnet connection
echo -e "\n${BLUE}[2/4] Configuring GUI for testnet mode...${NC}"

GUI_CONFIG_DIR="$HOME/Library/Application Support/citrate-core"
mkdir -p "$GUI_CONFIG_DIR"

# Create configuration that connects to testnet as a peer
cat > "$GUI_CONFIG_DIR/testnet-config.json" << EOF
{
  "dataDir": "./gui-data/testnet",
  "network": "testnet",
  "rpcPort": 18545,
  "wsPort": 18546,
  "p2pPort": 30304,
  "restPort": 3001,
  "maxPeers": 10,
  "bootnodes": ["127.0.0.1:30303"],
  "rewardAddress": null,
  "enableNetwork": true,
  "discovery": false,
  "externalRpc": "http://localhost:8545",
  "consensus": {
    "blockTimeSeconds": 2,
    "blueScoreK": 18
  },
  "mempool": {
    "chainId": 42069
  }
}
EOF

echo -e "${GREEN}[✓]${NC} Configuration created"

# Step 3: Clean GUI data to force fresh sync
echo -e "\n${BLUE}[3/4] Cleaning GUI blockchain data...${NC}"
rm -rf "$GUI_CONFIG_DIR/testnet" 2>/dev/null || true
rm -rf "$GUI_CONFIG_DIR/gui-data" 2>/dev/null || true
rm -f "$GUI_CONFIG_DIR/db.sqlite" 2>/dev/null || true
echo -e "${GREEN}[✓]${NC} GUI data cleaned"

# Step 4: Start GUI with testnet connection
echo -e "\n${BLUE}[4/4] Starting GUI...${NC}"

cd gui/citrate-core

# Check if dependencies are installed
if [ ! -d "node_modules" ]; then
    echo "Installing dependencies..."
    npm install > /dev/null 2>&1
fi

echo -e "${CYAN}Starting GUI in testnet mode...${NC}"
echo -e "${YELLOW}The GUI will:${NC}"
echo -e "  1. Connect to testnet node at localhost:30303 as a peer"
echo -e "  2. Sync blocks from the testnet"
echo -e "  3. Display the DAG visualization with synced data"
echo -e ""
echo -e "${GREEN}Starting GUI now...${NC}"

# Set environment to use testnet config
export CITRATE_GUI_CONFIG="$GUI_CONFIG_DIR/testnet-config.json"
export CITRATE_NETWORK="testnet"
export CITRATE_BOOTNODES="127.0.0.1:30303"

npm run tauri dev