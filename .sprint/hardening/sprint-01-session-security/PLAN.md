# Sprint 01: Session Security & Frontend Exposure

**Status**: CURRENT
**Priority**: P0 Critical
**Duration**: 2-3 days
**Depends On**: None

---

## Problem Statement

The wallet backend has comprehensive session management (15-minute timeout, rate limiting, re-auth for high-value transactions), but **none of this is exposed to the frontend**. Users must enter their password for every transaction, creating poor UX despite having production-grade security infrastructure.

### Current State
- Backend: `SessionManager` with 15-min timeout, `touch_session()`, `is_session_valid()`
- Frontend: **No session awareness** - always prompts for password
- Location: `gui/citrate-core/src-tauri/src/wallet/mod.rs` (lines 183-250)

### Target State
- Frontend shows session status indicator
- Transactions use session if active (no password prompt)
- User can manually lock wallet
- High-value transactions (>= 10 SALT) still require password

---

## Work Breakdown

### Task 1: Add Tauri Session Commands (Backend)

**File**: `gui/citrate-core/src-tauri/src/lib.rs`

Add these three Tauri commands:

```rust
#[tauri::command]
async fn get_session_remaining(
    state: State<'_, AppState>,
    address: String,
) -> Result<u64, String> {
    match state.wallet_manager.get_session_remaining(&address).await {
        Some(remaining) => Ok(remaining),
        None => Ok(0),
    }
}

#[tauri::command]
async fn lock_wallet(
    state: State<'_, AppState>,
    address: String,
) -> Result<(), String> {
    state.wallet_manager.end_session(&address).await;
    Ok(())
}

#[tauri::command]
async fn is_session_active(
    state: State<'_, AppState>,
    address: String,
) -> Result<bool, String> {
    Ok(state.wallet_manager.is_session_valid(&address).await)
}
```

**Register in invoke_handler**:
```rust
.invoke_handler(tauri::generate_handler![
    // ... existing commands ...
    get_session_remaining,
    lock_wallet,
    is_session_active,
])
```

**Acceptance Criteria**:
- [ ] `get_session_remaining` returns seconds remaining (0 if no session)
- [ ] `lock_wallet` ends session for address
- [ ] `is_session_active` returns true/false

---

### Task 2: Create SessionStatus Component (Frontend)

**File**: `gui/citrate-core/src/components/SessionStatus.tsx` (NEW)

```typescript
import React, { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/tauri';

interface SessionStatusProps {
  address: string;
  onSessionExpired?: () => void;
}

export const SessionStatus: React.FC<SessionStatusProps> = ({ address, onSessionExpired }) => {
  const [remaining, setRemaining] = useState<number>(0);
  const [isActive, setIsActive] = useState<boolean>(false);

  useEffect(() => {
    const checkSession = async () => {
      try {
        const secs = await invoke<number>('get_session_remaining', { address });
        setRemaining(secs);
        setIsActive(secs > 0);

        if (secs === 0 && onSessionExpired) {
          onSessionExpired();
        }
      } catch (e) {
        setIsActive(false);
        setRemaining(0);
      }
    };

    checkSession();
    const interval = setInterval(checkSession, 5000); // Poll every 5s
    return () => clearInterval(interval);
  }, [address, onSessionExpired]);

  const handleLock = async () => {
    try {
      await invoke('lock_wallet', { address });
      setIsActive(false);
      setRemaining(0);
    } catch (e) {
      console.error('Failed to lock wallet:', e);
    }
  };

  const formatTime = (secs: number) => {
    const mins = Math.floor(secs / 60);
    const s = secs % 60;
    return `${mins}:${s.toString().padStart(2, '0')}`;
  };

  if (!isActive) {
    return (
      <div className="flex items-center gap-2 text-sm text-gray-500">
        <span className="w-2 h-2 rounded-full bg-gray-400"></span>
        <span>Session inactive</span>
      </div>
    );
  }

  return (
    <div className="flex items-center gap-3 text-sm">
      <div className="flex items-center gap-2 text-green-600">
        <span className="w-2 h-2 rounded-full bg-green-500 animate-pulse"></span>
        <span>Session active ({formatTime(remaining)})</span>
      </div>
      <button
        onClick={handleLock}
        className="px-2 py-1 text-xs bg-red-100 text-red-600 rounded hover:bg-red-200"
      >
        Lock
      </button>
    </div>
  );
};

export default SessionStatus;
```

