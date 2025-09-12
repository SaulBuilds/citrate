#!/bin/bash

echo "🚀 Starting Lattice Blockchain..."
echo "================================"

# Clean up any existing processes
pkill -f lattice 2>/dev/null
sleep 1

# Start the chain with proper logging
echo "Starting chain with devnet mode..."
RUST_LOG=info,lattice_api=info,lattice_sequencer=info ./node/target/release/lattice devnet

# Note: Press Ctrl+C to stop the chain