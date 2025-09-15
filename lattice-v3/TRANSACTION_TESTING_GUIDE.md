# Transaction Testing Guide

This guide helps you test transactions on the Lattice blockchain using the newly built wallet CLI and helper scripts.

## Prerequisites

1. **Build the wallet CLI** (already completed):
   ```bash
   cargo build -p lattice-wallet --bin wallet
   ```

2. **Build the node binary** (required for testing):
   ```bash
   cargo build -p lattice-node --bin lattice
   ```

3. **Optional: Install expect for automated testing**:
   ```bash
   # On macOS
   brew install expect
   
   # On Ubuntu/Debian  
   sudo apt-get install expect
   ```

## Built Binaries

After building, you'll have:
- **Wallet CLI**: `/Users/soleilklosowski/Downloads/lattice/lattice-v3/target/debug/wallet`
- **Node binary**: `/Users/soleilklosowski/Downloads/lattice/lattice-v3/target/debug/lattice`

## Testing Scripts

Two helper scripts have been created for you:

### 1. `scripts/wallet_helper.sh` - Interactive Wallet Operations

A comprehensive wrapper around the wallet CLI with simplified commands:

```bash
# Show wallet and chain info
./scripts/wallet_helper.sh info

# Create a new account
./scripts/wallet_helper.sh create my-account

# List all accounts
./scripts/wallet_helper.sh list

# Check account balance
./scripts/wallet_helper.sh balance 0

# Send a transaction
./scripts/wallet_helper.sh send 0 0x742d35Cc6e6B37b5ba5B27CFbCB2dFeB7b17b91c 1.5

# Run a quick transaction test
./scripts/wallet_helper.sh quick-test 0 0.1

# Debug RPC connection
./scripts/wallet_helper.sh debug-rpc

# Start interactive mode
./scripts/wallet_helper.sh interactive
```

### 2. `scripts/test_transaction.sh` - End-to-End Transaction Test

A fully automated test that creates accounts, checks balances, and sends transactions:

```bash
# Basic test with defaults
./scripts/test_transaction.sh

# Custom parameters via environment variables
AMOUNT=2.5 TO_ADDRESS=0x123...abc ./scripts/test_transaction.sh

# Import existing private key
IMPORT_PRIVATE_KEY=0xabc123... ./scripts/test_transaction.sh

# Keep test keystore after completion
KEEP_KEYSTORE=true ./scripts/test_transaction.sh
```

## Step-by-Step Testing Process

### Step 1: Start the Lattice Node

First, start a local node for testing:

```bash
# Start in one terminal
cargo run --bin lattice -- --data-dir .lattice-devnet
```

The node will start with default settings:
- RPC server on http://localhost:8545
- Chain ID 1337
- GhostDAG consensus

### Step 2: Verify RPC Connection

Test that the RPC server is working:

```bash
./scripts/wallet_helper.sh debug-rpc
```

You should see:
```
✓ RPC server responds to HTTP requests
✓ eth_blockNumber: 0x1
✓ eth_gasPrice: 0x4a817c800
✓ eth_chainId: 0x539
```

### Step 3: Test Wallet Operations

#### Create or Import Account

```bash
# Create a new account
./scripts/wallet_helper.sh create validator-account

# Or import the validator account with 9.22 LATT
# (You'll need the private key from your existing validator)
./scripts/wallet_helper.sh import 0xYOUR_PRIVATE_KEY validator-account
```

#### Check Balance

```bash
./scripts/wallet_helper.sh balance 0
```

Expected output with your validator account:
```
Address: 0x1234567890123456789012345678901234567890
Balance: 9.22 LATT
Nonce: 0
```

### Step 4: Send Test Transaction

#### Manual Transaction

```bash
# Send 1 LATT to test address
./scripts/wallet_helper.sh send 0 0x742d35Cc6e6B37b5ba5B27CFbCB2dFeB7b17b91c 1.0
```

