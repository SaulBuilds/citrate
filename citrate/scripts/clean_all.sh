#!/bin/bash
# Quick cleanup script - removes all data and wallet files
# Use this when you just want to start fresh without rebuilding

set -e

echo "🗑️  CITRATE DATA CLEANUP"
echo "======================="
echo ""
echo "⚠️  This will DELETE:"
echo "   ✗ All blockchain data (.citrate-* directories)"
echo "   ✗ All wallet accounts and keys (macOS Keychain entries)"
echo "   ✗ GUI application data"
echo "   ✗ Build artifacts (optional)"
echo ""
read -p "Continue? (yes/no): " confirm

if [ "$confirm" != "yes" ]; then
    echo "Aborted."
    exit 1
fi

cd /Users/soleilklosowski/Downloads/citrate/citrate

echo ""
echo "Cleaning blockchain data..."
rm -rf .citrate-devnet
rm -rf .citrate-testnet
rm -rf .citrate-mainnet
echo "✅ Blockchain data removed"

echo ""
echo "Cleaning GUI application data..."
rm -rf ~/Library/Application\ Support/citrate-core
rm -rf ~/Library/Application\ Support/citrate-gui
echo "✅ GUI data removed"

echo ""
echo "Cleaning wallet keychain entries..."
# Note: macOS Keychain entries for "citrate-core" service
echo "   (Wallet keys in macOS Keychain will persist - remove manually if needed)"
echo "   To remove: Open Keychain Access.app → Search 'citrate-core' → Delete entries"

echo ""
read -p "Also clean build artifacts? (yes/no): " clean_builds

if [ "$clean_builds" == "yes" ]; then
    echo ""
    echo "Cleaning Rust builds..."
    cargo clean
    echo "✅ Rust builds cleaned"

    echo ""
    echo "Cleaning GUI builds..."
    cd gui/citrate-core
    rm -rf node_modules/.vite
    rm -rf dist
    rm -rf src-tauri/target
    echo "✅ GUI builds cleaned"
fi

echo ""
echo "✅ CLEANUP COMPLETE!"
echo ""
echo "Next steps:"
echo "   1. Run: ./scripts/fresh_deploy.sh (full redeployment)"
echo "   OR"
echo "   2. Just start the node: cargo run --release --bin citrate-node devnet"
echo ""
