#!/bin/bash

# Test transaction sending
echo "Testing transaction sending..."

# Simple value transfer transaction (already signed)
# From: 0x3333333333333333333333333333333333333333 
# To: 0x4444444444444444444444444444444444444444
# Value: 1 ETH
# This is a pre-signed transaction for testing

TX_DATA="0xf86b808504a817c800825208944444444444444444444444444444444444444444880de0b6b3a76400008025a0b0c5e2a2d8c5e2a2d8c5e2a2d8c5e2a2d8c5e2a2d8c5e2a2d8c5e2a2d8c5e2a2a0b0c5e2a2d8c5e2a2d8c5e2a2d8c5e2a2d8c5e2a2d8c5e2a2d8c5e2a2d8c5e2a2"

echo "Sending transaction..."
RESULT=$(curl -s -X POST http://localhost:8545 \
  -H "Content-Type: application/json" \
  -d "{\"jsonrpc\":\"2.0\",\"method\":\"eth_sendRawTransaction\",\"params\":[\"$TX_DATA\"],\"id\":1}")

echo "Result: $RESULT"

# Extract transaction hash from result
TX_HASH=$(echo $RESULT | grep -o '"result":"[^"]*' | cut -d'"' -f4)

if [ ! -z "$TX_HASH" ]; then
    echo "Transaction hash: $TX_HASH"
    
    # Wait a moment
    sleep 2
    
    # Check transaction receipt
    echo "Checking transaction receipt..."
    RECEIPT=$(curl -s -X POST http://localhost:8545 \
      -H "Content-Type: application/json" \
      -d "{\"jsonrpc\":\"2.0\",\"method\":\"eth_getTransactionReceipt\",\"params\":[\"$TX_HASH\"],\"id\":1}")
    
    echo "Receipt: $RECEIPT"
fi