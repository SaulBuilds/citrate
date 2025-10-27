#!/bin/bash

# Test script for sending transactions to the Citrate testnet

# Configuration
RPC_URL=${RPC_URL:-"http://127.0.0.1:8545"}
NUM_TXS=${NUM_TXS:-10}

# Colors
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m'

echo -e "${GREEN}Citrate Testnet Transaction Tester${NC}"
echo "==================================="
echo "RPC URL: $RPC_URL"
echo "Number of transactions: $NUM_TXS"
echo ""

# Test accounts (these should have balance from genesis)
declare -a ACCOUNTS=(
    "0x1234567890123456789012345678901234567890"
    "0x2345678901234567890123456789012345678901"
    "0x3456789012345678901234567890123456789012"
    "0x4567890123456789012345678901234567890123"
    "0x5678901234567890123456789012345678901234"
)

# Check if node is reachable
echo -e "${YELLOW}Checking node connectivity...${NC}"
if ! curl -s -X POST $RPC_URL \
    -H "Content-Type: application/json" \
    -d '{"jsonrpc":"2.0","method":"net_version","params":[],"id":1}' > /dev/null 2>&1; then
    echo -e "${RED}Cannot connect to node at $RPC_URL${NC}"
    exit 1
fi
echo -e "${GREEN}✅ Node is reachable${NC}"
echo ""

# Get initial block number
INITIAL_BLOCK=$(curl -s -X POST $RPC_URL \
    -H "Content-Type: application/json" \
    -d '{"jsonrpc":"2.0","method":"eth_blockNumber","params":[],"id":1}' | \
    grep -o '"result":"0x[0-9a-f]*"' | cut -d'"' -f4 | xargs printf "%d\n")

echo "Initial block height: $INITIAL_BLOCK"
echo ""

# Function to send a transaction
send_transaction() {
    local from=$1
    local to=$2
    local value=$3
    local nonce=$4

    # Get current nonce if not provided
    if [ -z "$nonce" ]; then
        nonce=$(curl -s -X POST $RPC_URL \
            -H "Content-Type: application/json" \
            -d "{\"jsonrpc\":\"2.0\",\"method\":\"eth_getTransactionCount\",\"params\":[\"$from\",\"pending\"],\"id\":1}" | \
            grep -o '"result":"0x[0-9a-f]*"' | cut -d'"' -f4 | xargs printf "%d\n")
    fi

    # Create transaction data
    local tx_data=$(cat <<EOF
{
    "jsonrpc": "2.0",
    "method": "eth_sendTransaction",
    "params": [{
        "from": "$from",
        "to": "$to",
        "value": "0x$(printf '%x' $value)",
        "gas": "0x5208",
        "gasPrice": "0x3b9aca00",
        "nonce": "0x$(printf '%x' $nonce)"
    }],
    "id": 1
}
EOF
    )

    # Send transaction
    local result=$(curl -s -X POST $RPC_URL \
        -H "Content-Type: application/json" \
        -d "$tx_data")

    # Extract transaction hash
    local tx_hash=$(echo "$result" | grep -o '"result":"0x[0-9a-f]*"' | cut -d'"' -f4)

    if [ -n "$tx_hash" ]; then
        echo -e "${GREEN}✅ Transaction sent: $tx_hash${NC}"
        echo "$tx_hash"
    else
        echo -e "${RED}❌ Transaction failed: $result${NC}"
        echo ""
    fi
}

# Send test transactions
echo -e "${YELLOW}Sending test transactions...${NC}"
declare -a TX_HASHES

for ((i=0; i<$NUM_TXS; i++)); do
    # Rotate through accounts
    FROM_IDX=$((i % ${#ACCOUNTS[@]}))
    TO_IDX=$(((i + 1) % ${#ACCOUNTS[@]}))

    FROM_ADDR=${ACCOUNTS[$FROM_IDX]}
    TO_ADDR=${ACCOUNTS[$TO_IDX]}
    VALUE=$((1000000000000000 * (i + 1)))  # Increasing amounts

    echo "Transaction $((i + 1))/$NUM_TXS: $FROM_ADDR -> $TO_ADDR"
    TX_HASH=$(send_transaction $FROM_ADDR $TO_ADDR $VALUE)

    if [ -n "$TX_HASH" ] && [ "$TX_HASH" != "" ]; then
        TX_HASHES+=($TX_HASH)
    fi

    # Small delay between transactions
    sleep 0.5
done

echo ""
echo -e "${GREEN}Sent ${#TX_HASHES[@]} transactions successfully${NC}"
echo ""

# Wait for blocks to be mined
echo -e "${YELLOW}Waiting for transactions to be mined...${NC}"
sleep 10

# Check final block number
FINAL_BLOCK=$(curl -s -X POST $RPC_URL \
    -H "Content-Type: application/json" \
    -d '{"jsonrpc":"2.0","method":"eth_blockNumber","params":[],"id":1}' | \
    grep -o '"result":"0x[0-9a-f]*"' | cut -d'"' -f4 | xargs printf "%d\n")

echo "Final block height: $FINAL_BLOCK"
echo "New blocks: $((FINAL_BLOCK - INITIAL_BLOCK))"
echo ""

# Check transaction receipts
echo -e "${YELLOW}Checking transaction receipts...${NC}"
CONFIRMED=0

for tx_hash in "${TX_HASHES[@]}"; do
    if [ -n "$tx_hash" ]; then
        receipt=$(curl -s -X POST $RPC_URL \
            -H "Content-Type: application/json" \
            -d "{\"jsonrpc\":\"2.0\",\"method\":\"eth_getTransactionReceipt\",\"params\":[\"$tx_hash\"],\"id\":1}")

        if echo "$receipt" | grep -q '"status":"0x1"'; then
            ((CONFIRMED++))
            echo -e "  $tx_hash: ${GREEN}✅ Confirmed${NC}"
        elif echo "$receipt" | grep -q '"result":null'; then
            echo -e "  $tx_hash: ${YELLOW}⏳ Pending${NC}"
        else
            echo -e "  $tx_hash: ${RED}❌ Failed${NC}"
        fi
    fi
done

echo ""
echo "==================================="
echo -e "${GREEN}Test Complete${NC}"
echo "Transactions sent: ${#TX_HASHES[@]}"
echo "Transactions confirmed: $CONFIRMED"
echo "New blocks: $((FINAL_BLOCK - INITIAL_BLOCK))"
echo ""

# Get mempool status
MEMPOOL=$(curl -s -X POST $RPC_URL \
    -H "Content-Type: application/json" \
    -d '{"jsonrpc":"2.0","method":"txpool_status","params":[],"id":1}')

echo "Mempool status:"
echo "$MEMPOOL" | grep -o '"pending":"0x[0-9a-f]*"' || echo "  Pending: 0"
echo "$MEMPOOL" | grep -o '"queued":"0x[0-9a-f]*"' || echo "  Queued: 0"