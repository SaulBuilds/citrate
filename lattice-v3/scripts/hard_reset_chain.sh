#!/bin/bash

# Complete hard reset of Lattice V3 blockchain
# Cleans all data, caches, wallets, and starts fresh from genesis

set -e

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
MAGENTA='\033[0;35m'
NC='\033[0m'

echo -e "${RED}╔══════════════════════════════════════════════════════════════════════╗${NC}"
echo -e "${RED}║           LATTICE V3 HARD RESET - COMPLETE CHAIN WIPE               ║${NC}"
echo -e "${RED}╚══════════════════════════════════════════════════════════════════════╝${NC}"

echo -e "\n${YELLOW}⚠️  WARNING: This will delete ALL blockchain data, wallets, and caches!${NC}"
echo -e "${YELLOW}Press Ctrl+C to cancel, or wait 5 seconds to continue...${NC}"
sleep 5

# Step 1: Kill all processes
echo -e "\n${BLUE}[1/8] Killing all Lattice processes...${NC}"
pkill -f "lattice" 2>/dev/null || true
pkill -f "tauri" 2>/dev/null || true
pkill -f "cargo.*lattice" 2>/dev/null || true
sleep 2
echo -e "${GREEN}[✓]${NC} All processes stopped"

# Step 2: Remove all blockchain data directories
echo -e "\n${BLUE}[2/8] Removing blockchain data directories...${NC}"
DIRS_TO_REMOVE=(
    ".lattice-testnet"
    ".lattice-devnet"
    ".lattice-mainnet"
    ".lattice"
    "data"
    "rocksdb"
    "storage"
    ".cache/lattice"
)

for dir in "${DIRS_TO_REMOVE[@]}"; do
    if [ -d "$dir" ]; then
        echo -e "  Removing: $dir"
        rm -rf "$dir"
    fi
done
echo -e "${GREEN}[✓]${NC} Blockchain data removed"

# Step 3: Clean GUI data
echo -e "\n${BLUE}[3/8] Cleaning GUI application data...${NC}"
GUI_DATA_DIR="$HOME/Library/Application Support/lattice-core"
if [ -d "$GUI_DATA_DIR" ]; then
    echo -e "  Removing: $GUI_DATA_DIR"
    rm -rf "$GUI_DATA_DIR"
fi

# Clean Tauri cache
TAURI_CACHE="$HOME/Library/Caches/lattice-core"
if [ -d "$TAURI_CACHE" ]; then
    echo -e "  Removing: $TAURI_CACHE"
    rm -rf "$TAURI_CACHE"
fi

# Clean GUI local storage
GUI_LOCAL="gui/lattice-core/gui-data"
if [ -d "$GUI_LOCAL" ]; then
    echo -e "  Removing: $GUI_LOCAL"
    rm -rf "$GUI_LOCAL"
fi
echo -e "${GREEN}[✓]${NC} GUI data cleaned"

# Step 4: Remove all wallet data
echo -e "\n${BLUE}[4/8] Removing wallet data...${NC}"
WALLET_DIRS=(
    "$HOME/.lattice-wallet"
    "$HOME/.lattice-wallet-test"
    ".lattice-wallet"
    "wallet-data"
    "keystore"
)

for dir in "${WALLET_DIRS[@]}"; do
    if [ -d "$dir" ]; then
        echo -e "  Removing: $dir"
        rm -rf "$dir"
    fi
done
echo -e "${GREEN}[✓]${NC} Wallet data removed"

# Step 5: Clean temporary files
echo -e "\n${BLUE}[5/8] Cleaning temporary files...${NC}"
rm -f /tmp/lattice*.log 2>/dev/null || true
rm -f /tmp/tauri*.log 2>/dev/null || true
rm -rf /tmp/.lattice* 2>/dev/null || true

# Remove lock files
find . -name "LOCK" -o -name "*.lock" 2>/dev/null | grep -E "(rocksdb|data|storage)" | while read lock; do
    echo -e "  Removing lock: $lock"
    rm -f "$lock"
done
echo -e "${GREEN}[✓]${NC} Temporary files cleaned"

