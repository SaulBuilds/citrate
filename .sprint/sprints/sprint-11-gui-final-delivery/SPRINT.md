# Sprint 11: GUI Final Delivery & Documentation

**Sprint Duration**: 2 weeks
**Started**: 2025-12-25
**Status**: PLANNING

---

## Sprint Goal

Finalize the Citrate GUI application for public release with:
1. Complete, accurate README documentation
2. Working installers (DMG, EXE, DEB, AppImage) available on GitHub Releases
3. Out-of-the-box experience with automatic model download
4. Verified CLI/SDK/Node operations from terminal

---

## Current State Assessment

### What Works
- [x] macOS DMG builds (~408MB)
- [x] Automatic model download on first run (Qwen2.5-Coder-7B)
- [x] ReAct tool orchestration pattern
- [x] Embedded node in GUI
- [x] Wallet operations
- [x] Build compiles without bundled models (fresh clone friendly)

### Issues Found
1. **Rust Tests**: 314/315 pass, 1 fails (async runtime issue in storage test - pre-existing)
2. **Contract Tests**: 42/43 pass, 1 fails (`testAddReview` arithmetic overflow)
3. **SDK Tests**: Jest not installed in sdk/javascript
4. **README**: References explorer/docs deployments to remove
5. **GitHub Releases**: Need to set up CI/CD for automated builds

---

## Work Packages

### WP-11.1: README Cleanup & Accuracy (Priority: HIGH)

**Goal**: Update README to reflect current reality

**Tasks**:
- [ ] Remove Block Explorer deployment section (keep local dev instructions)
- [ ] Remove Documentation site deployment section
- [ ] Update GitHub Releases link to actual repository
- [ ] Add direct download links for installers (once releases exist)
- [ ] Verify all build commands work on fresh clone
- [ ] Update model download instructions
- [ ] Add troubleshooting for common issues

**Files to Update**:
- `citrate/README.md`

---

### WP-11.2: GitHub Releases Setup (Priority: HIGH)

**Goal**: Make installers downloadable from GitHub

**Tasks**:
- [ ] Create v1.0.0 GitHub release
- [ ] Upload macOS DMG (aarch64)
- [ ] Build and upload Windows NSIS installer (needs Windows CI)
- [ ] Build and upload Linux DEB/AppImage (needs Linux CI)
- [ ] Add release notes with first-run instructions

**Build Targets**:
| Platform | Target | Output |
|----------|--------|--------|
| macOS ARM | aarch64-apple-darwin | Citrate_1.0.0_aarch64.dmg |
| macOS Intel | x86_64-apple-darwin | Citrate_1.0.0_x64.dmg |
| Windows | x86_64-pc-windows-msvc | Citrate_1.0.0_x64-setup.exe |
| Linux | x86_64-unknown-linux-gnu | citrate_1.0.0_amd64.deb |

---

### WP-11.3: Fix Remaining Test Failures (Priority: MEDIUM)

**Goal**: All tests should pass

**Tasks**:
- [ ] Fix `testAddReview` arithmetic overflow in ModelMarketplace.t.sol
- [ ] Fix `test_storage_creation` async runtime issue
- [ ] Install SDK dependencies and verify tests pass
- [ ] Run full test suite and document any skipped tests

---

### WP-11.4: CLI/SDK/Node Verification (Priority: MEDIUM)

**Goal**: Ensure terminal operations work out of the box

**Tasks**:
- [ ] Verify `cargo run --bin citrate-node -- devnet` works
- [ ] Verify `cargo run --bin citrate-wallet` commands work
- [ ] Verify `@citrate-ai/sdk` npm package works
- [ ] Document required environment variables
- [ ] Test transaction flow end-to-end

---

### WP-11.5: First-Run Experience Polish (Priority: MEDIUM)

**Goal**: Smooth onboarding for new users

**Tasks**:
- [ ] Verify model download works on all platforms
- [ ] Test onboarding modal flow
- [ ] Ensure app works immediately with bundled 0.5B model
- [ ] Background download of 7B model doesn't block UI

---

### WP-11.6: CI/CD Pipeline (Priority: LOW)

**Goal**: Automated builds on push/release

**Tasks**:
- [ ] Review existing GitHub Actions workflows
- [ ] Add Windows build workflow
- [ ] Add Linux build workflow
- [ ] Auto-upload to GitHub Releases on tag push

---

## Definition of Done

1. README accurately describes installation for all platforms
2. GitHub Releases has working installers for macOS/Windows/Linux
3. Fresh clone + `npm run tauri build` succeeds
4. All tests pass (or known issues documented)
5. CLI wallet can send transactions to local devnet
6. GUI opens and can interact with embedded node

---

## Key Files Reference

| Purpose | Path |
|---------|------|
| Main README | `citrate/README.md` |
| Tauri Config | `gui/citrate-core/src-tauri/tauri.conf.json` |
| Model Download | `gui/citrate-core/src-tauri/src/agent/download_models.rs` |
| Onboarding | `gui/citrate-core/src/components/OnboardingModal.tsx` |
| Node Binary | `node/src/main.rs` |
| Wallet CLI | `wallet/src/main.rs` |
| SDK Package | `sdk/javascript/package.json` |
| Contract Tests | `contracts/test/ModelMarketplace.t.sol` |
| GitHub Actions | `.github/workflows/` |

---

## Blockers

1. Windows build requires Windows machine or CI runner
2. Linux build requires Linux machine or CI runner
3. GitHub Releases requires push access to main repo

---

## Notes

- Explorer/docs can be removed from README (user doesn't need them)
- Focus on GUI app + CLI tools for v1.0 release
- Model download happens at runtime, not bundled (keeps installer size reasonable)
