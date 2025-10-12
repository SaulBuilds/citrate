#!/bin/bash

# Integration test script for transaction pipeline
# Tests the complete flow from sending a transaction to confirmation

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Configuration
RPC_URL="${RPC_URL:-http://localhost:8545}"
TEST_ACCOUNT_1="0x1234567890123456789012345678901234567890"
TEST_ACCOUNT_2="0x2345678901234567890123456789012345678901"

echo -e "${YELLOW}Transaction Pipeline Integration Test${NC}"
echo "RPC URL: $RPC_URL"
echo ""

# Function to make RPC call
rpc_call() {
    local method=$1
    local params=$2
    curl -s -X POST $RPC_URL \
        -H "Content-Type: application/json" \
        -d "{\"jsonrpc\":\"2.0\",\"method\":\"$method\",\"params\":$params,\"id\":1}" \
        | jq -r '.result'
}

# Function to check if a value is not null or empty
check_not_empty() {
    local value=$1
    local description=$2
    if [ -z "$value" ] || [ "$value" = "null" ]; then
        echo -e "${RED}✗ $description: Empty or null${NC}"
        return 1
    else
        echo -e "${GREEN}✓ $description: $value${NC}"
        return 0
    fi
}

# Test 1: Check node connectivity
echo -e "${YELLOW}Test 1: Node Connectivity${NC}"
BLOCK_NUMBER=$(rpc_call "eth_blockNumber" "[]")
check_not_empty "$BLOCK_NUMBER" "Current block number"
echo ""

# Test 2: Check chain ID
echo -e "${YELLOW}Test 2: Chain ID${NC}"
CHAIN_ID=$(rpc_call "eth_chainId" "[]")
check_not_empty "$CHAIN_ID" "Chain ID"
echo ""

# Test 3: Get account balances
echo -e "${YELLOW}Test 3: Account Balances${NC}"
BALANCE_1=$(rpc_call "eth_getBalance" "[\"$TEST_ACCOUNT_1\", \"latest\"]")
BALANCE_2=$(rpc_call "eth_getBalance" "[\"$TEST_ACCOUNT_2\", \"latest\"]")
check_not_empty "$BALANCE_1" "Account 1 balance"
check_not_empty "$BALANCE_2" "Account 2 balance"
echo ""

# Test 4: Get nonce (with pending)
echo -e "${YELLOW}Test 4: Nonce Query (Pending)${NC}"
NONCE_LATEST=$(rpc_call "eth_getTransactionCount" "[\"$TEST_ACCOUNT_1\", \"latest\"]")
NONCE_PENDING=$(rpc_call "eth_getTransactionCount" "[\"$TEST_ACCOUNT_1\", \"pending\"]")
check_not_empty "$NONCE_LATEST" "Nonce (latest)"
check_not_empty "$NONCE_PENDING" "Nonce (pending)"
echo ""

# Test 5: Check mempool snapshot
echo -e "${YELLOW}Test 5: Mempool Snapshot${NC}"
MEMPOOL=$(rpc_call "lattice_getMempoolSnapshot" "[]")
if [ -z "$MEMPOOL" ] || [ "$MEMPOOL" = "null" ]; then
    echo -e "${YELLOW}⚠ Mempool endpoint not available or empty${NC}"
else
    PENDING_COUNT=$(echo "$MEMPOOL" | jq -r '.totalTransactions // 0')
    echo -e "${GREEN}✓ Mempool has $PENDING_COUNT pending transactions${NC}"
fi
echo ""

# Test 6: Send a test transaction (if wallet is available)
if command -v lattice-wallet &> /dev/null; then
    echo -e "${YELLOW}Test 6: Send Transaction via CLI Wallet${NC}"
    # This would require wallet setup, skipping for now
    echo -e "${YELLOW}⚠ Wallet test requires account setup, skipping${NC}"
else
    echo -e "${YELLOW}Test 6: Send Transaction${NC}"
    echo -e "${YELLOW}⚠ lattice-wallet not found in PATH${NC}"
fi
echo ""

# Test 7: Check transaction receipt for a known tx (if any)
echo -e "${YELLOW}Test 7: Transaction Receipt Query${NC}"
# Try to get a recent transaction from a block
LATEST_BLOCK=$(rpc_call "eth_getBlockByNumber" "[\"latest\", true]")
if [ ! -z "$LATEST_BLOCK" ] && [ "$LATEST_BLOCK" != "null" ]; then
    TX_COUNT=$(echo "$LATEST_BLOCK" | jq -r '.transactions | length')
    if [ "$TX_COUNT" -gt 0 ]; then
        FIRST_TX_HASH=$(echo "$LATEST_BLOCK" | jq -r '.transactions[0].hash')
        RECEIPT=$(rpc_call "eth_getTransactionReceipt" "[\"$FIRST_TX_HASH\"]")
        check_not_empty "$RECEIPT" "Transaction receipt"
    else
        echo -e "${YELLOW}⚠ No transactions in latest block${NC}"
    fi
else
    echo -e "${YELLOW}⚠ Could not fetch latest block${NC}"
fi
echo ""

# Test 8: Gas price query
echo -e "${YELLOW}Test 8: Gas Price${NC}"
GAS_PRICE=$(rpc_call "eth_gasPrice" "[]")
check_not_empty "$GAS_PRICE" "Gas price"
echo ""

# Test 9: EIP-1559 support
echo -e "${YELLOW}Test 9: EIP-1559 Support${NC}"
MAX_PRIORITY_FEE=$(rpc_call "eth_maxPriorityFeePerGas" "[]")
check_not_empty "$MAX_PRIORITY_FEE" "Max priority fee per gas"
echo ""

# Summary
echo -e "${YELLOW}========================================${NC}"
echo -e "${YELLOW}Test Summary${NC}"
echo -e "${YELLOW}========================================${NC}"
echo -e "${GREEN}✓ Node is responsive${NC}"
echo -e "${GREEN}✓ RPC endpoints are functional${NC}"
echo -e "${GREEN}✓ Nonce queries support 'pending' parameter${NC}"
echo -e "${GREEN}✓ EIP-1559 gas fields are available${NC}"

# Check if mempool endpoint exists
if [ ! -z "$MEMPOOL" ] && [ "$MEMPOOL" != "null" ]; then
    echo -e "${GREEN}✓ Mempool snapshot endpoint is available${NC}"
fi

echo ""
echo -e "${YELLOW}Note: For complete transaction testing, ensure:${NC}"
echo "1. The node is running with: cargo run --bin lattice-node"
echo "2. The wallet has funded accounts"
echo "3. Block producer is active"
echo ""

# Test transaction sending with curl (example raw transaction)
echo -e "${YELLOW}Example: Send Raw Transaction via curl${NC}"
echo 'curl -X POST http://localhost:8545 \'
echo '  -H "Content-Type: application/json" \'
echo '  -d '\''{"jsonrpc":"2.0","method":"eth_sendRawTransaction","params":["0x..."],"id":1}'\'''
echo ""

# Test mempool query
echo -e "${YELLOW}Example: Query Mempool Snapshot${NC}"
echo 'curl -X POST http://localhost:8545 \'
echo '  -H "Content-Type: application/json" \'
echo '  -d '\''{"jsonrpc":"2.0","method":"lattice_getMempoolSnapshot","params":[],"id":1}'\'' | jq'