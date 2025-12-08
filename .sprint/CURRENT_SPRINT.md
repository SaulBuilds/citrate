# Current Sprint

**Active Sprint**: Sprint 9 - Bundled Models & Agent-Guided Onboarding
**Phase**: v1.0 Release - User Experience
**Status**: IN PROGRESS
**Started**: 2025-12-07

---

## Sprint Goal

Transform the initial user experience with:
1. Bundled AI model (Qwen2 0.5B) in DMG for offline use
2. IPFS pinning flow with local deletion option
3. Multi-provider API key settings (OpenAI, Anthropic, Gemini, xAI)
4. Agent-guided onboarding that walks users through setup

---

## Quick Reference

| Work Package | Status | Points |
|--------------|--------|--------|
| WP-9.1: Bundle Model in DMG | IN PROGRESS | 8 |
| WP-9.2: First-Run Detection | PENDING | 5 |
| WP-9.3: IPFS Pinning Flow | PENDING | 8 |
| WP-9.4: Multi-Provider API Keys | PENDING | 8 |
| WP-9.5: API Key Settings UI | PENDING | 5 |
| WP-9.6: Agent Onboarding | PENDING | 8 |
| WP-9.7: Testing & Polish | PENDING | 5 |

**Total Points**: 47

---

## Today's Focus

### Immediate Tasks
1. Configure Tauri to bundle Qwen2 model
2. Create first-run detection module
3. Add API key settings to Settings page
4. Wire up agent onboarding flow

### Blocked By
- Need Qwen2 0.5B GGUF model downloaded

---

## Key Files

| Purpose | Path |
|---------|------|
| Sprint Plan | `.sprint/sprints/sprint-09-bundled-models-onboarding/SPRINT.md` |
| Tauri Config | `gui/citrate-core/src-tauri/tauri.conf.json` |
| Agent Config | `gui/citrate-core/src-tauri/src/agent/config.rs` |
| IPFS Manager | `gui/citrate-core/src-tauri/src/ipfs/mod.rs` |
| Settings UI | `gui/citrate-core/src/components/Settings.tsx` |
| Onboarding | `gui/citrate-core/src-tauri/src/agent/onboarding.rs` |

---

## Previous Sprint (Sprint 8)

**Status**: COMPLETED
**Achievements**:
- Fixed styled-jsx Babel configuration
- Fixed rpc_call → agent_send_message
- Built production DMG
- Branded loading screen

---

*Last updated: 2025-12-07*
