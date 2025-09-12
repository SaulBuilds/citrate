#!/bin/bash

# Lattice v3 GUI Testnet Connection Script
# This script helps connect the GUI to an external testnet node

set -e

SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Default configuration
TESTNET_RPC="http://localhost:8545"
TESTNET_WS="ws://localhost:8546"
CHAIN_ID=42069

echo "========================================="
echo "  Lattice v3 GUI Testnet Connector"
echo "========================================="
echo ""

# Function to check if testnet is running
check_testnet() {
    echo -e "${YELLOW}Checking testnet connection...${NC}"
    
    # Try to get chain ID from RPC
    response=$(curl -s -X POST $TESTNET_RPC \
        -H "Content-Type: application/json" \
        -d '{"jsonrpc":"2.0","method":"eth_chainId","params":[],"id":1}' 2>/dev/null || echo "")
    
    if [ -z "$response" ]; then
        return 1
    fi
    
    # Extract chain ID from response
    chain_id_hex=$(echo $response | grep -o '"result":"[^"]*"' | cut -d'"' -f4)
    if [ -z "$chain_id_hex" ]; then
        return 1
    fi
    
    # Convert hex to decimal
    chain_id_dec=$((16#${chain_id_hex#0x}))
    
    if [ "$chain_id_dec" -eq "$CHAIN_ID" ]; then
        return 0
    else
        echo -e "${RED}Warning: Chain ID mismatch. Expected $CHAIN_ID, got $chain_id_dec${NC}"
        return 1
    fi
}

# Function to start testnet if not running
start_testnet_if_needed() {
    if ! check_testnet; then
        echo -e "${YELLOW}Testnet not running. Starting testnet...${NC}"
        
        # Start testnet in background
        "$PROJECT_ROOT/scripts/start_testnet.sh" &
        TESTNET_PID=$!
        
        # Wait for testnet to be ready
        echo -e "${YELLOW}Waiting for testnet to start...${NC}"
        for i in {1..30}; do
            sleep 2
            if check_testnet; then
                echo -e "${GREEN}Testnet started successfully!${NC}"
                return 0
            fi
        done
        
        echo -e "${RED}Failed to start testnet${NC}"
        return 1
    else
        echo -e "${GREEN}Testnet is already running${NC}"
        return 0
    fi
}

# Function to build GUI if needed
build_gui_if_needed() {
    if [ ! -d "$PROJECT_ROOT/gui/lattice-core/node_modules" ]; then
        echo -e "${YELLOW}Installing GUI dependencies...${NC}"
        cd "$PROJECT_ROOT/gui/lattice-core"
        npm install
    fi
}

# Main execution
echo -e "${BLUE}Step 1: Checking testnet status${NC}"
if ! start_testnet_if_needed; then
    echo -e "${RED}Cannot proceed without testnet. Please start testnet manually:${NC}"
    echo "  ./scripts/start_testnet.sh"
    exit 1
fi

echo ""
echo -e "${BLUE}Step 2: Getting testnet info${NC}"

# Get current block number
block_response=$(curl -s -X POST $TESTNET_RPC \
    -H "Content-Type: application/json" \
    -d '{"jsonrpc":"2.0","method":"eth_blockNumber","params":[],"id":1}')

block_hex=$(echo $block_response | grep -o '"result":"[^"]*"' | cut -d'"' -f4)
if [ ! -z "$block_hex" ]; then
    block_number=$((16#${block_hex#0x}))
    echo -e "${GREEN}Current block: $block_number${NC}"
fi

echo ""
echo -e "${BLUE}Step 3: Preparing GUI${NC}"
build_gui_if_needed

echo ""
echo "========================================="
echo -e "${GREEN}Testnet Connection Info:${NC}"
echo "========================================="
echo "RPC Endpoint:  $TESTNET_RPC"
echo "WebSocket:     $TESTNET_WS"
echo "Chain ID:      $CHAIN_ID (0x$(printf '%x' $CHAIN_ID))"
echo ""
echo -e "${YELLOW}To connect GUI to testnet:${NC}"
echo ""
echo "1. Start the GUI:"
echo "   cd gui/lattice-core && npm run tauri dev"
echo ""
echo "2. In the GUI, connect to external testnet:"
echo "   - Open developer console (F12)"
echo "   - Run this command:"
echo "   await window.__TAURI__.invoke('connect_to_external_testnet', {"
echo "     rpcUrl: '$TESTNET_RPC'"
echo "   })"
echo ""
echo "3. Or use the GUI's network settings (if available) to:"
echo "   - Select 'External RPC' mode"
echo "   - Enter RPC URL: $TESTNET_RPC"
echo "   - Click 'Connect'"
echo ""
echo -e "${GREEN}Once connected, transactions from the GUI will be sent${NC}"
echo -e "${GREEN}to the testnet node instead of the embedded node.${NC}"
echo ""
echo "========================================="
echo ""

# Option to start GUI
read -p "Do you want to start the GUI now? (y/n) " -n 1 -r
echo
if [[ $REPLY =~ ^[Yy]$ ]]; then
    echo -e "${YELLOW}Starting GUI...${NC}"
    cd "$PROJECT_ROOT/gui/lattice-core"
    npm run tauri dev
fi