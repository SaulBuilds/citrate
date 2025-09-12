#!/usr/bin/env bash
set -euo pipefail

# Lattice reset script
# - Cleans data dirs for the selected mode (devnet/testnet)
# - Waits for RPC to become available
# - Prints current height and genesis hash, and confirms 0 -> 1 advance
#
# Usage examples:
#   MODE=devnet ./scripts/reset.sh
#   MODE=testnet ./scripts/reset.sh
#   RPC=http://127.0.0.1:8545 MODE=testnet ./scripts/reset.sh

# --- Configuration ---
MODE="${MODE:-devnet}"                   # devnet | testnet
RPC="${RPC:-http://127.0.0.1:8545}"     # core node RPC endpoint

# Core node data dirs
CORE_DIR_DEFAULT="$HOME/.lattice"       # default core dir (NodeConfig.storage.data_dir)
CORE_DEVNET_DIR="./.lattice-devnet"     # used by `lattice --devnet`
CORE_DIR="${CORE_DIR:-$CORE_DIR_DEFAULT}"

# GUI (Tauri) data dirs (adjust to your GUI NodeConfig.data_dir roots)
GUI_DEVNET_DIR="./gui-data/devnet/chain"
GUI_TESTNET_DIR="./gui-data/testnet/chain"

rpc() {
  local METHOD="$1"; shift
  local PARAMS="${1:-[]}"
  curl -s -X POST -H 'Content-Type: application/json' \
    --data "{\"jsonrpc\":\"2.0\",\"method\":\"${METHOD}\",\"params\":${PARAMS},\"id\":1}" \
    "$RPC"
}

get_block_number() {
  rpc eth_blockNumber | sed -nE 's/.*"result"\s*:\s*"([^"]+)".*/\1/p'
}

get_block_by_number() {
  local NUM="$1"
  rpc eth_getBlockByNumber "[\"$NUM\", false]"
}

hex_to_dec() {
  python3 - "$1" << 'PY'
import sys
s=sys.argv[1]
print(int(s,16))
PY
}

echo "== Lattice Reset =="
echo "Mode: $MODE"
echo "RPC:  $RPC"
echo
echo "Stopping any running nodes (ensure processes are stopped)."
echo
echo "Cleaning core and GUI data..."
if [ "$MODE" = "devnet" ]; then
  rm -rf "$CORE_DEVNET_DIR" || true
  rm -rf "$GUI_DEVNET_DIR" || true
  echo "Removed $CORE_DEVNET_DIR and $GUI_DEVNET_DIR"
else
  rm -rf "$CORE_DIR" || true
  rm -rf "$GUI_TESTNET_DIR" || true
  echo "Removed $CORE_DIR and $GUI_TESTNET_DIR"
fi

echo
echo "Start your core node and GUI now in a second terminal..."
echo "Waiting for RPC to respond at $RPC"
for i in {1..30}; do
  if curl -s "$RPC" > /dev/null; then break; fi
  sleep 1
done

BN_HEX=$(get_block_number || echo "0x0")
echo "Current eth_blockNumber: $BN_HEX"
BN_DEC=$(hex_to_dec "${BN_HEX:-0x0}" 2>/dev/null || echo 0)
echo "Block height (dec): $BN_DEC"

echo
echo "Fetching genesis block (0x0)..."
GENESIS=$(get_block_by_number "0x0")
G_HASH=$(echo "$GENESIS" | sed -nE 's/.*"hash"\s*:\s*"([^"]+)".*/\1/p')
echo "Genesis hash: ${G_HASH:-<null>}"

echo
echo "Waiting up to 20s for height to advance (0 -> 1) ..."
for i in {1..20}; do
  sleep 1
  BN_HEX_NEW=$(get_block_number || echo "0x0")
  if [ "$BN_HEX_NEW" != "$BN_HEX" ]; then
    echo "Height advanced: $BN_HEX -> $BN_HEX_NEW"
    break
  fi
done

echo "Done."

