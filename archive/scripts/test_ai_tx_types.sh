#!/bin/bash

# Test AI transaction types in Citrate v3

echo "=== Testing AI Transaction Types ==="
echo ""

# RPC endpoint
RPC_URL="http://127.0.0.1:8545"

# Test 1: Model Deploy Transaction
echo "1. Testing Model Deploy Transaction (0x01000000)"
MODEL_DEPLOY_DATA="0x01000000$(openssl rand -hex 32)"  # Model deploy opcode + model hash
curl -X POST $RPC_URL \
  -H "Content-Type: application/json" \
  -d "{
    \"jsonrpc\": \"2.0\",
    \"method\": \"eth_sendTransaction\",
    \"params\": [{
      \"from\": \"0x0101010101010101010101010101010101010101\",
      \"to\": \"0x0000000000000000000000000000000000000001\",
      \"data\": \"$MODEL_DEPLOY_DATA\",
      \"gas\": \"0x186a0\",
      \"gasPrice\": \"0x3b9aca00\"
    }],
    \"id\": 1
  }" 2>/dev/null | jq -r '.result // .error.message'
echo ""

# Test 2: Inference Request Transaction
echo "2. Testing Inference Request Transaction (0x03000000)"
INFERENCE_DATA="0x03000000$(openssl rand -hex 32)$(echo -n 'Test input' | xxd -p)"
curl -X POST $RPC_URL \
  -H "Content-Type: application/json" \
  -d "{
    \"jsonrpc\": \"2.0\",
    \"method\": \"eth_sendTransaction\",
    \"params\": [{
      \"from\": \"0x0101010101010101010101010101010101010101\",
      \"to\": \"0x0000000000000000000000000000000000000001\",
      \"data\": \"$INFERENCE_DATA\",
      \"gas\": \"0xc350\",
      \"gasPrice\": \"0x3b9aca00\"
    }],
    \"id\": 2
  }" 2>/dev/null | jq -r '.result // .error.message'
echo ""

# Test 3: Training Job Transaction
echo "3. Testing Training Job Transaction (0x04000000)"
TRAINING_DATA="0x04000000$(openssl rand -hex 32)$(openssl rand -hex 32)"  # Training opcode + model + dataset
curl -X POST $RPC_URL \
  -H "Content-Type: application/json" \
  -d "{
    \"jsonrpc\": \"2.0\",
    \"method\": \"eth_sendTransaction\",
    \"params\": [{
      \"from\": \"0x0101010101010101010101010101010101010101\",
      \"to\": \"0x0000000000000000000000000000000000000001\",
      \"data\": \"$TRAINING_DATA\",
      \"gas\": \"0x30d40\",
      \"gasPrice\": \"0x3b9aca00\"
    }],
    \"id\": 3
  }" 2>/dev/null | jq -r '.result // .error.message'
echo ""

# Test 4: Check mempool for AI transactions
echo "4. Checking mempool content"
curl -X POST $RPC_URL \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "method": "txpool_content",
    "params": [],
    "id": 4
  }' 2>/dev/null | jq -r '.result // "Mempool check failed"' | head -20
echo ""

# Test 5: Get latest block to see if AI transactions are included
echo "5. Getting latest block info"
curl -X POST $RPC_URL \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "method": "eth_getBlockByNumber",
    "params": ["latest", true],
    "id": 5
  }' 2>/dev/null | jq -r '{
    number: .result.number,
    hash: .result.hash,
    stateRoot: .result.stateRoot,
    transactions: .result.transactions | length
  }'
echo ""

echo "=== AI Transaction Type Test Complete ==="