# Sprint 02: Tracked Addresses Backend

**Status**: Blocked (waiting for Sprint 01)
**Priority**: P0 Critical
**Duration**: 1-2 days
**Depends On**: Sprint 01

---

## Problem Statement

The "Tracked Addresses" feature in the GUI stores addresses in localStorage only. There's no backend RPC support for fetching balances or activity for arbitrary addresses, making the feature non-functional.

### Current State
- Frontend: Stores tracked addresses in `localStorage`
- Frontend calls: `walletService.getObservedBalance()` - **UNDEFINED**
- Frontend calls: `walletService.getAccountActivity()` - **UNDEFINED**
- Backend: No RPC endpoints for observing external addresses

### Target State
- Tracked addresses persist in backend storage
- `eth_getObservedBalance` RPC returns balance for any address
- `citrate_getObservedActivity` RPC returns transaction history
- Feature works end-to-end

---

## Work Breakdown

### Task 1: Add RPC Endpoints

**File**: `core/api/src/eth_rpc.rs`

```rust
/// Get balance for any observed address
pub async fn eth_get_observed_balance(
    &self,
    address: String,
    block_tag: String,
) -> Result<String, JsonRpcError> {
    let addr = parse_address(&address)?;
    let balance = self.executor.get_balance(&addr);
    Ok(format!("0x{:x}", balance))
}

/// Get transaction activity for any address
pub async fn citrate_get_observed_activity(
    &self,
    address: String,
    limit: u32,
    offset: u32,
) -> Result<Vec<TransactionSummary>, JsonRpcError> {
    let addr = parse_address(&address)?;

    // Query transaction storage for all txs involving this address
    let txs = self.storage.transactions
        .get_transactions_for_address(&addr, limit, offset)
        .map_err(|e| JsonRpcError::internal_error(e.to_string()))?;

    Ok(txs.into_iter().map(|tx| TransactionSummary {
        hash: tx.hash,
        from: tx.from,
        to: tx.to,
        value: tx.value,
        timestamp: tx.timestamp,
        status: tx.status,
    }).collect())
}
```

Register methods:
```rust
io_handler.add_method("eth_getObservedBalance", ...);
io_handler.add_method("citrate_getObservedActivity", ...);
```

**Acceptance Criteria**:
- [ ] `eth_getObservedBalance` returns balance for any address
- [ ] `citrate_getObservedActivity` returns transaction history
- [ ] Works for addresses not owned by wallet

---

### Task 2: Add Transaction Index for Address Lookup

**File**: `core/storage/src/transaction_store.rs`

Add address-based transaction indexing:

```rust
impl TransactionStore {
    /// Get all transactions involving an address (as sender or recipient)
    pub fn get_transactions_for_address(
        &self,
        address: &Address,
        limit: u32,
        offset: u32,
    ) -> Result<Vec<TransactionRecord>, StorageError> {
        // Query index: address -> [tx_hash, ...]
        let tx_hashes = self.address_tx_index
            .get_range(address, offset, limit)?;

        // Fetch full transaction records
        let mut records = Vec::new();
        for hash in tx_hashes {
            if let Some(record) = self.get_transaction(&hash)? {
                records.push(record);
            }
        }

        Ok(records)
    }

    /// Index a transaction by sender and recipient addresses
    pub fn index_transaction(&self, tx: &Transaction) -> Result<(), StorageError> {
        let from_addr = Address::from_public_key(&tx.from);
        self.address_tx_index.add(&from_addr, &tx.hash)?;

        if let Some(to) = &tx.to {
            let to_addr = Address::from_public_key(to);
            self.address_tx_index.add(&to_addr, &tx.hash)?;
        }

        Ok(())
    }
}
```

**Acceptance Criteria**:
- [ ] Transactions indexed by sender address
- [ ] Transactions indexed by recipient address
- [ ] Efficient range queries with limit/offset

---

### Task 3: Add Tauri Commands for Tracked Addresses

**File**: `gui/citrate-core/src-tauri/src/lib.rs`

