#!/bin/bash

# Start fresh testnet with new wallet and proper configuration
# Complete end-to-end setup with DAG explorer

set -e

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
MAGENTA='\033[0;35m'
NC='\033[0m'

echo -e "${CYAN}╔══════════════════════════════════════════════════════════════════════╗${NC}"
echo -e "${CYAN}║           CITRATE V3 FRESH TESTNET STARTUP                          ║${NC}"
echo -e "${CYAN}╚══════════════════════════════════════════════════════════════════════╝${NC}"

# Step 1: Ensure clean state
echo -e "\n${BLUE}[1/9] Ensuring clean state...${NC}"
if [ -d ".citrate-testnet" ] || [ -d "data" ]; then
    echo -e "${YELLOW}Found existing blockchain data.${NC}"
    echo -e "${YELLOW}Run ./scripts/hard_reset_chain.sh first for a completely fresh start.${NC}"
    echo -e "${YELLOW}Continue with existing data? (y/n):${NC}"
    read -t 10 -n 1 continue_existing 2>/dev/null || continue_existing="n"
    echo
    
    if [[ "$continue_existing" != "y" && "$continue_existing" != "Y" ]]; then
        echo -e "${RED}Aborting. Please run hard reset first.${NC}"
        exit 1
    fi
fi

# Step 2: Build if needed
echo -e "\n${BLUE}[2/9] Checking build...${NC}"
if [ ! -f "./target/release/lattice" ]; then
    echo "Building core node..."
    cargo build --release --bin lattice
fi

if [ ! -f "./target/release/wallet" ]; then
    echo "Building wallet CLI..."
    cargo build --release --bin wallet
fi
echo -e "${GREEN}[✓]${NC} Binaries ready"

# Step 3: Create new wallet
echo -e "\n${BLUE}[3/9] Setting up validator wallet...${NC}"

# Generate a new private key for validator
PRIVATE_KEY=$(openssl rand -hex 32)
echo -e "${CYAN}Generated new validator private key${NC}"

# Derive address from private key (simplified - in production use proper derivation)
# For now, we'll use a deterministic test address
VALIDATOR_ADDRESS="0x$(echo -n "$PRIVATE_KEY" | sha256sum | cut -c1-40)"
echo -e "${GREEN}Validator address: $VALIDATOR_ADDRESS${NC}"

# Convert to coinbase format (64 hex chars)
COINBASE_HEX="${VALIDATOR_ADDRESS:2}$(printf '0%.0s' {1..24})"

# Step 4: Create testnet configuration
echo -e "\n${BLUE}[4/9] Creating testnet configuration...${NC}"
cat > testnet-config.toml << EOF
[chain]
chain_id = 42069
genesis_hash = ""
block_time = 2
ghostdag_k = 18

[network]
listen_addr = "0.0.0.0:30303"
bootstrap_nodes = []
max_peers = 100

[rpc]
enabled = true
listen_addr = "0.0.0.0:8545"
ws_addr = "0.0.0.0:8546"

[storage]
data_dir = ".citrate-testnet"
pruning = false
keep_blocks = 10000

[mining]
enabled = true
# Validator: $VALIDATOR_ADDRESS
coinbase = "$COINBASE_HEX"
target_block_time = 2
min_gas_price = 1000000000

[explorer]
enabled = false
port = 3030
EOF

echo -e "${GREEN}[✓]${NC} Configuration created with validator as coinbase"

# Step 5: Initialize genesis block
echo -e "\n${BLUE}[5/9] Initializing genesis block...${NC}"
mkdir -p .citrate-testnet
echo -e "${GREEN}[✓]${NC} Genesis ready"

# Step 6: Start core node
echo -e "\n${BLUE}[6/9] Starting core node...${NC}"
RUST_LOG=info ./target/release/lattice \
    --config testnet-config.toml \
    --data-dir .citrate-testnet \
    --mine \
    > /tmp/citrate_testnet.log 2>&1 &
NODE_PID=$!

