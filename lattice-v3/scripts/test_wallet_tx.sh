#!/bin/bash

# Test transaction script for Lattice V3 wallet system
# Sends a test transaction and verifies execution

set -e

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m'

# Default addresses
DEFAULT_FROM="0x48a3f6698a1384e6f97ae9cc2f6d6c94504ba8a5"
DEFAULT_TO="0x92b3f6698a1384e6f97ae9cc2f6d6c94504ba8a6"
DEFAULT_AMOUNT="1000000000000000000"  # 1 LATT in wei

# Parse arguments
FROM=${1:-$DEFAULT_FROM}
TO=${2:-$DEFAULT_TO}
AMOUNT=${3:-$DEFAULT_AMOUNT}

echo -e "${CYAN}╔══════════════════════════════════════════════════════════════════════╗${NC}"
echo -e "${CYAN}║           LATTICE V3 TRANSACTION TEST                               ║${NC}"
echo -e "${CYAN}╚══════════════════════════════════════════════════════════════════════╝${NC}"

echo -e "\n${BLUE}Transaction Details:${NC}"
echo -e "From:   ${CYAN}$FROM${NC}"
echo -e "To:     ${CYAN}$TO${NC}"
echo -e "Amount: ${CYAN}$(echo "scale=6; $AMOUNT / 1000000000000000000" | bc) LATT${NC}"

# Step 1: Check sender balance
echo -e "\n${BLUE}[1/5] Checking sender balance...${NC}"
balance_hex=$(curl -s -X POST http://localhost:8545 \
    -H "Content-Type: application/json" \
    -d "{\"jsonrpc\":\"2.0\",\"method\":\"eth_getBalance\",\"params\":[\"$FROM\",\"latest\"],\"id\":1}" \
    | grep -o '"result":"[^"]*"' | cut -d'"' -f4)

if [ -z "$balance_hex" ] || [ "$balance_hex" == "0x0" ]; then
    echo -e "${RED}[✗] Sender has no balance! Mine some blocks first.${NC}"
    exit 1
fi

balance_wei=$(printf "%d" "$balance_hex")
balance_latt=$(echo "scale=2; $balance_wei / 1000000000000000000" | bc)
echo -e "${GREEN}[✓]${NC} Sender balance: $balance_latt LATT"

if [ "$balance_wei" -lt "$AMOUNT" ]; then
    echo -e "${RED}[✗] Insufficient balance for transaction${NC}"
    exit 1
fi

# Step 2: Get nonce
echo -e "\n${BLUE}[2/5] Getting account nonce...${NC}"
nonce_hex=$(curl -s -X POST http://localhost:8545 \
    -H "Content-Type: application/json" \
    -d "{\"jsonrpc\":\"2.0\",\"method\":\"eth_getTransactionCount\",\"params\":[\"$FROM\",\"pending\"],\"id\":1}" \
    | grep -o '"result":"[^"]*"' | cut -d'"' -f4)

nonce=$(printf "%d" "${nonce_hex:-0x0}")
echo -e "${GREEN}[✓]${NC} Account nonce: $nonce"

# Step 3: Create transaction
echo -e "\n${BLUE}[3/5] Creating transaction...${NC}"

# Build raw transaction (simplified for testing)
tx_data=$(cat <<EOF
{
  "from": "$FROM",
  "to": "$TO",
  "value": "0x$(printf "%x" $AMOUNT)",
  "gas": "0x5208",
  "gasPrice": "0x3b9aca00",
  "nonce": "$nonce_hex",
  "chainId": 42069
}
EOF
)

echo -e "${GREEN}[✓]${NC} Transaction created"
echo -e "${CYAN}$tx_data${NC}" | jq '.' 2>/dev/null || echo "$tx_data"

# Step 4: Send transaction (using wallet CLI if available)
echo -e "\n${BLUE}[4/5] Sending transaction...${NC}"

# Check if wallet CLI exists
if [ -f "./target/release/wallet" ]; then
    # First check if we have a keystore
    KEYSTORE_DIR=".lattice-wallet"
    if [ ! -d "$KEYSTORE_DIR" ]; then
        echo -e "${YELLOW}Creating wallet keystore...${NC}"
        mkdir -p "$KEYSTORE_DIR"
        
        # Import the test private key (this is a test key for development)
        # In production, you'd generate a new key or import securely
        echo "48a3f6698a1384e6f97ae9cc2f6d6c94504ba8a5deadbeefdeadbeefdeadbeef" | \
            ./target/release/wallet import --keystore "$KEYSTORE_DIR" 2>&1 | head -5
    fi
    
    # Use wallet CLI for proper signing
    echo -e "${CYAN}Using wallet CLI to sign and send transaction...${NC}"
    
    # Send with wallet (account index 0)
    tx_response=$(./target/release/wallet send \
        --keystore "$KEYSTORE_DIR" \
        --from 0 \
        --to "$TO" \
        --amount "$(echo "scale=6; $AMOUNT / 1000000000000000000" | bc)" \
        --rpc http://localhost:8545 \
        --chain-id 42069 2>&1)
    
    if echo "$tx_response" | grep -q "Transaction sent:"; then
        tx_hash=$(echo "$tx_response" | grep -o "0x[a-f0-9]\{64\}" | head -1)
        echo -e "${GREEN}[✓]${NC} Transaction sent via wallet CLI!"
    else
        echo -e "${RED}[✗] Transaction failed${NC}"
        echo "$tx_response"
        exit 1
    fi
