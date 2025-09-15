#!/bin/bash
set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Script configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
LATTICE_DIR="$SCRIPT_DIR/.."
WALLET_CLI="$LATTICE_DIR/target/release/wallet"
RPC_URL="${RPC_URL:-http://localhost:8545}"
CHAIN_ID="${CHAIN_ID:-42069}"
KEYSTORE_DIR="$HOME/.lattice-wallet-test"

# Default test values
DEFAULT_FROM_INDEX=0
DEFAULT_TO_ADDRESS="0x742d35Cc6e6B37b5ba5B27CFbCB2dFeB7b17b91c"
DEFAULT_AMOUNT="1.0"
DEFAULT_GAS_PRICE="20"
DEFAULT_GAS_LIMIT="21000"

print_header() {
    echo -e "${BLUE}===============================================${NC}"
    echo -e "${BLUE}           Lattice Transaction Test${NC}"
    echo -e "${BLUE}===============================================${NC}"
    echo
}

print_step() {
    echo -e "${YELLOW}Step $1: $2${NC}"
}

print_success() {
    echo -e "${GREEN}✓ $1${NC}"
}

print_error() {
    echo -e "${RED}✗ $1${NC}"
}

print_info() {
    echo -e "${BLUE}ℹ $1${NC}"
}

check_prerequisites() {
    print_step "1" "Checking prerequisites"
    
    # Check if wallet CLI exists
    if [ ! -f "$WALLET_CLI" ]; then
        print_error "Wallet CLI binary not found at $WALLET_CLI"
        echo "Please run: cargo build -p lattice-wallet --bin wallet"
        exit 1
    fi
    print_success "Wallet CLI binary found"
    
    # Check if RPC server is running
    if ! curl -s "$RPC_URL" > /dev/null; then
        print_error "RPC server not accessible at $RPC_URL"
        echo "Please start the Lattice node with: cargo run --bin lattice-node"
        exit 1
    fi
    print_success "RPC server is accessible"
    
    echo
}

setup_test_keystore() {
    print_step "2" "Setting up test keystore"
    
    # Create clean keystore directory
    rm -rf "$KEYSTORE_DIR"
    mkdir -p "$KEYSTORE_DIR"
    print_success "Created clean keystore directory: $KEYSTORE_DIR"
    
    echo
}

check_wallet_info() {
    print_step "3" "Checking wallet connection"
    
    echo "Wallet connection information:"
    "$WALLET_CLI" --keystore "$KEYSTORE_DIR" --rpc "$RPC_URL" --chain-id "$CHAIN_ID" info || {
        print_error "Failed to connect to wallet/RPC"
        exit 1
    }
    print_success "Wallet connection verified"
    
    echo
}

import_or_create_account() {
    print_step "4" "Setting up test account"
    
    # Check if user wants to import existing key or create new
    if [ -n "$IMPORT_PRIVATE_KEY" ]; then
        echo "Importing account from private key..."
        echo "$IMPORT_PRIVATE_KEY" | "$WALLET_CLI" --keystore "$KEYSTORE_DIR" --rpc "$RPC_URL" --chain-id "$CHAIN_ID" import --key "$IMPORT_PRIVATE_KEY" --alias "test-account" || {
            print_error "Failed to import account"
            exit 1
        }
        print_success "Account imported successfully"
    else
        echo "Creating new test account..."
        echo -e "test123\ntest123" | "$WALLET_CLI" --keystore "$KEYSTORE_DIR" --rpc "$RPC_URL" --chain-id "$CHAIN_ID" new --alias "test-account" || {
            print_error "Failed to create account"
            exit 1
        }
        print_success "New account created"
        
        print_info "To use an existing account, set IMPORT_PRIVATE_KEY environment variable"
    fi
    
    echo
}

list_accounts() {
    print_step "5" "Listing accounts"
    
    echo -e "test123" | "$WALLET_CLI" --keystore "$KEYSTORE_DIR" --rpc "$RPC_URL" --chain-id "$CHAIN_ID" list || {
        print_error "Failed to list accounts"
        exit 1
    }
    
    echo
}

check_balance() {
    print_step "6" "Checking account balance"
    
    local from_index="${FROM_INDEX:-$DEFAULT_FROM_INDEX}"
    
    echo "Checking balance for account index $from_index:"
    "$WALLET_CLI" --keystore "$KEYSTORE_DIR" --rpc "$RPC_URL" --chain-id "$CHAIN_ID" balance "$from_index" || {
        print_error "Failed to check balance"
        exit 1
    }
    
    echo
}

