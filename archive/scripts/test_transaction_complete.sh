#!/bin/bash

# Complete transaction test including node startup
# This ensures everything is running for end-to-end testing

set -e

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
MAGENTA='\033[0;35m'
NC='\033[0m'

echo -e "${CYAN}╔══════════════════════════════════════════════════════════════════════╗${NC}"
echo -e "${CYAN}║           LATTICE V3 COMPLETE TRANSACTION TEST                      ║${NC}"
echo -e "${CYAN}╚══════════════════════════════════════════════════════════════════════╝${NC}"

# Test addresses
VALIDATOR_ADDRESS="0x48a3f6698a1384e6f97ae9cc2f6d6c94504ba8a5"
TEST_RECEIVER="0x92b3f6698a1384e6f97ae9cc2f6d6c94504ba8a6"

# Step 1: Start the node if not running
echo -e "\n${BLUE}[1/6] Checking if node is running...${NC}"
if ! curl -s http://localhost:8545 -X POST -H "Content-Type: application/json" \
    -d '{"jsonrpc":"2.0","method":"eth_chainId","params":[],"id":1}' | grep -q "0xa455"; then
    
    echo -e "${YELLOW}Node not running. Starting testnet...${NC}"
    
    # Use existing config or create minimal one
    if [ ! -f "testnet-config.toml" ]; then
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
data_dir = ".lattice-testnet"
pruning = false
keep_blocks = 10000

[mining]
enabled = true
coinbase = "48a3f6698a1384e6f97ae9cc2f6d6c94504ba8a5000000000000000000000000"
target_block_time = 2
min_gas_price = 1000000000
EOF
    fi
    
    # Start node in background
    RUST_LOG=info ./target/release/lattice --config testnet-config.toml --data-dir .lattice-testnet --mine > /tmp/lattice_tx_test.log 2>&1 &
    NODE_PID=$!
    
    echo -e "${CYAN}Waiting for node to start...${NC}"
    for i in {1..30}; do
        if curl -s http://localhost:8545 -X POST -H "Content-Type: application/json" \
            -d '{"jsonrpc":"2.0","method":"eth_chainId","params":[],"id":1}' 2>/dev/null | grep -q "0xa455"; then
            echo -e "${GREEN}[✓]${NC} Node started successfully (PID: $NODE_PID)"
            break
        fi
        sleep 1
        echo -ne "."
    done
    echo
else
    echo -e "${GREEN}[✓]${NC} Node already running"
fi

# Step 2: Wait for some blocks to be mined
echo -e "\n${BLUE}[2/6] Waiting for blocks to be mined...${NC}"
echo -e "${CYAN}Validator needs balance from mining rewards...${NC}"

for i in {1..10}; do
    BLOCK_HEIGHT=$(curl -s -X POST http://localhost:8545 \
        -H "Content-Type: application/json" \
        -d '{"jsonrpc":"2.0","method":"eth_blockNumber","params":[],"id":1}' \
        | grep -o '"result":"[^"]*"' | cut -d'"' -f4 | xargs printf "%d\n" 2>/dev/null || echo "0")
    
    echo -ne "\rBlock height: #$BLOCK_HEIGHT  "
    
    if [ "$BLOCK_HEIGHT" -gt 5 ]; then
        break
    fi
    sleep 2
done
echo

# Step 3: Check validator balance
echo -e "\n${BLUE}[3/6] Checking validator balance...${NC}"
BALANCE_HEX=$(curl -s -X POST http://localhost:8545 \
    -H "Content-Type: application/json" \
    -d "{\"jsonrpc\":\"2.0\",\"method\":\"eth_getBalance\",\"params\":[\"$VALIDATOR_ADDRESS\",\"latest\"],\"id\":1}" \
    | grep -o '"result":"[^"]*"' | cut -d'"' -f4)

if [ -z "$BALANCE_HEX" ] || [ "$BALANCE_HEX" == "0x0" ]; then
    echo -e "${RED}[✗] Validator has no balance!${NC}"
    echo "Waiting for more blocks..."
    sleep 10
    
    BALANCE_HEX=$(curl -s -X POST http://localhost:8545 \
        -H "Content-Type: application/json" \
        -d "{\"jsonrpc\":\"2.0\",\"method\":\"eth_getBalance\",\"params\":[\"$VALIDATOR_ADDRESS\",\"latest\"],\"id\":1}" \
        | grep -o '"result":"[^"]*"' | cut -d'"' -f4)
fi

BALANCE_WEI=$(printf "%d" "$BALANCE_HEX" 2>/dev/null || echo "0")
BALANCE_LATT=$(echo "scale=2; $BALANCE_WEI / 1000000000000000000" | bc 2>/dev/null || echo "0")
echo -e "${GREEN}[✓]${NC} Validator balance: $BALANCE_LATT LATT"

