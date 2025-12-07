#!/bin/bash
# Safe cleanup script - removes only regenerable build artifacts
# Created: 2025-10-26

echo "🧹 Starting safe cleanup of build artifacts..."
echo ""

cd citrate

# Track space freed
echo "📊 Calculating current sizes..."
TARGET_SIZE=$(du -sh target 2>/dev/null | awk '{print $1}')
echo "  - Rust target/: $TARGET_SIZE"

# Clean Rust build artifacts
echo ""
echo "🦀 Cleaning Rust build artifacts (target/)..."
if [ -d "target" ]; then
    rm -rf target/
    echo "  ✅ Removed target/ ($TARGET_SIZE)"
else
    echo "  ℹ️  No target/ directory found"
fi

# Clean Foundry artifacts
echo ""
echo "⚒️  Cleaning Foundry build artifacts (contracts/out/)..."
if [ -d "contracts/out" ]; then
    OUT_SIZE=$(du -sh contracts/out 2>/dev/null | awk '{print $1}')
    rm -rf contracts/out/
    echo "  ✅ Removed contracts/out/ ($OUT_SIZE)"
else
    echo "  ℹ️  No contracts/out/ directory found"
fi

# Note about node_modules (leaving them for now - users can run npm ci)
echo ""
echo "ℹ️  Note: node_modules directories preserved (run 'npm ci' to restore if deleted)"
echo ""

echo "✨ Cleanup complete!"
echo ""
echo "To regenerate:"
echo "  - Rust: cargo build --release"
echo "  - Contracts: forge build"
echo "  - Node packages: npm ci && npm run build"
