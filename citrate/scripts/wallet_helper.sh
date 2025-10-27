#!/bin/bash
set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# Configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
CITRATE_DIR="$SCRIPT_DIR/.."
WALLET_CLI="$CITRATE_DIR/target/debug/wallet"
RPC_URL="${RPC_URL:-http://localhost:8545}"
CHAIN_ID="${CHAIN_ID:-1337}"
KEYSTORE_DIR="${KEYSTORE_DIR:-$HOME/.citrate-wallet}"

# Utility functions
print_header() {
    echo -e "${BLUE}===============================================${NC}"
    echo -e "${BLUE}           Citrate Wallet Helper${NC}"
    echo -e "${BLUE}===============================================${NC}"
    echo
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

print_warning() {
    echo -e "${YELLOW}⚠ $1${NC}"
}

check_prerequisites() {
    local errors=0
    
    echo -e "${CYAN}Checking prerequisites...${NC}"
    echo
    
    # Check wallet binary
    if [ ! -f "$WALLET_CLI" ]; then
        print_error "Wallet CLI not found at $WALLET_CLI"
        echo "  Run: cargo build -p citrate-wallet --bin wallet"
        errors=$((errors + 1))
    else
        print_success "Wallet CLI found"
    fi
    
    # Check RPC connection
    if curl -s -f "$RPC_URL" >/dev/null 2>&1; then
        print_success "RPC server accessible at $RPC_URL"
    else
        print_warning "RPC server not accessible at $RPC_URL"
        echo "  Start node with: cargo run --bin citrate-node"
        echo "  Or use different RPC_URL environment variable"
    fi
    
    # Check keystore directory
    if [ -d "$KEYSTORE_DIR" ]; then
        print_info "Keystore directory: $KEYSTORE_DIR"
    else
        print_info "Keystore will be created at: $KEYSTORE_DIR"
    fi
    
    echo
    return $errors
}

run_wallet_command() {
    local cmd="$1"
    shift
    echo -e "${CYAN}Running: wallet $cmd $@${NC}"
    "$WALLET_CLI" --keystore "$KEYSTORE_DIR" --rpc "$RPC_URL" --chain-id "$CHAIN_ID" "$cmd" "$@"
}

cmd_info() {
    echo -e "${CYAN}=== Wallet Info ===${NC}"
    run_wallet_command info
    echo
}

cmd_create() {
    echo -e "${CYAN}=== Create Account ===${NC}"
    local alias="${1:-new-account}"
    echo "Creating account with alias: $alias"
    run_wallet_command new --alias "$alias"
    echo
}

cmd_import() {
    echo -e "${CYAN}=== Import Account ===${NC}"
    local private_key="$1"
    local alias="${2:-imported-account}"
    
    if [ -z "$private_key" ]; then
        echo "Usage: $0 import <private_key> [alias]"
        echo "Example: $0 import 0x1234...abcd my-account"
        exit 1
    fi
    
    echo "Importing account with alias: $alias"
    run_wallet_command import --key "$private_key" --alias "$alias"
    echo
}

cmd_list() {
    echo -e "${CYAN}=== Account List ===${NC}"
    run_wallet_command list
    echo
}

cmd_balance() {
    echo -e "${CYAN}=== Balance Check ===${NC}"
    local account="$1"
    
    if [ -z "$account" ]; then
        echo "Checking all account balances:"
        run_wallet_command balance
    else
        echo "Checking balance for: $account"
        run_wallet_command balance "$account"
    fi
    echo
}

cmd_send() {
    echo -e "${CYAN}=== Send Transaction ===${NC}"
    local from="$1"
    local to="$2"
    local amount="$3"
    local gas_price="${4:-20}"
    local gas_limit="${5:-21000}"
    
    if [ -z "$from" ] || [ -z "$to" ] || [ -z "$amount" ]; then
        echo "Usage: $0 send <from_index> <to_address> <amount> [gas_price] [gas_limit]"
        echo "Example: $0 send 0 0x742d35Cc6e6B37b5ba5B27CFbCB2dFeB7b17b91c 1.5 20 21000"
        exit 1
    fi
    
    echo "Transaction details:"
    echo "  From index: $from"
    echo "  To address: $to"
    echo "  Amount:     $amount LATT"
    echo "  Gas price:  $gas_price gwei"
    echo "  Gas limit:  $gas_limit"
    echo
    
    run_wallet_command send --from "$from" --to "$to" --amount "$amount" --gas-price "$gas_price" --gas-limit "$gas_limit"
    echo
}

cmd_export() {
    echo -e "${CYAN}=== Export Private Key ===${NC}"
    local index="$1"
    
    if [ -z "$index" ]; then
        echo "Usage: $0 export <account_index>"
        echo "Example: $0 export 0"
        exit 1
    fi
    
    print_warning "SECURITY WARNING: Private keys should be kept secret!"
    echo
    
    run_wallet_command export "$index"
    echo
}

cmd_interactive() {
    echo -e "${CYAN}=== Interactive Mode ===${NC}"
    run_wallet_command interactive
}

cmd_quick_test() {
    echo -e "${CYAN}=== Quick Transaction Test ===${NC}"
    local from_index="${1:-0}"
    local amount="${2:-0.1}"
    local to_address="${3:-0x742d35Cc6e6B37b5ba5B27CFbCB2dFeB7b17b91c}"
    
    echo "Running quick transaction test:"
    echo "  From index: $from_index"
    echo "  Amount:     $amount LATT"
    echo "  To address: $to_address"
    echo
    
    # Check balance before
    echo "Balance before transaction:"
    cmd_balance "$from_index"
    
    # Send transaction
    echo "Sending transaction..."
    cmd_send "$from_index" "$to_address" "$amount"
    
    # Check balance after
    echo "Balance after transaction:"
    cmd_balance "$from_index"
}

cmd_debug_rpc() {
    echo -e "${CYAN}=== RPC Debug Information ===${NC}"
    
    # Test basic RPC calls
    echo "Testing RPC connectivity..."
    
    # Check if server responds
    if curl -s -f "$RPC_URL" >/dev/null 2>&1; then
        print_success "RPC server responds to HTTP requests"
    else
        print_error "RPC server not responding"
        return 1
    fi
    
    # Test JSON-RPC methods
    echo
    echo "Testing JSON-RPC methods:"
    
    # eth_blockNumber
    local block_response=$(curl -s -X POST "$RPC_URL" \
        -H "Content-Type: application/json" \
        -d '{"jsonrpc":"2.0","method":"eth_blockNumber","params":[],"id":1}')
    
    if echo "$block_response" | grep -q "result"; then
        local block_number=$(echo "$block_response" | grep -o '"result":"[^"]*"' | cut -d'"' -f4)
        print_success "eth_blockNumber: $block_number"
    else
        print_error "eth_blockNumber failed"
        echo "Response: $block_response"
    fi
    
    # eth_gasPrice
    local gas_response=$(curl -s -X POST "$RPC_URL" \
        -H "Content-Type: application/json" \
        -d '{"jsonrpc":"2.0","method":"eth_gasPrice","params":[],"id":1}')
    
    if echo "$gas_response" | grep -q "result"; then
        local gas_price=$(echo "$gas_response" | grep -o '"result":"[^"]*"' | cut -d'"' -f4)
        print_success "eth_gasPrice: $gas_price"
    else
        print_error "eth_gasPrice failed"
        echo "Response: $gas_response"
    fi
    
    # eth_chainId
    local chain_response=$(curl -s -X POST "$RPC_URL" \
        -H "Content-Type: application/json" \
        -d '{"jsonrpc":"2.0","method":"eth_chainId","params":[],"id":1}')
    
    if echo "$chain_response" | grep -q "result"; then
        local chain_id=$(echo "$chain_response" | grep -o '"result":"[^"]*"' | cut -d'"' -f4)
        print_success "eth_chainId: $chain_id"
    else
        print_error "eth_chainId failed"
        echo "Response: $chain_response"
    fi
    
    echo
}

show_usage() {
    echo "Citrate Wallet Helper - Simplified wallet operations"
    echo
    echo "Usage: $0 <command> [arguments]"
    echo
    echo "Commands:"
    echo "  info                           - Show wallet and chain information"
    echo "  create [alias]                 - Create new account"
    echo "  import <private_key> [alias]   - Import account from private key"
    echo "  list                          - List all accounts"
    echo "  balance [account]             - Check balance (account index or address)"
    echo "  send <from> <to> <amount> [gas_price] [gas_limit]"
    echo "                                - Send transaction"
    echo "  export <account_index>        - Export private key"
    echo "  interactive                   - Start interactive mode"
    echo "  quick-test [from] [amount] [to] - Run quick transaction test"
    echo "  debug-rpc                     - Debug RPC connection"
    echo "  help                          - Show this help"
    echo
    echo "Environment Variables:"
    echo "  RPC_URL       - RPC server URL (default: http://localhost:8545)"
    echo "  CHAIN_ID      - Chain ID (default: 1337)"
    echo "  KEYSTORE_DIR  - Keystore directory (default: ~/.citrate-wallet)"
    echo
    echo "Examples:"
    echo "  $0 info                                     # Show wallet info"
    echo "  $0 create my-account                        # Create account"
    echo "  $0 list                                     # List accounts"
    echo "  $0 balance 0                               # Check account 0 balance"
    echo "  $0 send 0 0x742d35...7b91c 1.5            # Send 1.5 LATT"
    echo "  $0 quick-test 0 0.1                       # Quick test with 0.1 LATT"
    echo "  $0 debug-rpc                               # Debug RPC connection"
    echo
    echo "Config:"
    echo "  RPC URL:     $RPC_URL"
    echo "  Chain ID:    $CHAIN_ID"  
    echo "  Keystore:    $KEYSTORE_DIR"
    echo "  Wallet CLI:  $WALLET_CLI"
}

main() {
    local command="$1"
    shift || true
    
    case "$command" in
        "info")
            print_header
            check_prerequisites
            cmd_info
            ;;
        "create")
            print_header
            check_prerequisites
            cmd_create "$@"
            ;;
        "import")
            print_header
            check_prerequisites
            cmd_import "$@"
            ;;
        "list")
            print_header
            check_prerequisites
            cmd_list
            ;;
        "balance")
            print_header
            check_prerequisites
            cmd_balance "$@"
            ;;
        "send")
            print_header
            check_prerequisites
            cmd_send "$@"
            ;;
        "export")
            print_header
            check_prerequisites
            cmd_export "$@"
            ;;
        "interactive")
            print_header
            check_prerequisites
            cmd_interactive
            ;;
        "quick-test")
            print_header
            check_prerequisites
            cmd_quick_test "$@"
            ;;
        "debug-rpc")
            print_header
            cmd_debug_rpc
            ;;
        "help"|"--help"|"-h"|"")
            show_usage
            ;;
        *)
            print_error "Unknown command: $command"
            echo
            show_usage
            exit 1
            ;;
    esac
}

main "$@"