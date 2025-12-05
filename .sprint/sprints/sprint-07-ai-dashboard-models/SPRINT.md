# Sprint 7: AI-First Dashboard & Model Integration

**Sprint Duration**: 2 weeks
**Sprint Goal**: Transform dashboard to chat-first interface, integrate IPFS for model management, prepare coding agent infrastructure
**Phase**: Phase 5 (Polish & Release) → Phase 6 (AI Agent Enhancement)
**Status**: PLANNING

---

## Sprint Overview

Sprint 7 transforms Citrate from a traditional dashboard into an AI-first conversational interface where all tooling is accessible through the chat agent. Additionally, this sprint establishes the infrastructure for:
1. Local IPFS node management for model storage
2. HuggingFace OAuth integration for model discovery
3. Coding agent preparation with Qwen model integration

---

## Architecture Vision

### Before (Current)
```
┌─────────────────────────────────────────────────┐
│ Sidebar Menu                                    │
│ ┌─────────┐                                     │
│ │Dashboard│  ┌─────────────────────────────┐    │
│ │Wallet   │  │     Content Area            │    │
│ │DAG      │  │     (varies by menu)        │    │
│ │Models   │  │                             │    │
│ │AI Chat  │◄─│                             │    │
│ │Settings │  └─────────────────────────────┘    │
│ └─────────┘                                     │
└─────────────────────────────────────────────────┘
```

### After (AI-First)
```
┌─────────────────────────────────────────────────┐
│ Compact Sidebar         Chat-First Dashboard    │
│ ┌─────────┐  ┌─────────────────────────────────┐│
│ │ ≡ Menu  │  │ AI Agent Chat Interface        ││
│ │ ○ Node  │  │ ┌─────────────────────────────┐││
│ │ ◐ Sync  │  │ │ "Deploy a new ERC-20..."   │││
│ │ ⚙ Quick │  │ │                             │││
│ │   Tools │  │ │ [Agent processes request]  │││
│ │         │  │ │ [Opens Terminal/Editor]    │││
│ │         │  │ │ [Shows contract preview]   │││
│ └─────────┘  │ └─────────────────────────────┘││
│              │ ┌─────────────────────────────┐││
│              │ │ Quick Actions / Tool Panel  │││
│              │ └─────────────────────────────┘││
│              └─────────────────────────────────┘│
└─────────────────────────────────────────────────┘
```

---

## Work Packages

### WP-7.1: Chat-First Dashboard Transformation (13 pts) - P0
**Objective**: Restructure dashboard to make chat the primary interface

**Scope**:
- Remove "AI Chat" from sidebar menu
- Make chat interface the main dashboard content
- Create collapsible/minimal sidebar with status indicators
- Integrate all tools as agent-accessible commands
- Add quick-action buttons below chat for common tasks
- Floating tool panels (Terminal, Editor, Preview)

**Key Changes**:
```typescript
// New Dashboard Layout
- src/components/Dashboard.tsx (major refactor)
- src/components/layout/MinimalSidebar.tsx (new)
- src/components/layout/ChatDashboard.tsx (new)
- src/components/chat/AgentChatInterface.tsx (enhanced)
- src/components/chat/QuickActions.tsx (new)
- src/components/chat/ToolPanel.tsx (new)
```

**Acceptance Criteria**:
- [ ] Dashboard opens directly to chat interface
- [ ] Sidebar is minimal/collapsible showing only node status
- [ ] Agent can access all tools (wallet, DAG, contracts, terminal)
- [ ] Previous menu items accessible via agent commands
- [ ] Quick action buttons for common operations

---

### WP-7.2: IPFS Node Integration (13 pts) - P0
**Objective**: Automatic IPFS node setup and management

**Scope**:
- Auto-detect or install IPFS daemon on first run
- Configure IPFS for model storage (large files)
- Implement IPFS pinning service integration
- Create model download/upload pipeline
- Support external IPFS gateway fallback
- Health monitoring for IPFS daemon

**Architecture**:
```
┌─────────────────────────────────────────────────┐
│                  IPFS Manager                   │
├─────────────────────────────────────────────────┤
│ 1. Check for existing IPFS installation        │
│ 2. If missing, guide user through install      │
│ 3. Configure for Citrate (model storage)       │
│ 4. Start daemon in background                  │
│ 5. Monitor health & connectivity               │
└─────────────────────────────────────────────────┘
           │
           ▼
┌─────────────────────────────────────────────────┐
│              Model Storage Layer                │
├─────────────────────────────────────────────────┤
│ - Pin models locally for offline access        │
│ - Chunk large models (10B+ params)             │
│ - Track download progress                      │
│ - Verify integrity (CID validation)            │
│ - Fallback to gateway if daemon unavailable    │
└─────────────────────────────────────────────────┘
```

