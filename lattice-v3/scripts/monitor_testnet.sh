#!/bin/bash

# Lattice V3 Testnet Monitor
# Monitors the status of running testnet nodes

# Configuration
BASE_DIR=${BASE_DIR:-"$HOME/.lattice-testnet"}
BASE_RPC_PORT=${BASE_RPC_PORT:-8545}

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

# Detect number of nodes
NUM_NODES=0
for dir in "$BASE_DIR"/node*; do
    if [ -d "$dir" ]; then
        ((NUM_NODES++))
    fi
done

if [ $NUM_NODES -eq 0 ]; then
    echo -e "${RED}No testnet nodes found. Run launch_testnet.sh first.${NC}"
    exit 1
fi

clear

echo -e "${GREEN}╔══════════════════════════════════════════════════════╗${NC}"
echo -e "${GREEN}║        Lattice V3 Testnet Monitor                   ║${NC}"
echo -e "${GREEN}╚══════════════════════════════════════════════════════╝${NC}"
echo ""

# Function to get node metrics
get_node_metrics() {
    local port=$1
    local node_num=$2

    # Initialize variables
    local status="❌ Offline"
    local peers=0
    local height=0
    local txpool=0
    local blue_score=0

    # Check if node is responding
    if curl -s -X POST http://127.0.0.1:$port \
        -H "Content-Type: application/json" \
        -d '{"jsonrpc":"2.0","method":"net_peerCount","params":[],"id":1}' > /dev/null 2>&1; then

        status="✅ Online"

        # Get peer count
        peers=$(curl -s -X POST http://127.0.0.1:$port \
            -H "Content-Type: application/json" \
            -d '{"jsonrpc":"2.0","method":"net_peerCount","params":[],"id":1}' | \
            grep -o '"result":"0x[0-9a-f]*"' | cut -d'"' -f4 | xargs printf "%d\n" 2>/dev/null || echo "0")

        # Get block height
        height=$(curl -s -X POST http://127.0.0.1:$port \
            -H "Content-Type: application/json" \
            -d '{"jsonrpc":"2.0","method":"eth_blockNumber","params":[],"id":1}' | \
            grep -o '"result":"0x[0-9a-f]*"' | cut -d'"' -f4 | xargs printf "%d\n" 2>/dev/null || echo "0")

        # Get mempool size
        txpool=$(curl -s -X POST http://127.0.0.1:$port \
            -H "Content-Type: application/json" \
            -d '{"jsonrpc":"2.0","method":"txpool_status","params":[],"id":1}' | \
            grep -o '"pending":"0x[0-9a-f]*"' | cut -d'"' -f4 | xargs printf "%d\n" 2>/dev/null || echo "0")

        # Get latest block for blue score
        local latest_block=$(curl -s -X POST http://127.0.0.1:$port \
            -H "Content-Type: application/json" \
            -d '{"jsonrpc":"2.0","method":"eth_getBlockByNumber","params":["latest", false],"id":1}')

        # Extract blue score if available
        blue_score=$(echo "$latest_block" | grep -o '"blueScore":"0x[0-9a-f]*"' | cut -d'"' -f4 | xargs printf "%d\n" 2>/dev/null || echo "0")
    fi

    echo "$status|$peers|$height|$txpool|$blue_score"
}

# Main monitoring loop
while true; do
    # Clear screen and show header
    clear
    echo -e "${GREEN}╔══════════════════════════════════════════════════════╗${NC}"
    echo -e "${GREEN}║        Lattice V3 Testnet Monitor                   ║${NC}"
    echo -e "${GREEN}╚══════════════════════════════════════════════════════╝${NC}"
    echo ""
    echo "Time: $(date '+%Y-%m-%d %H:%M:%S')"
    echo "Nodes: $NUM_NODES"
    echo ""

    # Table header
    printf "${BLUE}%-8s %-12s %-8s %-10s %-10s %-12s${NC}\n" "Node" "Status" "Peers" "Height" "Mempool" "Blue Score"
    echo "──────────────────────────────────────────────────────────────"

    # Collect metrics for all nodes
    declare -a metrics
    max_height=0
    total_peers=0
    total_txpool=0
    online_nodes=0

    for ((i=0; i<$NUM_NODES; i++)); do
        RPC_PORT=$((BASE_RPC_PORT + i))
        metrics[$i]=$(get_node_metrics $RPC_PORT $i)

        # Parse metrics
        IFS='|' read -r status peers height txpool blue_score <<< "${metrics[$i]}"

        # Update totals
        if [[ "$status" == *"Online"* ]]; then
            ((online_nodes++))
            total_peers=$((total_peers + peers))
            total_txpool=$((total_txpool + txpool))
            if [ $height -gt $max_height ]; then
                max_height=$height
            fi
        fi

        # Display node metrics
        printf "%-8s %-12s %-8s %-10s %-10s %-12s\n" \
            "Node $i" "$status" "$peers" "$height" "$txpool" "$blue_score"
    done

    echo "──────────────────────────────────────────────────────────────"
    echo ""

    # Summary statistics
    echo -e "${GREEN}Network Statistics:${NC}"
    echo "  Online Nodes: $online_nodes/$NUM_NODES"
    echo "  Total Peers: $total_peers"
    echo "  Max Height: $max_height"
    echo "  Total Mempool: $total_txpool txs"
    echo ""

    # Check consensus health
    if [ $online_nodes -gt $((NUM_NODES / 2)) ]; then
        echo -e "  Consensus: ${GREEN}✅ Healthy (>50% nodes online)${NC}"
    else
        echo -e "  Consensus: ${RED}❌ Unhealthy (<50% nodes online)${NC}"
    fi

    # Check if nodes are synced
    height_variance=0
    for ((i=0; i<$NUM_NODES; i++)); do
        IFS='|' read -r status peers height txpool blue_score <<< "${metrics[$i]}"
        if [[ "$status" == *"Online"* ]]; then
            diff=$((max_height - height))
            if [ $diff -gt 2 ]; then
                ((height_variance++))
            fi
        fi
    done

    if [ $height_variance -eq 0 ]; then
        echo -e "  Sync Status: ${GREEN}✅ All nodes synced${NC}"
    else
        echo -e "  Sync Status: ${YELLOW}⚠ $height_variance nodes behind${NC}"
    fi

    echo ""
    echo "──────────────────────────────────────────────────────────────"
    echo -e "${YELLOW}Press Ctrl+C to exit monitor${NC}"

    # Refresh every 5 seconds
    sleep 5
done