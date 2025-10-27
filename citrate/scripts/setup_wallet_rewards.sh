#!/bin/bash

# Complete wallet and rewards setup for Citrate V3
# This script ensures proper token distribution and wallet integration

set -e

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
MAGENTA='\033[0;35m'
NC='\033[0m'

echo -e "${CYAN}╔══════════════════════════════════════════════════════════════════════╗${NC}"
echo -e "${CYAN}║           CITRATE V3 WALLET & REWARDS SETUP                         ║${NC}"
echo -e "${CYAN}╚══════════════════════════════════════════════════════════════════════╝${NC}"

# Default test wallet addresses (these would normally come from GUI wallet creation)
DEFAULT_VALIDATOR="0x48a3f6698a1384e6f97ae9cc2f6d6c94504ba8a5"
DEFAULT_RECEIVER="0x92b3f6698a1384e6f97ae9cc2f6d6c94504ba8a6"
TREASURY_ADDRESS="0x1111111111111111111111111111111111111111"

# Function to convert address to 64-char hex for coinbase
address_to_coinbase() {
    local addr=$1
    # Remove 0x prefix if present
    addr=${addr#0x}
    # Pad to 64 characters with zeros
    printf "%s%0$((64-${#addr}))s" "$addr" | tr ' ' '0'
}

# Step 1: Kill existing processes
echo -e "\n${BLUE}[1/8] Stopping existing processes...${NC}"
pkill -f "citrate" 2>/dev/null || true
pkill -f "tauri" 2>/dev/null || true
sleep 2
echo -e "${GREEN}[✓]${NC} All processes stopped"

# Step 2: Clean state
echo -e "\n${BLUE}[2/8] Cleaning state and preparing fresh environment...${NC}"
rm -rf .citrate-testnet 2>/dev/null || true
rm -rf .citrate-devnet 2>/dev/null || true
GUI_DATA_DIR="$HOME/Library/Application Support/citrate-core"
if [ -d "$GUI_DATA_DIR" ]; then
    find "$GUI_DATA_DIR" -name "LOCK" -o -name "*.lock" 2>/dev/null | while read lock; do
        rm -f "$lock"
    done
    rm -rf "$GUI_DATA_DIR/testnet" 2>/dev/null || true
fi
echo -e "${GREEN}[✓]${NC} State cleaned"

# Step 3: Build with latest fixes
echo -e "\n${BLUE}[3/8] Building core node with reward fixes...${NC}"
cargo build --release --bin lattice 2>&1 | tail -5
echo -e "${GREEN}[✓]${NC} Core node built"

# Step 4: Create wallet-aware config
echo -e "\n${BLUE}[4/8] Creating wallet-aware testnet configuration...${NC}"

# Ask user if they want to use their own address
echo -e "${YELLOW}Do you want to set a custom validator address? (y/n)${NC}"
read -t 10 -n 1 use_custom 2>/dev/null || use_custom="n"
echo

if [[ "$use_custom" == "y" || "$use_custom" == "Y" ]]; then
    echo -e "${CYAN}Enter your wallet address (e.g., 0x...):${NC}"
    read validator_address
    if [[ ! "$validator_address" =~ ^0x[0-9a-fA-F]{40}$ ]]; then
        echo -e "${RED}Invalid address format. Using default.${NC}"
        validator_address=$DEFAULT_VALIDATOR
    fi
else
    validator_address=$DEFAULT_VALIDATOR
    echo -e "${CYAN}Using default test validator: $validator_address${NC}"
fi

# Convert to coinbase format
coinbase_hex=$(address_to_coinbase "$validator_address")
echo -e "${GREEN}[✓]${NC} Validator configured: $validator_address"

# Create config with proper validator
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
# Validator address: $validator_address
coinbase = "$coinbase_hex"
target_block_time = 2
min_gas_price = 1000000000
EOF

echo -e "${GREEN}[✓]${NC} Configuration created with proper validator"

# Step 5: Start core node with rewards
echo -e "\n${BLUE}[5/8] Starting core node with reward distribution...${NC}"
RUST_LOG=info ./target/release/lattice --config testnet-config.toml --data-dir .citrate-testnet --mine > /tmp/citrate_core.log 2>&1 &
CORE_PID=$!

sleep 5

# Verify core is running
if curl -s http://localhost:8545 -X POST -H "Content-Type: application/json" \
    -d '{"jsonrpc":"2.0","method":"eth_chainId","params":[],"id":1}' | grep -q "0xa455"; then
    echo -e "${GREEN}[✓]${NC} Core node running with rewards to $validator_address"
else
    echo -e "${RED}[✗]${NC} Core node failed to start"
    tail -20 /tmp/citrate_core.log
    exit 1
fi

# Step 6: Monitor initial balance
echo -e "\n${BLUE}[6/8] Monitoring validator balance accrual...${NC}"
echo -e "${CYAN}Waiting for first blocks to be mined...${NC}"

for i in {1..10}; do
    sleep 2
    
    # Check validator balance
    balance_hex=$(curl -s -X POST http://localhost:8545 \
        -H "Content-Type: application/json" \
        -d "{\"jsonrpc\":\"2.0\",\"method\":\"eth_getBalance\",\"params\":[\"$validator_address\",\"latest\"],\"id\":1}" \
        | grep -o '"result":"[^"]*"' | cut -d'"' -f4)
    
    if [ ! -z "$balance_hex" ] && [ "$balance_hex" != "0x0" ]; then
        balance_wei=$(printf "%d" "$balance_hex" 2>/dev/null || echo "0")
        balance_latt=$(echo "scale=2; $balance_wei / 1000000000000000000" | bc 2>/dev/null || echo "0")
        echo -e "${GREEN}Block #$i mined! Validator balance: $balance_latt LATT${NC}"
    else
        echo -ne "."
    fi
done

# Step 7: Check treasury accumulation
echo -e "\n${BLUE}[7/8] Checking treasury accumulation...${NC}"
treasury_balance=$(curl -s -X POST http://localhost:8545 \
    -H "Content-Type: application/json" \
    -d "{\"jsonrpc\":\"2.0\",\"method\":\"eth_getBalance\",\"params\":[\"$TREASURY_ADDRESS\",\"latest\"],\"id\":1}" \
    | grep -o '"result":"[^"]*"' | cut -d'"' -f4)

if [ ! -z "$treasury_balance" ] && [ "$treasury_balance" != "0x0" ]; then
    treasury_wei=$(printf "%d" "$treasury_balance" 2>/dev/null || echo "0")
    treasury_latt=$(echo "scale=2; $treasury_wei / 1000000000000000000" | bc 2>/dev/null || echo "0")
    echo -e "${GREEN}[✓]${NC} Treasury balance: $treasury_latt LATT"
else
    echo -e "${YELLOW}[!]${NC} Treasury balance still 0 (may need more blocks)"
fi

# Step 8: Build and start GUI
echo -e "\n${BLUE}[8/8] Building and starting GUI wallet...${NC}"
cd gui/citrate-core

# Configure GUI for testnet with reward address
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
  "rewardAddress": "$validator_address",
  "enableNetwork": true,
  "discovery": false
}
EOF

