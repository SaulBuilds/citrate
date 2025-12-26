# Sprint 01 Changes

**Sprint**: Session Security & Frontend Exposure
**Completed**: 2025-12-26

---

## Summary

Exposed backend session management to the frontend GUI, enabling:
- Password-free signing during active sessions (15 min timeout)
- Visual session status indicators with countdown timers
- Manual lock functionality at wallet and global levels
- High-value transaction re-authentication (>10 SALT)

---

## Files Changed

### New Files

| File | Purpose |
|------|---------|
| `gui/citrate-core/src/components/SessionStatus.tsx` | Session indicator component with countdown |

### Modified Files

| File | Changes |
|------|---------|
| `gui/citrate-core/src-tauri/src/lib.rs` | Added 5 Tauri commands, modified send_transaction |
| `gui/citrate-core/src-tauri/src/wallet/mod.rs` | Added key caching to SessionManager |
| `gui/citrate-core/src/components/Wallet.tsx` | Integrated SessionStatus, smart password modal |
| `gui/citrate-core/src/components/Settings.tsx` | Added Session Security section |
| `gui/citrate-core/src/services/tauri.ts` | Added session service functions |

---

## API Changes

### New Tauri Commands

```typescript
// Check if session is active
invoke<boolean>('is_session_active', { address: string })

// Get remaining session time in seconds
invoke<number>('get_session_remaining', { address: string })

// End session for address
invoke('lock_wallet', { address: string })

// End all sessions, return count
invoke<number>('lock_all_wallets')

// Check if password required for transaction
invoke<boolean>('check_password_required', { address: string, value: string })
```

### Modified Tauri Commands

```typescript
// send_transaction now accepts optional password
invoke('send_transaction', {
  request: TransactionRequest,
  password: string | null  // Changed from required to optional
})
```

---

## Backend Changes

### SessionManager Enhancements (wallet/mod.rs)

```rust
pub struct SessionManager {
    sessions: HashMap<String, (Instant, Instant)>,
    cached_keys: HashMap<String, SigningKey>,  // NEW: key caching
}

// New methods
pub fn create_session_with_key(&mut self, address: &str, signing_key: SigningKey)
pub fn get_cached_key(&self, address: &str) -> Option<&SigningKey>
pub fn clear_cached_key(&mut self, address: &str)

// Modified to clear cached keys
pub fn end_session(&mut self, address: &str)
pub fn cleanup_expired(&mut self)
```

### Sign Transaction Flow

1. Check if session is active AND cached key exists
2. If yes and not high-value: use cached key (no password needed)
3. If no or high-value: require password, validate, cache key

---

## Behavior Changes

| Before | After |
|--------|-------|
| Password required for every transaction | Password only when session expired or high-value |
| No session visibility | Session countdown visible in account cards |
| No manual lock | Lock button in each account + Settings |
| No session info in settings | Security section shows active sessions |

---

## Configuration

No configuration changes required. Session timeout remains 15 minutes (SESSION_TIMEOUT_SECS = 900).

High-value threshold is 10 SALT (REAUTH_THRESHOLD_SALT = 10e18).

---

## Test Results

| Category | Passed | Failed |
|----------|--------|--------|
| Wallet Unit Tests | 25 | 0 |
| Build Verification | 2 | 0 |
| Code Review Tests | 10 | 0 |
| **Total** | **37** | **0** |

---

## Migration

No migration required. This is purely additive functionality.

---

## Known Issues

None identified.

---

## Rollback

To rollback:
1. Revert lib.rs Tauri commands
2. Revert wallet/mod.rs SessionManager changes
3. Remove SessionStatus.tsx
4. Revert Wallet.tsx integration
5. Revert Settings.tsx security section
6. Revert tauri.ts service functions

The wallet will return to always-require-password behavior.

---

## Git Commit

**Message**: `feat(gui): Add session-based signing and frontend session management`

**Files**:
```
A  gui/citrate-core/src/components/SessionStatus.tsx
M  gui/citrate-core/src-tauri/src/lib.rs
M  gui/citrate-core/src-tauri/src/wallet/mod.rs
M  gui/citrate-core/src/components/Wallet.tsx
M  gui/citrate-core/src/components/Settings.tsx
M  gui/citrate-core/src/services/tauri.ts
```

**Tag**: `hardening-sprint-01`
