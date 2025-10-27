#!/usr/bin/env bash
set -euo pipefail

# Start a 2-node local network for E2E testing
SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

# Clean up any existing nodes and old data
if [ -f "$PROJECT_ROOT/scripts/stop_nodes.sh" ]; then
  bash "$PROJECT_ROOT/scripts/stop_nodes.sh" || true
fi
CONFIG_DIR="$PROJECT_ROOT/node/config"

CHAIN_ID=${CHAIN_ID:-1337}

# Node 1 ports and paths
RPC1=${RPC1:-8545}
WS1=${WS1:-8546}
P2P1=${P2P1:-30303}
DATA1="$PROJECT_ROOT/.citrate-mn-1"
CONF1="$CONFIG_DIR/testnet-1.toml"

# Node 2 ports and paths
RPC2=${RPC2:-8547}
WS2=${WS2:-8548}
P2P2=${P2P2:-30305}
DATA2="$PROJECT_ROOT/.citrate-mn-2"
CONF2="$CONFIG_DIR/testnet-2.toml"

mkdir -p "$CONFIG_DIR"
rm -rf "$DATA1" "$DATA2"

echo "Writing multi-node configs..."
cat > "$CONF1" << EOF
[chain]
chain_id = $CHAIN_ID
genesis_hash = ""
block_time = 2
ghostdag_k = 18

[network]
listen_addr = "127.0.0.1:$P2P1"
bootstrap_nodes = []
max_peers = 50

[rpc]
enabled = true
listen_addr = "127.0.0.1:$RPC1"
ws_addr = "127.0.0.1:$WS1"

[storage]
data_dir = "$DATA1"
pruning = false
keep_blocks = 100000

[mining]
enabled = true
coinbase = "0000000000000000000000000000000000000000000000000000000000000000"
target_block_time = 2
min_gas_price = 1000000000
EOF

cat > "$CONF2" << EOF
[chain]
chain_id = $CHAIN_ID
genesis_hash = ""
block_time = 2
ghostdag_k = 18

[network]
listen_addr = "127.0.0.1:$P2P2"
bootstrap_nodes = ["127.0.0.1:$P2P1"]
max_peers = 50

[rpc]
enabled = true
listen_addr = "127.0.0.1:$RPC2"
ws_addr = "127.0.0.1:$WS2"

[storage]
data_dir = "$DATA2"
pruning = false
keep_blocks = 100000

[mining]
enabled = true
coinbase = "0000000000000000000000000000000000000000000000000000000000000000"
target_block_time = 2
min_gas_price = 1000000000
EOF

echo "Building lattice node (debug)..."
cd "$PROJECT_ROOT"
cargo build -p citrate-node >/dev/null

echo "Starting Node 1 (RPC :$RPC1, P2P :$P2P1)"
CITRATE_REQUIRE_VALID_SIGNATURE=0 \
RUST_LOG=${RUST_LOG:-info} \
./target/debug/lattice --config "$CONF1" --data-dir "$DATA1" --mine \
  > "$PROJECT_ROOT/mn-node1.log" 2>&1 &
echo $! > "$PROJECT_ROOT/mn-node1.pid"

echo "Starting Node 2 (RPC :$RPC2, P2P :$P2P2)"
CITRATE_REQUIRE_VALID_SIGNATURE=0 \
RUST_LOG=${RUST_LOG:-info} \
./target/debug/lattice --config "$CONF2" --data-dir "$DATA2" --mine \
  > "$PROJECT_ROOT/mn-node2.log" 2>&1 &
echo $! > "$PROJECT_ROOT/mn-node2.pid"

echo "Waiting for RPCs..."
for port in $RPC1 $RPC2; do
  for i in $(seq 1 60); do
    if curl -s -X POST http://127.0.0.1:$port -H 'Content-Type: application/json' \
      -d '{"jsonrpc":"2.0","method":"eth_blockNumber","params":[],"id":1}' | grep -q '"result"'; then
      echo "RPC :$port is up"; break
    fi
    sleep 1
    if [ $i -eq 60 ]; then echo "RPC :$port not responding"; exit 1; fi
  done
done

echo "Multi-node started. PIDs: $(cat mn-node1.pid), $(cat mn-node2.pid)"
echo "Logs: $PROJECT_ROOT/mn-node1.log, $PROJECT_ROOT/mn-node2.log"
