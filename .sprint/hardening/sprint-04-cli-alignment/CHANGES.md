# Sprint 04 Changes

**Sprint**: CLI Signature Alignment
**Completed**: TBD

---

## Summary

Aligned CLI to use ed25519 signature scheme, matching the wallet. All CLI commands now fully implemented.

---

## Breaking Changes

- CLI now uses ed25519 instead of secp256k1
- Existing CLI-created accounts will not work (re-import from wallet)

---

## Files Changed

| File | Changes |
|------|---------|
| `cli/Cargo.toml` | Switch to ed25519-dalek |
| `cli/src/commands/account.rs` | ed25519 key generation |
| `cli/src/commands/transaction.rs` | ed25519 signing |
| `cli/src/commands/model.rs` | Complete implementation |
| `cli/src/commands/contract.rs` | Complete implementation |

---

## Git Commit

**Message**: `feat(cli): Align signature scheme with wallet - Sprint 04`
**Tag**: `hardening-sprint-04`
