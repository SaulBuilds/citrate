#!/bin/bash

# Lattice v3 Genesis Startup Script
# This script automates the complete startup process from genesis

set -e  # Exit on error

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# Configuration
LATTICE_DIR="/Users/soleilklosowski/Downloads/lattice/lattice-v3"
CHAIN_ID=42069
RPC_PORT=8545
WS_PORT=8546
P2P_PORT=30303
GUI_RPC_PORT=18545
GUI_P2P_PORT=30304

# Logging functions
log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[✓]${NC} $1"
}

log_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

log_step() {
    echo -e "\n${CYAN}═══════════════════════════════════════════════════════════════${NC}"
    echo -e "${CYAN}STEP $1: $2${NC}"
    echo -e "${CYAN}═══════════════════════════════════════════════════════════════${NC}\n"
}

# Function to check if port is in use
check_port() {
    local port=$1
    if lsof -Pi :$port -sTCP:LISTEN -t >/dev/null 2>&1; then
        return 0  # Port is in use
    else
        return 1  # Port is free
    fi
}

# Function to wait for RPC to be ready
wait_for_rpc() {
    local port=$1
    local max_attempts=30
    local attempt=0
    
    log_info "Waiting for RPC on port $port to be ready..."
    
    while [ $attempt -lt $max_attempts ]; do
        if curl -s -X POST http://localhost:$port \
            -H "Content-Type: application/json" \
            -d '{"jsonrpc":"2.0","method":"eth_chainId","params":[],"id":1}' \
            2>/dev/null | grep -q "result"; then
            log_success "RPC on port $port is ready!"
            return 0
        fi
        
        sleep 1
        attempt=$((attempt + 1))
        echo -n "."
    done
    
    echo ""
    log_error "RPC on port $port failed to start after $max_attempts seconds"
    return 1
}

