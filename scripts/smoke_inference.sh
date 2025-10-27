#!/usr/bin/env bash
set -euo pipefail

RPC_URL=${RPC_URL:-http://127.0.0.1:8545}

say() { printf "\033[1;34m[smoke]\033[0m %s\n" "$*"; }

rpc() {
  local method=$1; shift
  local params=${1:-[]}
  curl -s "$RPC_URL" -H 'Content-Type: application/json' -d "{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"$method\",\"params\":$params}"
}

say "RPC: $RPC_URL"

say "Check net_version"
rpc net_version | sed 's/.*/  &/'

say "List models (citrate_listModels)"
LIST=$(rpc citrate_listModels)
echo "$LIST" | sed 's/.*/  &/'

# Extract first 64-hex model id from the result array and prefix 0x
MID=$(echo "$LIST" | grep -oE '([a-f0-9]{64})' | head -n1)
if [ -z "${MID:-}" ]; then
  say "No models found; ensure genesis model registered or deploy one."
  exit 1
fi
MID_HEX="0x$MID"
say "Using model id: $MID_HEX"

say "Get model info"
rpc citrate_getModel "[\"$MID_HEX\"]" | sed 's/.*/  &/'

say "Run inference"
INF=$(rpc citrate_runInference "[{\"model_id\":\"$MID_HEX\",\"input\":{\"text\":\"hello lattice\"},\"max_gas\":1000000,\"with_proof\":false}]")
echo "$INF" | sed 's/.*/  &/'

say "Smoke test completed"

