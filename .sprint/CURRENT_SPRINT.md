# Current Sprint

**Active Sprint**: Sprint 8 - Stable Release Preparation
**Phase**: Final Push to v1.0
**Status**: IN PROGRESS
**Started**: 2025-12-06

---

## Sprint Goal

Prepare a stable, downloadable release with:
1. All builds clean and fresh
2. IPFS integration operational with required models
3. RPC endpoint configured (ngrok + future api.citrate.ai)
4. AI onboarding agent fully functional
5. Cross-platform packages ready (macOS, Windows, Linux)

---

## Prerequisites Checklist

| Task | Status | Notes |
|------|--------|-------|
| Clean all caches and builds | [x] Done | `target/`, `node_modules/`, `.citrate/` removed |
| Kill background processes | [x] Done | All citrate/tauri processes stopped |
| Verify genesis models | [~] In Progress | BGE-M3 (417MB) present, Mistral 7B CID configured |
| RPC URL configuration | [ ] Pending | Need to verify localhost:8545 + ngrok setup |
| IPFS daemon configuration | [ ] Pending | Need kubo installation verification |

---

## Work Packages

### WP-8.1: Infrastructure Cleanup (P0) - 5 points
- [x] Clean all Rust target directories
- [x] Clean all node_modules
- [x] Clear chain data (~/.citrate, ~/.citrate-devnet)
- [ ] Verify `cargo build --release` succeeds
- [ ] Verify `npm install && npm run build` in GUI

### WP-8.2: Genesis & Model Setup (P0) - 8 points
- [ ] Verify BGE-M3 embedding model in assets
- [ ] Download/pin Mistral 7B Instruct to IPFS (CID: QmUsYyxg71bV8USRQ6Ccm3SdMqeWgEEVnCYkgNDaxvBTZB)
- [ ] Download/pin Qwen2 0.5B for fast inference
- [ ] Create model download script for first-run setup
- [ ] Test model loading in GUI agent

### WP-8.3: RPC Configuration (P0) - 5 points
- [ ] Document RPC endpoint locations:
  - Local: `http://localhost:8545` (JSON-RPC)
  - Local WS: `ws://localhost:8546`
  - Future: `https://api.citrate.ai` (when DNS configured)
- [ ] Test ngrok tunnel setup: `ngrok http 8545`
- [ ] Create production RPC config template
- [ ] Verify GUI connects to embedded node RPC

### WP-8.4: AI Onboarding Agent (P0) - 13 points
- [ ] Implement skill-level assessment questions
- [ ] Create personalized onboarding flow:
  - Beginner: Explain blockchain basics, guide wallet setup
  - Intermediate: Show development tools, explain dApp creation
  - Advanced: Technical deep-dive, API documentation
- [ ] Add contextual help for each operation
- [ ] Implement progressive disclosure of features
- [ ] Test conversation flow end-to-end

### WP-8.5: IPFS Integration (P0) - 8 points
- [ ] Verify kubo (go-ipfs) installation instructions
- [ ] Implement fallback gateway support
- [ ] Add model download progress UI
- [ ] Implement CID verification for downloaded models
- [ ] Create first-run IPFS setup wizard

### WP-8.6: Cross-Platform Packaging (P0) - 13 points
- [ ] macOS: Build .dmg with code signing
- [ ] Windows: Build .msi installer
- [ ] Linux: Build .AppImage and .deb
- [ ] Include bundled dependencies
- [ ] Create installation documentation
- [ ] Test on fresh systems

### WP-8.7: SDK & Tooling (P1) - 8 points
- [ ] Verify JavaScript SDK builds and publishes
- [ ] Verify Python SDK installs
- [ ] Test CLI tools on all platforms
- [ ] Document SDK usage examples
- [ ] Create quickstart guides

### WP-8.8: Documentation & Polish (P1) - 5 points
- [ ] Update README with fresh installation steps
- [ ] Create QUICK_START.md for new users
- [ ] Document ngrok RPC setup
- [ ] Add troubleshooting guide
- [ ] Create release notes

**Total Points**: 65

---

## Domain Configuration

### Current Status
- Domain: `citrate.ai` (purchased)
- Planned subdomains:
  - `api.citrate.ai` - Production RPC endpoint
  - `docs.citrate.ai` - Documentation portal
  - `explorer.citrate.ai` - Block explorer

### Temporary Setup (ngrok)
```bash
# Start RPC tunnel
ngrok http 8545

# This provides a URL like: https://xxxx.ngrok-free.app
# Use this for testing until api.citrate.ai DNS is configured
```

---

## Model Requirements

### Genesis Embedded
| Model | Size | Status | Location |
|-------|------|--------|----------|
| BGE-M3 (Q4) | 417 MB | [x] Present | `node/assets/bge-m3-q4.gguf` |

### IPFS Pinned (Required)
| Model | CID | Size | Status |
|-------|-----|------|--------|
| Mistral 7B Instruct v0.3 | QmUsYyxg71bV8USRQ6Ccm3SdMqeWgEEVnCYkgNDaxvBTZB | 4.1 GB | [ ] Need to verify |
| Qwen2 0.5B (Q4) | TBD | ~500 MB | [ ] Need to add |

---

## Critical Paths

```
Clean Build ──► Model Setup ──► RPC Config ──► Agent Test ──► Packaging
     │              │              │              │              │
     ▼              ▼              ▼              ▼              ▼
  WP-8.1        WP-8.2         WP-8.3         WP-8.4         WP-8.6
```

---

## Blockers & Risks

| Risk | Severity | Mitigation |
|------|----------|------------|
| Model download fails | HIGH | Use multiple IPFS gateways, include fallback |
| Cross-platform build fails | MEDIUM | Test incrementally, use CI/CD |
| ngrok rate limiting | LOW | Document self-hosting options |
| Agent responses slow | MEDIUM | Use smaller models, implement caching |

---

*Sprint 8 - Last updated: 2025-12-06*
