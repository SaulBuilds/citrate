# Sprint 02 Validation

**Sprint**: Tracked Addresses Backend
**Validation Date**: TBD

---

## Automated Tests

```bash
cargo test -p citrate-api observed
cargo test -p citrate-storage address_index
```

| Test | Status | Notes |
|------|--------|-------|
| test_get_observed_balance | | |
| test_get_observed_activity | | |
| test_address_tx_index | | |

---

## Manual Tests

### Test 1: Add Tracked Address
**Steps**: Enter valid address, click Track
**Expected**: Address appears in tracked list
**Result**: [ ] Pass [ ] Fail

### Test 2: Balance Displays
**Steps**: Track an address with known balance
**Expected**: Correct balance shown
**Result**: [ ] Pass [ ] Fail

### Test 3: Activity Displays
**Steps**: Track address with transaction history
**Expected**: Transaction list shown
**Result**: [ ] Pass [ ] Fail

### Test 4: Persist Across Restart
**Steps**: Add address, close app, reopen
**Expected**: Address still in tracked list
**Result**: [ ] Pass [ ] Fail

### Test 5: Remove Tracked
**Steps**: Click remove on tracked address
**Expected**: Address removed from list
**Result**: [ ] Pass [ ] Fail

### Test 6: Invalid Address Handling
**Steps**: Try to track invalid address
**Expected**: Error message, address not added
**Result**: [ ] Pass [ ] Fail

---

## Sign-Off

**Validated By**:
**Date**:
