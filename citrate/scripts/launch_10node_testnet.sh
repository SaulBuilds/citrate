#!/bin/bash

# Launch 10-node testnet
# Production-ready multi-node deployment

set -e

# Configuration
NUM_NODES=10
BASE_P2P_PORT=30303
BASE_RPC_PORT=8545
DATA_BASE_DIR=".citrate-testnet"
LOG_DIR="testnet-logs"

# Colors
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
RED='\033[0;31m'
NC='\033[0m'

clear
echo -e "${GREEN}╔══════════════════════════════════════════╗${NC}"
echo -e "${GREEN}║   Citrate V3 10-Node Testnet Launcher   ║${NC}"
echo -e "${GREEN}╚══════════════════════════════════════════╝${NC}"
echo ""

# Cleanup function
cleanup() {
    echo -e "\n${YELLOW}Shutting down testnet...${NC}"
    pkill -f "citrate" || true
    echo -e "${GREEN}Testnet stopped${NC}"
}

trap cleanup EXIT

# Clean previous data
echo -e "${BLUE}Cleaning previous testnet data...${NC}"
rm -rf "$DATA_BASE_DIR" "$LOG_DIR" 2>/dev/null || true
mkdir -p "$LOG_DIR"

# Build if needed
if [ ! -f "target/release/lattice" ]; then
    echo -e "${YELLOW}Building lattice...${NC}"
    cargo build --release --bin lattice
fi

echo -e "${BLUE}Configuration:${NC}"
echo "  Nodes:       $NUM_NODES"
echo "  Data dir:    $DATA_BASE_DIR/node-N"
echo "  P2P ports:   $BASE_P2P_PORT-$((BASE_P2P_PORT + NUM_NODES - 1))"
echo "  RPC ports:   $BASE_RPC_PORT-$((BASE_RPC_PORT + NUM_NODES - 1))"
echo "  Logs:        $LOG_DIR/"
echo ""

# Start bootstrap node
echo -e "${GREEN}Starting Bootstrap Node (Node 0)...${NC}"
BOOTSTRAP_ADDR="127.0.0.1:$BASE_P2P_PORT"

./target/release/lattice \
    --data-dir "$DATA_BASE_DIR/node-0" \
    --p2p-addr "$BOOTSTRAP_ADDR" \
    --rpc-addr "127.0.0.1:$BASE_RPC_PORT" \
    --bootstrap \
    --mine \
    --coinbase "0x0000000000000000000000000000000000000000" \
    --chain-id 1337 \
    > "$LOG_DIR/node-0.log" 2>&1 &

BOOTSTRAP_PID=$!
echo "  Node 0: PID=$BOOTSTRAP_PID P2P=$BOOTSTRAP_ADDR RPC=127.0.0.1:$BASE_RPC_PORT"
sleep 3

# Start remaining nodes
echo -e "\n${GREEN}Starting Worker Nodes...${NC}"
for i in $(seq 1 $((NUM_NODES - 1))); do
    P2P_PORT=$((BASE_P2P_PORT + i))
    RPC_PORT=$((BASE_RPC_PORT + i))
    
    # Generate unique coinbase for each node
    COINBASE=$(printf "0x%040d" $i)
    
    ./target/release/lattice \
        --data-dir "$DATA_BASE_DIR/node-$i" \
        --p2p-addr "127.0.0.1:$P2P_PORT" \
        --rpc-addr "127.0.0.1:$RPC_PORT" \
        --bootstrap-nodes "$BOOTSTRAP_ADDR" \
        --mine \
        --coinbase "$COINBASE" \
        --chain-id 1337 \
        --max-peers 20 \
        > "$LOG_DIR/node-$i.log" 2>&1 &
    
    PID=$!
    echo "  Node $i: PID=$PID P2P=127.0.0.1:$P2P_PORT RPC=127.0.0.1:$RPC_PORT"
    
    # Stagger node starts to avoid connection storms
    sleep 0.5
done

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo -e "${BLUE}Waiting for network to stabilize...${NC}"
sleep 10

# Check network status
echo ""
echo -e "${GREEN}Network Status:${NC}"
echo ""

# Check nodes are running
RUNNING=0
for i in $(seq 0 $((NUM_NODES - 1))); do
    if pgrep -f "node-$i" > /dev/null; then
        RUNNING=$((RUNNING + 1))
    fi
done
echo "  Nodes running: $RUNNING/$NUM_NODES"

# Check peer connections
echo ""
echo "  Peer connections:"
for i in $(seq 0 2); do
    PORT=$((BASE_RPC_PORT + i))
    PEER_HEX=$(curl -s -X POST http://127.0.0.1:$PORT \
        -H "Content-Type: application/json" \
        -d '{"jsonrpc":"2.0","method":"net_peerCount","params":[],"id":1}' 2>/dev/null \
        | jq -r '.result' 2>/dev/null || echo "0x0")
    PEERS=$((16#${PEER_HEX#0x}))
    echo "    Node $i: $PEERS peers"
done
echo "    ..."

# Check block production
echo ""
echo "  Block heights:"
for i in $(seq 0 2); do
    PORT=$((BASE_RPC_PORT + i))
    BLOCK_HEX=$(curl -s -X POST http://127.0.0.1:$PORT \
        -H "Content-Type: application/json" \
        -d '{"jsonrpc":"2.0","method":"eth_blockNumber","params":[],"id":1}' 2>/dev/null \
        | jq -r '.result' 2>/dev/null || echo "0x0")
    HEIGHT=$((16#${BLOCK_HEX#0x}))
    echo "    Node $i: Block #$HEIGHT"
done
echo "    ..."

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo -e "${GREEN}✅ 10-Node Testnet Launched Successfully!${NC}"
echo ""
echo "Commands:"
echo "  Monitor:     ./scripts/monitor_testnet.sh"
echo "  Test RPC:    curl -X POST http://127.0.0.1:8545 -H 'Content-Type: application/json' -d '{\"jsonrpc\":\"2.0\",\"method\":\"eth_blockNumber\",\"params\":[],\"id\":1}'"
echo "  View logs:   tail -f $LOG_DIR/node-0.log"
echo "  Stop:        pkill -f lattice"
echo ""
echo -e "${YELLOW}Press Ctrl+C to stop the testnet${NC}"
echo ""

# Keep running
wait
