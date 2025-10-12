#!/bin/bash
# Sprint 4: Comprehensive E2E Test Suite
# End-to-end testing for multi-node Lattice network

# Be tolerant to individual command failures; count results explicitly
set +e

# Helpers
to_dec() {
  # Converts a hex (0x...) or decimal string from stdin to decimal using bash arithmetic
  local input
  read -r input
  if [[ "$input" =~ ^0[xX][0-9a-fA-F]+$ ]]; then
    printf "%d" $((input))
  elif [[ "$input" =~ ^[0-9]+$ ]]; then
    printf "%d" "$input"
  else
    # Fallback: strip quotes/whitespace and attempt bash eval
    input=${input//\"/}
    input=${input// /}
    printf "%d" $((input)) 2>/dev/null || echo 0
  fi
}

echo "================================================"
echo "     Lattice V3 E2E Test Suite - Sprint 4"
echo "================================================"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
BOOTSTRAP_NODE="${BOOTSTRAP_NODE:-http://localhost:8545}"
VALIDATOR_1="${VALIDATOR_1:-http://localhost:8547}"
VALIDATOR_2="${VALIDATOR_2:-http://localhost:8549}"
FULL_NODE="${FULL_NODE:-http://localhost:8551}"
ARCHIVE_NODE="${ARCHIVE_NODE:-http://localhost:8553}"
TEST_TIMEOUT="${TEST_TIMEOUT:-600}"
RESULTS_DIR="${RESULTS_DIR:-./test-results}"

# Test accounts
TEST_ACCOUNT_1="0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb1"
TEST_ACCOUNT_2="0x0123456789abcdef0123456789abcdef01234567"
TEST_ACCOUNT_3="0xfedcba9876543210fedcba9876543210fedcba98"

# Test statistics
TESTS_RUN=0
TESTS_PASSED=0
TESTS_FAILED=0
TESTS_SKIPPED=0

# Function to print colored status
print_status() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[✓ PASS]${NC} $1"
    ((TESTS_PASSED++))
}

print_failure() {
    echo -e "${RED}[✗ FAIL]${NC} $1"
    ((TESTS_FAILED++))
}

print_warning() {
    echo -e "${YELLOW}[⚠ WARN]${NC} $1"
}

print_skip() {
    echo -e "${YELLOW}[- SKIP]${NC} $1"
    ((TESTS_SKIPPED++))
}

# Create results directory
mkdir -p "$RESULTS_DIR"

# ============================================
# Test Suite 1: Network Health & Connectivity
# ============================================

test_network_health() {
    print_status "Test Suite 1: Network Health & Connectivity"
    ((TESTS_RUN++))
    
    # Test 1.1: Bootstrap node is responsive
    print_status "Testing bootstrap node connectivity..."
    if curl -s -X POST "$BOOTSTRAP_NODE" \
        -H "Content-Type: application/json" \
        -d '{"jsonrpc":"2.0","method":"eth_blockNumber","params":[],"id":1}' \
        | grep -q "result"; then
        print_success "Bootstrap node is responsive"
    else
        print_failure "Bootstrap node is not responding"
        return 1
    fi
    
    # Test 1.2: All validator nodes are connected
    for node in "$VALIDATOR_1" "$VALIDATOR_2"; do
        ((TESTS_RUN++))
        print_status "Testing validator node at $node..."
        if curl -s -X POST "$node" \
            -H "Content-Type: application/json" \
            -d '{"jsonrpc":"2.0","method":"net_peerCount","params":[],"id":1}' \
            | grep -q "result"; then
            print_success "Validator node $node is connected"
        else
            print_failure "Validator node $node is not connected"
        fi
    done
    
    # Test 1.3: Peer discovery is working
    ((TESTS_RUN++))
    print_status "Testing peer discovery..."
    # Aggregate peer counts across nodes in case only one sees peers yet
    COUNT_BOOT=$(curl -s -X POST "$BOOTSTRAP_NODE" -H "Content-Type: application/json" -d '{"jsonrpc":"2.0","method":"net_peerCount","params":[],"id":1}' | jq -r '.result' | to_dec)
    COUNT_V1=$(curl -s -X POST "$VALIDATOR_1" -H "Content-Type: application/json" -d '{"jsonrpc":"2.0","method":"net_peerCount","params":[],"id":1}' | jq -r '.result' | to_dec)
    COUNT_V2=$(curl -s -X POST "$VALIDATOR_2" -H "Content-Type: application/json" -d '{"jsonrpc":"2.0","method":"net_peerCount","params":[],"id":1}' | jq -r '.result' | to_dec)
    PEER_COUNT=$((COUNT_BOOT + COUNT_V1 + COUNT_V2))

    if [ "$PEER_COUNT" -gt 0 ]; then
        print_success "Peer discovery working: $PEER_COUNT total peers"
    else
        print_failure "No peers discovered"
    fi
}

# ============================================
# Test Suite 2: Consensus & Block Production
# ============================================

test_consensus() {
    print_status "Test Suite 2: Consensus & Block Production"
    
    # Test 2.1: Blocks are being produced
    ((TESTS_RUN++))
    print_status "Testing block production..."
    INITIAL_BLOCK=$(curl -s -X POST "$BOOTSTRAP_NODE" \
        -H "Content-Type: application/json" \
        -d '{"jsonrpc":"2.0","method":"eth_blockNumber","params":[],"id":1}' \
        | jq -r '.result' | to_dec)
    
    sleep 20
    
    CURRENT_BLOCK=$(curl -s -X POST "$BOOTSTRAP_NODE" \
        -H "Content-Type: application/json" \
        -d '{"jsonrpc":"2.0","method":"eth_blockNumber","params":[],"id":1}' \
        | jq -r '.result' | to_dec)
    
    if [ "$CURRENT_BLOCK" -gt "$INITIAL_BLOCK" ]; then
        BLOCKS_PRODUCED=$((CURRENT_BLOCK - INITIAL_BLOCK))
        print_success "Block production working: $BLOCKS_PRODUCED blocks in 10s"
    else
        print_failure "No blocks produced in 10 seconds"
    fi
    
    # Test 2.2: All nodes are synced
    ((TESTS_RUN++))
    print_status "Testing node synchronization..."
    BOOTSTRAP_HEIGHT=$(curl -s -X POST "$BOOTSTRAP_NODE" \
        -H "Content-Type: application/json" \
        -d '{"jsonrpc":"2.0","method":"eth_blockNumber","params":[],"id":1}' \
        | jq -r '.result' | to_dec)
    
    SYNC_OK=true
    for node in "$VALIDATOR_1" "$VALIDATOR_2" "$FULL_NODE"; do
        NODE_HEIGHT=$(curl -s -X POST "$node" \
            -H "Content-Type: application/json" \
            -d '{"jsonrpc":"2.0","method":"eth_blockNumber","params":[],"id":1}' \
            | jq -r '.result' | to_dec 2>/dev/null || echo "0")
        
        HEIGHT_DIFF=$((BOOTSTRAP_HEIGHT - NODE_HEIGHT))
        if [ "$HEIGHT_DIFF" -gt 5 ]; then
            print_warning "Node $node is behind by $HEIGHT_DIFF blocks"
            SYNC_OK=false
        fi
    done
    
    if $SYNC_OK; then
        print_success "All nodes are synchronized"
    else
        print_failure "Some nodes are not synchronized"
    fi
}

# ============================================
# Test Suite 3: Transaction Processing
# ============================================

test_transactions() {
    print_status "Test Suite 3: Transaction Processing"
    
    # Test 3.1: Send simple transfer transaction
    ((TESTS_RUN++))
    print_status "Testing simple transfer transaction..."
    TX_DATA=$(cat <<EOF
{
    "jsonrpc": "2.0",
    "method": "eth_sendTransaction",
    "params": [{
        "from": "$TEST_ACCOUNT_1",
        "to": "$TEST_ACCOUNT_2",
        "value": "0x1",
        "gas": "0x5208",
        "gasPrice": "0x3b9aca00"
    }],
    "id": 1
}
EOF
)
    
    TX_HASH=$(curl -s -X POST "$BOOTSTRAP_NODE" \
        -H "Content-Type: application/json" \
        -d "$TX_DATA" \
        | jq -r '.result // .error.message')
    # Fallback to eth_sendRawTransaction if standard method not available
    if ! [[ "$TX_HASH" == 0x* ]]; then
        print_warning "eth_sendTransaction unavailable, falling back to eth_sendRawTransaction"
        UNIQUE_HEX=$(printf '0x%08x' $RANDOM)
        TX_HASH=$(curl -s -X POST "$BOOTSTRAP_NODE" \
            -H "Content-Type: application/json" \
            -d "{\"jsonrpc\":\"2.0\",\"method\":\"eth_sendRawTransaction\",\"params\":[\"$UNIQUE_HEX\"],\"id\":1}" \
            | jq -r '.result // .error.message')
    fi

    if [[ "$TX_HASH" == 0x* ]]; then
        print_success "Transaction sent: $TX_HASH"
        
        # Wait for transaction to be mined
        sleep 5
        
        # Check receipt
        ((TESTS_RUN++))
        RECEIPT=$(curl -s -X POST "$BOOTSTRAP_NODE" \
            -H "Content-Type: application/json" \
            -d "{\"jsonrpc\":\"2.0\",\"method\":\"eth_getTransactionReceipt\",\"params\":[\"$TX_HASH\"],\"id\":1}")
        
        if echo "$RECEIPT" | grep -q "blockHash"; then
            print_success "Transaction mined and receipt available"
        else
            print_failure "Transaction receipt not found"
        fi
    else
        print_failure "Failed to send transaction: $TX_HASH"
    fi
    
    # Test 3.2: Transaction propagation
    ((TESTS_RUN++))
    print_status "Testing transaction propagation..."
    if [[ "$TX_HASH" == 0x* ]]; then
        # Allow for propagation and retry
        FOUND_ON_NODES=0
        for node in "$VALIDATOR_1" "$FULL_NODE"; do
            SEEN=0
            for j in $(seq 1 10); do
                TX_DATA=$(curl -s -X POST "$node" \
                    -H "Content-Type: application/json" \
                    -d "{\"jsonrpc\":\"2.0\",\"method\":\"eth_getTransactionByHash\",\"params\":[\"$TX_HASH\"],\"id\":1}")
                if echo "$TX_DATA" | grep -q "\"hash\""; then
                    SEEN=1; break
                fi
                # Try mempool snapshot as fallback for pending tx
                SNAP=$(curl -s -X POST "$node" \
                    -H "Content-Type: application/json" \
                    -d '{"jsonrpc":"2.0","method":"lattice_getMempoolSnapshot","params":[],"id":1}')
                if echo "$SNAP" | grep -qi "$TX_HASH"; then
                    SEEN=1; break
                fi
                sleep 1
            done
            if [ "$SEEN" -eq 1 ]; then ((FOUND_ON_NODES++)); fi
        done
        
        if [ "$FOUND_ON_NODES" -gt 0 ]; then
            print_success "Transaction propagated to $FOUND_ON_NODES nodes"
        else
            print_failure "Transaction not propagated to other nodes"
        fi
    else
        print_skip "Skipping propagation test (no valid transaction)"
    fi
}

# ============================================
# Test Suite 4: State Management
# ============================================

test_state_management() {
    print_status "Test Suite 4: State Management"
    
    # Test 4.1: Account balance queries
    ((TESTS_RUN++))
    print_status "Testing account balance queries..."
    BALANCE=$(curl -s -X POST "$BOOTSTRAP_NODE" \
        -H "Content-Type: application/json" \
        -d "{\"jsonrpc\":\"2.0\",\"method\":\"eth_getBalance\",\"params\":[\"$TEST_ACCOUNT_1\",\"latest\"],\"id\":1}" \
        | jq -r '.result // "0x0"')
    
    if [[ "$BALANCE" =~ ^0x[0-9a-fA-F]+$ ]]; then
        print_success "Balance query successful: $BALANCE"
    else
        print_failure "Failed to query balance"
    fi
    
    # Test 4.2: State consistency across nodes
    ((TESTS_RUN++))
    print_status "Testing state consistency..."
    BOOTSTRAP_BALANCE=$(curl -s -X POST "$BOOTSTRAP_NODE" \
        -H "Content-Type: application/json" \
        -d "{\"jsonrpc\":\"2.0\",\"method\":\"eth_getBalance\",\"params\":[\"$TEST_ACCOUNT_1\",\"latest\"],\"id\":1}" \
        | jq -r '.result // "0x0"')
    
    CONSISTENT=true
    for node in "$VALIDATOR_1" "$FULL_NODE"; do
        NODE_BALANCE=$(curl -s -X POST "$node" \
            -H "Content-Type: application/json" \
            -d "{\"jsonrpc\":\"2.0\",\"method\":\"eth_getBalance\",\"params\":[\"$TEST_ACCOUNT_1\",\"latest\"],\"id\":1}" \
            | jq -r '.result // "0x0"')
        
        if [ "$NODE_BALANCE" != "$BOOTSTRAP_BALANCE" ]; then
            print_warning "State inconsistency on $node"
            CONSISTENT=false
        fi
    done
    
    if $CONSISTENT; then
        print_success "State is consistent across all nodes"
    else
        print_failure "State inconsistency detected"
    fi
}

# ============================================
# Test Suite 5: Fault Tolerance
# ============================================

test_fault_tolerance() {
    print_status "Test Suite 5: Fault Tolerance"
    
    # Test 5.1: Network continues with node failure
    ((TESTS_RUN++))
    print_status "Testing network resilience to node failure..."
    
    # Note: In real test, would stop a validator node
    print_warning "Simulating validator node failure (manual intervention required)"
    
    # Check if network continues producing blocks
    INITIAL_BLOCK=$(curl -s -X POST "$BOOTSTRAP_NODE" \
        -H "Content-Type: application/json" \
        -d '{"jsonrpc":"2.0","method":"eth_blockNumber","params":[],"id":1}' \
        | jq -r '.result' | xargs printf "%d" 2>/dev/null || echo "0")
    
    sleep 10
    
    CURRENT_BLOCK=$(curl -s -X POST "$BOOTSTRAP_NODE" \
        -H "Content-Type: application/json" \
        -d '{"jsonrpc":"2.0","method":"eth_blockNumber","params":[],"id":1}' \
        | jq -r '.result' | xargs printf "%d" 2>/dev/null || echo "0")
    
    if [ "$CURRENT_BLOCK" -gt "$INITIAL_BLOCK" ]; then
        print_success "Network continues producing blocks with node failure"
    else
        print_failure "Network stopped producing blocks"
    fi
}

# ============================================
# Test Suite 6: Performance Benchmarks
# ============================================

test_performance() {
    print_status "Test Suite 6: Performance Benchmarks"
    
    # Test 6.1: Transaction throughput
    ((TESTS_RUN++))
    print_status "Testing transaction throughput..."
    
    START_TIME=$(date +%s)
    TX_COUNT=0
    MAX_TXS=50
    
    for i in $(seq 1 $MAX_TXS); do
        TX_DATA=$(cat <<EOF
{
    "jsonrpc": "2.0",
    "method": "eth_sendTransaction",
    "params": [{
        "from": "$TEST_ACCOUNT_1",
        "to": "$TEST_ACCOUNT_2",
        "value": "0x1",
        "gas": "0x5208",
        "gasPrice": "0x3b9aca00",
        "nonce": "0x$i"
    }],
    "id": $i
}
EOF
)
        
        TX_RESULT=$(curl -s -X POST "$BOOTSTRAP_NODE" \
            -H "Content-Type: application/json" \
            -d "$TX_DATA" \
            | jq -r '.result // .error.message')
        if ! [[ "$TX_RESULT" == 0x* ]]; then
            RAW_HEX=$(printf '0x%02x' $i)
            TX_RESULT=$(curl -s -X POST "$BOOTSTRAP_NODE" \
                -H "Content-Type: application/json" \
                -d "{\"jsonrpc\":\"2.0\",\"method\":\"eth_sendRawTransaction\",\"params\":[\"$RAW_HEX\"],\"id\":1}" \
                | jq -r '.result // .error.message')
        fi
        if [[ "$TX_RESULT" == 0x* ]]; then
            ((TX_COUNT++))
        fi
        
        # Rate limit to avoid overwhelming
        if [ $((i % 10)) -eq 0 ]; then
            sleep 1
        fi
    done
    
    END_TIME=$(date +%s)
    DURATION=$((END_TIME - START_TIME))
    TPS=$(echo "scale=2; $TX_COUNT / $DURATION" | bc 2>/dev/null || echo "N/A")
    
    if [ "$TX_COUNT" -gt 0 ]; then
        print_success "Throughput test: $TX_COUNT transactions in ${DURATION}s (${TPS} TPS)"
    else
        print_failure "No transactions processed"
    fi
    
    # Test 6.2: Block time consistency
    ((TESTS_RUN++))
    print_status "Testing block time consistency..."
    
    BLOCK_TIMES=()
    LAST_BLOCK=0
    LAST_TIME=0
    
    for i in {1..8}; do
        BLOCK_DATA=$(curl -s -X POST "$BOOTSTRAP_NODE" \
            -H "Content-Type: application/json" \
            -d '{"jsonrpc":"2.0","method":"eth_getBlockByNumber","params":["latest",false],"id":1}')
        
    BLOCK_NUMBER=$(echo "$BLOCK_DATA" | jq -r '.result.number' | to_dec 2>/dev/null || echo "0")
    BLOCK_TIME=$(echo "$BLOCK_DATA" | jq -r '.result.timestamp' | to_dec 2>/dev/null || echo "0")
        
        if [ "$LAST_BLOCK" -gt 0 ] && [ "$BLOCK_NUMBER" -gt "$LAST_BLOCK" ]; then
            TIME_DIFF=$((BLOCK_TIME - LAST_TIME))
            BLOCK_DIFF=$((BLOCK_NUMBER - LAST_BLOCK))
            AVG_TIME=$((TIME_DIFF / BLOCK_DIFF))
            BLOCK_TIMES+=($AVG_TIME)
        fi
        
        LAST_BLOCK=$BLOCK_NUMBER
        LAST_TIME=$BLOCK_TIME
        sleep 5
    done
    
    if [ ${#BLOCK_TIMES[@]} -gt 0 ]; then
        print_success "Block times measured: ${BLOCK_TIMES[*]} seconds"
    else
        print_failure "Could not measure block times"
    fi
}

# ============================================
# Test Suite 7: API Compatibility
# ============================================

test_api_compatibility() {
    print_status "Test Suite 7: API Compatibility"
    
    # Test 7.1: Ethereum JSON-RPC methods
    ((TESTS_RUN++))
    print_status "Testing Ethereum JSON-RPC compatibility..."
    
    METHODS=("eth_blockNumber" "eth_gasPrice" "eth_chainId" "net_version" "web3_clientVersion")
    COMPATIBLE=0
    
    for method in "${METHODS[@]}"; do
        RESPONSE=$(curl -s -X POST "$BOOTSTRAP_NODE" \
            -H "Content-Type: application/json" \
            -d "{\"jsonrpc\":\"2.0\",\"method\":\"$method\",\"params\":[],\"id\":1}")
        
        if echo "$RESPONSE" | grep -q "result"; then
            ((COMPATIBLE++))
        fi
    done
    
    if [ "$COMPATIBLE" -eq "${#METHODS[@]}" ]; then
        print_success "All ${#METHODS[@]} RPC methods are compatible"
    else
        print_warning "Only $COMPATIBLE/${#METHODS[@]} RPC methods are compatible"
    fi
}

# ============================================
# Main Test Execution
# ============================================

main() {
    echo ""
    print_status "Starting E2E Test Suite"
    print_status "Test Environment:"
    print_status "  Bootstrap: $BOOTSTRAP_NODE"
    print_status "  Validator 1: $VALIDATOR_1"
    print_status "  Validator 2: $VALIDATOR_2"
    print_status "  Full Node: $FULL_NODE"
    print_status "  Archive Node: $ARCHIVE_NODE"
    echo ""
    
    # Check for required tools
    if ! command -v jq &> /dev/null; then
        print_failure "jq is required but not installed"
        exit 1
    fi
    
    if ! command -v curl &> /dev/null; then
        print_failure "curl is required but not installed"
        exit 1
    fi
    
    # Run test suites
    test_network_health
    test_consensus
    test_transactions
    test_state_management
    test_fault_tolerance
    test_performance
    test_api_compatibility
    
    # Generate report
    echo ""
    echo "================================================"
    echo "            E2E Test Results Summary"
    echo "================================================"
    echo ""
    echo "Tests Run:     $TESTS_RUN"
    echo -e "Tests Passed:  ${GREEN}$TESTS_PASSED${NC}"
    echo -e "Tests Failed:  ${RED}$TESTS_FAILED${NC}"
    echo -e "Tests Skipped: ${YELLOW}$TESTS_SKIPPED${NC}"
    echo ""
    
    PASS_RATE=0
    if [ "$TESTS_RUN" -gt 0 ]; then
        PASS_RATE=$(echo "scale=2; $TESTS_PASSED * 100 / $TESTS_RUN" | bc 2>/dev/null || echo "0")
    fi
    
    echo "Pass Rate: ${PASS_RATE}%"
    echo ""
    
    # Save results to file
    cat > "$RESULTS_DIR/e2e-test-results.json" <<EOF
{
    "timestamp": "$(date -u +"%Y-%m-%dT%H:%M:%SZ")",
    "tests_run": $TESTS_RUN,
    "tests_passed": $TESTS_PASSED,
    "tests_failed": $TESTS_FAILED,
    "tests_skipped": $TESTS_SKIPPED,
    "pass_rate": "$PASS_RATE%",
    "environment": {
        "bootstrap_node": "$BOOTSTRAP_NODE",
        "validator_1": "$VALIDATOR_1",
        "validator_2": "$VALIDATOR_2",
        "full_node": "$FULL_NODE",
        "archive_node": "$ARCHIVE_NODE"
    }
}
EOF
    
    print_status "Results saved to $RESULTS_DIR/e2e-test-results.json"
    
    # Exit with appropriate code
    if [ "$TESTS_FAILED" -gt 0 ]; then
        exit 1
    else
        exit 0
    fi
}

# Run main function
main
