#!/bin/bash
# Fresh Citrate Deployment Script
# Cleans all databases, builds, and redeploys with genesis models

set -e  # Exit on error

echo "🧹 CITRATE FRESH DEPLOYMENT"
echo "=============================="
echo ""
echo "⚠️  WARNING: This will DELETE all existing data!"
echo "   - All blockchain data"
echo "   - All wallet data"
echo "   - All build artifacts"
echo "   - GUI application data"
echo ""
read -p "Are you sure you want to continue? (yes/no): " confirm

if [ "$confirm" != "yes" ]; then
    echo "Aborted."
    exit 1
fi

echo ""
echo "🗑️  Step 1: Cleaning databases and data directories..."

# Clean Rust build artifacts
echo "   → Cleaning Rust builds..."
cd /Users/soleilklosowski/Downloads/citrate/citrate
cargo clean

# Clean node data directories
echo "   → Cleaning node databases..."
rm -rf .citrate-devnet
rm -rf .citrate-testnet
rm -rf .citrate-mainnet

# Clean GUI application data (macOS)
echo "   → Cleaning GUI data..."
rm -rf ~/Library/Application\ Support/citrate-core
rm -rf ~/Library/Application\ Support/citrate-gui

# Clean GUI build artifacts
echo "   → Cleaning GUI builds..."
cd gui/citrate-core
rm -rf node_modules/.vite
rm -rf dist
rm -rf src-tauri/target

echo ""
echo "🔧 Step 2: Rebuilding core components..."

# Rebuild Rust workspace
echo "   → Building Rust workspace (release mode)..."
cd /Users/soleilklosowski/Downloads/citrate/citrate
cargo build --release --bin citrate-node

# Rebuild GUI
echo "   → Installing GUI dependencies..."
cd gui/citrate-core
npm install

echo ""
echo "📦 Step 3: Preparing genesis models..."
echo ""
echo "Choose genesis model strategy:"
echo "   1) Minimal (placeholder only, deploy models later) - FAST"
echo "   2) Core 3 models (DistilBERT, ResNet-50, DistilGPT-2) - RECOMMENDED"
echo "   3) Full suite (7 models) - COMPREHENSIVE"
echo ""
read -p "Enter choice (1/2/3): " model_choice

case $model_choice in
    1)
        echo "   → Using minimal genesis (placeholder model only)"
        MODEL_STRATEGY="minimal"
        ;;
    2)
        echo "   → Preparing Core 3 models..."
        MODEL_STRATEGY="core3"
        # Check if Python tools are available
        if ! command -v python3 &> /dev/null; then
            echo "   ⚠️  Python3 not found. Will use minimal genesis instead."
            MODEL_STRATEGY="minimal"
        fi
        ;;
    3)
        echo "   → Preparing full model suite..."
        MODEL_STRATEGY="full"
        if ! command -v python3 &> /dev/null; then
            echo "   ⚠️  Python3 not found. Will use minimal genesis instead."
            MODEL_STRATEGY="minimal"
        fi
        ;;
    *)
        echo "   Invalid choice. Using minimal genesis."
        MODEL_STRATEGY="minimal"
        ;;
esac

echo ""
echo "⚙️  Step 4: Configure network settings..."
echo ""
echo "Choose network configuration:"
echo "   1) Devnet (local, single node)"
echo "   2) Testnet (multi-node, mining enabled)"
echo ""
read -p "Enter choice (1/2): " network_choice

case $network_choice in
    1)
        echo "   → Configuring for devnet..."
        NETWORK="devnet"
        CHAIN_ID=1337
        ;;
    2)
        echo "   → Configuring for testnet..."
        NETWORK="testnet"
        CHAIN_ID=42069
        ;;
    *)
        echo "   Invalid choice. Using devnet."
        NETWORK="devnet"
        CHAIN_ID=1337
        ;;
esac

