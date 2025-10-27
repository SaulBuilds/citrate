#!/bin/bash

# Test script for Citrate devnet
# Sends RPC calls to test the running node

# Colors
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
RED='\033[0;31m'
NC='\033[0m'

RPC_URL=${RPC_URL:-"http://127.0.0.1:8545"}

echo -e "${GREEN}Citrate Devnet Tester${NC}"
echo "====================="
echo "RPC URL: $RPC_URL"
echo ""

# Check if node is running
echo -e "${BLUE}1. Checking node connectivity...${NC}"
RESPONSE=$(curl -s -X POST $RPC_URL \
    -H "Content-Type: application/json" \
    -d '{"jsonrpc":"2.0","method":"net_version","params":[],"id":1}' 2>/dev/null)

if [ -z "$RESPONSE" ]; then
    echo -e "${RED}❌ Cannot connect to node at $RPC_URL${NC}"
    echo "   Make sure the devnet is running: ./scripts/start_devnet.sh"
    exit 1
fi

echo -e "${GREEN}✅ Node is reachable${NC}"
echo "   Response: $RESPONSE"
echo ""

# Get chain ID
echo -e "${BLUE}2. Getting chain ID...${NC}"
curl -s -X POST $RPC_URL \
    -H "Content-Type: application/json" \
    -d '{"jsonrpc":"2.0","method":"eth_chainId","params":[],"id":1}' | jq -r '.result' | xargs printf "   Chain ID: %d\n" 2>/dev/null || echo "   Chain ID: (unable to parse)"
echo ""

# Get block number
echo -e "${BLUE}3. Getting latest block number...${NC}"
BLOCK_HEX=$(curl -s -X POST $RPC_URL \
    -H "Content-Type: application/json" \
    -d '{"jsonrpc":"2.0","method":"eth_blockNumber","params":[],"id":1}' | jq -r '.result' 2>/dev/null)

if [ -n "$BLOCK_HEX" ] && [ "$BLOCK_HEX" != "null" ]; then
    BLOCK_NUM=$(printf "%d" "$BLOCK_HEX" 2>/dev/null || echo "0")
    echo -e "   Block Height: ${GREEN}$BLOCK_NUM${NC}"
else
    echo "   Block Height: 0"
fi
echo ""

# Get peer count
echo -e "${BLUE}4. Getting peer count...${NC}"
PEER_HEX=$(curl -s -X POST $RPC_URL \
    -H "Content-Type: application/json" \
    -d '{"jsonrpc":"2.0","method":"net_peerCount","params":[],"id":1}' | jq -r '.result' 2>/dev/null)

if [ -n "$PEER_HEX" ] && [ "$PEER_HEX" != "null" ]; then
    PEER_COUNT=$(printf "%d" "$PEER_HEX" 2>/dev/null || echo "0")
    echo "   Peers: $PEER_COUNT"
else
    echo "   Peers: 0"
fi
echo ""

# Get latest block details
echo -e "${BLUE}5. Getting latest block details...${NC}"
BLOCK=$(curl -s -X POST $RPC_URL \
    -H "Content-Type: application/json" \
    -d '{"jsonrpc":"2.0","method":"eth_getBlockByNumber","params":["latest", false],"id":1}')

if echo "$BLOCK" | jq -e '.result' > /dev/null 2>&1; then
    echo "   Hash:         $(echo "$BLOCK" | jq -r '.result.hash // "N/A"')"
    echo "   Parent Hash:  $(echo "$BLOCK" | jq -r '.result.parentHash // "N/A"')"
    echo "   Timestamp:    $(echo "$BLOCK" | jq -r '.result.timestamp // "N/A"' | xargs printf "%d" 2>/dev/null || echo "N/A")"
    echo "   Transactions: $(echo "$BLOCK" | jq -r '.result.transactions | length // 0')"
else
    echo "   Unable to get block details"
fi
echo ""

# Check treasury balance
echo -e "${BLUE}6. Checking treasury balance...${NC}"
TREASURY="0x0f572e5295c57f15886f9b263e2f6d2d6c7b5ec6"
BALANCE=$(curl -s -X POST $RPC_URL \
    -H "Content-Type: application/json" \
    -d "{\"jsonrpc\":\"2.0\",\"method\":\"eth_getBalance\",\"params\":[\"$TREASURY\",\"latest\"],\"id\":1}" | jq -r '.result' 2>/dev/null)

if [ -n "$BALANCE" ] && [ "$BALANCE" != "null" ]; then
    # Convert from hex to decimal (wei)
    BALANCE_WEI=$(printf "%d" "$BALANCE" 2>/dev/null || echo "0")
    # Convert to a readable format (assuming 18 decimals)
    BALANCE_ETH=$(echo "scale=4; $BALANCE_WEI / 1000000000000000000" | bc 2>/dev/null || echo "N/A")
    echo "   Treasury Balance: $BALANCE_ETH tokens"
else
    echo "   Treasury Balance: Unable to retrieve"
fi
echo ""

# Get mempool status
echo -e "${BLUE}7. Checking mempool status...${NC}"
MEMPOOL=$(curl -s -X POST $RPC_URL \
    -H "Content-Type: application/json" \
    -d '{"jsonrpc":"2.0","method":"txpool_status","params":[],"id":1}')

if echo "$MEMPOOL" | jq -e '.result' > /dev/null 2>&1; then
    echo "   Pending: $(echo "$MEMPOOL" | jq -r '.result.pending // "0"' | xargs printf "%d" 2>/dev/null || echo "0")"
    echo "   Queued:  $(echo "$MEMPOOL" | jq -r '.result.queued // "0"' | xargs printf "%d" 2>/dev/null || echo "0")"
else
    echo "   Mempool status unavailable"
fi
echo ""

# Summary
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo -e "${GREEN}✅ Devnet Health Check Complete${NC}"
echo ""

# Show useful commands
echo -e "${YELLOW}Useful commands:${NC}"
echo ""
echo "Watch blocks being produced:"
echo -e "  ${BLUE}watch -n 1 'curl -s $RPC_URL -X POST -H \"Content-Type: application/json\" -d \"{\\\"jsonrpc\\\":\\\"2.0\\\",\\\"method\\\":\\\"eth_blockNumber\\\",\\\"params\\\":[],\\\"id\\\":1}\" | jq -r \".result\" | xargs printf \"Block: %d\\\\n\"'${NC}"
echo ""
echo "Send a test transaction (requires wallet):"
echo -e "  ${BLUE}cargo run --bin lattice-wallet -- --rpc-url $RPC_URL transfer --to 0x123...456 --value 1000000000000000000${NC}"
echo ""