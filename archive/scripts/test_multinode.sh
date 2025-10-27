#!/bin/bash

# Test script for multi-node networking
# Tests P2P connectivity between nodes

set -e

# Colors
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
RED='\033[0;31m'
NC='\033[0m'

echo -e "${GREEN}Citrate Multi-Node Test${NC}"
echo "========================"
echo ""

# Cleanup function
cleanup() {
    echo -e "\n${YELLOW}Stopping all nodes...${NC}"
    pkill -f "citrate" || true
    rm -rf .lattice-test-node* 2>/dev/null || true
    echo -e "${GREEN}Cleanup complete${NC}"
}

# Set trap for cleanup
trap cleanup EXIT

# Build if needed
if [ ! -f "target/release/lattice" ]; then
    echo -e "${YELLOW}Building lattice...${NC}"
    cargo build --release --bin lattice
fi

echo -e "${BLUE}Starting 3-node test network...${NC}"
echo ""

# Node 1: Bootstrap node
echo -e "${GREEN}Starting Node 1 (Bootstrap)...${NC}"
./target/release/lattice \
    --data-dir .lattice-test-node1 \
    --p2p-addr 127.0.0.1:30303 \
    --rpc-addr 127.0.0.1:8545 \
    --bootstrap \
    --mine \
    --coinbase "0x1111111111111111111111111111111111111111" \
    > node1.log 2>&1 &
NODE1_PID=$!
echo "  PID: $NODE1_PID"
echo "  P2P: 127.0.0.1:30303"
echo "  RPC: 127.0.0.1:8545"
echo "  Log: node1.log"
echo ""

sleep 3

# Node 2: Connect to bootstrap
echo -e "${GREEN}Starting Node 2...${NC}"
./target/release/lattice \
    --data-dir .lattice-test-node2 \
    --p2p-addr 127.0.0.1:30304 \
    --rpc-addr 127.0.0.1:8546 \
    --bootstrap-nodes "127.0.0.1:30303" \
    --mine \
    --coinbase "0x2222222222222222222222222222222222222222" \
    > node2.log 2>&1 &
NODE2_PID=$!
echo "  PID: $NODE2_PID"
echo "  P2P: 127.0.0.1:30304"
echo "  RPC: 127.0.0.1:8546"
echo "  Log: node2.log"
echo ""

sleep 3

# Node 3: Connect to bootstrap
echo -e "${GREEN}Starting Node 3...${NC}"
./target/release/lattice \
    --data-dir .lattice-test-node3 \
    --p2p-addr 127.0.0.1:30305 \
    --rpc-addr 127.0.0.1:8547 \
    --bootstrap-nodes "127.0.0.1:30303" \
    --mine \
    --coinbase "0x3333333333333333333333333333333333333333" \
    > node3.log 2>&1 &
NODE3_PID=$!
echo "  PID: $NODE3_PID"
echo "  P2P: 127.0.0.1:30305"
echo "  RPC: 127.0.0.1:8547"
echo "  Log: node3.log"
echo ""

sleep 5

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo -e "${BLUE}Testing Network Connectivity...${NC}"
echo ""

# Check if nodes are running
for PID in $NODE1_PID $NODE2_PID $NODE3_PID; do
    if kill -0 $PID 2>/dev/null; then
        echo -e "  Node PID $PID: ${GREEN}✓ Running${NC}"
    else
        echo -e "  Node PID $PID: ${RED}✗ Failed${NC}"
    fi
done
echo ""

# Check RPC connectivity
echo -e "${BLUE}Testing RPC Endpoints...${NC}"
for PORT in 8545 8546 8547; do
    RESPONSE=$(curl -s -X POST http://127.0.0.1:$PORT \
        -H "Content-Type: application/json" \
        -d '{"jsonrpc":"2.0","method":"net_version","params":[],"id":1}' 2>/dev/null || echo "failed")
    if [ "$RESPONSE" != "failed" ] && [ -n "$RESPONSE" ]; then
        echo -e "  Port $PORT: ${GREEN}✓ Responding${NC}"
    else
        echo -e "  Port $PORT: ${RED}✗ Not responding${NC}"
    fi
done
echo ""

# Check peer counts
echo -e "${BLUE}Checking Peer Connections...${NC}"
for PORT in 8545 8546 8547; do
    PEER_COUNT=$(curl -s -X POST http://127.0.0.1:$PORT \
        -H "Content-Type: application/json" \
        -d '{"jsonrpc":"2.0","method":"net_peerCount","params":[],"id":1}' 2>/dev/null \
        | jq -r '.result' 2>/dev/null || echo "0x0")
    PEERS=$((16#${PEER_COUNT#0x}))
    echo "  Node on port $PORT: $PEERS peers"
done
echo ""

# Check block heights
echo -e "${BLUE}Checking Block Production...${NC}"
for PORT in 8545 8546 8547; do
    BLOCK_HEX=$(curl -s -X POST http://127.0.0.1:$PORT \
        -H "Content-Type: application/json" \
        -d '{"jsonrpc":"2.0","method":"eth_blockNumber","params":[],"id":1}' 2>/dev/null \
        | jq -r '.result' 2>/dev/null || echo "0x0")
    BLOCK_NUM=$((16#${BLOCK_HEX#0x}))
    echo "  Node on port $PORT: Block #$BLOCK_NUM"
done
echo ""

# Show last few lines of logs
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo -e "${BLUE}Recent Log Output:${NC}"
echo ""
echo "Node 1 (Bootstrap):"
tail -5 node1.log | sed 's/^/  /'
echo ""
echo "Node 2:"
tail -5 node2.log | sed 's/^/  /'
echo ""
echo "Node 3:"
tail -5 node3.log | sed 's/^/  /'
echo ""

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo -e "${GREEN}Multi-node test running!${NC}"
echo ""
echo "Nodes are producing blocks and should be syncing."
echo "Check the logs for P2P messages:"
echo "  - tail -f node1.log"
echo "  - tail -f node2.log"
echo "  - tail -f node3.log"
echo ""
echo -e "${YELLOW}Press Ctrl+C to stop all nodes${NC}"
echo ""

# Keep running
wait
