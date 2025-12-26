# Sprint 04 Changes

**Sprint**: CLI Signature Alignment
**Completed**: 2025-12-26

---

## Summary

Migrated CLI from secp256k1 to ed25519 signature scheme to align with the wallet. Accounts created in the CLI are now compatible with the wallet and vice versa.

---

## Breaking Changes

- CLI now uses **ed25519** instead of **secp256k1**
- Existing CLI-created keystores (version 1) will not work
- Re-create accounts or import from wallet

---

## Files Changed

### Modified Files

| File | Changes |
|------|---------|
| `cli/Cargo.toml` | Replaced secp256k1 with ed25519-dalek |
| `cli/src/utils/keystore.rs` | Ed25519 SigningKey, version 2 format |
| `cli/src/commands/account.rs` | Ed25519 key generation, address derivation |

---

## API Changes

### Keystore Format (Version 2)

**Before (Version 1)**:
```json
{
  "version": 1,
  "encrypted_key": "...",
  "salt": "...",
  "nonce": "..."
}
```

**After (Version 2)**:
```json
{
  "version": 2,
  "key_type": "ed25519",
  "encrypted_key": "...",
  "salt": "...",
  "nonce": "...",
  "public_key": "..."
}
```

### Address Derivation

Same logic as `citrate-execution/src/types.rs`:
1. Check if pubkey has embedded EVM address (20 bytes + 12 zeros)
2. If yes: use first 20 bytes directly
3. If no: Keccak256 hash of 32-byte pubkey, take last 20 bytes

---

## Behavior Changes

| Before | After |
|--------|-------|
| secp256k1 key generation | ed25519 key generation |
| 65-byte uncompressed pubkey | 32-byte ed25519 pubkey |
| Ethereum-style address derivation | Citrate-style address derivation |
| CLI accounts incompatible with wallet | CLI accounts portable to wallet |

---

## Test Results

| Category | Passed | Failed |
|----------|--------|--------|
| Build Verification | 1 | 0 |
| Code Review | 12 | 0 |
| CLI Commands | 7 | 0 |
| **Total** | **20** | **0** |

---

## Migration

For existing CLI accounts:
1. Export private key from old keystore (if backup exists)
2. Create new account in wallet
3. Import private key into new CLI keystore

Or simply create new accounts using either tool.

---

## Rollback

To rollback:
1. Revert `cli/Cargo.toml` to use secp256k1
2. Revert `cli/src/utils/keystore.rs` to SecretKey
3. Revert `cli/src/commands/account.rs` to secp256k1 derivation

---

## Git Commit

**Message**: `feat(cli): Migrate to ed25519 for wallet compatibility - Sprint 04`

**Files**:
```
M  cli/Cargo.toml
M  cli/src/utils/keystore.rs
M  cli/src/commands/account.rs
```

**Tag**: `hardening-sprint-04`
