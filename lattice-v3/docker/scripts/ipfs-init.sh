#!/bin/sh
set -e

# Initialize IPFS if not already done
if [ ! -f /data/ipfs/config ]; then
    echo "Initializing IPFS..."
    ipfs init --profile server

    # Configure IPFS for production use
    ipfs config Addresses.API /ip4/0.0.0.0/tcp/5001
    ipfs config Addresses.Gateway /ip4/0.0.0.0/tcp/8080
    ipfs config --json API.HTTPHeaders.Access-Control-Allow-Origin '["*"]'
    ipfs config --json API.HTTPHeaders.Access-Control-Allow-Methods '["PUT", "GET", "POST"]'

    # Enable experimental features for Lattice integration
    ipfs config --json Experimental.FilestoreEnabled true
    ipfs config --json Experimental.UrlstoreEnabled true
    ipfs config --json Experimental.P2pHttpProxy true

    # Set storage and networking optimizations
    ipfs config Datastore.StorageMax 100GB
    ipfs config --json Swarm.ConnMgr.HighWater 900
    ipfs config --json Swarm.ConnMgr.LowWater 600

    echo "IPFS initialized for Lattice integration"
fi

# Start IPFS daemon
echo "Starting IPFS daemon..."
exec ipfs daemon --migrate=true --agent-version-suffix=lattice