**Acceptance Criteria**:
- [ ] Shows "Session inactive" when no session
- [ ] Shows countdown when session active
- [ ] Lock button ends session immediately
- [ ] Polls every 5 seconds

---

### Task 3: Integrate SessionStatus into Wallet Component

**File**: `gui/citrate-core/src/components/Wallet.tsx`

Add import and component:

```typescript
import SessionStatus from './SessionStatus';

// In the component, add to wallet header area:
{selectedAccount && (
  <SessionStatus
    address={selectedAccount.address}
    onSessionExpired={() => {
      // Optional: Show notification
      console.log('Session expired');
    }}
  />
)}
```

**Acceptance Criteria**:
- [ ] SessionStatus visible in wallet view
- [ ] Updates when switching accounts
- [ ] Shows correct state for each account

---

### Task 4: Modify send_transaction for Smart Password Caching

**File**: `gui/citrate-core/src-tauri/src/lib.rs`

Modify `send_transaction` to check session first:

```rust
#[tauri::command]
async fn send_transaction(
    state: State<'_, AppState>,
    request: TransactionRequest,
    password: Option<String>, // Make password optional
) -> Result<String, String> {
    let from_address = &request.from;

    // Check if session is valid
    if state.wallet_manager.is_session_valid(from_address).await {
        // Session active - check if high-value transaction
        let value = request.value.unwrap_or(0);
        let threshold = 10_000_000_000_000_000_000u128; // 10 SALT

        if value >= threshold {
            // High-value: require password regardless of session
            let pwd = password.ok_or("High-value transactions require password confirmation")?;
            return execute_transaction(&state, request, &pwd).await;
        }

        // Regular transaction with active session - use cached credentials
        state.wallet_manager.touch_session(from_address).await;
        return execute_transaction_with_session(&state, request).await;
    }

    // No session - require password
    let pwd = password.ok_or("Session expired. Please enter your password.")?;
    execute_transaction(&state, request, &pwd).await
}
```

**Note**: This requires adding a helper function that uses the session to sign without re-entering password. The session should store a temporary decryption capability.

**Acceptance Criteria**:
- [ ] Session-active + low-value = no password prompt
- [ ] Session-active + high-value = password prompt
- [ ] Session-expired = password prompt
- [ ] Session touched on successful transaction

---

### Task 5: Update Transaction Modal for Smart Prompting

**File**: `gui/citrate-core/src/components/ContractInteraction.tsx` (and similar)

```typescript
const [sessionActive, setSessionActive] = useState(false);
const [password, setPassword] = useState('');
const [isHighValue, setIsHighValue] = useState(false);

useEffect(() => {
  const checkSession = async () => {
    if (selectedAccount) {
      const active = await invoke<boolean>('is_session_active', {
        address: selectedAccount.address
      });
      setSessionActive(active);
    }
  };
  checkSession();
}, [selectedAccount]);

// Check if high-value when amount changes
useEffect(() => {
  const threshold = 10; // 10 SALT
  setIsHighValue(parseFloat(amount || '0') >= threshold);
}, [amount]);

const handleSend = async () => {
  try {
    const needsPassword = !sessionActive || isHighValue;

    const result = await invoke('send_transaction', {
      request: txRequest,
      password: needsPassword ? password : null,
    });

    toast.success(`Transaction sent: ${result}`);
    setPassword(''); // Clear password
  } catch (e: any) {
    toast.error(e.toString());
  }
};

// In render:
{(!sessionActive || isHighValue) && (
  <div className="space-y-2">
    <label className="text-sm text-gray-600">
      {isHighValue
        ? 'High-value transaction - password required'
        : 'Enter password to confirm'
      }
    </label>
    <input
      type="password"
      value={password}
      onChange={(e) => setPassword(e.target.value)}
      className="w-full px-3 py-2 border rounded"
      placeholder="Password"
    />
  </div>
)}

{sessionActive && !isHighValue && (
  <div className="flex items-center gap-2 text-green-600 text-sm">
    <span className="w-2 h-2 rounded-full bg-green-500"></span>
    <span>Session active - no password needed</span>
  </div>
)}
```

