#!/bin/bash
# Test genesis block creation and verification

set -e

echo "==================================="
echo "Genesis Block Verification Test"
echo "==================================="
echo ""

# Find project root
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
cd "$PROJECT_ROOT"

# Create temporary data directory
TEST_DATA_DIR=$(mktemp -d -t citrate-genesis-test)
echo "Test data directory: $TEST_DATA_DIR"
echo ""

# Initialize devnet (this creates genesis)
echo "Creating genesis block..."
./target/release/citrate --data-dir "$TEST_DATA_DIR" devnet --initialize-only 2>&1 | head -20 || {
    # If the command doesn't support --initialize-only, just capture the log
    timeout 2 ./target/release/citrate --data-dir "$TEST_DATA_DIR" devnet 2>&1 | grep -i "genesis" | head -20 || true
}

echo ""
echo "Checking genesis block file..."

# Check if genesis block was created
if [ -f "$TEST_DATA_DIR/genesis.dat" ]; then
    GENESIS_SIZE=$(stat -f%z "$TEST_DATA_DIR/genesis.dat" 2>/dev/null || stat -c%s "$TEST_DATA_DIR/genesis.dat" 2>/dev/null)
    GENESIS_SIZE_MB=$(echo "scale=2; $GENESIS_SIZE / 1024 / 1024" | bc)

    echo "✓ Genesis block created"
    echo "  File: $TEST_DATA_DIR/genesis.dat"
    echo "  Size: $GENESIS_SIZE bytes ($GENESIS_SIZE_MB MB)"
    echo ""

    # Verify size is approximately 437 MB for embedded BGE-M3
    if (( $(echo "$GENESIS_SIZE_MB >= 400 && $GENESIS_SIZE_MB <= 500" | bc -l) )); then
        echo "✓ Genesis size looks correct (~437 MB for BGE-M3)"
    else
        echo "⚠ Warning: Genesis size is outside expected range (400-500 MB)"
        echo "  This might indicate missing or incorrect embedded model"
    fi
else
    echo "✗ Genesis block file not found"
    echo "  Looking in: $TEST_DATA_DIR/"
    ls -lh "$TEST_DATA_DIR/" || echo "Directory empty or doesn't exist"
fi

echo ""
echo "Checking logs for genesis info..."
echo "-----------------------------------"

# Try to extract genesis info from logs
if [ -f "$TEST_DATA_DIR/citrate.log" ]; then
    grep -i "genesis\|embedded\|model" "$TEST_DATA_DIR/citrate.log" | head -20 || echo "No relevant log entries found"
else
    echo "No log file found at $TEST_DATA_DIR/citrate.log"
fi

echo ""
echo "-----------------------------------"
echo "Cleaning up test directory..."
rm -rf "$TEST_DATA_DIR"
echo "✓ Test complete"
