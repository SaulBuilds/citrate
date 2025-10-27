#!/bin/bash

# Citrate v3 Quick Genesis Startup Script
# This script quickly starts from genesis (assumes already built)

set -e  # Exit on error

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# Configuration
CITRATE_DIR="/Users/soleilklosowski/Downloads/lattice/citrate"
CHAIN_ID=42069

# Logging functions
log_info() { echo -e "${BLUE}[INFO]${NC} $1"; }
log_success() { echo -e "${GREEN}[✓]${NC} $1"; }
log_warning() { echo -e "${YELLOW}[WARNING]${NC} $1"; }
log_error() { echo -e "${RED}[ERROR]${NC} $1"; }

clear
echo -e "${CYAN}╔══════════════════════════════════════════════════════════════════════╗${NC}"
echo -e "${CYAN}║                 CITRATE V3 QUICK GENESIS STARTUP                    ║${NC}"
echo -e "${CYAN}╚══════════════════════════════════════════════════════════════════════╝${NC}\n"

cd "$CITRATE_DIR"

# Step 1: Clean and kill
log_info "Stopping existing processes..."
pkill -f "citrate" 2>/dev/null || true
pkill -f "tauri" 2>/dev/null || true
sleep 2

log_info "Cleaning state..."
rm -rf .lattice* 2>/dev/null || true
rm -rf gui/citrate-core/src-tauri/gui-data 2>/dev/null || true
find . -name "LOCK" -delete 2>/dev/null || true
log_success "Clean state ready"

# Step 2: Start core node
log_info "Starting core testnet node..."
./scripts/start_testnet.sh > /tmp/citrate_core.log 2>&1 &
CORE_PID=$!

# Wait for RPC
sleep 5
if curl -s http://localhost:8545 -X POST -H "Content-Type: application/json" \
    -d '{"jsonrpc":"2.0","method":"eth_chainId","params":[],"id":1}' | grep -q result; then
    log_success "Core node running (PID: $CORE_PID)"
else
    log_error "Core node failed to start"
    tail -10 /tmp/citrate_core.log
    exit 1
fi

# Step 3: Start GUI
log_info "Starting GUI..."
cd gui/citrate-core
npm run tauri dev > /tmp/citrate_gui.log 2>&1 &
GUI_PID=$!
cd "$CITRATE_DIR"

sleep 10
if ps -p $GUI_PID > /dev/null; then
    log_success "GUI running (PID: $GUI_PID)"
else
    log_error "GUI failed to start"
    exit 1
fi

echo -e "\n${GREEN}═══════════════════════════════════════════════════════════════${NC}"
echo -e "${GREEN}GENESIS STARTUP COMPLETE!${NC}"
echo -e "${GREEN}═══════════════════════════════════════════════════════════════${NC}\n"

echo -e "${YELLOW}NOW DO THIS IN THE GUI:${NC}"
echo -e "1. Go to Settings → Network"
echo -e "2. Change dropdown to 'testnet'"
echo -e "3. Click 'Connect Bootnodes'"
echo -e "4. Check DAG Explorer for blocks\n"

echo -e "${CYAN}Commands:${NC}"
echo -e "• Core logs: tail -f /tmp/citrate_core.log"
echo -e "• GUI logs: tail -f /tmp/citrate_gui.log"
echo -e "• Stop all: pkill -f lattice && pkill -f tauri\n"

echo "Press Ctrl+C to stop everything..."

# Cleanup on exit
trap "kill $CORE_PID $GUI_PID 2>/dev/null; echo 'Stopped.'; exit" INT TERM

# Keep running
while true; do
    sleep 5
    ps -p $CORE_PID > /dev/null || { echo "Core stopped!"; exit 1; }
    ps -p $GUI_PID > /dev/null || { echo "GUI stopped!"; exit 1; }
done