send_transaction() {
    print_step "7" "Sending test transaction"
    
    local from_index="${FROM_INDEX:-$DEFAULT_FROM_INDEX}"
    local to_address="${TO_ADDRESS:-$DEFAULT_TO_ADDRESS}"
    local amount="${AMOUNT:-$DEFAULT_AMOUNT}"
    local gas_price="${GAS_PRICE:-$DEFAULT_GAS_PRICE}"
    local gas_limit="${GAS_LIMIT:-$DEFAULT_GAS_LIMIT}"
    
    echo "Transaction details:"
    echo "  From index: $from_index"
    echo "  To address: $to_address"
    echo "  Amount:     $amount LATT"
    echo "  Gas price:  $gas_price gwei"
    echo "  Gas limit:  $gas_limit"
    echo
    
    # Use expect to automate password input and confirmation
    if command -v expect >/dev/null 2>&1; then
        expect << EOF
spawn "$WALLET_CLI" --keystore "$KEYSTORE_DIR" --rpc "$RPC_URL" --chain-id "$CHAIN_ID" send --from "$from_index" --to "$to_address" --amount "$amount" --gas-price "$gas_price" --gas-limit "$gas_limit"
expect "Enter password to unlock wallet:"
send "test123\r"
expect "Send transaction?"
send "y\r"
expect eof
EOF
    else
        # Fallback without expect
        print_info "For automated testing, install 'expect' package"
        echo "test123" | "$WALLET_CLI" --keystore "$KEYSTORE_DIR" --rpc "$RPC_URL" --chain-id "$CHAIN_ID" send --from "$from_index" --to "$to_address" --amount "$amount" --gas-price "$gas_price" --gas-limit "$gas_limit" || {
            print_error "Transaction failed"
            exit 1
        }
    fi
    
    print_success "Transaction sent successfully"
    
    echo
}

final_balance_check() {
    print_step "8" "Final balance check"
    
    local from_index="${FROM_INDEX:-$DEFAULT_FROM_INDEX}"
    
    echo "Checking balance after transaction:"
    "$WALLET_CLI" --keystore "$KEYSTORE_DIR" --rpc "$RPC_URL" --chain-id "$CHAIN_ID" balance "$from_index" || {
        print_error "Failed to check final balance"
        exit 1
    }
    
    echo
}

cleanup() {
    if [ "$KEEP_KEYSTORE" != "true" ]; then
        print_info "Cleaning up test keystore (set KEEP_KEYSTORE=true to preserve)"
        rm -rf "$KEYSTORE_DIR"
    else
        print_info "Test keystore preserved at: $KEYSTORE_DIR"
    fi
}

show_usage() {
    echo "Usage: $0 [OPTIONS]"
    echo
    echo "Environment Variables:"
    echo "  RPC_URL           RPC server URL (default: http://localhost:8545)"
    echo "  CHAIN_ID          Chain ID (default: 1337)"
    echo "  FROM_INDEX        Sender account index (default: 0)"
    echo "  TO_ADDRESS        Recipient address (default: test address)"
    echo "  AMOUNT            Amount in LATT (default: 1.0)"
    echo "  GAS_PRICE         Gas price in gwei (default: 20)"
    echo "  GAS_LIMIT         Gas limit (default: 21000)"
    echo "  IMPORT_PRIVATE_KEY Private key to import (optional)"
    echo "  KEEP_KEYSTORE     Set to 'true' to preserve test keystore"
    echo
    echo "Examples:"
    echo "  $0                                    # Basic test with defaults"
    echo "  AMOUNT=0.5 $0                       # Send 0.5 LATT"
    echo "  TO_ADDRESS=0x123... $0              # Send to specific address"
    echo "  IMPORT_PRIVATE_KEY=0xabc... $0      # Use existing private key"
    echo "  KEEP_KEYSTORE=true $0               # Keep test keystore after run"
}

main() {
    if [ "$1" = "--help" ] || [ "$1" = "-h" ]; then
        show_usage
        exit 0
    fi
    
    print_header
    
    # Set trap for cleanup
    trap cleanup EXIT
    
    # Run test steps
    check_prerequisites
    setup_test_keystore
    check_wallet_info
    import_or_create_account
    list_accounts
    check_balance
    send_transaction
    final_balance_check
    
    print_success "Transaction test completed successfully!"
    
    echo
    print_info "Test Summary:"
    echo "  - Wallet CLI: Working ✓"
    echo "  - RPC Connection: Working ✓" 
    echo "  - Account Management: Working ✓"
    echo "  - Transaction Sending: Working ✓"
    echo "  - Balance Queries: Working ✓"
}

# Run main function
main "$@"