# Function to get block number
get_block_number() {
    local port=$1
    local result=$(curl -s -X POST http://localhost:$port \
        -H "Content-Type: application/json" \
        -d '{"jsonrpc":"2.0","method":"eth_blockNumber","params":[],"id":1}' \
        2>/dev/null | grep -o '"result":"[^"]*"' | sed 's/"result":"0x\([^"]*\)"/\1/')
    
    if [ -z "$result" ]; then
        echo "0"
    else
        echo $((16#$result))
    fi
}

# Function to get chain ID
get_chain_id() {
    local port=$1
    local result=$(curl -s -X POST http://localhost:$port \
        -H "Content-Type: application/json" \
        -d '{"jsonrpc":"2.0","method":"eth_chainId","params":[],"id":1}' \
        2>/dev/null | grep -o '"result":"[^"]*"' | sed 's/"result":"0x\([^"]*\)"/\1/')
    
    if [ -z "$result" ]; then
        echo "0"
    else
        echo $((16#$result))
    fi
}

# Main script starts here
clear
echo -e "${CYAN}╔══════════════════════════════════════════════════════════════════════╗${NC}"
echo -e "${CYAN}║                  LATTICE V3 GENESIS STARTUP SCRIPT                  ║${NC}"
echo -e "${CYAN}║                                                                      ║${NC}"
echo -e "${CYAN}║  This script will:                                                  ║${NC}"
echo -e "${CYAN}║  • Clean all existing state                                         ║${NC}"
echo -e "${CYAN}║  • Build the core node and GUI                                      ║${NC}"
echo -e "${CYAN}║  • Start a fresh testnet from genesis                               ║${NC}"
echo -e "${CYAN}║  • Launch the GUI and configure for testnet                         ║${NC}"
echo -e "${CYAN}║  • Verify everything is synced and working                          ║${NC}"
echo -e "${CYAN}╚══════════════════════════════════════════════════════════════════════╝${NC}\n"

# Check if we're in the right directory
if [ ! -d "$LATTICE_DIR" ]; then
    log_error "Lattice directory not found at $LATTICE_DIR"
    exit 1
fi

cd "$LATTICE_DIR"
log_info "Working directory: $(pwd)"

# Step 1: Kill existing processes
log_step "1" "STOPPING EXISTING PROCESSES"

log_info "Killing any existing Lattice processes..."
pkill -f "lattice" 2>/dev/null && log_success "Killed existing lattice processes" || log_info "No lattice processes running"
pkill -f "tauri" 2>/dev/null && log_success "Killed existing tauri processes" || log_info "No tauri processes running"

# Check if any critical ports are still in use
for port in $RPC_PORT $WS_PORT $P2P_PORT $GUI_RPC_PORT $GUI_P2P_PORT; do
    if check_port $port; then
        log_warning "Port $port is still in use, attempting to free it..."
        lsof -ti:$port | xargs kill -9 2>/dev/null || true
        sleep 1
    fi
done

log_success "All processes stopped"

# Step 2: Clean state
log_step "2" "CLEANING ALL STATE DATA"

log_info "Removing all blockchain state directories..."
rm -rf .lattice* 2>/dev/null && log_success "Removed .lattice directories" || log_info "No .lattice directories found"
rm -rf target/ 2>/dev/null && log_success "Removed target directory" || log_info "No target directory found"
rm -rf gui/lattice-core/src-tauri/gui-data 2>/dev/null && log_success "Removed GUI data" || log_info "No GUI data found"
rm -rf gui/lattice-core/.lattice-gui-temp 2>/dev/null && log_success "Removed GUI temp data" || log_info "No GUI temp data found"

log_info "Cleaning build artifacts..."
cargo clean 2>/dev/null && log_success "Cargo clean completed" || log_warning "Cargo clean had issues"

log_success "All state data cleaned"

# Step 3: Build everything
log_step "3" "BUILDING CORE NODE AND GUI"

log_info "Building core node (this may take a few minutes)..."
if cargo build --release -p lattice-node 2>&1 | tail -5; then
    log_success "Core node built successfully"
else
    log_error "Failed to build core node"
    exit 1
fi

log_info "Building CLI wallet..."
if cargo build --release -p lattice-wallet 2>&1 | tail -5; then
    log_success "CLI wallet built successfully"
else
    log_error "Failed to build CLI wallet"
    exit 1
fi

log_info "Building GUI (this may take several minutes)..."
cd gui/lattice-core

if [ ! -d "node_modules" ]; then
    log_info "Installing GUI dependencies..."
    npm install > /dev/null 2>&1 && log_success "Dependencies installed" || log_error "Failed to install dependencies"
fi

if npm run tauri:build 2>&1 | tail -10; then
    log_success "GUI built successfully"
else
    log_error "Failed to build GUI"
    exit 1
fi

cd "$LATTICE_DIR"

# Step 4: Start core testnet node
log_step "4" "STARTING CORE TESTNET NODE"

log_info "Creating testnet configuration..."
cat > /tmp/lattice_testnet_config.toml << EOF
[chain]
chain_id = $CHAIN_ID
genesis_hash = ""
block_time = 2
ghostdag_k = 18

[network]
p2p_port = $P2P_PORT
max_peers = 100
enable_discovery = true

[rpc]
enabled = true
listen_addr = "127.0.0.1:$RPC_PORT"
ws_port = $WS_PORT

[mining]
enabled = true
threads = 2

[storage]
data_dir = ".lattice-testnet"
EOF

log_info "Starting testnet node with fresh genesis..."
./scripts/start_testnet.sh > /tmp/lattice_core.log 2>&1 &
CORE_PID=$!

log_info "Core node PID: $CORE_PID"

# Wait for RPC to be ready
if wait_for_rpc $RPC_PORT; then
    log_success "Core node started successfully"
else
    log_error "Core node failed to start. Check /tmp/lattice_core.log for details"
    tail -20 /tmp/lattice_core.log
    exit 1
fi

# Step 5: Verify genesis block
log_step "5" "VERIFYING GENESIS BLOCK"

log_info "Checking block number..."
BLOCK_NUM=$(get_block_number $RPC_PORT)
if [ "$BLOCK_NUM" -eq 0 ] || [ "$BLOCK_NUM" -eq 1 ]; then
    log_success "Starting from genesis (block $BLOCK_NUM)"
else
    log_warning "Block number is $BLOCK_NUM (expected 0 or 1)"
fi

log_info "Checking chain ID..."
ACTUAL_CHAIN_ID=$(get_chain_id $RPC_PORT)
if [ "$ACTUAL_CHAIN_ID" -eq "$CHAIN_ID" ]; then
    log_success "Chain ID verified: $ACTUAL_CHAIN_ID (0x$(printf '%x' $ACTUAL_CHAIN_ID))"
else
    log_error "Chain ID mismatch! Expected $CHAIN_ID, got $ACTUAL_CHAIN_ID"
    exit 1
fi

# Step 6: Start GUI
log_step "6" "STARTING GUI APPLICATION"

log_info "Launching GUI in development mode..."
cd gui/lattice-core

# Start GUI in background
npm run tauri dev > /tmp/lattice_gui.log 2>&1 &
GUI_PID=$!

log_info "GUI PID: $GUI_PID"
log_info "Waiting for GUI to initialize (15 seconds)..."
sleep 15

if ps -p $GUI_PID > /dev/null; then
    log_success "GUI is running"
else
    log_error "GUI failed to start. Check /tmp/lattice_gui.log for details"
    tail -20 /tmp/lattice_gui.log
    exit 1
fi

cd "$LATTICE_DIR"

# Step 7: Display status
log_step "7" "SYSTEM STATUS"

echo -e "${GREEN}╔══════════════════════════════════════════════════════════════════════╗${NC}"
echo -e "${GREEN}║                        GENESIS STARTUP COMPLETE                      ║${NC}"
echo -e "${GREEN}╚══════════════════════════════════════════════════════════════════════╝${NC}\n"

echo -e "${CYAN}Core Node:${NC}"
echo -e "  • Status: ${GREEN}Running${NC} (PID: $CORE_PID)"
echo -e "  • Chain ID: $ACTUAL_CHAIN_ID (0x$(printf '%x' $ACTUAL_CHAIN_ID))"
echo -e "  • Current Block: $BLOCK_NUM"
echo -e "  • RPC: http://localhost:$RPC_PORT"
echo -e "  • WebSocket: ws://localhost:$WS_PORT"
echo -e "  • P2P: localhost:$P2P_PORT"
echo -e "  • Logs: /tmp/lattice_core.log"

echo -e "\n${CYAN}GUI Application:${NC}"
echo -e "  • Status: ${GREEN}Running${NC} (PID: $GUI_PID)"
echo -e "  • P2P: localhost:$GUI_P2P_PORT"
echo -e "  • Logs: /tmp/lattice_gui.log"

echo -e "\n${YELLOW}IMPORTANT NEXT STEPS:${NC}"
echo -e "1. In the GUI, go to Settings"
echo -e "2. Change Network dropdown from 'devnet' to 'testnet'"
echo -e "3. Click 'Connect Bootnodes' to sync with core node"
echo -e "4. Check DAG Explorer to see blocks appearing"

echo -e "\n${CYAN}Useful Commands:${NC}"
echo -e "• Check core logs: ${BLUE}tail -f /tmp/lattice_core.log${NC}"
echo -e "• Check GUI logs: ${BLUE}tail -f /tmp/lattice_gui.log${NC}"
echo -e "• Get latest block: ${BLUE}curl -X POST http://localhost:$RPC_PORT -H \"Content-Type: application/json\" -d '{\"jsonrpc\":\"2.0\",\"method\":\"eth_blockNumber\",\"params\":[],\"id\":1}'${NC}"
echo -e "• Stop everything: ${BLUE}pkill -f lattice && pkill -f tauri${NC}"

echo -e "\n${GREEN}System is ready! The GUI should be visible on your screen.${NC}"

# Keep script running and monitor
log_info "Press Ctrl+C to stop all processes..."

# Trap to clean up on exit
cleanup() {
    echo -e "\n${YELLOW}Shutting down...${NC}"
    kill $CORE_PID 2>/dev/null && log_success "Core node stopped"
    kill $GUI_PID 2>/dev/null && log_success "GUI stopped"
    log_success "Cleanup complete"
    exit 0
}

trap cleanup INT TERM

# Monitor loop
while true; do
    sleep 5
    
    # Check if processes are still running
    if ! ps -p $CORE_PID > /dev/null; then
        log_error "Core node has stopped unexpectedly!"
        tail -20 /tmp/lattice_core.log
        cleanup
    fi
    
    if ! ps -p $GUI_PID > /dev/null; then
        log_error "GUI has stopped unexpectedly!"
        tail -20 /tmp/lattice_gui.log
        cleanup
    fi
done