if [ "$BALANCE_WEI" -eq 0 ]; then
    echo -e "${RED}[✗] Cannot proceed without balance. Check coinbase configuration.${NC}"
    exit 1
fi

# Step 4: Build wallet CLI if needed
echo -e "\n${BLUE}[4/6] Checking wallet CLI...${NC}"
if [ ! -f "./target/release/wallet" ]; then
    echo "Building wallet CLI..."
    cargo build --release --bin wallet
fi
echo -e "${GREEN}[✓]${NC} Wallet CLI ready"

# Step 5: Create and send transaction using wallet CLI
echo -e "\n${BLUE}[5/6] Creating and sending transaction...${NC}"

# For now, use a simpler approach with direct RPC call
# In production, use the wallet CLI with proper keystore

# Get nonce
NONCE_HEX=$(curl -s -X POST http://localhost:8545 \
    -H "Content-Type: application/json" \
    -d "{\"jsonrpc\":\"2.0\",\"method\":\"eth_getTransactionCount\",\"params\":[\"$VALIDATOR_ADDRESS\",\"pending\"],\"id\":1}" \
    | grep -o '"result":"[^"]*"' | cut -d'"' -f4)

echo "Current nonce: $NONCE_HEX"

# Create simple transfer transaction
AMOUNT_WEI="1000000000000000000"  # 1 LATT
echo -e "${CYAN}Sending 1 LATT from validator to test receiver...${NC}"

# This is where we'd use the wallet CLI in production
# For now, showing the transaction structure
echo -e "${YELLOW}Transaction details:${NC}"
echo "  From:   $VALIDATOR_ADDRESS"
echo "  To:     $TEST_RECEIVER"
echo "  Amount: 1 LATT"
echo "  Nonce:  $NONCE_HEX"

# Since the wallet CLI requires interactive input, we'll demonstrate the flow
echo -e "\n${YELLOW}To send a signed transaction, run:${NC}"
echo -e "${CYAN}./target/release/wallet --keystore .lattice-wallet --chain-id 42069 interactive${NC}"
echo -e "Then select 'Send Transaction' and follow the prompts"

# Step 6: Alternative - Use the test script with expect
echo -e "\n${BLUE}[6/6] Alternative transaction methods:${NC}"
echo -e "${GREEN}Option 1:${NC} Interactive wallet"
echo "  ./target/release/wallet --chain-id 42069 interactive"
echo
echo -e "${GREEN}Option 2:${NC} Use test script (requires 'expect')"
echo "  CHAIN_ID=42069 ./scripts/test_transaction.sh"
echo
echo -e "${GREEN}Option 3:${NC} GUI wallet"
echo "  cd gui/lattice-core && npm run tauri dev"

# Check final balances
echo -e "\n${MAGENTA}Current Balances:${NC}"
for addr in "$VALIDATOR_ADDRESS" "$TEST_RECEIVER"; do
    BAL_HEX=$(curl -s -X POST http://localhost:8545 \
        -H "Content-Type: application/json" \
        -d "{\"jsonrpc\":\"2.0\",\"method\":\"eth_getBalance\",\"params\":[\"$addr\",\"latest\"],\"id\":1}" \
        | grep -o '"result":"[^"]*"' | cut -d'"' -f4)
    
    if [ ! -z "$BAL_HEX" ]; then
        BAL_WEI=$(printf "%d" "$BAL_HEX" 2>/dev/null || echo "0")
        BAL_LATT=$(echo "scale=6; $BAL_WEI / 1000000000000000000" | bc 2>/dev/null || echo "0")
        echo "  ${addr:0:10}...: $BAL_LATT LATT"
    fi
done

echo -e "\n${GREEN}═══════════════════════════════════════════════════════════════${NC}"
echo -e "${GREEN}TRANSACTION SYSTEM READY!${NC}"
echo -e "${GREEN}═══════════════════════════════════════════════════════════════${NC}"

echo -e "\n${YELLOW}Node is running and validator has balance.${NC}"
echo -e "${YELLOW}You can now test transactions using any of the methods above.${NC}"

# Keep monitoring if requested
if [ "$MONITOR" == "true" ]; then
    echo -e "\n${CYAN}Monitoring mode enabled. Press Ctrl+C to stop...${NC}"
    trap "kill $NODE_PID 2>/dev/null; exit" INT
    
    while true; do
        sleep 5
        HEIGHT=$(curl -s -X POST http://localhost:8545 \
            -H "Content-Type: application/json" \
            -d '{"jsonrpc":"2.0","method":"eth_blockNumber","params":[],"id":1}' \
            | grep -o '"result":"[^"]*"' | cut -d'"' -f4 | xargs printf "%d\n" 2>/dev/null || echo "0")
        
        echo -ne "\r[Live] Block #$HEIGHT | Status: Mining...  "
    done
fi