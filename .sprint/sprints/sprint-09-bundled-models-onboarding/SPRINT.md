# Sprint 9 - Bundled Models & Agent-Guided Onboarding

**Sprint Goal**: Bundle base AI model with DMG, implement IPFS pinning flow, add multi-provider API keys, and create seamless agent-guided onboarding.

**Duration**: 2 weeks
**Started**: 2025-12-07
**Status**: IN PROGRESS

---

## Overview

This sprint transforms the initial user experience by:
1. Bundling a small, capable AI model directly in the DMG
2. Implementing automatic IPFS pinning with local deletion option
3. Adding API key configuration for OpenAI, Anthropic, Gemini, and xAI
4. Creating agent-guided onboarding that walks users through setup

---

## Architecture

### Model Loading Flow
```
DMG Installation
       │
       ▼
┌─────────────────────┐
│  First Run Check    │
│  (no model loaded?) │
└──────────┬──────────┘
           │
           ▼
┌─────────────────────┐
│  Detect Bundled     │
│  Model in Resources │
└──────────┬──────────┘
           │
           ▼
┌─────────────────────┐
│  Copy to Data Dir   │
│  (~/.citrate/models)│
└──────────┬──────────┘
           │
           ▼
┌─────────────────────┐
│  Load Model for     │
│  Agent Use          │
└──────────┬──────────┘
           │
           ▼
┌─────────────────────┐
│  Offer IPFS Pin     │
│  (background)       │
└──────────┬──────────┘
           │
           ▼
┌─────────────────────┐
│  Option to Delete   │
│  Local (keep CID)   │
└─────────────────────┘
```

### API Key Provider Flow
```
Settings Page
       │
       ▼
┌─────────────────────┐
│  AI Provider Config │
│  ┌─────────────────┐│
│  │ OpenAI          ││
│  │ Anthropic       ││
│  │ Google Gemini   ││
│  │ xAI (Grok)      ││
│  │ Local (default) ││
│  └─────────────────┘│
└──────────┬──────────┘
           │
           ▼
┌─────────────────────┐
│  Agent uses         │
│  configured provider│
│  (fallback: local)  │
└─────────────────────┘
```

---

## Work Packages

### WP-9.1: Bundle Model in DMG (P0) - 8 points
**Goal**: Include Qwen2 0.5B Q4 (~350MB) in the DMG bundle

**Tasks**:
- [x] Download Qwen2-0.5B-Instruct-Q4_K_M.gguf
- [ ] Configure Tauri to include model in resources
- [ ] Update tauri.conf.json resources section
- [ ] Verify model is accessible from Rust backend
- [ ] Test DMG size remains reasonable (~400MB total)

**Files to modify**:
- `src-tauri/tauri.conf.json` - Add resource bundle
- `src-tauri/Cargo.toml` - May need `include_bytes!` or resource loading

### WP-9.2: First-Run Model Detection (P0) - 5 points
**Goal**: Automatically detect and load bundled model on first run

**Tasks**:
- [ ] Create `FirstRunManager` in Rust backend
- [ ] Check for existing model in `~/.citrate/models/`
- [ ] If missing, copy from bundled resources
- [ ] Emit Tauri event when model is ready
- [ ] Show loading state in UI during copy

**Files to create/modify**:
- `src-tauri/src/first_run.rs` - New module
- `src-tauri/src/lib.rs` - Register commands
- `src/components/layout/FirstRunScreen.tsx` - New component

### WP-9.3: IPFS Pinning Flow (P0) - 8 points
**Goal**: Pin bundled model to IPFS, enable local deletion

**Tasks**:
- [ ] Add `pin_bundled_model` command
- [ ] Implement progress tracking for IPFS upload
- [ ] Store CID in local config
- [ ] Add "Delete Local, Keep in IPFS" option
- [ ] Implement re-download from IPFS CID
- [ ] Add IPFS status indicator in UI

**Files to modify**:
- `src-tauri/src/ipfs/mod.rs` - Add model pinning
- `src-tauri/src/models/mod.rs` - Add CID storage
- `src/components/Settings.tsx` - Add IPFS controls

### WP-9.4: Multi-Provider API Keys (P0) - 8 points
**Goal**: Support OpenAI, Anthropic, Gemini, and xAI API keys

**Tasks**:
- [ ] Extend `AgentConfig` with provider-specific keys
- [ ] Add API key validation for each provider
- [ ] Create secure key storage (keychain/credential manager)
- [ ] Implement provider selection logic
- [ ] Add fallback chain: selected → local → error

**Files to modify**:
- `src-tauri/src/agent/config.rs` - Add providers
- `src-tauri/src/agent/llm.rs` - Implement providers
- `src/components/Settings.tsx` - Add API key fields

### WP-9.5: API Key Settings UI (P0) - 5 points
**Goal**: Beautiful, secure API key configuration interface

**Tasks**:
- [ ] Create "AI Providers" section in Settings
- [ ] Add masked input fields for API keys
- [ ] Implement "Test Connection" button per provider
- [ ] Show provider status (connected/disconnected)
- [ ] Add model selection per provider

**New component structure**:
```
Settings.tsx
  └── AIProviderSettings.tsx (new)
        ├── ProviderCard (OpenAI)
        ├── ProviderCard (Anthropic)
        ├── ProviderCard (Gemini)
        ├── ProviderCard (xAI)
        └── LocalModelCard
```