echo ""
echo "🎬 Step 5: Starting deployment..."

# Start the node to generate genesis
echo "   → Starting Citrate node (generating genesis block)..."
cd /Users/soleilklosowski/Downloads/citrate/citrate

if [ "$NETWORK" == "devnet" ]; then
    ./target/release/citrate-node devnet &
    NODE_PID=$!
else
    ./target/release/citrate-node --config node/config/testnet.toml &
    NODE_PID=$!
fi

echo "   → Waiting for node to initialize (30 seconds)..."
sleep 30

# Check if node is running
if ! ps -p $NODE_PID > /dev/null; then
    echo "   ❌ Node failed to start. Check logs."
    exit 1
fi

echo "   ✅ Node started successfully (PID: $NODE_PID)"

# Deploy models if needed
if [ "$MODEL_STRATEGY" == "core3" ]; then
    echo ""
    echo "📦 Step 6: Deploying core models to genesis..."

    # Check if IPFS is running
    if ! pgrep -x "ipfs" > /dev/null; then
        echo "   → Starting IPFS daemon..."
        ipfs daemon &
        IPFS_PID=$!
        sleep 5
    fi

    cd /Users/soleilklosowski/Downloads/citrate/citrate/tools

    echo "   → Deploying DistilBERT..."
    python3 import_model.py huggingface distilbert-base-uncased --optimize || echo "   ⚠️  Failed to deploy DistilBERT"

    echo "   → Deploying ResNet-50..."
    python3 import_model.py huggingface microsoft/resnet-50 --optimize || echo "   ⚠️  Failed to deploy ResNet-50"

    echo "   → Deploying DistilGPT-2..."
    python3 import_model.py huggingface distilgpt2 --optimize || echo "   ⚠️  Failed to deploy DistilGPT-2"

elif [ "$MODEL_STRATEGY" == "full" ]; then
    echo ""
    echo "📦 Step 6: Deploying full model suite..."

    # Start IPFS if needed
    if ! pgrep -x "ipfs" > /dev/null; then
        echo "   → Starting IPFS daemon..."
        ipfs daemon &
        IPFS_PID=$!
        sleep 5
    fi

    cd /Users/soleilklosowski/Downloads/citrate/citrate/tools

    # Text models
    python3 import_model.py huggingface distilbert-base-uncased --optimize || true
    python3 import_model.py huggingface bert-base-uncased --optimize || true

    # Generation models
    python3 import_model.py huggingface distilgpt2 --optimize || true

    # Vision models
    python3 import_model.py huggingface microsoft/resnet-50 --optimize || true
    python3 import_model.py huggingface google/vit-base-patch16-224 --optimize || true
fi

echo ""
echo "✅ DEPLOYMENT COMPLETE!"
echo "=============================="
echo ""
echo "📊 Deployment Summary:"
echo "   Network: $NETWORK"
echo "   Chain ID: $CHAIN_ID"
echo "   Model Strategy: $MODEL_STRATEGY"
echo "   Node PID: $NODE_PID"
echo ""
echo "🔗 Connection Details:"
echo "   RPC: http://localhost:8545"
echo "   WebSocket: ws://localhost:8546"
if [ "$NETWORK" == "testnet" ]; then
    echo "   P2P: 30304"
fi
echo ""
echo "🎮 Next Steps:"
echo "   1. Open GUI: cd gui/citrate-core && npm run tauri dev"
echo "   2. Create wallet account (set new password)"
echo "   3. Start mining to earn rewards"
echo ""
echo "📝 Useful Commands:"
echo "   View logs: tail -f .citrate-$NETWORK/logs/*.log"
echo "   Stop node: kill $NODE_PID"
echo "   Check status: curl -X POST http://localhost:8545 -H 'Content-Type: application/json' -d '{\"jsonrpc\":\"2.0\",\"method\":\"eth_blockNumber\",\"params\":[],\"id\":1}'"
echo ""
