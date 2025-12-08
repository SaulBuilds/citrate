# Sprint 8: Stable Release Preparation

**Sprint Goal**: Prepare Citrate for stable v1.0 release with all components operational

**Duration**: 1-2 weeks
**Started**: 2025-12-06
**Status**: IN PROGRESS

---

## Executive Summary

This sprint focuses on preparing a clean, stable release of Citrate with:
- Fresh builds from clean state
- All required AI models available via IPFS
- RPC endpoints configured (local + ngrok + future production)
- AI onboarding agent that adapts to user skill level
- Cross-platform packages for macOS, Windows, and Linux

---

## Work Breakdown Structure

### WP-8.1: Infrastructure Cleanup (5 points)

**Priority**: P0 (Blocker)
**Owner**: Core Team

| Task | Status | Estimate | Notes |
|------|--------|----------|-------|
| Clean Rust target directories | [x] | 0.5h | All `/target` removed |
| Clean node_modules | [x] | 0.5h | All `node_modules` removed |
| Clear chain data | [x] | 0.25h | `~/.citrate*` removed |
| Verify release build | [x] | 2h | `cargo build --release` ✅ |
| Verify GUI build | [x] | 1h | `npm run build` ✅ |

**Acceptance Criteria**:
- `cargo build --release` completes without errors
- `npm run tauri build` produces valid binaries
- No stale data in chain directories

---

### WP-8.2: Genesis & Model Setup (8 points)

**Priority**: P0 (Blocker)
**Owner**: ML/Infra Team

| Task | Status | Estimate | Notes |
|------|--------|----------|-------|
| Verify BGE-M3 in assets | [x] | 0.25h | 417MB at `node/assets/` ✅ |
| Pin Mistral 7B to IPFS | [x] | 2h | CID verified accessible (HTTP 200) ✅ |
| Pin Qwen2 0.5B to IPFS | [x] | 1h | CID: QmZj4ZaG9v6nXKnT5yqwi8YaH5bm48zooNdh9ff4CHGTY4 ✅ |
| Create model download script | [x] | 3h | `scripts/download-models.sh` ✅ |
| Test model loading in agent | [ ] | 2h | End-to-end test |

**Acceptance Criteria**:
- BGE-M3 loads successfully in genesis block
- Mistral 7B downloadable from IPFS
- Qwen2 0.5B available for fast responses
- Agent can run inference using downloaded models

**Model CIDs**:
```
mistral-7b-instruct-v0.3: QmUsYyxg71bV8USRQ6Ccm3SdMqeWgEEVnCYkgNDaxvBTZB
qwen2-0.5b-q4:            QmZj4ZaG9v6nXKnT5yqwi8YaH5bm48zooNdh9ff4CHGTY4
bge-m3-q4:                Embedded in binary
```

---

### WP-8.3: RPC Configuration (5 points)

**Priority**: P0 (Blocker)
**Owner**: Backend Team

| Task | Status | Estimate | Notes |
|------|--------|----------|-------|
| Document RPC endpoints | [x] | 1h | See docs below ✅ |
| Test ngrok setup | [x] | 1h | ngrok v3.33.1 verified ✅ |
| Create config templates | [x] | 1h | `mainnet.toml` created ✅ |
| Verify GUI-to-node RPC | [x] | 2h | RPC responding (block 0x6fc3a) ✅ |
| Create external RPC docs | [x] | 1h | `docs/guides/external-rpc-access.md` ✅ |

**RPC Endpoint Matrix**:

| Environment | URL | Protocol | Notes |
|-------------|-----|----------|-------|
| Local Dev | `http://localhost:8545` | JSON-RPC | Default |
| Local WS | `ws://localhost:8546` | WebSocket | Subscriptions |
| ngrok (temp) | `https://xxx.ngrok-free.app` | JSON-RPC | External testing |
| Production | `https://api.citrate.ai` | JSON-RPC | When DNS ready |

**Acceptance Criteria**:
- Local RPC responds to `eth_blockNumber`
- ngrok tunnel works for external access
- GUI connects to embedded node without errors

---

### WP-8.4: AI Onboarding Agent (13 points)

**Priority**: P0 (Blocker)
**Owner**: Agent Team

| Task | Status | Estimate | Notes |
|------|--------|----------|-------|
| Skill assessment questions | [x] | 3h | 4 questions implemented ✅ |
| Beginner flow | [x] | 4h | 5-step wallet setup path ✅ |
| Intermediate flow | [x] | 3h | Developer onboarding path ✅ |
| Advanced flow | [x] | 2h | Power user setup path ✅ |
| Test conversation flows | [ ] | 4h | E2E testing pending |

**Implementation**: `gui/citrate-core/src-tauri/src/agent/onboarding.rs`

**Onboarding Flow**:

```
Welcome ──► Skill Assessment ──► Personalized Path
                │
    ┌───────────┼───────────┐
    ▼           ▼           ▼
 Beginner  Intermediate  Advanced
    │           │           │
    ▼           ▼           ▼
 Wallet      dApp Dev    API Docs
 Basics      Tools       Deep Dive
```

**Skill Assessment Questions**:
1. "Have you used a blockchain wallet before?"
2. "Have you written smart contracts?"
3. "Are you familiar with AI model inference?"

**Acceptance Criteria**:
- Agent correctly classifies user skill level
- Each path provides relevant guidance
- User can switch paths at any time
- Agent remembers context across sessions

---

### WP-8.5: IPFS Integration (8 points)

**Priority**: P0 (Blocker)
**Owner**: Infra Team

