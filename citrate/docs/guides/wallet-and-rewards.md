# Citrate V3: Complete Wallet & Rewards System Guide

## Current Issues & Fixes

### 🔴 Critical Issue: Validator Address is Zero
**Problem**: Block rewards are being minted to `0x0000000000000000000000000000000000000000`
**Impact**: No real wallets receive tokens for network participation

### 🟡 Secondary Issue: GUI Not Showing Blocks
**Problem**: DAG Explorer might show empty even when blocks are synced
**Cause**: GUI node needs to be started to populate its storage

## Token Economics Overview

### Block Rewards Distribution
```
Total Block Reward: 10 LATT (10^19 wei)
├── Validator (90%): 9 LATT → Goes to block producer
└── Treasury (10%): 1 LATT → Goes to 0x1111111111111111111111111111111111111111
```

### Reward Configuration (node/src/producer.rs)
```rust
RewardConfig {
    block_reward: 10,              // 10 LATT per block
    halving_interval: 2_100_000,   // Halving every ~2.1M blocks
    inference_bonus: 1,             // AI inference rewards
    model_deployment_bonus: 1,      // Model deployment rewards
    treasury_percentage: 10,        // 10% to treasury
    treasury_address: [0x11; 20],   // Treasury address
}
```

## Fix #1: Set Proper Validator Address

### For Core Node (Immediate Fix)
```bash
# Stop current node
pkill -f citrate

# Start with a real coinbase address (your wallet)
# Replace with your actual wallet address from GUI
cat > testnet-config-fixed.toml << 'EOF'
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
data_dir = ".citrate-testnet"
pruning = false
keep_blocks = 10000

[mining]
enabled = true
# USE YOUR WALLET ADDRESS HERE (without 0x prefix, padded to 64 chars)
coinbase = "48a3f6698a1384e6f97ae9cc2f6d6c94504ba8a5000000000000000000000000"
target_block_time = 2
min_gas_price = 1000000000
EOF

# Restart with proper address
./target/release/citrate-node --config testnet-config-fixed.toml --data-dir .citrate-testnet --mine
```

### For GUI Node
The GUI automatically uses the reward address from your wallet:
```typescript
// GUI uses first wallet address as validator
const account = await walletService.getAccounts()[0];
await nodeService.setRewardAddress(account.address);
```

## Fix #2: Ensure Proper Token Accrual

### Check Your Balance
```bash
# Via CLI - Replace with your address
curl -X POST http://localhost:8545 -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","method":"eth_getBalance","params":["0x48a3f6698a1384e6f97ae9cc2f6d6c94504ba8a5","latest"],"id":1}'
```

### Monitor Rewards
```bash
# Watch validator rewards
tail -f /tmp/citrate_core.log | grep "Minted.*validator"

# Check treasury accumulation
curl -X POST http://localhost:8545 -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","method":"eth_getBalance","params":["0x1111111111111111111111111111111111111111","latest"],"id":1}'
```

## Complete Transaction Flow

### 1. Create & Sign Transaction (GUI)
```typescript
// wallet_manager.rs
let tx = Transaction {
    hash: calculate_tx_hash(&raw_tx),
    from: PublicKey::new(pubkey_bytes),
    to: Some(PublicKey::new(to_bytes)),
    value: U256::from_dec_str(&value)?,
    nonce: nonce,
    gas_price: U256::from(1_000_000_000u64),
    gas_limit: 21000,
    input: vec![],
    signature: Some(Signature { bytes: sig_vec }),
    tx_type: TransactionType::Legacy,
};
```

### 2. Broadcast via P2P
```rust
// Transaction enters mempool
mempool.add_transaction(tx, TxClass::Standard)

// Gossip to peers
peer_manager.broadcast(&NetworkMessage::NewTransaction { transaction })
```

### 3. Block Inclusion
```rust
// Producer selects from mempool
let txs = mempool.select_transactions(max_size, min_gas_price)

// Execute transactions
for tx in txs {
    let receipt = executor.execute_transaction(&tx, &state)
    receipts.push(receipt)
}
```

### 4. State Updates
```rust
// Transfer value
executor.transfer(from, to, value)

// Update nonces
state.increment_nonce(from)

// Deduct gas
let gas_used = receipt.gas_used
executor.deduct_balance(from, gas_used * gas_price)
```

## Testing End-to-End

