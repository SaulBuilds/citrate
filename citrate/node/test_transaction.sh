#!/bin/bash

echo "📤 Testing Citrate Transaction..."
echo "================================"

# Test eth_blockNumber
echo -e "\n1. Testing eth_blockNumber..."
curl -s -X POST http://localhost:8545 \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","method":"eth_blockNumber","params":[],"id":1}' | jq

# Test eth_gasPrice
echo -e "\n2. Testing eth_gasPrice..."
curl -s -X POST http://localhost:8545 \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","method":"eth_gasPrice","params":[],"id":1}' | jq

# Test sending a raw transaction
echo -e "\n3. Sending test transaction..."
curl -s -X POST http://localhost:8545 \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "method": "eth_sendRawTransaction",
    "params": ["0xf86380843b9aca00825208940000000000000000000000000000000000000000880de0b6b3a76400008025a0c9cf86333bcb065d140032ecaab5d9281bde80f21b9687b3e94161de42d51895a0727a108a0b8d101465414033c3f705a9c7b826e596766046ee1183dbc8aeaa6"],
    "id": 1
  }' | jq

echo -e "\n✅ Transaction test complete!"
