# Sprint 2 - Frontend Redesign

## Overview
**Duration**: 2 weeks
**Start Date**: 2025-12-02
**Points**: 39
**Phase**: Phase 2 - Frontend Redesign
**Status**: IN PROGRESS

## Objective
Transform the React frontend from tab-based to chat-centric interface. Build all core UI components for the AI-first conversational experience.

---

## Sprint Goals

1. Create ChatInterface as the main container component
2. Implement AgentContext for centralized state management
3. Build MessageThread with real-time streaming support
4. Create TransactionCard for approve/reject flows
5. Build StatusSidebar showing node/wallet status
6. Create ChainResultCard for query result display
7. Implement keyboard shortcuts for power users
8. Add error boundaries and loading states

---

## Work Packages

### WP-2.1: ChatInterface Component
**Points**: 8 | **Priority**: P0 | **Status**: [ ] Not Started

**Description**:
Create the main chat container component that houses the message thread, input area, and manages the overall chat layout.

**Tasks**:
- [ ] Create `src/components/chat/ChatInterface.tsx`
- [ ] Implement chat layout with:
  - Header with session controls
  - Message thread area (scrollable)
  - Input area (fixed at bottom)
  - Optional sidebar toggle
- [ ] Add session management:
  - Create new session
  - Switch between sessions
  - Clear current session
- [ ] Implement auto-scroll behavior
- [ ] Add responsive layout for different screen sizes
- [ ] Connect to AgentContext for state

**Acceptance Criteria**:
- [ ] Chat interface renders correctly
- [ ] Can create and switch sessions
- [ ] Auto-scrolls on new messages
- [ ] Responsive on all screen sizes

**Files to Create**:
- `gui/citrate-core/src/components/chat/ChatInterface.tsx`
- `gui/citrate-core/src/components/chat/ChatHeader.tsx`
- `gui/citrate-core/src/components/chat/index.ts`

**Dependencies**: None

---

### WP-2.2: AgentContext for State Management
**Points**: 5 | **Priority**: P0 | **Status**: [ ] Not Started

**Description**:
Create a React context that manages all agent-related state including sessions, messages, streaming status, and configuration.

**Tasks**:
- [ ] Create `src/contexts/AgentContext.tsx`
- [ ] Implement state management for:
  - Current session ID
  - Message history per session
  - Streaming status (active, tokens received)
  - Pending tool approvals
  - Agent configuration
  - Connection status
- [ ] Create hooks:
  - `useAgent()` - Main context hook
  - `useMessages()` - Message history hook
  - `useStreaming()` - Streaming state hook
  - `usePendingTools()` - Tool approval hook
- [ ] Add Tauri event listeners:
  - `agent-token` - Handle streaming tokens
  - `agent-complete` - Handle message completion
  - `agent-error` - Handle errors
  - `agent-tool-call` - Handle tool invocations
- [ ] Implement message sending logic
- [ ] Add persistence for session list

**Acceptance Criteria**:
- [ ] Context provides all agent state
- [ ] Streaming updates reflect in real-time
- [ ] Tool approvals tracked correctly
- [ ] Sessions persist across refreshes

**Files to Create**:
- `gui/citrate-core/src/contexts/AgentContext.tsx`
- `gui/citrate-core/src/hooks/useAgent.ts`
- `gui/citrate-core/src/types/agent.ts`

**Dependencies**: None

---

### WP-2.3: MessageThread with Streaming Support
**Points**: 8 | **Priority**: P0 | **Status**: [ ] Not Started

**Description**:
Build the message thread component that displays conversation history with real-time streaming support.

**Tasks**:
- [ ] Create `src/components/chat/MessageThread.tsx`
- [ ] Implement message types:
  - User message (right-aligned, colored)
  - Assistant message (left-aligned, markdown support)
  - System message (centered, muted)
  - Tool result message (card-based)
- [ ] Add streaming support:
  - Cursor animation during streaming
  - Token-by-token text append
  - Smooth scroll during stream
- [ ] Implement markdown rendering:
  - Code blocks with syntax highlighting
  - Tables, lists, links
  - Inline code formatting
- [ ] Add message actions:
  - Copy message content
  - Regenerate response
  - Delete message
- [ ] Implement virtualization for long threads

**Acceptance Criteria**:
- [ ] All message types render correctly
- [ ] Streaming shows token-by-token
- [ ] Markdown renders properly
- [ ] Long conversations performant

**Files to Create**:
- `gui/citrate-core/src/components/chat/MessageThread.tsx`
- `gui/citrate-core/src/components/chat/Message.tsx`
- `gui/citrate-core/src/components/chat/StreamingMessage.tsx`
- `gui/citrate-core/src/components/chat/MessageActions.tsx`

