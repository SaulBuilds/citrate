# Sprint 6: Polish & Release

**Sprint Duration**: 2 weeks
**Sprint Goal**: Polish the application, fix bugs, test thoroughly, and prepare for beta release
**Phase**: Phase 5 (Polish & Release)
**Status**: IN PROGRESS

---

## Sprint Overview

Sprint 6 is the final sprint before v1.0 release. Focus areas:
1. Integration testing - ensure all components work together
2. Bug fixing - address issues found during testing
3. Performance optimization - ensure smooth UX
4. Cross-platform validation - macOS, Windows, Linux
5. Documentation and onboarding improvements
6. Release packaging and signing

---

## Backlog Items Addressed

### From Product Backlog:
- RL-001: Error handling audit
- RL-002: Performance optimization
- RL-003: E2E integration tests
- RL-004: Cross-platform testing
- RL-007: User documentation
- RL-008: Onboarding flow
- TL-012: SEARCH_WEB tool (deferred - requires external API key)

### Sprint 4 Carryover:
- SEARCH_WEB tool deferred (requires external API integration)

---

## Work Packages

### WP-6.1: Integration Testing & Bug Fixes (13 pts) - P0
**Objective**: Run the application, identify bugs, and fix them

**Scope**:
- Start the Tauri dev server
- Test all Sprint 0-5 features end-to-end
- Fix compilation errors and runtime issues
- Verify agent functionality
- Test window management
- Validate terminal PTY
- Verify persistence and IPC

**Acceptance Criteria**:
- [ ] `cargo build` succeeds with no errors
- [ ] `npm run dev` starts frontend
- [ ] `npm run tauri dev` runs full app
- [ ] Agent chat works (create session, send message)
- [ ] Terminal opens and accepts input
- [ ] Windows open/close/focus correctly
- [ ] No console errors in normal usage

---

### WP-6.2: Error Handling Improvements (8 pts) - P0
**Objective**: Improve error handling throughout the application

**Scope**:
- Add try-catch around critical operations
- Improve error messages for users
- Add error boundaries in React
- Graceful degradation when services unavailable
- Logging improvements for debugging

**Key Files**:
```
src/components/ErrorBoundary.tsx   # React error boundary
src/contexts/ToastContext.tsx      # Toast notifications
src-tauri/src/lib.rs               # Better error returns
```

---

### WP-6.3: Performance Optimization (5 pts) - P1
**Objective**: Ensure smooth user experience

**Scope**:
- Profile React component renders
- Optimize state updates
- Lazy load heavy components (Monaco, xterm)
- Reduce bundle size
- Memory leak prevention

**Metrics**:
- Time to interactive < 3s
- No jank during chat scroll
- Terminal responsive
- Editor loads in < 2s

---

### WP-6.4: Component Integration (8 pts) - P0
**Objective**: Wire up all Sprint 5 components to main app

**Scope**:
- Add window open buttons to main UI
- Integrate terminal tool with new TerminalWindow
- Wire preview to scaffold/deploy tools
- Connect code editor to agent suggestions
- Add keyboard shortcuts for window management

**Key Integration Points**:
```
- ChatBot → WindowContext (open terminal/preview/editor)
- Agent tools → Terminal component
- Agent scaffold → AppPreview
- Agent suggestions → CodeEditor
```

---

### WP-6.5: Onboarding & First-Run Experience (5 pts) - P1
**Objective**: Smooth experience for new users

**Scope**:
- First-time wallet setup flow
- Node connection guidance
- Sample commands/prompts
- Welcome message from agent
- Help command improvements

---

### WP-6.6: Documentation Updates (3 pts) - P2
**Objective**: Update documentation to reflect new features

**Scope**:
- Update README with new features
- Document window management
- Document terminal usage
- Document agent capabilities
- Add troubleshooting guide

---

### WP-6.7: Cross-Platform Validation (5 pts) - P1
**Objective**: Ensure app works on all platforms

**Scope**:
- Test on macOS (primary)
- Document Windows requirements
- Document Linux requirements
- Platform-specific fixes

---

### WP-6.8: Release Preparation (3 pts) - P0
**Objective**: Prepare for beta distribution

**Scope**:
- Update version numbers
- Generate changelog
- Test build process
- Create release artifacts

---

## Sprint Metrics

| Metric | Value |
|--------|-------|
| **Total Points** | 50 |
| **P0 Points** | 32 |
| **P1 Points** | 15 |
| **P2 Points** | 3 |
| **Work Packages** | 8 |

---

## Testing Checklist

### Core Functionality
- [ ] Start node (embedded)
- [ ] Create wallet
- [ ] View balance
- [ ] Send transaction
- [ ] View DAG
- [ ] Deploy model

### Agent Features
- [ ] Create agent session
- [ ] Send message and receive response
- [ ] Execute chain query via agent
- [ ] Run inference via agent
- [ ] Approve/reject tool actions

### Multi-Window
- [ ] Open terminal window
- [ ] Write/read from terminal
- [ ] Resize terminal
- [ ] Open preview window
- [ ] Navigate in preview
- [ ] Open editor window
- [ ] Edit and save file
- [ ] Window persistence on reload

### IPC
- [ ] Messages flow between windows
- [ ] Agent can control terminal
- [ ] Agent suggestions appear in editor

---

## Risk Assessment

| Risk | Impact | Probability | Mitigation |
|------|--------|-------------|------------|
| Build failures | High | Medium | Fix incrementally, test often |
| PTY issues on Windows | Medium | High | Document workaround, test thoroughly |
| Performance issues | Medium | Low | Profile early, optimize hot paths |
| Missing dependencies | Low | Medium | Verify package.json and Cargo.toml |

---

## Daily Plan

### Day 1: Build & Compile
- Run cargo build, fix any Rust errors
- Run npm install, fix any JS errors
- Run npm run tauri dev, capture issues

### Day 2: Core Feature Testing
- Test node start/stop
- Test wallet operations
- Test DAG visualization
- Fix discovered bugs

### Day 3: Agent Testing
- Test agent session creation
- Test message sending
- Test tool execution
- Fix agent bugs

### Day 4: Window Testing
- Test terminal creation
- Test terminal input/output
- Test window management
- Fix window bugs

### Day 5: Integration Testing
- Test full workflows
- Test IPC communication
- Test persistence
- Performance profiling

### Days 6-10: Bug Fixes & Polish
- Address all discovered issues
- Performance optimization
- Documentation
- Release preparation

---

## Dependencies

- All Sprint 0-5 work must be complete
- Node.js 18+, Rust 1.75+, Tauri 2.x
- Platform build tools (Xcode/MSVC/gcc)

---

*Created: 2025-12-03*
