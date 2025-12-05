# Citrate AI-First GUI Transformation Roadmap

## Executive Summary

Transform the Citrate GUI from a traditional tab-based blockchain wallet into an AI-first conversational interface where users interact with the blockchain, build dApps, generate art, and manage assets through natural language with an intelligent agent.

**Timeline**: 14 weeks (7 sprints)
**Start**: December 2024
**Target v1.0**: March 2025

---

## Phase Overview

```
┌─────────────────────────────────────────────────────────────────────────────┐
│  PHASE 0: CRITICAL FIXES          │  Sprint 0  │  2 weeks  │  BLOCKER      │
├─────────────────────────────────────────────────────────────────────────────┤
│  PHASE 1: AGENT FOUNDATION        │  Sprint 1  │  2 weeks  │  CORE         │
├─────────────────────────────────────────────────────────────────────────────┤
│  PHASE 2: FRONTEND REDESIGN       │  Sprint 2  │  2 weeks  │  UI/UX        │
├─────────────────────────────────────────────────────────────────────────────┤
│  PHASE 3: TOOL IMPLEMENTATION     │  Sprint 3-4│  4 weeks  │  FEATURES     │
├─────────────────────────────────────────────────────────────────────────────┤
│  PHASE 4: MULTI-WINDOW & TERMINAL │  Sprint 5  │  2 weeks  │  ADVANCED     │
├─────────────────────────────────────────────────────────────────────────────┤
│  PHASE 5: POLISH & RELEASE        │  Sprint 6  │  2 weeks  │  RELEASE      │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## Phase 0: Critical Fixes (Sprint 0)
**Duration**: 2 weeks
**Priority**: BLOCKER - Must complete before any other work
**Status**: NOT STARTED

### Objective
Fix all critical blockers preventing the blockchain from functioning correctly in a distributed environment.

### Work Packages

| WP | Title | Points | Priority |
|----|-------|--------|----------|
| WP-0.1 | Fix consensus compilation (missing struct fields) | 2 | P0 |
| WP-0.2 | Implement total ordering (mergeset topological sort) | 13 | P0 |
| WP-0.3 | Fix network peer ID bugs (block/tx propagation) | 5 | P0 |
| WP-0.4 | Implement basic finality mechanism | 13 | P0 |
| WP-0.5 | Fix executor panic points (unwrap → error handling) | 5 | P1 |
| WP-0.6 | Implement ECRECOVER precompile | 5 | P1 |
| WP-0.7 | Add comprehensive test coverage | 8 | P1 |

**Total Points**: 51

### Success Criteria
- [ ] `cargo test --workspace` passes
- [ ] Blocks propagate correctly between nodes
- [ ] Transactions execute in deterministic order
- [ ] Finality achieved within 12 seconds
- [ ] No panic on malformed transactions

### Deliverables
- Working multi-node testnet
- All critical consensus bugs resolved
- Test coverage >70% on core modules

---

## Phase 1: Agent Foundation (Sprint 1)
**Duration**: 2 weeks
**Priority**: CORE
**Status**: BLOCKED BY PHASE 0

### Objective
Build the core agent infrastructure in Rust that powers the AI-first interface.

### Work Packages

| WP | Title | Points | Priority |
|----|-------|--------|----------|
| WP-1.1 | Create AgentOrchestrator module | 8 | P0 |
| WP-1.2 | Implement IntentClassifier (fast patterns + LLM) | 8 | P0 |
| WP-1.3 | Create ToolDispatcher with MCP bindings | 13 | P0 |
| WP-1.4 | Implement streaming response infrastructure | 5 | P0 |
| WP-1.5 | Add GGUF engine integration for local LLM | 5 | P1 |
| WP-1.6 | Create Tauri commands for agent interaction | 3 | P0 |
| WP-1.7 | Add context management (conversation history) | 5 | P1 |

**Total Points**: 47

### Success Criteria
- [ ] Agent responds to basic queries ("What's my balance?")
- [ ] Intent classification >90% accuracy on test set
- [ ] Streaming responses work end-to-end
- [ ] Local LLM inference functional

### Deliverables
- `gui/citrate-core/src-tauri/src/agent/` module
- Working intent classification
- Basic tool execution framework

---

## Phase 2: Frontend Redesign (Sprint 2)
**Duration**: 2 weeks
**Priority**: UI/UX
**Status**: BLOCKED BY PHASE 1

### Objective
Transform the React frontend from tab-based to chat-centric interface.

### Work Packages

| WP | Title | Points | Priority |
|----|-------|--------|----------|
| WP-2.1 | Create ChatInterface component | 8 | P0 |
| WP-2.2 | Create AgentContext for state management | 5 | P0 |
| WP-2.3 | Build MessageThread with streaming support | 8 | P0 |
| WP-2.4 | Create TransactionCard (approve/reject UI) | 5 | P0 |
| WP-2.5 | Build StatusSidebar (node/wallet status) | 3 | P1 |
| WP-2.6 | Create ChainResultCard (query results) | 5 | P1 |
| WP-2.7 | Implement keyboard shortcuts | 2 | P2 |
| WP-2.8 | Add error boundaries and loading states | 3 | P1 |

**Total Points**: 39

### Success Criteria
- [ ] Chat interface renders and accepts input
- [ ] Messages stream token-by-token
- [ ] Transaction approval flow works
- [ ] Status sidebar shows live data

### Deliverables
- New `src/components/chat/` directory
- AgentContext implementation
- Working chat UI prototype

---

## Phase 3: Tool Implementation (Sprint 3-4)
**Duration**: 4 weeks (2 sprints)
**Priority**: FEATURES
**Status**: BLOCKED BY PHASE 2

### Objective
Implement all MCP tools that power agent capabilities.

### Sprint 3: Core Tools

| WP | Title | Points | Priority |
|----|-------|--------|----------|
| WP-3.1 | CHAIN_QUERY tool (balance, block, tx, receipt) | 5 | P0 |
| WP-3.2 | SEND_TRANSACTION tool | 5 | P0 |
| WP-3.3 | DEPLOY_CONTRACT tool | 8 | P0 |
| WP-3.4 | CALL_CONTRACT tool | 5 | P0 |
| WP-3.5 | RUN_INFERENCE tool (model execution) | 8 | P0 |
| WP-3.6 | Tool result formatting for chat | 5 | P1 |

**Sprint 3 Points**: 36

### Sprint 4: Advanced Tools

| WP | Title | Points | Priority |
|----|-------|--------|----------|
| WP-4.1 | SEARCH_MARKETPLACE tool | 5 | P0 |
| WP-4.2 | GENERATE_IMAGE tool (with LoRA support) | 13 | P1 |
| WP-4.3 | SCAFFOLD_DAPP tool (project templates) | 13 | P1 |
| WP-4.4 | EXECUTE_TERMINAL tool | 8 | P0 |
| WP-4.5 | SEARCH_WEB tool | 5 | P2 |
| WP-4.6 | UPLOAD_IPFS tool | 3 | P1 |

**Sprint 4 Points**: 47

### Success Criteria
- [ ] User can query chain via chat
- [ ] User can send transactions via chat
- [ ] User can deploy contracts via chat
- [ ] User can run model inference via chat
- [ ] User can scaffold new dApp projects
- [ ] Terminal commands execute successfully

### Deliverables
- Complete tool registry with 12+ tools
- Integration tests for each tool
- Documentation for tool capabilities

---

## Phase 4: Multi-Window & Terminal (Sprint 5)
**Duration**: 2 weeks
**Priority**: ADVANCED
**Status**: BLOCKED BY PHASE 3

### Objective
Add secondary windows for app preview, terminal, and code editing.

### Work Packages

| WP | Title | Points | Priority |
|----|-------|--------|----------|
| WP-5.1 | WindowManager component (multi-window state) | 8 | P0 |
| WP-5.2 | AppPreviewWindow (deployed dApp preview) | 8 | P0 |
| WP-5.3 | TerminalWindow (PTY integration) | 13 | P0 |
| WP-5.4 | CodeEditorWindow (Monaco editor) | 8 | P1 |
| WP-5.5 | Inter-window communication (IPC) | 5 | P0 |
| WP-5.6 | Window positioning and persistence | 3 | P2 |

**Total Points**: 45

### Success Criteria
- [ ] App preview window shows deployed dApps
- [ ] Terminal window executes commands
- [ ] Agent can read terminal output
- [ ] Windows communicate via IPC

### Deliverables
- Multi-window Tauri configuration
- PTY terminal integration
- Monaco editor integration

---

## Phase 5: Polish & Release (Sprint 6)
**Duration**: 2 weeks
**Priority**: RELEASE
**Status**: BLOCKED BY PHASE 4

### Objective
Polish the experience and prepare for distribution.

### Work Packages

| WP | Title | Points | Priority |
|----|-------|--------|----------|
| WP-6.1 | Error handling and recovery | 8 | P0 |
| WP-6.2 | Performance optimization | 5 | P0 |
| WP-6.3 | End-to-end integration tests | 8 | P0 |
| WP-6.4 | User documentation | 5 | P1 |
| WP-6.5 | Onboarding flow improvements | 5 | P1 |
| WP-6.6 | Cross-platform testing (macOS, Windows, Linux) | 8 | P0 |
| WP-6.7 | Security audit and fixes | 8 | P0 |
| WP-6.8 | Release preparation (signing, packaging) | 5 | P0 |

**Total Points**: 52

### Success Criteria
- [ ] No critical bugs in testing
- [ ] Cross-platform builds work
- [ ] Documentation complete
- [ ] Security review passed
- [ ] Release binaries signed and packaged

### Deliverables
- v1.0 release binaries
- User documentation
- Release notes

---

## Total Story Points by Phase

| Phase | Sprint(s) | Points | Weeks |
|-------|-----------|--------|-------|
| Phase 0 | Sprint 0 | 51 | 2 |
| Phase 1 | Sprint 1 | 47 | 2 |
| Phase 2 | Sprint 2 | 39 | 2 |
| Phase 3 | Sprint 3-4 | 83 | 4 |
| Phase 4 | Sprint 5 | 45 | 2 |
| Phase 5 | Sprint 6 | 52 | 2 |
| **Total** | **7 Sprints** | **317** | **14 weeks** |

---

## Risk Register

| Risk | Impact | Probability | Mitigation |
|------|--------|-------------|------------|
| Consensus fixes take longer | High | Medium | Start with compilation fix, parallelize |
| LLM inference too slow | Medium | Low | Use smaller models, optimize prompts |
| Multi-window complex in Tauri | Medium | Medium | Fall back to tabs if needed |
| PTY integration issues | Low | Medium | Use simpler command execution |
| Cross-platform bugs | Medium | High | Test early on all platforms |

---

## Dependencies

```
Phase 0 ─────┐
             ├─────► Phase 1 ─────► Phase 2 ─────► Phase 3 ─────► Phase 4 ─────► Phase 5
             │
             └─ BLOCKER: Nothing proceeds until Phase 0 complete
```

---

## Milestones

| Milestone | Date | Criteria |
|-----------|------|----------|
| M0: Testnet Working | Week 2 | Multi-node consensus functional |
| M1: Agent Responds | Week 4 | Basic queries answered |
| M2: Chat UI Live | Week 6 | Full chat interface functional |
| M3: Tools Complete | Week 10 | All 12 tools working |
| M4: Multi-Window | Week 12 | Secondary windows functional |
| M5: v1.0 Release | Week 14 | Production release |

---

## Current Status

**Active Phase**: Phase 0 (Critical Fixes)
**Active Sprint**: Sprint 0
**Next Review**: End of Sprint 0

---

*Last Updated: December 2024*
*Version: 1.0*