#### Automated Transaction Test

```bash
# Run complete automated test
./scripts/test_transaction.sh
```

## Environment Variables

Customize behavior with these environment variables:

```bash
# RPC Configuration
export RPC_URL="http://localhost:8545"
export CHAIN_ID="1337"

# Keystore Location  
export KEYSTORE_DIR="$HOME/.lattice-wallet-test"

# Transaction Parameters
export FROM_INDEX="0"
export TO_ADDRESS="0x742d35Cc6e6B37b5ba5B27CFbCB2dFeB7b17b91c"
export AMOUNT="1.0"
export GAS_PRICE="20"
export GAS_LIMIT="21000"

# Testing Options
export IMPORT_PRIVATE_KEY="0x..."  # Use existing private key
export KEEP_KEYSTORE="true"        # Don't delete test keystore
```

## Troubleshooting

### Common Issues

1. **"Wallet CLI not found"**
   ```bash
   cargo build -p lattice-wallet --bin wallet
   ```

2. **"RPC server not accessible"**
   ```bash
   # Start the node
   cargo run --bin lattice -- --data-dir .lattice-devnet
   
   # Or check if something is running on port 8545
   lsof -i :8545
   ```

3. **"Account not found"**
   ```bash
   # List available accounts
   ./scripts/wallet_helper.sh list
   ```

4. **"Insufficient balance"**
   ```bash
   # Check balance first
   ./scripts/wallet_helper.sh balance 0
   
   # Use smaller amount or fund the account
   ```

### Debug Information

Get detailed debug information:

```bash
# Check RPC methods
./scripts/wallet_helper.sh debug-rpc

# Show wallet configuration
./scripts/wallet_helper.sh info

# List accounts with balances
./scripts/wallet_helper.sh list
```

### Manual RPC Testing

Test RPC methods directly:

```bash
# Get block number
curl -X POST http://localhost:8545 \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","method":"eth_blockNumber","params":[],"id":1}'

# Get balance
curl -X POST http://localhost:8545 \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","method":"eth_getBalance","params":["0xYOUR_ADDRESS","latest"],"id":1}'
```

## Known Issues and Solutions

Based on the CLAUDE.md documentation, there are known transaction pipeline issues:

1. **GUI Producer Issue**: If using GUI, transactions may not execute properly
   - **Solution**: Use the CLI wallet with standalone node (this setup)

2. **Address Mismatch**: 20-byte EVM addresses in 32-byte fields
   - **Solution**: Wallet CLI handles this correctly

3. **Nonce Reading**: May use "latest" instead of "pending"
   - **Monitor**: Check if sequential transactions work properly

4. **RPC Decoder Gaps**: Limited EIP-1559 support
   - **Workaround**: Use legacy transaction format (default in wallet CLI)

## Success Criteria

A successful test run should show:

1. ✅ Wallet CLI builds and runs
2. ✅ RPC connection established
3. ✅ Account creation/import works
4. ✅ Balance queries return correct values
5. ✅ Transactions are submitted successfully
6. ✅ Transaction receipts are received
7. ✅ Balances update after transactions

## Next Steps

After successful testing:

1. **Test with MetaMask**: Try connecting external wallets
2. **Smart Contract Deployment**: Use Foundry to deploy contracts
3. **Load Testing**: Run multiple parallel transactions
4. **GUI Integration**: Test the Tauri GUI wallet

## Files Created

This testing setup created the following files:

- `/Users/soleilklosowski/Downloads/lattice/lattice-v3/scripts/wallet_helper.sh` - Interactive wallet operations
- `/Users/soleilklosowski/Downloads/lattice/lattice-v3/scripts/test_transaction.sh` - Automated transaction testing
- `/Users/soleilklosowski/Downloads/lattice/lattice-v3/TRANSACTION_TESTING_GUIDE.md` - This guide

All scripts are executable and ready to use!