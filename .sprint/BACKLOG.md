# Product Backlog

## Overview
Prioritized list of all work items for the AI-First GUI Transformation initiative.

**Priority Levels**:
- **P0**: Must have - blocks release
- **P1**: Should have - important for UX
- **P2**: Nice to have - defer if needed

**Status**:
- `BACKLOG` - Not yet scheduled
- `SPRINT-X` - Assigned to sprint
- `DONE` - Completed

---

## Epic: Critical Infrastructure Fixes

### P0 - Blockers

| ID | Title | Points | Status | Notes |
|----|-------|--------|--------|-------|
| BL-001 | Fix consensus test compilation | 2 | SPRINT-0 | Missing embedded_models, required_pins |
| BL-002 | Implement mergeset total ordering | 13 | SPRINT-0 | Required for deterministic execution |
| BL-003 | Fix block propagation peer IDs | 3 | SPRINT-0 | Random IDs break routing |
| BL-004 | Fix transaction gossip peer IDs | 2 | SPRINT-0 | Same issue as BL-003 |
| BL-005 | Implement basic finality | 13 | SPRINT-0 | Depth-based confirmation |
| BL-006 | Fix executor unwrap panics | 5 | SPRINT-0 | Convert to proper errors |
| BL-007 | Implement ECRECOVER precompile | 5 | SPRINT-0 | Signature verification |

### P1 - High Priority

| ID | Title | Points | Status | Notes |
|----|-------|--------|--------|-------|
| BL-008 | Implement MODEXP precompile | 5 | BACKLOG | RSA operations |
| BL-009 | Implement ECADD/ECMUL precompiles | 8 | BACKLOG | BN128 curve operations |
| BL-010 | Add BLOCKHASH implementation | 3 | BACKLOG | Currently returns zero |
| BL-011 | Complete encrypted IPFS | 8 | BACKLOG | Real ECIES encryption |
| BL-012 | Add state root commitment | 8 | BACKLOG | Proper MPT root |

### P2 - Lower Priority

| ID | Title | Points | Status | Notes |
|----|-------|--------|--------|-------|
| BL-013 | Implement proper VRF (ECVRF) | 8 | BACKLOG | Replace SHA3-based |
| BL-014 | Add network partition recovery | 5 | BACKLOG | DHT discovery |
| BL-015 | Cache size limits in consensus | 3 | BACKLOG | Prevent memory bloat |

---

## Epic: Agent Layer

### P0 - Core Agent

| ID | Title | Points | Status | Notes |
|----|-------|--------|--------|-------|
| AG-001 | AgentOrchestrator module | 8 | BACKLOG | Task routing, context |
| AG-002 | IntentClassifier - fast patterns | 3 | BACKLOG | Regex/keyword matching |
| AG-003 | IntentClassifier - LLM fallback | 5 | BACKLOG | GGUF integration |
| AG-004 | ToolDispatcher framework | 5 | BACKLOG | Tool registry, execution |
| AG-005 | MCP tool bindings | 8 | BACKLOG | Connect to existing services |
| AG-006 | Streaming response infra | 5 | BACKLOG | SSE via Tauri |
| AG-007 | Tauri agent commands | 3 | BACKLOG | IPC interface |

### P1 - Agent Features

| ID | Title | Points | Status | Notes |
|----|-------|--------|--------|-------|
| AG-008 | Conversation history | 5 | BACKLOG | Context window |
| AG-009 | Tool result formatting | 5 | BACKLOG | Render in chat |
| AG-010 | Agent configuration UI | 3 | BACKLOG | Model selection |
| AG-011 | Prompt templates | 3 | BACKLOG | Reusable prompts |

---

## Epic: Chat Frontend

### P0 - Core UI

| ID | Title | Points | Status | Notes |
|----|-------|--------|--------|-------|
| UI-001 | ChatInterface component | 8 | BACKLOG | Main chat container |
| UI-002 | MessageThread component | 5 | BACKLOG | Message history |
| UI-003 | MessageInput component | 3 | BACKLOG | Input with attachments |
| UI-004 | AgentContext | 5 | BACKLOG | State management |
| UI-005 | StreamService | 3 | BACKLOG | Handle streaming |
| UI-006 | TransactionCard | 5 | BACKLOG | Approve/reject |

### P1 - Enhanced UI

| ID | Title | Points | Status | Notes |
|----|-------|--------|--------|-------|
| UI-007 | ChainResultCard | 5 | BACKLOG | Display query results |
| UI-008 | StatusSidebar | 3 | BACKLOG | Node/wallet status |
| UI-009 | ArtifactViewer | 5 | BACKLOG | Show generated files |
| UI-010 | ImagePreview | 3 | BACKLOG | Generated images |
| UI-011 | Error boundaries | 3 | BACKLOG | Graceful failures |
| UI-012 | Loading skeletons | 2 | BACKLOG | Loading states |

