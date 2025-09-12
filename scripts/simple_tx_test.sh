#!/bin/bash

echo "=== Simple Transaction Test ==="

# Create a simple transfer transaction
# From: Treasury (0x1111111111111111111111111111111111111111)
# To: Test account (0x2222222222222222222222222222222222222222)
# Value: 1 LATT (1000000000000000000 wei)

TX_DATA='{
  "jsonrpc": "2.0",
  "method": "eth_sendTransaction",
  "params": [{
    "from": "0x1111111111111111111111111111111111111111",
    "to": "0x2222222222222222222222222222222222222222",
    "value": "0xde0b6b3a7640000",
    "gas": "0x5208",
    "gasPrice": "0x3b9aca00"
  }],
  "id": 1
}'

echo "Sending transaction..."
curl -s -X POST http://localhost:8545 \
  -H "Content-Type: application/json" \
  -d "$TX_DATA" | jq

echo ""
echo "Checking latest block..."
curl -s -X POST http://localhost:8545 \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","method":"eth_getBlockByNumber","params":["latest", true],"id":1}' | jq '.result | {number, hash, transactions}'

echo ""
echo "Checking recipient balance..."
curl -s -X POST http://localhost:8545 \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","method":"eth_getBalance","params":["0x2222222222222222222222222222222222222222", "latest"],"id":1}' | jq