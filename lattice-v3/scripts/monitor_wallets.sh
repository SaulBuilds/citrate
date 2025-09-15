#!/bin/bash

# Real-time wallet monitoring for Lattice V3
# Shows balances and block rewards as they accrue

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
MAGENTA='\033[0;35m'
NC='\033[0m'

# Wallet addresses to monitor
VALIDATOR="0x48a3f6698a1384e6f97ae9cc2f6d6c94504ba8a5"
TREASURY="0x1111111111111111111111111111111111111111"
TEST_WALLET="0x92b3f6698a1384e6f97ae9cc2f6d6c94504ba8a6"

# Allow custom validator address
if [ ! -z "$1" ]; then
    VALIDATOR="$1"
fi

clear

echo -e "${CYAN}╔══════════════════════════════════════════════════════════════════════╗${NC}"
echo -e "${CYAN}║           LATTICE V3 WALLET MONITOR                                 ║${NC}"
echo -e "${CYAN}╚══════════════════════════════════════════════════════════════════════╝${NC}"

echo -e "\n${YELLOW}Monitoring Addresses:${NC}"
echo -e "Validator: ${CYAN}$VALIDATOR${NC}"
echo -e "Treasury:  ${CYAN}$TREASURY${NC}"
echo -e "Test:      ${CYAN}$TEST_WALLET${NC}"

echo -e "\n${YELLOW}Press Ctrl+C to exit${NC}\n"

# Function to get balance in LATT
get_balance() {
    local addr=$1
    local balance_hex=$(curl -s -X POST http://localhost:8545 \
        -H "Content-Type: application/json" \
        -d "{\"jsonrpc\":\"2.0\",\"method\":\"eth_getBalance\",\"params\":[\"$addr\",\"latest\"],\"id\":1}" \
        | grep -o '"result":"[^"]*"' | cut -d'"' -f4)
    
    if [ ! -z "$balance_hex" ] && [ "$balance_hex" != "null" ]; then
        local balance_wei=$(printf "%d" "$balance_hex" 2>/dev/null || echo "0")
        echo "scale=6; $balance_wei / 1000000000000000000" | bc 2>/dev/null || echo "0"
    else
        echo "0"
    fi
}

# Function to get block height
get_block_height() {
    local height_hex=$(curl -s -X POST http://localhost:8545 \
        -H "Content-Type: application/json" \
        -d '{"jsonrpc":"2.0","method":"eth_blockNumber","params":[],"id":1}' \
        | grep -o '"result":"[^"]*"' | cut -d'"' -f4)
    
    if [ ! -z "$height_hex" ] && [ "$height_hex" != "null" ]; then
        printf "%d" "$height_hex" 2>/dev/null || echo "0"
    else
        echo "0"
    fi
}

# Track previous balances for delta calculation
prev_validator=0
prev_treasury=0
prev_test=0
prev_height=0

# Main monitoring loop
while true; do
    # Get current data
    height=$(get_block_height)
    validator_balance=$(get_balance "$VALIDATOR")
    treasury_balance=$(get_balance "$TREASURY")
    test_balance=$(get_balance "$TEST_WALLET")
    
    # Calculate deltas
    height_delta=$((height - prev_height))
    validator_delta=$(echo "$validator_balance - $prev_validator" | bc 2>/dev/null || echo "0")
    treasury_delta=$(echo "$treasury_balance - $prev_treasury" | bc 2>/dev/null || echo "0")
    test_delta=$(echo "$test_balance - $prev_test" | bc 2>/dev/null || echo "0")
    
    # Clear previous output (keep header)
    printf "\033[6A"  # Move cursor up 6 lines
    
    # Display current state
    echo -e "${MAGENTA}══════════════════════════════════════════════════════════════════════${NC}"
    echo -e "${GREEN}Block Height:${NC} #$height $(if [ $height_delta -gt 0 ]; then echo -e "${GREEN}(+$height_delta)${NC}"; fi)"
    echo -e "${MAGENTA}══════════════════════════════════════════════════════════════════════${NC}"
    
    # Validator balance with delta
    echo -ne "${BLUE}Validator:${NC} ${CYAN}$(printf "%12.6f" $validator_balance) LATT${NC}"
    if (( $(echo "$validator_delta > 0" | bc -l) )); then
        echo -e " ${GREEN}(+$(printf "%.6f" $validator_delta))${NC}   "
    else
        echo "                    "
    fi
    
    # Treasury balance with delta
    echo -ne "${BLUE}Treasury: ${NC} ${CYAN}$(printf "%12.6f" $treasury_balance) LATT${NC}"
    if (( $(echo "$treasury_delta > 0" | bc -l) )); then
        echo -e " ${GREEN}(+$(printf "%.6f" $treasury_delta))${NC}   "
    else
        echo "                    "
    fi
    
    # Test wallet balance with delta
    echo -ne "${BLUE}Test:     ${NC} ${CYAN}$(printf "%12.6f" $test_balance) LATT${NC}"
    if (( $(echo "$test_delta != 0" | bc -l) )); then
        if (( $(echo "$test_delta > 0" | bc -l) )); then
            echo -e " ${GREEN}(+$(printf "%.6f" $test_delta))${NC}   "
        else
            echo -e " ${RED}($(printf "%.6f" $test_delta))${NC}   "
        fi
    else
        echo "                    "
    fi
    
    # Update previous values
    prev_height=$height
    prev_validator=$validator_balance
    prev_treasury=$treasury_balance
    prev_test=$test_balance
    
    # Wait before next update
    sleep 2
done