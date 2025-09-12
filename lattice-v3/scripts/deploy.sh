#!/bin/bash

# Lattice V3 Deployment Script
# Deploys the Lattice AI-native blockchain

set -e

# Colors for output
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

# Configuration
NETWORK=${1:-mainnet}
DATA_DIR=${2:-~/.lattice}
RPC_PORT=${3:-8545}
WS_PORT=${4:-8546}
REST_PORT=${5:-3000}
P2P_PORT=${6:-30303}

echo "==========================================="
echo "       LATTICE V3 DEPLOYMENT SCRIPT"
echo "==========================================="
echo ""
echo "Network: $NETWORK"
echo "Data Directory: $DATA_DIR"
echo "RPC Port: $RPC_PORT"
echo "WebSocket Port: $WS_PORT"
echo "REST API Port: $REST_PORT"
echo "P2P Port: $P2P_PORT"
echo ""

# Function to check if port is available
check_port() {
    local port=$1
    if lsof -Pi :$port -sTCP:LISTEN -t >/dev/null 2>&1; then
        echo -e "${RED}✗${NC} Port $port is already in use"
        return 1
    else
        echo -e "${GREEN}✓${NC} Port $port is available"
        return 0
    fi
}

# Step 1: Check prerequisites
echo -e "${YELLOW}Checking prerequisites...${NC}"
echo ""

# Check Rust installation
if command -v cargo &> /dev/null; then
    echo -e "${GREEN}✓${NC} Rust installed ($(cargo --version))"
else
    echo -e "${RED}✗${NC} Rust not installed. Please install from https://rustup.rs/"
    exit 1
fi

# Check ports
echo ""
echo -e "${YELLOW}Checking port availability...${NC}"
check_port $RPC_PORT || exit 1
check_port $WS_PORT || exit 1
check_port $REST_PORT || exit 1
check_port $P2P_PORT || exit 1

# Step 2: Build the project
echo ""
echo -e "${YELLOW}Building Lattice V3...${NC}"
echo ""

if [ "$NETWORK" = "mainnet" ] || [ "$NETWORK" = "testnet" ]; then
    echo "Building release version..."
    cargo build --release
    BINARY="./target/release/lattice"
else
    echo "Building debug version for devnet..."
    cargo build
    BINARY="./target/debug/lattice"
fi

if [ -f "$BINARY" ]; then
    echo -e "${GREEN}✓${NC} Build successful"
else
    echo -e "${RED}✗${NC} Build failed"
    exit 1
fi

# Step 3: Create data directory
echo ""
echo -e "${YELLOW}Setting up data directory...${NC}"
mkdir -p $DATA_DIR
mkdir -p $DATA_DIR/chain
mkdir -p $DATA_DIR/state
mkdir -p $DATA_DIR/models
mkdir -p $DATA_DIR/logs

echo -e "${GREEN}✓${NC} Data directory created at $DATA_DIR"

# Step 4: Generate configuration
echo ""
echo -e "${YELLOW}Generating configuration...${NC}"

cat > $DATA_DIR/config.toml << EOF
# Lattice V3 Configuration
[network]
id = "$NETWORK"
listen_addr = "0.0.0.0:$P2P_PORT"
max_peers = 50
enable_discovery = true

[consensus]
type = "ghostdag"
k_parameter = 16
block_time_seconds = 2
finality_depth = 100

[api]
rpc_enabled = true
rpc_port = $RPC_PORT
rpc_host = "127.0.0.1"
ws_enabled = true
ws_port = $WS_PORT
ws_host = "127.0.0.1"
rest_enabled = true
rest_port = $REST_PORT
rest_host = "127.0.0.1"

[storage]
db_path = "$DATA_DIR/chain"
state_path = "$DATA_DIR/state"
model_storage = "$DATA_DIR/models"
cache_size_mb = 1024

[ai]
enable_inference = true
enable_training = true
max_model_size_mb = 5000
inference_timeout_seconds = 30
cache_inference_results = true

[logging]
level = "info"
file = "$DATA_DIR/logs/lattice.log"
rotate_size_mb = 100
EOF

