#!/bin/bash

# Script to fix lock issues and restart with proper sync

set -e

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

echo -e "${YELLOW}Fixing lock issues and restarting...${NC}"

# Kill all processes
echo -e "${BLUE}Stopping all Citrate processes...${NC}"
pkill -f "citrate" 2>/dev/null || true
pkill -f "tauri" 2>/dev/null || true
sleep 2

# Find and remove ALL lock files
echo -e "${BLUE}Cleaning up lock files...${NC}"

# GUI lock files
GUI_DATA_DIR="$HOME/Library/Application Support/citrate-core"
if [ -d "$GUI_DATA_DIR" ]; then
    find "$GUI_DATA_DIR" -name "LOCK" -o -name "*.lock" | while read lock; do
        echo "Removing: $lock"
        rm -f "$lock"
    done
fi

# Local lock files
find . -name "LOCK" -o -name "*.lock" 2>/dev/null | while read lock; do
    echo "Removing: $lock"
    rm -f "$lock"
done

# Clean testnet data for fresh sync
echo -e "${BLUE}Cleaning testnet data...${NC}"
rm -rf .citrate-testnet
rm -rf "$GUI_DATA_DIR/testnet"

echo -e "${GREEN}Lock files cleaned!${NC}"

# Start core node
echo -e "${BLUE}Starting core testnet node...${NC}"
cd /Users/soleilklosowski/Downloads/lattice/citrate
./scripts/start_testnet.sh > /tmp/citrate_core.log 2>&1 &
CORE_PID=$!

sleep 5

# Verify core node is running
if curl -s http://localhost:8545 -X POST -H "Content-Type: application/json" \
    -d '{"jsonrpc":"2.0","method":"eth_chainId","params":[],"id":1}' | grep -q result; then
    echo -e "${GREEN}Core node running (PID: $CORE_PID)${NC}"
else
    echo -e "${RED}Core node failed to start${NC}"
    tail -20 /tmp/citrate_core.log
    exit 1
fi

# Start GUI
echo -e "${BLUE}Starting GUI...${NC}"
cd gui/citrate-core
npm run tauri dev > /tmp/citrate_gui.log 2>&1 &
GUI_PID=$!

sleep 10

echo -e "${GREEN}═══════════════════════════════════════════════════════════════${NC}"
echo -e "${GREEN}System restarted successfully!${NC}"
echo -e "${GREEN}═══════════════════════════════════════════════════════════════${NC}"

echo -e "\n${YELLOW}IMPORTANT:${NC}"
echo -e "1. In GUI: Settings → Network → Select 'testnet'"
echo -e "2. Click 'Connect Bootnodes'"
echo -e "3. Watch sync progress in logs"

echo -e "\n${CYAN}Monitor sync progress:${NC}"
echo -e "tail -f /tmp/citrate_gui.log | grep -E 'Sync:|Stored|height'"

echo -e "\n${CYAN}Check DAG data:${NC}"
echo -e "tail -f /tmp/citrate_gui.log | grep 'DAG:'"

echo -e "\nPress Ctrl+C to stop..."

trap "kill $CORE_PID $GUI_PID 2>/dev/null; exit" INT
while true; do sleep 1; done