```rust
#[tauri::command]
async fn get_observed_balance(
    state: State<'_, AppState>,
    address: String,
) -> Result<String, String> {
    let balance = state.rpc_client
        .call("eth_getObservedBalance", vec![address, "latest".to_string()])
        .await
        .map_err(|e| e.to_string())?;
    Ok(balance)
}

#[tauri::command]
async fn get_observed_activity(
    state: State<'_, AppState>,
    address: String,
    limit: u32,
    offset: u32,
) -> Result<Vec<TransactionSummary>, String> {
    let activity = state.rpc_client
        .call("citrate_getObservedActivity", vec![address, limit, offset])
        .await
        .map_err(|e| e.to_string())?;
    Ok(activity)
}

#[tauri::command]
async fn save_tracked_addresses(
    state: State<'_, AppState>,
    addresses: Vec<String>,
) -> Result<(), String> {
    state.config_store
        .set("tracked_addresses", &addresses)
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
async fn get_tracked_addresses(
    state: State<'_, AppState>,
) -> Result<Vec<String>, String> {
    state.config_store
        .get::<Vec<String>>("tracked_addresses")
        .map_err(|e| e.to_string())
        .map(|v| v.unwrap_or_default())
}
```

**Acceptance Criteria**:
- [ ] `get_observed_balance` works via Tauri
- [ ] `get_observed_activity` works via Tauri
- [ ] Tracked addresses persist in config store

---

### Task 4: Update Wallet Component

**File**: `gui/citrate-core/src/components/Wallet.tsx`

Replace localStorage with Tauri commands:

```typescript
// Load tracked addresses from backend
useEffect(() => {
  const loadTracked = async () => {
    try {
      const addresses = await invoke<string[]>('get_tracked_addresses');
      setTracked(addresses);
    } catch (e) {
      console.error('Failed to load tracked addresses:', e);
      setTracked([]);
    }
  };
  loadTracked();
}, []);

// Save tracked addresses to backend
const persistTracked = async (list: string[]) => {
  try {
    await invoke('save_tracked_addresses', { addresses: list });
    setTracked(list);
  } catch (e) {
    console.error('Failed to save tracked addresses:', e);
  }
};

// Refresh balance for tracked address
const refreshTrackedOne = async (addr: string) => {
  try {
    const balance = await invoke<string>('get_observed_balance', { address: addr });
    const activity = await invoke<TransactionSummary[]>('get_observed_activity', {
      address: addr,
      limit: 25,
      offset: 0,
    });
    setTrackedData(prev => ({
      ...prev,
      [addr]: { balance, activity }
    }));
  } catch (e) {
    console.error(`Failed to refresh tracked address ${addr}:`, e);
  }
};
```

**Acceptance Criteria**:
- [ ] Tracked addresses load from backend
- [ ] Tracked addresses persist across sessions
- [ ] Balance updates work for tracked addresses
- [ ] Activity history works for tracked addresses

---

## Testing Checklist

### Unit Tests
- [ ] `eth_getObservedBalance` returns correct balance
- [ ] `citrate_getObservedActivity` returns transaction list
- [ ] Transaction indexing works correctly

### Manual Tests
| Test | Steps | Expected |
|------|-------|----------|
| Add tracked address | Enter address, click Track | Address added to list |
| Balance displays | Add tracked address | Shows correct balance |
| Activity displays | Add tracked address with history | Shows transactions |
| Persist across restart | Add address, close app, reopen | Address still tracked |
| Remove tracked | Click remove on tracked address | Address removed |

---

## Files Modified

| File | Changes |
|------|---------|
| `core/api/src/eth_rpc.rs` | Add 2 RPC methods |
| `core/storage/src/transaction_store.rs` | Add address indexing |
| `gui/citrate-core/src-tauri/src/lib.rs` | Add 4 Tauri commands |
| `gui/citrate-core/src/components/Wallet.tsx` | Use Tauri commands instead of localStorage |

---

## Definition of Done

- [ ] All 4 tasks completed
- [ ] RPC endpoints respond correctly
- [ ] Tracked addresses persist
- [ ] Balance and activity load
- [ ] Manual tests pass
- [ ] Git commit: "Sprint 02: Tracked Addresses Backend"