echo -e "${GREEN}✓${NC} Configuration generated at $DATA_DIR/config.toml"

# Step 5: Generate genesis block (for new networks)
if [ ! -f "$DATA_DIR/chain/genesis.json" ]; then
    echo ""
    echo -e "${YELLOW}Generating genesis block...${NC}"
    
    cat > $DATA_DIR/chain/genesis.json << EOF
{
  "version": 1,
  "network_id": "$NETWORK",
  "timestamp": $(date +%s),
  "initial_supply": "1000000000000000000000000000",
  "treasury_address": "0x1111111111111111111111111111111111111111",
  "consensus": {
    "type": "ghostdag",
    "k": 16,
    "difficulty": 1000
  },
  "ai_models": [
    {
      "id": "0x0000000000000000000000000000000000000000000000000000000000000001",
      "name": "Genesis Model",
      "owner": "0x1111111111111111111111111111111111111111",
      "framework": "PyTorch",
      "version": "1.0.0"
    }
  ],
  "validators": [],
  "allocations": {
    "0x1111111111111111111111111111111111111111": "100000000000000000000000000",
    "0x2222222222222222222222222222222222222222": "50000000000000000000000000"
  }
}
EOF
    
    echo -e "${GREEN}✓${NC} Genesis block generated"
fi

# Step 6: Create systemd service (optional)
if [ "$NETWORK" = "mainnet" ] || [ "$NETWORK" = "testnet" ]; then
    echo ""
    echo -e "${YELLOW}Creating systemd service...${NC}"
    
    sudo cat > /etc/systemd/system/lattice.service << EOF
[Unit]
Description=Lattice V3 AI Blockchain
After=network.target

[Service]
Type=simple
User=$USER
WorkingDirectory=$(pwd)
ExecStart=$BINARY --config $DATA_DIR/config.toml
Restart=always
RestartSec=10
StandardOutput=append:$DATA_DIR/logs/lattice.log
StandardError=append:$DATA_DIR/logs/lattice.error.log

[Install]
WantedBy=multi-user.target
EOF
    
    sudo systemctl daemon-reload
    echo -e "${GREEN}✓${NC} Systemd service created"
    echo ""
    echo "To start the service:"
    echo "  sudo systemctl start lattice"
    echo "  sudo systemctl enable lattice  # To start on boot"
fi

# Step 7: Start the node
echo ""
echo -e "${YELLOW}Starting Lattice node...${NC}"
echo ""

if [ "$NETWORK" = "devnet" ]; then
    # Development mode - run in foreground
    echo "Starting in development mode..."
    echo ""
    echo "Press Ctrl+C to stop"
    echo ""
    
    RUST_LOG=info $BINARY \
        --config $DATA_DIR/config.toml \
        --network devnet \
        --rpc-port $RPC_PORT \
        --ws-port $WS_PORT \
        --rest-port $REST_PORT \
        --p2p-port $P2P_PORT
else
    # Production mode - run in background
    echo "Starting in production mode..."
    
    nohup $BINARY \
        --config $DATA_DIR/config.toml \
        --network $NETWORK \
        --rpc-port $RPC_PORT \
        --ws-port $WS_PORT \
        --rest-port $REST_PORT \
        --p2p-port $P2P_PORT \
        > $DATA_DIR/logs/lattice.log 2>&1 &
    
    PID=$!
    echo $PID > $DATA_DIR/lattice.pid
    
    echo -e "${GREEN}✓${NC} Lattice node started (PID: $PID)"
    echo ""
    echo "Logs: tail -f $DATA_DIR/logs/lattice.log"
    echo "Stop: kill \$(cat $DATA_DIR/lattice.pid)"
fi

echo ""
echo "==========================================="
echo "        DEPLOYMENT COMPLETE!"
echo "==========================================="
echo ""
echo "API Endpoints:"
echo "  JSON-RPC: http://localhost:$RPC_PORT"
echo "  WebSocket: ws://localhost:$WS_PORT"
echo "  REST API: http://localhost:$REST_PORT"
echo "  P2P: localhost:$P2P_PORT"
echo ""
echo "Documentation: https://github.com/lattice-network/lattice-v3"
echo ""