### P2 - Polish

| ID | Title | Points | Status | Notes |
|----|-------|--------|--------|-------|
| UI-013 | Keyboard shortcuts | 2 | BACKLOG | Power user UX |
| UI-014 | Dark/light theme | 3 | BACKLOG | Theme switching |
| UI-015 | Sound notifications | 1 | BACKLOG | Optional audio |

---

## Epic: Tools

### P0 - Core Tools

| ID | Title | Points | Status | Notes |
|----|-------|--------|--------|-------|
| TL-001 | CHAIN_QUERY tool | 5 | BACKLOG | Balance, block, tx |
| TL-002 | SEND_TRANSACTION tool | 5 | BACKLOG | Token transfers |
| TL-003 | DEPLOY_CONTRACT tool | 8 | BACKLOG | Compile + deploy |
| TL-004 | CALL_CONTRACT tool | 5 | BACKLOG | Read/write calls |
| TL-005 | RUN_INFERENCE tool | 8 | BACKLOG | Model execution |
| TL-006 | EXECUTE_TERMINAL tool | 8 | BACKLOG | Shell commands |

### P1 - Advanced Tools

| ID | Title | Points | Status | Notes |
|----|-------|--------|--------|-------|
| TL-007 | SEARCH_MARKETPLACE tool | 5 | BACKLOG | Model discovery |
| TL-008 | GENERATE_IMAGE tool | 13 | BACKLOG | Image generation |
| TL-009 | SCAFFOLD_DAPP tool | 13 | BACKLOG | Project templates |
| TL-010 | UPLOAD_IPFS tool | 3 | BACKLOG | File storage |
| TL-011 | APPLY_LORA tool | 5 | BACKLOG | LoRA adapters |

### P2 - Extended Tools

| ID | Title | Points | Status | Notes |
|----|-------|--------|--------|-------|
| TL-012 | SEARCH_WEB tool | 5 | BACKLOG | Web search - requires external API (deferred from Sprint 4) |
| TL-013 | READ_DOCS tool | 3 | BACKLOG | Documentation search |
| TL-014 | GIT_OPERATIONS tool | 5 | BACKLOG | Version control |

---

## Epic: Multi-Window

### P0 - Core Windows

| ID | Title | Points | Status | Notes |
|----|-------|--------|--------|-------|
| MW-001 | WindowManager component | 8 | BACKLOG | Multi-window state |
| MW-002 | AppPreviewWindow | 8 | BACKLOG | Preview dApps |
| MW-003 | TerminalWindow | 13 | BACKLOG | PTY integration |
| MW-004 | Inter-window IPC | 5 | BACKLOG | Communication |

### P1 - Enhanced Windows

| ID | Title | Points | Status | Notes |
|----|-------|--------|--------|-------|
| MW-005 | CodeEditorWindow | 8 | BACKLOG | Monaco editor |
| MW-006 | Window persistence | 3 | BACKLOG | Remember positions |
| MW-007 | Window docking | 5 | BACKLOG | Dock to sides |

---

## Epic: Release Preparation

### P0 - Must Have

| ID | Title | Points | Status | Notes |
|----|-------|--------|--------|-------|
| RL-001 | Error handling audit | 8 | BACKLOG | Comprehensive review |
| RL-002 | Performance optimization | 5 | BACKLOG | Profiling, fixes |
| RL-003 | E2E integration tests | 8 | BACKLOG | Full workflows |
| RL-004 | Cross-platform testing | 8 | BACKLOG | Mac/Win/Linux |
| RL-005 | Security audit | 8 | BACKLOG | Vulnerability scan |
| RL-006 | Release packaging | 5 | BACKLOG | Signed binaries |

### P1 - Should Have

| ID | Title | Points | Status | Notes |
|----|-------|--------|--------|-------|
| RL-007 | User documentation | 5 | BACKLOG | Usage guides |
| RL-008 | Onboarding flow | 5 | BACKLOG | First-time UX |
| RL-009 | Changelog generation | 2 | BACKLOG | Release notes |

---

## Summary

| Epic | P0 Points | P1 Points | P2 Points | Total |
|------|-----------|-----------|-----------|-------|
| Critical Fixes | 43 | 24 | 16 | 83 |
| Agent Layer | 37 | 16 | 0 | 53 |
| Chat Frontend | 29 | 21 | 6 | 56 |
| Tools | 39 | 39 | 13 | 91 |
| Multi-Window | 34 | 16 | 0 | 50 |
| Release | 42 | 12 | 0 | 54 |
| **Total** | **224** | **128** | **35** | **387** |

---

*Last Updated: December 2024*