**Key Files**:
```
src-tauri/src/ipfs/
├── mod.rs           # IPFS module exports
├── daemon.rs        # Daemon lifecycle management
├── config.rs        # IPFS configuration
├── downloader.rs    # Model download with progress
├── uploader.rs      # Model upload/pinning
└── health.rs        # Health monitoring

src/services/
├── ipfsService.ts   # Frontend IPFS service
├── modelDownloader.ts # Download UI/progress
└── ipfsSetup.tsx    # First-run setup wizard
```

**Acceptance Criteria**:
- [ ] IPFS daemon auto-starts with app (if installed)
- [ ] First-run wizard guides IPFS installation
- [ ] Models can be downloaded with progress indicator
- [ ] Large files (10GB+) handled via chunking
- [ ] Fallback to gateway when daemon unavailable
- [ ] External daemon URL configurable (e.g., ngrok endpoint)

---

### WP-7.3: HuggingFace OAuth Integration (8 pts) - P1
**Objective**: Enable HuggingFace model discovery and download

**Scope**:
- Implement HuggingFace OAuth 2.0 flow
- Model search API integration
- GGUF model filtering and recommendations
- Download queue management
- Token/auth persistence

**OAuth Flow**:
```
┌─────────┐     ┌─────────────┐     ┌──────────────┐
│ Citrate │────▶│ HF OAuth    │────▶│ HuggingFace  │
│   GUI   │◀────│ Callback    │◀────│   API        │
└─────────┘     └─────────────┘     └──────────────┘
     │                                     │
     │         Access Token                │
     │◀────────────────────────────────────│
     │                                     │
     │         Search Models               │
     │────────────────────────────────────▶│
     │                                     │
     │         Model List + Metadata       │
     │◀────────────────────────────────────│
```

**Key Files**:
```
src-tauri/src/huggingface/
├── mod.rs           # HF module exports
├── oauth.rs         # OAuth 2.0 implementation
├── api.rs           # HF API client
├── models.rs        # Model search/filter
└── download.rs      # Model download handler

src/components/models/
├── HuggingFaceLogin.tsx    # OAuth login button
├── ModelBrowser.tsx        # Model search UI
├── ModelFilters.tsx        # GGUF/size filters
└── DownloadQueue.tsx       # Download management
```

**Acceptance Criteria**:
- [ ] User can login to HuggingFace via OAuth
- [ ] Model search with filters (GGUF, size, task)
- [ ] One-click download to local IPFS
- [ ] Download progress tracking
- [ ] Auth token persisted securely

---

### WP-7.4: Qwen Coding Agent Setup (13 pts) - P0
**Objective**: Prepare infrastructure for Qwen 2.5 Coder model

**Scope**:
- Select appropriate Qwen 2.5 Coder model (7B or 14B)
- Configure llama.cpp for optimal inference
- Create Solidity-specific system prompts
- Design training data pipeline structure
- Implement code generation tools

**Model Selection**:
```
Recommended: Qwen2.5-Coder-7B-Instruct-GGUF
- Q4_K_M quantization (~5GB)
- Good balance of speed/quality
- Strong code generation capability
- Supports 32K context window

Alternative: Qwen2.5-Coder-14B-Instruct-GGUF
- Q4_K_M quantization (~9GB)
- Better quality for complex code
- Higher memory requirements
```

**System Prompt Structure**:
```
You are a Solidity smart contract developer with expertise in:
- OpenZeppelin contract patterns
- Foundry testing framework
- EIP/ERC standards implementation
- Gas optimization techniques
- Security best practices (reentrancy, overflow, etc.)

When generating contracts:
1. Follow OpenZeppelin patterns where applicable
2. Include NatSpec documentation
3. Add comprehensive Foundry tests
4. Consider upgrade patterns (proxy/beacon)
5. Implement proper access control
```

**Key Files**:
```
src-tauri/src/agent/
├── coding/
│   ├── mod.rs              # Coding agent module
│   ├── qwen_backend.rs     # Qwen-specific backend
│   ├── solidity_prompts.rs # Solidity system prompts
│   ├── code_tools.rs       # Code gen/edit tools
│   └── context.rs          # Code context management

src/components/coding/
├── CodeGeneration.tsx      # Code gen UI
├── ContractWizard.tsx      # Contract creation wizard
└── TestGenerator.tsx       # Foundry test generator
```

**Training Data Structure** (for future fine-tuning):
```
training-data/
├── openzeppelin/
│   ├── contracts/          # OZ contract examples
│   └── patterns/           # Common patterns
├── foundry/
│   ├── tests/              # Test examples
│   └── scripts/            # Deployment scripts
├── eips/
│   ├── erc20/              # ERC-20 implementations
│   ├── erc721/             # NFT implementations
│   ├── erc1155/            # Multi-token
│   └── erc4626/            # Vault standard
└── security/
    ├── vulnerabilities/    # Known vulnerability patterns
    └── best-practices/     # Secure coding patterns
```