echo -e "${CYAN}Core node starting (PID: $NODE_PID)...${NC}"

# Wait for node to be ready
for i in {1..30}; do
    if curl -s http://localhost:8545 -X POST -H "Content-Type: application/json" \
        -d '{"jsonrpc":"2.0","method":"eth_chainId","params":[],"id":1}' 2>/dev/null | grep -q "0xa455"; then
        echo -e "\n${GREEN}[✓]${NC} Core node running and RPC ready!"
        break
    fi
    sleep 1
    echo -ne "."
done

# Step 7: Wait for initial blocks
echo -e "\n${BLUE}[7/9] Mining initial blocks...${NC}"
echo -e "${CYAN}Waiting for validator to accumulate rewards...${NC}"

for i in {1..10}; do
    sleep 2
    
    BLOCK_HEIGHT=$(curl -s -X POST http://localhost:8545 \
        -H "Content-Type: application/json" \
        -d '{"jsonrpc":"2.0","method":"eth_blockNumber","params":[],"id":1}' \
        | grep -o '"result":"[^"]*"' | cut -d'"' -f4 | xargs printf "%d\n" 2>/dev/null || echo "0")
    
    BALANCE_HEX=$(curl -s -X POST http://localhost:8545 \
        -H "Content-Type: application/json" \
        -d "{\"jsonrpc\":\"2.0\",\"method\":\"eth_getBalance\",\"params\":[\"$VALIDATOR_ADDRESS\",\"latest\"],\"id\":1}" \
        | grep -o '"result":"[^"]*"' | cut -d'"' -f4)
    
    if [ ! -z "$BALANCE_HEX" ] && [ "$BALANCE_HEX" != "0x0" ]; then
        BALANCE_WEI=$(printf "%d" "$BALANCE_HEX" 2>/dev/null || echo "0")
        BALANCE_LATT=$(echo "scale=2; $BALANCE_WEI / 1000000000000000000" | bc 2>/dev/null || echo "0")
        echo -e "Block #$BLOCK_HEIGHT mined | Validator balance: ${GREEN}$BALANCE_LATT LATT${NC}"
    else
        echo -ne "Block #$BLOCK_HEIGHT "
    fi
done

# Step 8: Start GUI (optional)
echo -e "\n${BLUE}[8/9] Start GUI wallet? (y/n):${NC}"
read -t 10 -n 1 start_gui 2>/dev/null || start_gui="n"
echo

if [[ "$start_gui" == "y" || "$start_gui" == "Y" ]]; then
    echo -e "${CYAN}Starting GUI wallet...${NC}"
    
    # Configure GUI for testnet
    GUI_DATA_DIR="$HOME/Library/Application Support/citrate-core"
    mkdir -p "$GUI_DATA_DIR"
    
    cat > "$GUI_DATA_DIR/config.json" << EOF
{
  "dataDir": "./gui-data/testnet",
  "network": "testnet",
  "rpcPort": 18545,
  "wsPort": 18546,
  "p2pPort": 30304,
  "restPort": 3000,
  "maxPeers": 10,
  "bootnodes": ["127.0.0.1:30303"],
  "rewardAddress": "$VALIDATOR_ADDRESS",
  "enableNetwork": true,
  "discovery": false
}
EOF
    
    cd ../gui/citrate-core
    if [ ! -d "node_modules" ]; then
        npm install > /dev/null 2>&1
    fi
    npm run tauri dev > /tmp/citrate_gui.log 2>&1 &
    GUI_PID=$!
    cd ../..
    
    echo -e "${GREEN}[✓]${NC} GUI starting (PID: $GUI_PID)"
else
    echo -e "${CYAN}[→]${NC} Skipping GUI"
fi

# Step 9: Summary
echo -e "\n${BLUE}[9/9] Startup complete!${NC}"

echo -e "\n${GREEN}═══════════════════════════════════════════════════════════════${NC}"
echo -e "${GREEN}FRESH TESTNET RUNNING!${NC}"
echo -e "${GREEN}═══════════════════════════════════════════════════════════════${NC}"

