#!/bin/bash

# Clean start script for Citrate v3
# Removes all state databases and build artifacts, then starts fresh

echo "🧹 Cleaning all Citrate state databases and build artifacts..."

# Kill any running processes on common ports
echo "⚡ Stopping any running services..."
lsof -i :3456 2>/dev/null | grep LISTEN | awk '{print $2}' | xargs kill -9 2>/dev/null || true
lsof -i :8545 2>/dev/null | grep LISTEN | awk '{print $2}' | xargs kill -9 2>/dev/null || true
pkill -f "citrate" 2>/dev/null || true
pkill -f "tauri" 2>/dev/null || true

# Remove all database directories
echo "🗑️  Removing state databases..."
rm -rf .citrate-devnet/
rm -rf .citrate-testnet/
rm -rf .citrate-mainnet/
rm -rf gui/citrate-core/.citrate-devnet/

# Clean build artifacts
echo "🗑️  Cleaning build artifacts..."
rm -rf target/release/
rm -rf target/debug/
cargo clean

# Clean GUI build artifacts
echo "🗑️  Cleaning GUI build artifacts..."
cd gui/citrate-core
rm -rf node_modules/.vite/
rm -rf dist/
cd ../..

# Remove any lock files
rm -f *.lock
rm -f Cargo.lock

# Remove any temp files
rm -f /tmp/citrate-*
rm -rf /tmp/cargo-*

echo "✅ All state databases and build artifacts cleaned"
echo ""
echo "📦 Building the project (this will take a few minutes)..."
cargo build --release

if [ $? -eq 0 ]; then
    echo "✅ Build successful!"
    echo ""
    echo "🚀 Starting fresh chain..."
    echo "Run one of the following commands to start:"
    echo "  - GUI: cd gui/citrate-core && npm run tauri dev"
    echo "  - CLI Node: ./target/release/lattice --chain devnet"
    echo "  - Test Script: ./scripts/test_transaction.sh"
else
    echo "❌ Build failed. Please check the errors above."
    echo ""
    echo "Try these steps manually:"
    echo "  1. cargo clean"
    echo "  2. rm -rf target/"
    echo "  3. cargo build --release"
fi