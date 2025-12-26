# Sprint 02 Changes

**Sprint**: Tracked Addresses Backend
**Completed**: TBD

---

## Summary

Added backend support for tracking external addresses, enabling balance and activity monitoring for any blockchain address.

---

## New RPC Methods

| Method | Parameters | Returns |
|--------|------------|---------|
| `eth_getObservedBalance` | address, blockTag | Balance (hex) |
| `citrate_getObservedActivity` | address, limit, offset | Transaction[] |

---

## New Tauri Commands

| Command | Parameters | Returns |
|---------|------------|---------|
| `get_observed_balance` | address | Balance (string) |
| `get_observed_activity` | address, limit, offset | TransactionSummary[] |
| `save_tracked_addresses` | addresses[] | void |
| `get_tracked_addresses` | none | addresses[] |

---

## Files Changed

| File | Changes |
|------|---------|
| `core/api/src/eth_rpc.rs` | +2 RPC methods |
| `core/storage/src/transaction_store.rs` | +address indexing |
| `gui/citrate-core/src-tauri/src/lib.rs` | +4 Tauri commands |
| `gui/citrate-core/src/components/Wallet.tsx` | Use backend storage |

---

## Git Commit

**Message**: `feat(api): Add tracked addresses backend support - Sprint 02`
**Tag**: `hardening-sprint-02`