| Task | Status | Estimate | Notes |
|------|--------|----------|-------|
| Verify kubo installation | [x] | 1h | kubo running locally ✅ |
| Implement gateway fallback | [x] | 3h | 4 gateways in download script ✅ |
| Model download progress UI | [ ] | 2h | Progress bar, ETA |
| CID verification | [ ] | 2h | Verify downloads |
| First-run setup wizard | [ ] | 2h | Guide users through setup |

**Gateway Fallback Implementation**: `scripts/download-models.sh`

**Gateway Fallback Order**:
1. Local daemon (`http://localhost:5001`)
2. Configured external gateway
3. Public gateways (ipfs.io, cloudflare-ipfs.com)

**Acceptance Criteria**:
- Models download successfully from IPFS
- Progress shown to user during download
- Downloaded files verified against CID
- Works even without local daemon (via gateway)

---

### WP-8.6: Cross-Platform Packaging (13 points)

**Priority**: P0 (Blocker)
**Owner**: Release Team

| Task | Status | Estimate | Notes |
|------|--------|----------|-------|
| macOS .dmg build | [x] | 4h | tauri.conf.json configured ✅ |
| Windows .msi build | [x] | 4h | NSIS installer configured ✅ |
| Linux packages | [x] | 3h | AppImage + deb configured ✅ |
| Bundle dependencies | [x] | 2h | Dependencies listed ✅ |
| Test on fresh systems | [ ] | 3h | VM testing pending |

**Build Matrix**:

| Platform | Format | Architecture | Status |
|----------|--------|--------------|--------|
| macOS | .dmg | arm64, x86_64 | [x] |
| Windows | .msi/.exe | x86_64 | [x] |
| Linux | .AppImage | x86_64 | [x] |
| Linux | .deb | x86_64 | [x] |

**Tauri Build**: `npm run tauri build` generates all packages

**Acceptance Criteria**:
- All packages install without errors
- Application launches successfully
- No missing dependencies
- Code signing verified (macOS, Windows)

---

### WP-8.7: SDK & Tooling (8 points)

**Priority**: P1
**Owner**: SDK Team

| Task | Status | Estimate | Notes |
|------|--------|----------|-------|
| JavaScript SDK build | [ ] | 2h | npm package |
| Python SDK install | [ ] | 2h | pip package |
| CLI tools test | [ ] | 2h | All platforms |
| SDK examples | [ ] | 1h | Usage docs |
| Quickstart guides | [ ] | 1h | Getting started |

**Acceptance Criteria**:
- `npm install @citrate-ai/sdk` works
- `pip install citrate-sdk` works
- CLI tools work on all platforms
- Example code runs successfully

---

### WP-8.8: Documentation & Polish (5 points)

**Priority**: P1
**Owner**: Docs Team

| Task | Status | Estimate | Notes |
|------|--------|----------|-------|
| Update README | [ ] | 1h | Fresh install steps |
| Create QUICK_START.md | [x] | 2h | `docs/QUICK_START.md` ✅ |
| Document ngrok setup | [x] | 1h | `docs/guides/external-rpc-access.md` ✅ |
| Troubleshooting guide | [x] | 1h | Included in QUICK_START.md ✅ |
| Release notes | [ ] | 1h | v1.0 changelog pending |

**Acceptance Criteria**:
- New users can install from README
- QUICK_START gets users running in <10 mins
- Common issues have documented solutions
- Release notes cover all changes since last release

---

## Dependencies

```
WP-8.1 (Cleanup) ────┐
                     ├──► WP-8.2 (Models) ──► WP-8.4 (Agent)
                     │
                     ├──► WP-8.3 (RPC) ───────────────────────┐
                     │                                         │
                     └──► WP-8.5 (IPFS) ──────────────────────┼──► WP-8.6 (Packaging)
                                                               │
                                        WP-8.7 (SDK) ──────────┘
                                              │
                                              └──► WP-8.8 (Docs)
```

---

## Daily Standup

### Day 1 (2025-12-06)
- [x] Infrastructure cleanup completed
- [x] Model verification and pinning
- [x] RPC configuration completed
- [x] AI onboarding agent implemented
- [x] Cross-platform packaging configured
- [x] Quick start documentation created

**Files Created:**
- `scripts/download-models.sh` - Model download with IPFS fallback
- `node/config/mainnet.toml` - Production config template
- `docs/guides/external-rpc-access.md` - ngrok & RPC docs
- `docs/QUICK_START.md` - User quick start guide
- `gui/citrate-core/src-tauri/src/agent/onboarding.rs` - Skill assessment module

**Model CIDs:**
- Mistral 7B: `QmUsYyxg71bV8USRQ6Ccm3SdMqeWgEEVnCYkgNDaxvBTZB`
- Qwen2 0.5B: `QmZj4ZaG9v6nXKnT5yqwi8YaH5bm48zooNdh9ff4CHGTY4`

### Day 2
- [ ] VM testing on fresh systems
- [ ] README updates
- [ ] Release notes
- [ ] E2E testing of onboarding flow

---

## Blockers

| Blocker | Status | Resolution |
|---------|--------|------------|
| None currently | - | - |

---

## Resources

- **Code**: `/Users/soleilklosowski/Downloads/citrate/citrate/`
- **Sprint Files**: `/Users/soleilklosowski/Downloads/citrate/.sprint/`
- **Models**: HuggingFace, IPFS
- **Domain**: citrate.ai

---

*Sprint 8 - Created 2025-12-06*
