# Sprint 02 Changes

**Sprint**: Tracked Addresses Backend
**Completed**: 2025-12-26

---

## Summary

Added backend persistence for tracked addresses. The feature now stores the list of tracked addresses in a JSON file instead of browser localStorage, enabling persistence across app restarts.

**Note**: Investigation revealed that RPC endpoints and Tauri commands for fetching balance/activity for arbitrary addresses already existed. Only list persistence was needed.

---

## Files Changed

### New Files

None - no new files created.

### Modified Files

| File | Changes |
|------|---------|
| `gui/citrate-core/src-tauri/src/lib.rs` | Added 2 Tauri commands for tracked address list persistence |
| `gui/citrate-core/src/services/tauri.ts` | Added service functions for tracked addresses |
| `gui/citrate-core/src/components/Wallet.tsx` | Replaced localStorage with backend persistence |

---

## API Changes

### New Tauri Commands

```typescript
// Get list of tracked addresses from backend storage
invoke<string[]>('get_tracked_addresses')

// Save list of tracked addresses to backend storage
invoke('save_tracked_addresses', { addresses: string[] })
```

### Pre-existing Commands (already worked for any address)

```typescript
// These were already implemented and work for tracked addresses:
invoke<string>('get_address_observed_balance', { address: string, blockWindow?: number })
invoke<TxActivity[]>('get_account_activity', { address: string, blockWindow?: number, limit?: number })
```

---

## Backend Changes

### Tracked Addresses Storage (lib.rs)

```rust
/// Storage location: dirs::data_dir()/citrate-core/tracked_addresses.json

fn tracked_addresses_path() -> std::path::PathBuf;

#[tauri::command]
async fn get_tracked_addresses() -> Result<Vec<String>, String>;

#[tauri::command]
async fn save_tracked_addresses(addresses: Vec<String>) -> Result<(), String>;
```

---

## Frontend Changes

### walletService (tauri.ts)

```typescript
// Added methods:
getTrackedAddresses: () => Promise<string[]>
saveTrackedAddresses: (addresses: string[]) => Promise<void>
```

### Wallet.tsx

| Before | After |
|--------|-------|
| `useState` initialized from localStorage | `useState` starts empty, loaded via useEffect |
| `persistTracked` writes to localStorage | `persistTracked` calls `walletService.saveTrackedAddresses()` |
| Sync save | Async save |

---

## Behavior Changes

| Before | After |
|--------|-------|
| Tracked addresses stored in browser localStorage | Tracked addresses stored in backend JSON file |
| Lost if browser data cleared | Persists with application data |
| Web-only storage | Cross-platform backend storage |

---

## Configuration

No configuration changes required. Storage location is automatic:
- macOS: `~/Library/Application Support/citrate-core/tracked_addresses.json`
- Linux: `~/.local/share/citrate-core/tracked_addresses.json`
- Windows: `%APPDATA%/citrate-core/tracked_addresses.json`

---

## Migration

Existing localStorage data is not automatically migrated. Users who had tracked addresses in localStorage will need to re-add them. This is a minor inconvenience since the feature was not widely used.

---

## Git Commit

**Message**: `feat(gui): Add backend persistence for tracked addresses - Sprint 02`

**Files**:
```
M  gui/citrate-core/src-tauri/src/lib.rs
M  gui/citrate-core/src/services/tauri.ts
M  gui/citrate-core/src/components/Wallet.tsx
```

**Tag**: `hardening-sprint-02`