# Quick dependency check
if [ ! -d "node_modules" ]; then
    echo "Installing dependencies..."
    npm install > /dev/null 2>&1
fi

npm run tauri dev > /tmp/citrate_gui.log 2>&1 &
GUI_PID=$!

echo -e "${GREEN}[✓]${NC} GUI wallet starting (PID: $GUI_PID)"

# Wait for GUI
echo -e "\n${CYAN}Waiting for GUI to compile and connect...${NC}"
for i in {1..120}; do
    if grep -q "Setting up node components" /tmp/citrate_gui.log 2>/dev/null; then
        echo -e "\n${GREEN}[✓]${NC} GUI compiled and running!"
        break
    fi
    if [ $((i % 10)) -eq 0 ]; then
        echo -ne "."
    fi
    sleep 1
done

echo -e "\n${GREEN}═══════════════════════════════════════════════════════════════${NC}"
echo -e "${GREEN}WALLET & REWARDS SYSTEM ACTIVE!${NC}"
echo -e "${GREEN}═══════════════════════════════════════════════════════════════${NC}"

echo -e "\n${MAGENTA}WALLET ADDRESSES:${NC}"
echo -e "Validator (receiving rewards): ${CYAN}$validator_address${NC}"
echo -e "Treasury (10% of rewards):     ${CYAN}$TREASURY_ADDRESS${NC}"
echo -e "Test receiver:                 ${CYAN}$DEFAULT_RECEIVER${NC}"