### 1. Setup Test Wallets
```bash
# In GUI:
# 1. Create Wallet A (will receive mining rewards)
# 2. Create Wallet B (for testing transfers)
# 3. Copy Wallet A address
```

### 2. Configure Mining to Wallet A
```bash
# Convert address to hex (remove 0x prefix)
# Example: 0x48a3f6698a1384e6f97ae9cc2f6d6c94504ba8a5
# Becomes: 48a3f6698a1384e6f97ae9cc2f6d6c94504ba8a5

# Pad to 64 characters with zeros
# Result: 48a3f6698a1384e6f97ae9cc2f6d6c94504ba8a5000000000000000000000000

# Update testnet-config.toml [mining] section
```

### 3. Monitor Balance Growth
```bash
# Watch balance increase every 2 seconds (9 LATT per block)
while true; do
  curl -s -X POST http://localhost:8545 \
    -H "Content-Type: application/json" \
    -d '{"jsonrpc":"2.0","method":"eth_getBalance","params":["YOUR_ADDRESS","latest"],"id":1}' \
    | jq -r '.result' | xargs printf "Balance: %d wei\n"
  sleep 2
done
```

### 4. Test Transfer
```javascript
// In GUI Console (F12)
const accounts = await walletService.getAccounts();
const txRequest = {
  from: accounts[0].address,
  to: accounts[1].address,
  value: BigInt("1000000000000000000"), // 1 LATT
  gasLimit: 21000,
  gasPrice: BigInt("1000000000"), // 1 gwei
  nonce: accounts[0].nonce,
  chainId: 42069
};

const txHash = await walletService.sendTransaction(txRequest, "your_password");
console.log("Transaction sent:", txHash);
```

### 5. Verify Transaction
```bash
# Check transaction receipt
curl -X POST http://localhost:8545 -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","method":"eth_getTransactionReceipt","params":["TX_HASH"],"id":1}'

# Check new balances
curl -X POST http://localhost:8545 -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","method":"eth_getBalance","params":["RECEIVER_ADDRESS","latest"],"id":1}'
```

## Network Security Features

### 1. Signature Verification
- **Ed25519**: Native Citrate signatures (32-byte keys)
- **ECDSA**: Ethereum compatibility (secp256k1)
- **Dual Support**: Both schemes validated in mempool

### 2. Nonce Management
- Sequential nonce enforcement
- Pending pool for out-of-order transactions
- Automatic nonce suggestion via RPC

### 3. Gas & Fees
- Minimum gas price: 1 gwei (10^9 wei)
- Standard transfer: 21,000 gas
- Contract execution: Variable based on complexity

### 4. P2P Security
- Peer scoring system
- Ban duration for misbehaving peers
- Message deduplication
- Signature verification on gossip

## Debugging Commands

### Check Sync Status
```bash
# GUI sync status
tail -f /tmp/citrate_gui.log | grep -E "Sync:|height"

# Core block production
tail -f /tmp/citrate_core.log | grep "Produced block"
```

### Inspect Mempool
```bash
# Pending transactions (needs custom endpoint)
curl http://localhost:8545/mempool

# Transaction count for address
curl -X POST http://localhost:8545 -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","method":"eth_getTransactionCount","params":["ADDRESS","pending"],"id":1}'
```

### DAG Explorer Issues
If DAG shows no blocks:
1. Ensure GUI node is started (not just compiled)
2. Check storage location: `~/Library/Application Support/citrate-core/testnet/chain/`
3. Verify sync: `tail -f /tmp/citrate_gui.log | grep "Stored block"`

## Summary of Required Actions

1. ✅ **Fix validator address** - Set proper coinbase in config
2. ✅ **Monitor balances** - Use RPC to track token accrual
3. ✅ **Test transfers** - Send transactions between wallets
4. ✅ **Verify receipts** - Check transaction execution
5. ✅ **Debug with logs** - Use tail commands for real-time monitoring

## Expected Behavior When Fixed

- **Every 2 seconds**: New block produced
- **Validator wallet**: Receives 9 LATT (9×10^18 wei)
- **Treasury**: Receives 1 LATT (10^18 wei)
- **Transfers**: Execute in next block
- **DAG Explorer**: Shows growing chain with transactions
- **Wallet UI**: Displays increasing balance

The system is fully functional once the validator address is set correctly!