**Acceptance Criteria**:
- [ ] Password field hidden when session active + low-value
- [ ] Password field shown when session expired
- [ ] Password field shown when high-value (with explanation)
- [ ] Success clears password field

---

### Task 6: Add Session Settings to Settings Panel

**File**: `gui/citrate-core/src/components/Settings.tsx`

Add a "Security" section:

```typescript
// Add to settings sections
<div className="space-y-4">
  <h3 className="text-lg font-medium">Wallet Security</h3>

  <div className="space-y-3">
    {/* Session Status */}
    <div className="flex justify-between items-center">
      <span className="text-sm">Active Sessions</span>
      <span className="text-sm text-gray-500">
        {activeSessions.length} account(s) unlocked
      </span>
    </div>

    {/* Lock All Button */}
    <button
      onClick={handleLockAll}
      className="w-full px-4 py-2 bg-red-500 text-white rounded hover:bg-red-600"
    >
      Lock All Wallets
    </button>

    {/* Session Timeout Info */}
    <div className="text-xs text-gray-500">
      Sessions automatically expire after 15 minutes of inactivity.
      High-value transactions (10+ SALT) always require password.
    </div>
  </div>
</div>
```

**Acceptance Criteria**:
- [ ] Shows number of active sessions
- [ ] "Lock All" button works
- [ ] Explains security policy

---

## Implementation Order

1. **Task 1**: Backend commands (required for everything else)
2. **Task 2**: SessionStatus component
3. **Task 3**: Integrate into Wallet
4. **Task 4**: Modify send_transaction
5. **Task 5**: Update transaction modals
6. **Task 6**: Settings panel

---

## Testing Checklist

### Unit Tests
- [ ] `get_session_remaining` returns correct values
- [ ] `lock_wallet` terminates session
- [ ] `is_session_active` accurate

### Integration Tests
- [ ] SessionStatus updates in real-time
- [ ] Password prompt appears correctly
- [ ] High-value detection works

### Manual Tests
| Test | Steps | Expected |
|------|-------|----------|
| Session creation | Enter password, send tx | Session shows in UI |
| Session countdown | Watch SessionStatus | Countdown decrements |
| Manual lock | Click Lock button | Session ends immediately |
| Auto-expire | Wait 15+ minutes | Session expires, shows inactive |
| Low-value no prompt | Session active, send 1 SALT | No password prompt |
| High-value prompt | Session active, send 15 SALT | Password prompt appears |

---

## Rollback Plan

If issues arise:
1. Revert Tauri command changes
2. Keep password always required (current behavior)
3. SessionStatus can remain as read-only indicator

---

## Files Modified

| File | Changes |
|------|---------|
| `gui/citrate-core/src-tauri/src/lib.rs` | Add 3 Tauri commands, modify send_transaction |
| `gui/citrate-core/src/components/SessionStatus.tsx` | NEW - Session indicator component |
| `gui/citrate-core/src/components/Wallet.tsx` | Add SessionStatus integration |
| `gui/citrate-core/src/components/ContractInteraction.tsx` | Smart password prompting |
| `gui/citrate-core/src/components/Settings.tsx` | Add security section |

---

## Definition of Done

- [ ] All 6 tasks completed
- [ ] All unit tests pass
- [ ] All integration tests pass
- [ ] All manual tests pass
- [ ] VALIDATION.md signed off
- [ ] CHANGES.md documents all modifications
- [ ] Git commit with message: "Sprint 01: Session Security Complete"