# Step 6: Reset configuration files
echo -e "\n${BLUE}[6/8] Resetting configuration files...${NC}"
rm -f testnet-config.toml 2>/dev/null || true
rm -f devnet-config.toml 2>/dev/null || true
rm -f mainnet-config.toml 2>/dev/null || true
rm -f config.toml 2>/dev/null || true
echo -e "${GREEN}[✓]${NC} Configuration files reset"

# Step 7: Clean build artifacts (optional)
echo -e "\n${BLUE}[7/8] Clean build artifacts?${NC}"
echo -e "${YELLOW}This will require rebuilding the project (y/n):${NC}"
read -t 10 -n 1 clean_build 2>/dev/null || clean_build="n"
echo

if [[ "$clean_build" == "y" || "$clean_build" == "Y" ]]; then
    echo -e "  Cleaning Rust build..."
    cargo clean
    echo -e "  Cleaning GUI build..."
    cd gui/lattice-core
    rm -rf node_modules dist src-tauri/target 2>/dev/null || true
    cd ../..
    echo -e "${GREEN}[✓]${NC} Build artifacts cleaned"
else
    echo -e "${CYAN}[→]${NC} Keeping build artifacts"
fi

# Step 8: Create fresh genesis configuration
echo -e "\n${BLUE}[8/8] Creating fresh genesis configuration...${NC}"

# Create clean testnet config
cat > testnet-config.toml << 'EOF'
[chain]
chain_id = 42069
genesis_hash = ""
block_time = 2
ghostdag_k = 18

[network]
listen_addr = "0.0.0.0:30303"
bootstrap_nodes = []
max_peers = 100

[rpc]
enabled = true
listen_addr = "0.0.0.0:8545"
ws_addr = "0.0.0.0:8546"

[storage]
data_dir = ".lattice-testnet"
pruning = false
keep_blocks = 10000

[mining]
enabled = true
# Fresh validator address - will be replaced when wallet is created
coinbase = "0000000000000000000000000000000000000000000000000000000000000000"
target_block_time = 2
min_gas_price = 1000000000
EOF

# Create clean devnet config
cat > devnet-config.toml << 'EOF'
[chain]
chain_id = 1337
genesis_hash = ""
block_time = 1
ghostdag_k = 8

[network]
listen_addr = "0.0.0.0:30303"
bootstrap_nodes = []
max_peers = 50

[rpc]
enabled = true
listen_addr = "0.0.0.0:8545"
ws_addr = "0.0.0.0:8546"

[storage]
data_dir = ".lattice-devnet"
pruning = false
keep_blocks = 1000

[mining]
enabled = true
coinbase = "0000000000000000000000000000000000000000000000000000000000000000"
target_block_time = 1
min_gas_price = 1000000000
EOF

echo -e "${GREEN}[✓]${NC} Fresh configuration created"

# Summary
echo -e "\n${GREEN}═══════════════════════════════════════════════════════════════${NC}"
echo -e "${GREEN}HARD RESET COMPLETE!${NC}"
echo -e "${GREEN}═══════════════════════════════════════════════════════════════${NC}"

echo -e "\n${MAGENTA}What was cleaned:${NC}"
echo -e "  ✓ All blockchain data"
echo -e "  ✓ All wallet keystores"
echo -e "  ✓ GUI application data"
echo -e "  ✓ Temporary files and logs"
echo -e "  ✓ Lock files"
echo -e "  ✓ Configuration files"
if [[ "$clean_build" == "y" ]]; then
    echo -e "  ✓ Build artifacts"
fi

echo -e "\n${CYAN}Next steps:${NC}"
echo -e "1. Create a new wallet:"
echo -e "   ${YELLOW}./target/release/wallet --chain-id 42069 new${NC}"
echo -e ""
echo -e "2. Update coinbase in testnet-config.toml with your wallet address"
echo -e ""
echo -e "3. Start fresh testnet:"
echo -e "   ${YELLOW}./scripts/start_fresh_testnet.sh${NC}"
echo -e ""
echo -e "4. Or use the setup script:"
echo -e "   ${YELLOW}./scripts/setup_wallet_rewards.sh${NC}"

echo -e "\n${GREEN}The chain is now completely reset to genesis state.${NC}"