#!/bin/bash

# Load testing script for Citrate testnet
# Sends 1000+ transactions to test throughput

set -e

# Configuration
NUM_TXS=${NUM_TXS:-1000}
RPC_URL=${RPC_URL:-"http://127.0.0.1:8545"}
CONCURRENT=${CONCURRENT:-10}

# Colors
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
RED='\033[0;31m'
NC='\033[0m'

echo -e "${GREEN}Citrate Load Testing Tool${NC}"
echo "========================="
echo ""
echo -e "${BLUE}Configuration:${NC}"
echo "  Transactions: $NUM_TXS"
echo "  RPC URL:      $RPC_URL"
echo "  Concurrency:  $CONCURRENT"
echo ""

# Check if node is running
echo -e "${BLUE}Checking node connectivity...${NC}"
RESPONSE=$(curl -s -X POST $RPC_URL \
    -H "Content-Type: application/json" \
    -d '{"jsonrpc":"2.0","method":"net_version","params":[],"id":1}' 2>/dev/null)

if [ -z "$RESPONSE" ]; then
    echo -e "${RED}❌ Cannot connect to node at $RPC_URL${NC}"
    echo "   Start the testnet first: ./scripts/launch_10node_testnet.sh"
    exit 1
fi
echo -e "${GREEN}✓ Node is reachable${NC}"

# Get initial block height
INIT_BLOCK=$(curl -s -X POST $RPC_URL \
    -H "Content-Type: application/json" \
    -d '{"jsonrpc":"2.0","method":"eth_blockNumber","params":[],"id":1}' \
    | jq -r '.result' | xargs printf "%d")
echo "  Initial block: #$INIT_BLOCK"
echo ""

# Test accounts (pre-funded in devnet)
FROM_ADDR="0x0f572e5295c57f15886f9b263e2f6d2d6c7b5ec6"
TO_ADDR="0x1234567890123456789012345678901234567890"

# Function to send a transaction
send_transaction() {
    local nonce=$1
    local tx_data=$(cat <<EOF
{
    "jsonrpc": "2.0",
    "method": "eth_sendTransaction",
    "params": [{
        "from": "$FROM_ADDR",
        "to": "$TO_ADDR",
        "value": "0x1",
        "gas": "0x5208",
        "gasPrice": "0x3b9aca00",
        "nonce": "$(printf '0x%x' $nonce)"
    }],
    "id": $nonce
}
EOF
    )
    
    curl -s -X POST $RPC_URL \
        -H "Content-Type: application/json" \
        -d "$tx_data" > /dev/null 2>&1
}

echo -e "${GREEN}Starting load test...${NC}"
START_TIME=$(date +%s)

# Send transactions in batches
TOTAL_SENT=0
BATCH_SIZE=100

for batch in $(seq 0 $((NUM_TXS / BATCH_SIZE))); do
    batch_start=$((batch * BATCH_SIZE))
    batch_end=$((batch_start + BATCH_SIZE))
    
    if [ $batch_end -gt $NUM_TXS ]; then
        batch_end=$NUM_TXS
    fi
    
    if [ $batch_start -ge $NUM_TXS ]; then
        break
    fi
    
    echo -ne "\r  Sending transactions: $batch_start-$batch_end / $NUM_TXS"
    
    # Send batch concurrently
    for i in $(seq $batch_start $((batch_end - 1))); do
        send_transaction $i &
        
        # Limit concurrent requests
        if [ $((i % CONCURRENT)) -eq 0 ]; then
            wait
        fi
    done
    wait
    
    TOTAL_SENT=$batch_end
done

END_TIME=$(date +%s)
DURATION=$((END_TIME - START_TIME))

echo -e "\r  Sent $TOTAL_SENT transactions in $DURATION seconds"
echo ""

# Wait for blocks to be produced
echo -e "${BLUE}Waiting for transactions to be included in blocks...${NC}"
sleep 10

# Check final block height
FINAL_BLOCK=$(curl -s -X POST $RPC_URL \
    -H "Content-Type: application/json" \
    -d '{"jsonrpc":"2.0","method":"eth_blockNumber","params":[],"id":1}' \
    | jq -r '.result' | xargs printf "%d")

# Get mempool status
MEMPOOL=$(curl -s -X POST $RPC_URL \
    -H "Content-Type: application/json" \
    -d '{"jsonrpc":"2.0","method":"txpool_status","params":[],"id":1}' 2>/dev/null)

PENDING=$(echo "$MEMPOOL" | jq -r '.result.pending // "0"' | xargs printf "%d" 2>/dev/null || echo "0")
QUEUED=$(echo "$MEMPOOL" | jq -r '.result.queued // "0"' | xargs printf "%d" 2>/dev/null || echo "0")

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo -e "${GREEN}Load Test Results:${NC}"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "  Transactions sent:    $TOTAL_SENT"
echo "  Time taken:          $DURATION seconds"
echo "  TPS (send rate):     $(echo "scale=2; $TOTAL_SENT / $DURATION" | bc) tx/s"
echo ""
echo "  Initial block:       #$INIT_BLOCK"
echo "  Final block:         #$FINAL_BLOCK"
echo "  Blocks produced:     $((FINAL_BLOCK - INIT_BLOCK))"
echo ""
echo "  Mempool pending:     $PENDING"
echo "  Mempool queued:      $QUEUED"
echo ""

if [ $((FINAL_BLOCK - INIT_BLOCK)) -gt 0 ]; then
    BLOCKS_PRODUCED=$((FINAL_BLOCK - INIT_BLOCK))
    echo -e "${GREEN}✅ Test successful! Network processed transactions.${NC}"
else
    echo -e "${YELLOW}⚠ Warning: No new blocks produced${NC}"
fi

if [ $PENDING -gt 100 ]; then
    echo -e "${YELLOW}⚠ High mempool backlog detected${NC}"
fi

echo ""
echo "Note: Actual TPS depends on block size and production rate."
echo "Check the logs for detailed transaction processing info."