echo -e "\n${MAGENTA}TOKEN DISTRIBUTION:${NC}"
echo -e "• Every 2 seconds: New block produced"
echo -e "• Validator receives: 9 LATT (90% of 10 LATT reward)"
echo -e "• Treasury receives:  1 LATT (10% of 10 LATT reward)"

echo -e "\n${YELLOW}MONITORING COMMANDS:${NC}"
echo -e "Watch validator balance:"
echo -e "  ${CYAN}while true; do curl -s -X POST http://localhost:8545 -H \"Content-Type: application/json\" -d '{\"jsonrpc\":\"2.0\",\"method\":\"eth_getBalance\",\"params\":[\"$validator_address\",\"latest\"],\"id\":1}' | jq -r '.result' | xargs printf \"Balance: %d wei\\\\n\"; sleep 2; done${NC}"

echo -e "\nWatch block production:"
echo -e "  ${CYAN}tail -f /tmp/citrate_core.log | grep -E \"Produced block|Minted.*validator\"${NC}"

echo -e "\nTest transaction (after wallet has balance):"
echo -e "  ${CYAN}cd ../.. && ./scripts/send_test_tx.sh --from $validator_address --to $DEFAULT_RECEIVER --amount 1${NC}"

echo -e "\n${YELLOW}IN THE GUI:${NC}"
echo -e "1. Your wallet should show increasing balance"
echo -e "2. DAG Explorer should display growing chain"
echo -e "3. You can send transactions once balance accrues"

echo -e "\n${YELLOW}Press Ctrl+C to stop everything...${NC}"

# Trap cleanup
trap "echo -e '\n${YELLOW}Stopping...${NC}'; kill $CORE_PID $GUI_PID 2>/dev/null; exit" INT

# Monitor loop
while true; do
    sleep 5
    
    # Check processes
    if ! ps -p $CORE_PID > /dev/null 2>&1; then
        echo -e "${RED}Core node stopped!${NC}"
        tail -20 /tmp/citrate_core.log
        break
    fi
    
    if ! ps -p $GUI_PID > /dev/null 2>&1; then
        echo -e "${RED}GUI stopped!${NC}"
        tail -20 /tmp/citrate_gui.log
        break
    fi
    
    # Get current stats
    BLOCK_HEIGHT=$(curl -s -X POST http://localhost:8545 \
        -H "Content-Type: application/json" \
        -d '{"jsonrpc":"2.0","method":"eth_blockNumber","params":[],"id":1}' \
        | grep -o '"result":"[^"]*"' | cut -d'"' -f4 | xargs printf "%d\n" 2>/dev/null || echo "0")
    
    BALANCE_HEX=$(curl -s -X POST http://localhost:8545 \
        -H "Content-Type: application/json" \
        -d "{\"jsonrpc\":\"2.0\",\"method\":\"eth_getBalance\",\"params\":[\"$validator_address\",\"latest\"],\"id\":1}" \
        | grep -o '"result":"[^"]*"' | cut -d'"' -f4)
    
    if [ ! -z "$BALANCE_HEX" ]; then
        BALANCE_WEI=$(printf "%d" "$BALANCE_HEX" 2>/dev/null || echo "0")
        BALANCE_LATT=$(echo "scale=2; $BALANCE_WEI / 1000000000000000000" | bc 2>/dev/null || echo "0")
    else
        BALANCE_LATT="0"
    fi
    
    echo -ne "\r${CYAN}[Live]${NC} Block #${BLOCK_HEIGHT} | Validator: ${BALANCE_LATT} LATT | Status: Mining...  "
done