**Dependencies**: WP-2.2

---

### WP-2.4: TransactionCard (Approve/Reject UI)
**Points**: 5 | **Priority**: P0 | **Status**: [ ] Not Started

**Description**:
Create a card component for displaying pending transactions that require user approval.

**Tasks**:
- [ ] Create `src/components/chat/TransactionCard.tsx`
- [ ] Display transaction details:
  - From/To addresses (truncated with copy)
  - Amount with token symbol
  - Gas estimate
  - Transaction type (send, deploy, call)
- [ ] Add approve/reject buttons with:
  - Loading states
  - Confirmation animation
  - Error feedback
- [ ] Show transaction simulation result
- [ ] Add timeout indicator (if applicable)
- [ ] Implement keyboard shortcuts (Enter to approve, Esc to reject)

**Acceptance Criteria**:
- [ ] Transaction details clearly visible
- [ ] Approve/reject flow works
- [ ] Loading and error states handled
- [ ] Keyboard shortcuts functional

**Files to Create**:
- `gui/citrate-core/src/components/chat/TransactionCard.tsx`
- `gui/citrate-core/src/components/chat/AddressDisplay.tsx`

**Dependencies**: WP-2.2

---

### WP-2.5: StatusSidebar (Node/Wallet Status)
**Points**: 3 | **Priority**: P1 | **Status**: [ ] Not Started

**Description**:
Create a sidebar component showing real-time status of node connection and wallet state.

**Tasks**:
- [ ] Create `src/components/sidebar/StatusSidebar.tsx`
- [ ] Display node status:
  - Connection indicator (green/red)
  - Block height
  - Peer count
  - Sync status
- [ ] Display wallet status:
  - Active account address
  - Balance
  - Pending transactions count
- [ ] Display agent status:
  - Current model
  - Streaming indicator
  - Token usage
- [ ] Add collapsible sections
- [ ] Implement auto-refresh

**Acceptance Criteria**:
- [ ] All status info displays correctly
- [ ] Updates in real-time
- [ ] Collapsible sections work
- [ ] Responsive on small screens

**Files to Create**:
- `gui/citrate-core/src/components/sidebar/StatusSidebar.tsx`
- `gui/citrate-core/src/components/sidebar/NodeStatus.tsx`
- `gui/citrate-core/src/components/sidebar/WalletStatus.tsx`
- `gui/citrate-core/src/components/sidebar/AgentStatus.tsx`

**Dependencies**: WP-2.2

---

### WP-2.6: ChainResultCard (Query Results)
**Points**: 5 | **Priority**: P1 | **Status**: [ ] Not Started

**Description**:
Create a card component for displaying blockchain query results in a readable format.

**Tasks**:
- [ ] Create `src/components/chat/ChainResultCard.tsx`
- [ ] Support result types:
  - Balance query (formatted amount)
  - Block info (expandable details)
  - Transaction receipt (status, logs)
  - Account info (nonce, code hash)
- [ ] Add formatting:
  - Large numbers with separators
  - Hex values with copy button
  - Timestamp formatting
  - Status badges (success/fail)
- [ ] Implement expandable sections for detailed data
- [ ] Add refresh button for re-query

**Acceptance Criteria**:
- [ ] All result types display correctly
- [ ] Formatting is human-readable
- [ ] Expandable sections work
- [ ] Copy functionality works

**Files to Create**:
- `gui/citrate-core/src/components/chat/ChainResultCard.tsx`
- `gui/citrate-core/src/components/chat/BlockInfoCard.tsx`
- `gui/citrate-core/src/components/chat/TransactionReceiptCard.tsx`

**Dependencies**: WP-2.2

---

### WP-2.7: Keyboard Shortcuts
**Points**: 2 | **Priority**: P2 | **Status**: [ ] Not Started

**Description**:
Implement keyboard shortcuts for power users to navigate and interact efficiently.

**Tasks**:
- [ ] Create keyboard shortcut system:
  - Global shortcuts (work everywhere)
  - Context-specific shortcuts
  - Shortcut help modal (Cmd+/)
- [ ] Implement shortcuts:
  - `Cmd+K` - New session
  - `Cmd+Enter` - Send message
  - `Cmd+Shift+C` - Copy last response
  - `Escape` - Cancel streaming/close modal
  - `Up/Down` - Navigate message history
  - `Cmd+/` - Show shortcuts help
- [ ] Add visual indicators for active shortcuts
- [ ] Make shortcuts configurable

**Acceptance Criteria**:
- [ ] All shortcuts work correctly
- [ ] Help modal shows all shortcuts
- [ ] No conflicts with system shortcuts
- [ ] Configurable in settings