**Acceptance Criteria**:
- [ ] Qwen model downloadable via IPFS/HuggingFace
- [ ] llama.cpp configured for Qwen inference
- [ ] Solidity-specific system prompts implemented
- [ ] Basic code generation working
- [ ] Training data directory structure created

---

### WP-7.5: Model Management UI (8 pts) - P1
**Objective**: Create comprehensive model management interface

**Scope**:
- Local model inventory display
- Model download/delete operations
- Model switching for agent
- Performance metrics display
- GPU/CPU utilization monitoring

**UI Components**:
```
┌─────────────────────────────────────────────────┐
│ Model Manager                              [×]  │
├─────────────────────────────────────────────────┤
│ Active Model: Qwen2.5-Coder-7B              ▼   │
│ Status: ● Running | Memory: 5.2GB | GPU: 45%   │
├─────────────────────────────────────────────────┤
│ Installed Models                                │
│ ┌───────────────────────────────────────────┐   │
│ │ ◉ Qwen2.5-Coder-7B    5.1GB   [Activate]  │   │
│ │ ○ Llama-3-8B          4.8GB   [Delete]    │   │
│ │ ○ Mistral-7B          4.2GB   [Delete]    │   │
│ └───────────────────────────────────────────┘   │
├─────────────────────────────────────────────────┤
│ [+ Download Model]  [HuggingFace Browse]        │
└─────────────────────────────────────────────────┘
```

**Acceptance Criteria**:
- [ ] View all installed models
- [ ] Switch active model
- [ ] Delete unused models
- [ ] Download new models
- [ ] View resource utilization

---

### WP-7.6: Agent Tool Integration (8 pts) - P0
**Objective**: Connect all existing tools to chat agent

**Scope**:
- Map all sidebar features to agent commands
- Implement natural language tool dispatch
- Create tool result formatting
- Add confirmation flows for sensitive ops

**Tool Mapping**:
```
User Request                    → Tool(s) Invoked
─────────────────────────────────────────────────
"Show my wallet balance"        → QUERY_BALANCE
"Send 10 ETH to 0x..."         → SEND_TRANSACTION (confirm)
"Show the DAG visualization"    → Opens DAG panel
"Deploy an ERC-20 token"        → Opens Terminal + Scaffold
"Open a terminal"               → Opens Terminal window
"Edit contract at..."           → Opens Editor window
"Search for NFT models"         → HuggingFace search
"Download Qwen coder model"     → IPFS download
"Check node status"             → NODE_STATUS
"View recent transactions"      → QUERY_TRANSACTIONS
```

**Acceptance Criteria**:
- [ ] All sidebar features accessible via chat
- [ ] Natural language understanding for commands
- [ ] Proper confirmation for sensitive operations
- [ ] Results displayed inline or in panels

---

## Sprint Metrics

| Metric | Value |
|--------|-------|
| **Total Points** | 63 |
| **P0 Points** | 47 |
| **P1 Points** | 16 |
| **Work Packages** | 6 |

---

## Technical Dependencies

### External Dependencies
- IPFS daemon (kubo) - https://docs.ipfs.tech/install/
- HuggingFace OAuth app registration
- llama.cpp with Qwen support

### Internal Dependencies
- Sprint 5 multi-window infrastructure
- Sprint 4 tool implementations
- Sprint 1-2 agent foundation

---

## Risk Assessment

| Risk | Impact | Probability | Mitigation |
|------|--------|-------------|------------|
| IPFS install complexity | High | Medium | Provide clear wizard, binary bundling option |
| Model download size (10GB+) | Medium | High | Chunked downloads, resume support, progress UI |
| HuggingFace API changes | Low | Low | Abstract API layer, version pinning |
| Qwen inference memory | High | Medium | Offer smaller quantizations, swap guidance |
| OAuth security | High | Low | Follow PKCE best practices, secure token storage |

---

## Model Training Roadmap (Future Sprint)

### Phase 1: Data Collection
- Scrape OpenZeppelin contracts
- Collect Foundry test examples
- Parse EIP/ERC specifications
- Gather audited contracts from Code4rena/Sherlock

### Phase 2: Data Preparation
- Format as instruction-response pairs
- Create diverse prompts for same patterns
- Include error cases and fixes
- Balance dataset across contract types

### Phase 3: Fine-tuning
- Use QLoRA for efficient fine-tuning
- Target 1000-5000 examples minimum
- Evaluate on held-out Solidity tasks
- Iterate on system prompts

---

*Created: 2025-12-03*
