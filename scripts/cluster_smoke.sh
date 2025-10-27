#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
RPC_URL=${RPC_URL:-http://127.0.0.1:28545}
TIMEOUT_SECS=${TIMEOUT_SECS:-120}

say() { printf "\033[1;34m[cluster]\033[0m %s\n" "$*"; }
rpc() {
  local method=$1; shift
  local params=${1:-[]}
  curl -s "$RPC_URL" -H 'Content-Type: application/json' -d "{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"$method\",\"params\":$params}"
}

say "Bringing up 5-node cluster via docker-compose profile..."
"$ROOT_DIR/scripts/lattice.sh" docker cluster up

say "Waiting for RPC at $RPC_URL ..."
deadline=$(( $(date +%s) + TIMEOUT_SECS ))
until curl -sf "$RPC_URL" >/dev/null 2>&1; do
  if [ $(date +%s) -gt $deadline ]; then
    echo "RPC did not become ready at $RPC_URL within $TIMEOUT_SECS seconds" >&2
    exit 1
  fi
  sleep 2
done

say "Polling net_peerCount until >= 4 ..."
deadline=$(( $(date +%s) + TIMEOUT_SECS ))
peers=0
while [ $peers -lt 4 ]; do
  out=$(rpc net_peerCount)
  peers_hex=$(echo "$out" | jq -r '.result // "0x0"')
  peers=$((16#${peers_hex#0x}))
  say "peerCount=$peers"
  if [ $(date +%s) -gt $deadline ]; then
    echo "peerCount did not reach 4 within $TIMEOUT_SECS seconds" >&2
    exit 1
  fi
  sleep 2
done

say "Checking block production..."
b1_hex=$(rpc eth_blockNumber | jq -r '.result // "0x0"')
b1=$((16#${b1_hex#0x}))
sleep 5
b2_hex=$(rpc eth_blockNumber | jq -r '.result // "0x0"')
b2=$((16#${b2_hex#0x}))
say "blockNumber initial=$b1 later=$b2"
if [ "$b2" -le "$b1" ]; then
  echo "Block height did not advance; check mining configuration" >&2
  exit 1
fi

say "Running lattice inference smoke test against cluster RPC..."
RPC_URL="$RPC_URL" "$ROOT_DIR/scripts/smoke_inference.sh"

say "Cluster smoke test passed."

