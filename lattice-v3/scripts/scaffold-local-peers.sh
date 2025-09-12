#!/usr/bin/env bash
set -euo pipefail

# Scaffold and run 5 local lattice-node instances bound to the machine's private IP.
#
# Usage:
#   scripts/scaffold-local-peers.sh up     # build + write configs + start 5 nodes
#   scripts/scaffold-local-peers.sh down   # stop nodes
#   scripts/scaffold-local-peers.sh clean  # remove scaffold directory
#
# Options via env:
#   HOST_IP=192.168.1.50   # override detected LAN IP
#   BASE_P2P=30303         # base P2P port (default 30303..30307)
#   BASE_RPC=8545          # base HTTP RPC port (default 8545..8549)
#   BASE_WS=8546           # base WS port (default 8546..8550)
#   CHAIN_ID=1337          # devnet chain id

ACTION="${1:-up}"
ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
SCAFFOLD_DIR="$ROOT_DIR/.lattice-multi"
BIN="$ROOT_DIR/target/release/lattice-node"

HOST_IP_DEFAULT() {
  # Try macOS first
  if command -v ipconfig >/dev/null 2>&1; then
    ipconfig getifaddr en0 2>/dev/null || ipconfig getifaddr en1 2>/dev/null || true
  fi
}

detect_host_ip() {
  if [[ -n "${HOST_IP:-}" ]]; then echo "$HOST_IP"; return; fi
  local ip
  ip=$(HOST_IP_DEFAULT)
  if [[ -z "$ip" ]]; then
    # Fallback: parse ifconfig, ignore loopback
    ip=$(ifconfig 2>/dev/null | awk '/inet / {print $2}' | grep -v '^127\.' | head -n1)
  fi
  echo "${ip:-127.0.0.1}"
}

BASE_P2P=${BASE_P2P:-30303}
BASE_RPC=${BASE_RPC:-8545}
BASE_WS=${BASE_WS:-8546}
CHAIN_ID=${CHAIN_ID:-1337}
HOST_ADDR=$(detect_host_ip)

write_config() {
  local idx=$1
  local dir="$SCAFFOLD_DIR/node$idx"
  local p2p=$((BASE_P2P + idx))
  local rpc=$((BASE_RPC + idx))
  local ws=$((BASE_WS + idx))

  mkdir -p "$dir"

  # Build bootstrap list: all other nodes
  local bootnodes=()
  for j in 0 1 2 3 4; do
    if [[ $j -ne $idx ]]; then
      bootnodes+=("\"$HOST_ADDR:$((BASE_P2P + j))\"")
    fi
  done
  local boot_list
  boot_list=$(IFS=,; echo "${bootnodes[*]}")

  cat >"$dir/config.toml" <<EOF
[chain]
chain_id = $CHAIN_ID
genesis_hash = ""
block_time = 2
ghostdag_k = 18

[network]
listen_addr = "$HOST_ADDR:$p2p"
bootstrap_nodes = [$boot_list]
max_peers = 32

[rpc]
enabled = true
listen_addr = "127.0.0.1:$rpc"
ws_addr = "127.0.0.1:$ws"

[storage]
data_dir = "$dir/data"
pruning = false
keep_blocks = 100000

[mining]
enabled = true
coinbase = "0000000000000000000000000000000000000000000000000000000000000000"
target_block_time = 2
min_gas_price = 1000000000
EOF
}

start_node() {
  local idx=$1
  local dir="$SCAFFOLD_DIR/node$idx"
  mkdir -p "$dir"
  echo "Starting node$idx on $HOST_ADDR:$((BASE_P2P + idx)) (RPC 127.0.0.1:$((BASE_RPC + idx)))"
  ("$BIN" --config "$dir/config.toml" >"$dir/node.log" 2>&1 & echo $! >"$dir/node.pid")
  sleep 0.2
}

stop_node() {
  local idx=$1
  local dir="$SCAFFOLD_DIR/node$idx"
  if [[ -f "$dir/node.pid" ]]; then
    local pid
    pid=$(cat "$dir/node.pid")
    if kill -0 "$pid" 2>/dev/null; then
      echo "Stopping node$idx (pid $pid)"
      kill "$pid" 2>/dev/null || true
    fi
    rm -f "$dir/node.pid"
  fi
}

case "$ACTION" in
  up)
    echo "Building lattice-node (release)…"
    (cd "$ROOT_DIR" && cargo build --release -p lattice-node)
    mkdir -p "$SCAFFOLD_DIR"
    echo "Using host address: $HOST_ADDR"
    for i in 0 1 2 3 4; do
      write_config "$i"
    done
    for i in 0 1 2 3 4; do
      start_node "$i"
    done
    echo "All nodes started. Logs under $SCAFFOLD_DIR/nodeX/node.log"
    ;;
  down)
    for i in 0 1 2 3 4; do
      stop_node "$i"
    done
    ;;
  clean)
    echo "Removing $SCAFFOLD_DIR"
    rm -rf "$SCAFFOLD_DIR"
    ;;
  *)
    echo "Unknown action: $ACTION" >&2
    exit 1
    ;;
esac