else
    # Fallback: try direct RPC (won't work without proper signing)
    echo -e "${YELLOW}[!] Wallet CLI not found. Attempting unsigned transaction...${NC}"
    
    # This would need proper signing - just for demonstration
    response=$(curl -s -X POST http://localhost:8545 \
        -H "Content-Type: application/json" \
        -d "{\"jsonrpc\":\"2.0\",\"method\":\"eth_sendTransaction\",\"params\":[$tx_data],\"id\":1}")
    
    if echo "$response" | grep -q "error"; then
        echo -e "${RED}[✗] Transaction failed: Needs proper signing${NC}"
        echo -e "${YELLOW}Build wallet CLI: cargo build --release --bin wallet${NC}"
        exit 1
    fi
    
    tx_hash=$(echo "$response" | grep -o '"result":"[^"]*"' | cut -d'"' -f4)
fi

if [ -z "$tx_hash" ]; then
    echo -e "${RED}[✗] Failed to send transaction${NC}"
    exit 1
fi

echo -e "${GREEN}[✓]${NC} Transaction sent!"
echo -e "Hash: ${CYAN}$tx_hash${NC}"

# Step 5: Wait for confirmation
echo -e "\n${BLUE}[5/5] Waiting for confirmation...${NC}"

for i in {1..10}; do
    sleep 2
    
    receipt=$(curl -s -X POST http://localhost:8545 \
        -H "Content-Type: application/json" \
        -d "{\"jsonrpc\":\"2.0\",\"method\":\"eth_getTransactionReceipt\",\"params\":[\"$tx_hash\"],\"id\":1}")
    
    if echo "$receipt" | grep -q '"status":"0x1"'; then
        block_num=$(echo "$receipt" | grep -o '"blockNumber":"[^"]*"' | cut -d'"' -f4)
        gas_used=$(echo "$receipt" | grep -o '"gasUsed":"[^"]*"' | cut -d'"' -f4)
        
        echo -e "${GREEN}[✓] Transaction confirmed!${NC}"
        echo -e "Block: ${CYAN}$(printf "%d" $block_num)${NC}"
        echo -e "Gas used: ${CYAN}$(printf "%d" $gas_used)${NC}"
        
        # Check new balances
        echo -e "\n${BLUE}Final Balances:${NC}"
        
        from_balance=$(curl -s -X POST http://localhost:8545 \
            -H "Content-Type: application/json" \
            -d "{\"jsonrpc\":\"2.0\",\"method\":\"eth_getBalance\",\"params\":[\"$FROM\",\"latest\"],\"id\":1}" \
            | grep -o '"result":"[^"]*"' | cut -d'"' -f4 | xargs printf "%d\n")
        
        to_balance=$(curl -s -X POST http://localhost:8545 \
            -H "Content-Type: application/json" \
            -d "{\"jsonrpc\":\"2.0\",\"method\":\"eth_getBalance\",\"params\":[\"$TO\",\"latest\"],\"id\":1}" \
            | grep -o '"result":"[^"]*"' | cut -d'"' -f4 | xargs printf "%d\n")
        
        from_latt=$(echo "scale=6; $from_balance / 1000000000000000000" | bc)
        to_latt=$(echo "scale=6; $to_balance / 1000000000000000000" | bc)
        
        echo -e "Sender:   ${CYAN}$from_latt LATT${NC}"
        echo -e "Receiver: ${CYAN}$to_latt LATT${NC}"
        
        echo -e "\n${GREEN}Transaction successful!${NC}"
        exit 0
    elif echo "$receipt" | grep -q '"status":"0x0"'; then
        echo -e "${RED}[✗] Transaction failed!${NC}"
        echo "$receipt" | jq '.'
        exit 1
    else
        echo -ne "."
    fi
done

echo -e "\n${YELLOW}[!] Transaction not confirmed after 20 seconds${NC}"
echo -e "Check status with: ${CYAN}curl -X POST http://localhost:8545 -H \"Content-Type: application/json\" -d '{\"jsonrpc\":\"2.0\",\"method\":\"eth_getTransactionReceipt\",\"params\":[\"$tx_hash\"],\"id\":1}'${NC}"