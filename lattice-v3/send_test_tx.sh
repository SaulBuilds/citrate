#!/bin/bash

WALLET="./target/release/wallet"
KEYSTORE="./test_wallet.json"
RPC="http://localhost:8545"

echo "=== Simple Transaction Test ==="

# Clean previous keystore
rm -f $KEYSTORE

echo "Creating new wallet..."
# Create wallet with test password
echo -e "password123\npassword123" | $WALLET -k $KEYSTORE new

echo ""
echo "Getting wallet address..."
ADDR=$($WALLET -k $KEYSTORE list | grep "Account 0" | awk '{print $3}')
echo "Wallet address: $ADDR"

echo ""
echo "Checking initial balance..."
$WALLET -k $KEYSTORE -r $RPC balance 0

echo ""
echo "Getting treasury balance..."
curl -s -X POST $RPC \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","method":"eth_getBalance","params":["0x1111111111111111111111111111111111111111", "latest"],"id":1}' | jq -r '.result' | xargs printf "Treasury balance: %d wei\n"

echo ""
echo "Importing treasury key for testing..."
echo -e "password123" | $WALLET -k $KEYSTORE import --key 0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef

echo ""
echo "Listing accounts..."
$WALLET -k $KEYSTORE list

echo ""
echo "Sending 1 LATT from treasury to first account..."
TREASURY_ADDR="0x1111111111111111111111111111111111111111"
echo -e "password123" | $WALLET -k $KEYSTORE -r $RPC send --from $TREASURY_ADDR --to $ADDR --amount 1000000000000000000

echo ""
echo "Waiting for block..."
sleep 3

echo ""
echo "Checking balances after transaction..."
$WALLET -k $KEYSTORE -r $RPC balance 0
curl -s -X POST $RPC \
  -H "Content-Type: application/json" \
  -d "{\"jsonrpc\":\"2.0\",\"method\":\"eth_getBalance\",\"params\":[\"$ADDR\", \"latest\"],\"id\":1}" | jq -r '.result' | xargs printf "Recipient balance: %d wei\n"

echo ""
echo "Checking latest block for transactions..."
curl -s -X POST $RPC \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","method":"eth_getBlockByNumber","params":["latest", true],"id":1}' | jq '.result | {number, transactions}'

echo "=== Test Complete ===="