### WP-9.6: Agent Onboarding Integration (P0) - 8 points
**Goal**: Agent guides user through first-run setup

**Tasks**:
- [ ] Add onboarding intents to agent classifier
- [ ] Create "setup_model" tool for agent
- [ ] Create "configure_api_key" tool for agent
- [ ] Implement guided conversation flow
- [ ] Handle "skip" and "later" responses
- [ ] Store onboarding completion state

**Agent flow**:
```
Agent: "Welcome! I'll help you get set up. First, let me check your AI model..."
Agent: "Great, I've loaded a local model so we can chat offline!"
Agent: "Would you like to connect to cloud AI providers for better responses?"
User: "Yes, I have an OpenAI key"
Agent: "Perfect! Go to Settings > AI Providers and enter your key."
Agent: "Also, would you like me to back up the local model to IPFS?"
```

### WP-9.7: Testing & Polish (P1) - 5 points
**Goal**: End-to-end testing of onboarding flow

**Tasks**:
- [ ] Test fresh install flow on macOS
- [ ] Test model loading from bundle
- [ ] Test API key configuration for all providers
- [ ] Test IPFS pin/unpin flow
- [ ] Test agent onboarding conversation
- [ ] Fix any UI/UX issues

---

## Technical Details

### Bundled Model Choice: Qwen2-0.5B-Instruct-Q4_K_M

**Why Qwen2 0.5B**:
- Size: ~350MB (small enough to bundle)
- Quality: Good instruction following
- Speed: Fast inference on CPU
- License: Apache 2.0 (permissive)
- GGUF support: Available

**Download**:
```bash
curl -L -o models/qwen2-0_5b-instruct-q4_k_m.gguf \
  "https://huggingface.co/Qwen/Qwen2-0.5B-Instruct-GGUF/resolve/main/qwen2-0_5b-instruct-q4_k_m.gguf"
```

### API Provider Configuration

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIProviderConfig {
    /// OpenAI configuration
    pub openai: Option<ProviderSettings>,
    /// Anthropic configuration
    pub anthropic: Option<ProviderSettings>,
    /// Google Gemini configuration
    pub gemini: Option<ProviderSettings>,
    /// xAI (Grok) configuration
    pub xai: Option<ProviderSettings>,
    /// Preferred provider order
    pub preferred_order: Vec<AIProvider>,
    /// Always fallback to local
    pub local_fallback: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderSettings {
    pub api_key: String,
    pub model_id: String,
    pub enabled: bool,
    pub base_url: Option<String>,
}
```

### Tauri Resource Bundling

```json
// tauri.conf.json
{
  "bundle": {
    "resources": [
      "resources/models/qwen2-0_5b-instruct-q4_k_m.gguf"
    ]
  }
}
```

### Secure Key Storage

Use `keyring` crate for secure credential storage:
```rust
use keyring::Entry;

fn store_api_key(provider: &str, key: &str) -> Result<()> {
    let entry = Entry::new("citrate", provider)?;
    entry.set_password(key)?;
    Ok(())
}

fn get_api_key(provider: &str) -> Result<String> {
    let entry = Entry::new("citrate", provider)?;
    entry.get_password()
}
```

---

## Dependencies

### External Dependencies
- `keyring` crate for secure credential storage
- Qwen2 0.5B GGUF model file
- IPFS (kubo) for pinning

### Internal Dependencies
- Existing `IpfsManager` module
- Existing `AgentConfig` and `AgentOrchestrator`
- Existing `OnboardingManager`

---

## Acceptance Criteria

### WP-9.1: Bundle Model
- [ ] DMG contains Qwen2 0.5B model
- [ ] Total DMG size < 500MB
- [ ] Model is accessible from Rust code

### WP-9.2: First-Run Detection
- [ ] Model auto-copies on first launch
- [ ] Progress indicator shown during copy
- [ ] Model loads successfully after copy

### WP-9.3: IPFS Pinning
- [ ] User can pin model to IPFS
- [ ] CID is stored and displayed
- [ ] Local file can be deleted after pinning
- [ ] Model can be re-downloaded from CID

### WP-9.4: API Key Settings
- [ ] All 4 providers configurable
- [ ] Keys stored securely
- [ ] Test connection works
- [ ] Provider switching works

### WP-9.5: Onboarding Flow
- [ ] Agent greets new users
- [ ] Agent explains model setup
- [ ] Agent offers API key configuration
- [ ] Agent offers IPFS backup
- [ ] Flow can be skipped

---

## Risks & Mitigations

| Risk | Impact | Mitigation |
|------|--------|------------|
| DMG size too large | HIGH | Use smaller quantization, lazy download |
| Model copy slow | MEDIUM | Show progress, async copy |
| API key security | HIGH | Use OS keychain, never log keys |
| IPFS unavailable | MEDIUM | Make IPFS optional, use gateways |

---

## Definition of Done

- [ ] All acceptance criteria met
- [ ] No console errors on fresh install
- [ ] Agent successfully guides through onboarding
- [ ] API keys work for all 4 providers
- [ ] IPFS pinning works end-to-end
- [ ] Code reviewed and merged to main
- [ ] New DMG built and tested

---

*Sprint 9 - Last updated: 2025-12-07*