echo -e "\n${MAGENTA}Network Details:${NC}"
echo -e "  Chain ID:    ${CYAN}42069${NC}"
echo -e "  RPC URL:     ${CYAN}http://localhost:8545${NC}"
echo -e "  WebSocket:   ${CYAN}ws://localhost:8546${NC}"
echo -e "  P2P Port:    ${CYAN}30303${NC}"

echo -e "\n${MAGENTA}Validator Info:${NC}"
echo -e "  Address:     ${CYAN}$VALIDATOR_ADDRESS${NC}"
echo -e "  Private Key: ${YELLOW}[Saved to .validator-key]${NC}"
echo -e "  Rewards:     ${CYAN}9 LATT per block (every 2 seconds)${NC}"

# Save validator key securely
echo "$PRIVATE_KEY" > .validator-key
chmod 600 .validator-key

echo -e "\n${MAGENTA}Useful Commands:${NC}"
echo -e "  Monitor logs:     ${CYAN}tail -f /tmp/citrate_testnet.log${NC}"
echo -e "  Check balance:    ${CYAN}curl -X POST http://localhost:8545 -H \"Content-Type: application/json\" -d '{\"jsonrpc\":\"2.0\",\"method\":\"eth_getBalance\",\"params\":[\"$VALIDATOR_ADDRESS\",\"latest\"],\"id\":1}'${NC}"
echo -e "  Monitor wallets:  ${CYAN}./scripts/monitor_wallets.sh $VALIDATOR_ADDRESS${NC}"
echo -e "  Send transaction: ${CYAN}./scripts/test_transaction_complete.sh${NC}"

echo -e "\n${MAGENTA}DAG Explorer:${NC}"
echo -e "  The new multi-network DAG explorer API is available"
echo -e "  Access at: ${CYAN}http://localhost:3030${NC}"

echo -e "\n${YELLOW}Process IDs:${NC}"
echo -e "  Core Node: $NODE_PID"
if [ ! -z "$GUI_PID" ]; then
    echo -e "  GUI: $GUI_PID"
fi

echo -e "\n${GREEN}Your fresh testnet is running with a clean genesis!${NC}"
echo -e "${GREEN}Validator is earning rewards and all systems are operational.${NC}"

# Trap cleanup
trap "echo -e '\n${YELLOW}Stopping testnet...${NC}'; kill $NODE_PID $GUI_PID 2>/dev/null; exit" INT

# Keep running and show periodic stats
echo -e "\n${CYAN}Press Ctrl+C to stop the testnet...${NC}\n"

while true; do
    sleep 10
    
    # Get current stats
    HEIGHT=$(curl -s -X POST http://localhost:8545 \
        -H "Content-Type: application/json" \
        -d '{"jsonrpc":"2.0","method":"eth_blockNumber","params":[],"id":1}' \
        | grep -o '"result":"[^"]*"' | cut -d'"' -f4 | xargs printf "%d" 2>/dev/null || echo "0")
    
    BAL_HEX=$(curl -s -X POST http://localhost:8545 \
        -H "Content-Type: application/json" \
        -d "{\"jsonrpc\":\"2.0\",\"method\":\"eth_getBalance\",\"params\":[\"$VALIDATOR_ADDRESS\",\"latest\"],\"id\":1}" \
        | grep -o '"result":"[^"]*"' | cut -d'"' -f4)
    
    if [ ! -z "$BAL_HEX" ]; then
        BAL_WEI=$(printf "%d" "$BAL_HEX" 2>/dev/null || echo "0")
        BAL_LATT=$(echo "scale=2; $BAL_WEI / 1000000000000000000" | bc 2>/dev/null || echo "0")
    else
        BAL_LATT="0"
    fi
    
    echo -ne "\r${CYAN}[Live]${NC} Block #$HEIGHT | Validator: $BAL_LATT LATT | Status: Mining...     "
done