**Files to Create**:
- `gui/citrate-core/src/hooks/useKeyboardShortcuts.ts`
- `gui/citrate-core/src/components/modals/ShortcutsHelp.tsx`

**Dependencies**: WP-2.1

---

### WP-2.8: Error Boundaries and Loading States
**Points**: 3 | **Priority**: P1 | **Status**: [ ] Not Started

**Description**:
Add comprehensive error handling and loading state management throughout the chat UI.

**Tasks**:
- [ ] Create error boundary components:
  - `ChatErrorBoundary` - Catches chat-related errors
  - `MessageErrorBoundary` - Per-message error handling
  - Generic error fallback UI
- [ ] Implement loading states:
  - Skeleton loaders for messages
  - Spinner for operations
  - Progress indicators for long operations
- [ ] Add error recovery:
  - Retry buttons
  - Error reporting
  - Graceful degradation
- [ ] Create toast notifications:
  - Success messages
  - Error alerts
  - Info notifications

**Acceptance Criteria**:
- [ ] Errors don't crash the app
- [ ] Loading states visible during operations
- [ ] Retry functionality works
- [ ] Toast notifications display correctly

**Files to Create**:
- `gui/citrate-core/src/components/common/ErrorBoundary.tsx`
- `gui/citrate-core/src/components/common/LoadingStates.tsx`
- `gui/citrate-core/src/components/common/Toast.tsx`
- `gui/citrate-core/src/contexts/ToastContext.tsx`

**Dependencies**: WP-2.1

---

## Architecture Overview

```
┌─────────────────────────────────────────────────────────────────┐
│                         App Container                           │
│  ┌─────────────────────────────────────────────────────────┐   │
│  │                     AgentProvider                         │   │
│  │  ┌─────────────────────────────────────────────────────┐ │   │
│  │  │                   ToastProvider                       │ │   │
│  │  │  ┌─────────────────────────────────────────────────┐│ │   │
│  │  │  │               Main Layout                        ││ │   │
│  │  │  │  ┌──────────┐  ┌──────────────────────────┐    ││ │   │
│  │  │  │  │ Status   │  │     ChatInterface        │    ││ │   │
│  │  │  │  │ Sidebar  │  │  ┌────────────────────┐  │    ││ │   │
│  │  │  │  │          │  │  │   ChatHeader       │  │    ││ │   │
│  │  │  │  │ - Node   │  │  ├────────────────────┤  │    ││ │   │
│  │  │  │  │ - Wallet │  │  │   MessageThread    │  │    ││ │   │
│  │  │  │  │ - Agent  │  │  │   - Messages       │  │    ││ │   │
│  │  │  │  │          │  │  │   - TransactionCard│  │    ││ │   │
│  │  │  │  │          │  │  │   - ChainResultCard│  │    ││ │   │
│  │  │  │  │          │  │  ├────────────────────┤  │    ││ │   │
│  │  │  │  │          │  │  │   MessageInput     │  │    ││ │   │
│  │  │  │  └──────────┘  │  └────────────────────┘  │    ││ │   │
│  │  │  │                └──────────────────────────┘    ││ │   │
│  │  │  └─────────────────────────────────────────────────┘│ │   │
│  │  └─────────────────────────────────────────────────────┘ │   │
│  └─────────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────────┘
```

---

## Component Hierarchy

```
ChatInterface
├── ChatHeader
│   ├── SessionSelector
│   ├── NewSessionButton
│   └── SettingsButton
├── MessageThread
│   ├── Message (user)
│   ├── Message (assistant)
│   │   └── StreamingMessage
│   ├── TransactionCard
│   ├── ChainResultCard
│   └── SystemMessage
├── MessageInput
│   ├── TextArea
│   ├── AttachButton
│   └── SendButton
└── StatusSidebar (optional)
    ├── NodeStatus
    ├── WalletStatus
    └── AgentStatus
```

---

## Success Criteria

- [ ] Chat interface is the primary view
- [ ] Messages stream token-by-token
- [ ] Transaction approval flow complete
- [ ] Status sidebar shows live data
- [ ] Keyboard shortcuts functional
- [ ] Error handling graceful
- [ ] All 8 work packages completed

---

## Risk Mitigation

| Risk | Mitigation |
|------|------------|
| Streaming performance issues | Batch UI updates, use requestAnimationFrame |
| State management complexity | Keep AgentContext focused, split if needed |
| Markdown rendering issues | Use proven library (react-markdown) |
| Virtualization complexity | Use react-window for long threads |

---

## Dependencies

- Sprint 1 complete (agent backend working)
- Existing React/Tauri setup
- Tailwind CSS configured
- TypeScript strict mode

---

*Last Updated: 2025-12-02*
