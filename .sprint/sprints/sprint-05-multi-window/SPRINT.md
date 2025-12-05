# Sprint 5: Multi-Window & Terminal Integration

**Sprint Duration**: 2 weeks
**Sprint Goal**: Add secondary windows for terminal, app preview, and code editing with full IPC communication
**Phase**: Phase 4 (Advanced Features)
**Status**: COMPLETE (45/45 points)

---

## Sprint Overview

Sprint 5 transforms the single-window Citrate GUI into a multi-window development environment. Users will be able to:
- Open dedicated terminal windows with interactive PTY
- Preview deployed dApps in separate windows
- Edit code in Monaco-powered windows
- All windows communicate via Tauri IPC for seamless agent integration

---

## Current State Analysis

### What Exists
- Single Tauri window (tauri.conf.json: 1 window at 800x600)
- Tab-based navigation in App.tsx (9 tabs: Dashboard, Wallet, DAG, Models, Marketplace, Chat, IPFS, Contracts, Settings)
- ChatBot component for AI interaction
- ExecuteCommandTool in backend (non-interactive command execution)
- No multi-window infrastructure

### What's Needed
- Tauri multi-window configuration
- WindowManager state management (React)
- PTY integration for interactive terminal
- Monaco editor integration
- dApp preview iframe/webview
- Inter-window communication (IPC)

---

## Work Packages

### WP-5.1: WindowManager Infrastructure (8 pts) - P0
**Objective**: Create the foundational multi-window state management

**Scope**:
- Create `WindowManager` React context for tracking open windows
- Define window types: `main`, `terminal`, `preview`, `editor`
- Implement window lifecycle (open, close, focus, minimize)
- Add window registry with unique IDs
- Create Tauri commands for window operations

**Files to Create**:
```
src/contexts/WindowContext.tsx       # Window state management
src/types/window.ts                  # Window type definitions
src-tauri/src/windows/mod.rs         # Rust window management
src-tauri/src/windows/manager.rs     # Window creation/control
```

**Key Interfaces**:
```typescript
// src/types/window.ts
interface WindowState {
  id: string;
  type: 'main' | 'terminal' | 'preview' | 'editor';
  title: string;
  isOpen: boolean;
  isFocused: boolean;
  position?: { x: number; y: number };
  size?: { width: number; height: number };
  data?: Record<string, unknown>;  // Window-specific data
}

interface WindowContextType {
  windows: WindowState[];
  openWindow: (type: WindowType, data?: Record<string, unknown>) => Promise<string>;
  closeWindow: (id: string) => Promise<void>;
  focusWindow: (id: string) => Promise<void>;
  sendToWindow: (id: string, message: WindowMessage) => Promise<void>;
}
```

**Rust Backend**:
```rust
// src-tauri/src/windows/manager.rs
pub struct WindowManager {
    windows: Arc<RwLock<HashMap<String, WindowHandle>>>,
}

impl WindowManager {
    pub fn create_window(&self, window_type: WindowType, label: &str) -> Result<String>;
    pub fn close_window(&self, id: &str) -> Result<()>;
    pub fn send_message(&self, id: &str, message: &str) -> Result<()>;
    pub fn get_window(&self, id: &str) -> Option<WindowHandle>;
}
```

**Tauri Commands**:
```rust
#[tauri::command]
async fn open_window(window_type: String, data: Option<serde_json::Value>) -> Result<String, String>;

#[tauri::command]
async fn close_window(window_id: String) -> Result<(), String>;

#[tauri::command]
async fn send_to_window(window_id: String, message: String) -> Result<(), String>;
```

**Acceptance Criteria**:
- [ ] WindowContext tracks all open windows
- [ ] Can open/close windows programmatically
- [ ] Window state persists during session
- [ ] Tauri commands work for window operations

---

### WP-5.2: TerminalWindow Component (13 pts) - P0
**Objective**: Create an interactive PTY-based terminal window

**Scope**:
- Integrate `portable-pty` crate for cross-platform PTY
- Create React terminal UI using xterm.js
- Implement bidirectional I/O (stdin/stdout/stderr)
- Add terminal session management
- Connect to agent for command suggestions

**Files to Create**:
```
src/components/windows/TerminalWindow.tsx    # React terminal component
src/components/terminal/XTerminal.tsx        # xterm.js wrapper
src/hooks/useTerminal.ts                     # Terminal state hook
src-tauri/src/terminal/mod.rs                # PTY module
src-tauri/src/terminal/pty.rs                # PTY process management
src-tauri/src/terminal/session.rs            # Session management
```

**Dependencies to Add**:
```toml
# Cargo.toml
portable-pty = "0.8"
```

```json
// package.json
"xterm": "^5.3.0",
"xterm-addon-fit": "^0.8.0",
"xterm-addon-web-links": "^0.9.0"
```

**Key Interfaces**:
```typescript
// src/hooks/useTerminal.ts
interface UseTerminalResult {
  terminalRef: React.RefObject<Terminal>;
  isConnected: boolean;
  sendInput: (data: string) => void;
  resize: (cols: number, rows: number) => void;
  clear: () => void;
  sessionId: string;
}

// Terminal events from Tauri
interface TerminalOutput {
  session_id: string;
  data: string;  // Base64 encoded
}
```

**Rust Backend**:
```rust
// src-tauri/src/terminal/pty.rs
pub struct PtySession {
    id: String,
    pty: Box<dyn portable_pty::MasterPty + Send>,
    child: Box<dyn portable_pty::Child + Send>,
    writer: Box<dyn std::io::Write + Send>,
}

impl PtySession {
    pub fn new(shell: &str, cwd: &Path, env: HashMap<String, String>) -> Result<Self>;
    pub fn write(&mut self, data: &[u8]) -> Result<usize>;
    pub fn resize(&mut self, rows: u16, cols: u16) -> Result<()>;
    pub fn kill(&mut self) -> Result<()>;
}
```

**Tauri Commands**:
```rust
#[tauri::command]
async fn create_terminal_session(cwd: Option<String>) -> Result<String, String>;

#[tauri::command]
async fn write_terminal(session_id: String, data: String) -> Result<(), String>;

#[tauri::command]
async fn resize_terminal(session_id: String, rows: u16, cols: u16) -> Result<(), String>;

#[tauri::command]
async fn close_terminal_session(session_id: String) -> Result<(), String>;
```

**Acceptance Criteria**:
- [ ] Terminal opens in separate window
- [ ] Can type commands and see output
- [ ] Supports colors and cursor positioning (ANSI)
- [ ] Resize works correctly
- [ ] Multiple terminal sessions supported
- [ ] Terminal output can be captured by agent

---

### WP-5.3: AppPreviewWindow Component (8 pts) - P0
**Objective**: Create a window to preview deployed dApps

**Scope**:
- Create webview-based preview window
- Support local dev server URLs (localhost)
- Support deployed app URLs
- Add reload, back, forward controls
- Implement secure sandboxing

**Files to Create**:
```
src/components/windows/AppPreviewWindow.tsx  # Preview window
src/components/preview/PreviewToolbar.tsx    # Navigation controls
src/hooks/usePreview.ts                      # Preview state
src-tauri/src/preview/mod.rs                 # Preview management
```

**Key Interfaces**:
```typescript
// src/components/windows/AppPreviewWindow.tsx
interface AppPreviewProps {
  url: string;
  title: string;
  onClose: () => void;
}

interface PreviewToolbarProps {
  url: string;
  canGoBack: boolean;
  canGoForward: boolean;
  onBack: () => void;
  onForward: () => void;
  onReload: () => void;
  onNavigate: (url: string) => void;
}
```

**Security Considerations**:
```typescript
// Allowed origins for preview
const ALLOWED_ORIGINS = [
  'http://localhost:*',
  'http://127.0.0.1:*',
  'https://*.ipfs.io',
  'https://*.pinata.cloud',
];
```

**Acceptance Criteria**:
- [ ] Preview window opens with URL
- [ ] Can navigate back/forward
- [ ] Reload works
- [ ] Sandbox prevents access to parent window
- [ ] Console logs captured (optional)

---

### WP-5.4: CodeEditorWindow Component (8 pts) - P1
**Objective**: Create Monaco-based code editor window

**Scope**:
- Integrate Monaco editor
- Support Solidity, TypeScript, JSON, Markdown
- Add file tabs for multiple files
- Implement save functionality
- Connect to agent for code suggestions

**Files to Create**:
```
src/components/windows/CodeEditorWindow.tsx  # Editor window
src/components/editor/MonacoEditor.tsx       # Monaco wrapper
src/components/editor/EditorTabs.tsx         # File tabs
src/components/editor/EditorToolbar.tsx      # Save, format, etc.
src/hooks/useEditor.ts                       # Editor state
```

**Dependencies to Add**:
```json
// package.json
"@monaco-editor/react": "^4.6.0"
```

**Key Interfaces**:
```typescript
// src/hooks/useEditor.ts
interface EditorFile {
  path: string;
  content: string;
  language: string;
  isDirty: boolean;
}

interface UseEditorResult {
  files: EditorFile[];
  activeFile: string | null;
  openFile: (path: string) => Promise<void>;
  saveFile: (path: string) => Promise<void>;
  closeFile: (path: string) => void;
  setContent: (path: string, content: string) => void;
}
```

**Acceptance Criteria**:
- [ ] Monaco editor loads in window
- [ ] Supports Solidity syntax highlighting
- [ ] Can open multiple files in tabs
- [ ] Save writes to filesystem
- [ ] Format on save (optional)

---

### WP-5.5: Inter-Window Communication (5 pts) - P0
**Objective**: Enable windows to communicate via Tauri IPC

**Scope**:
- Define message protocol between windows
- Implement event bus for window messages
- Add message types for common actions
- Connect agent to all windows

**Files to Create**:
```
src/services/windowIPC.ts                    # IPC service
src/types/ipc.ts                             # Message types
src-tauri/src/ipc/mod.rs                     # Rust IPC
src-tauri/src/ipc/events.rs                  # Event definitions
```

**Key Interfaces**:
```typescript
// src/types/ipc.ts
type WindowMessageType =
  | 'terminal:output'
  | 'terminal:command'
  | 'editor:save'
  | 'editor:open'
  | 'preview:navigate'
  | 'preview:reload'
  | 'agent:request'
  | 'agent:response';

interface WindowMessage {
  id: string;
  type: WindowMessageType;
  source: string;      // Source window ID
  target?: string;     // Target window ID (null = broadcast)
  payload: unknown;
  timestamp: number;
}

// src/services/windowIPC.ts
class WindowIPCService {
  subscribe(type: WindowMessageType, handler: (msg: WindowMessage) => void): () => void;
  send(type: WindowMessageType, payload: unknown, target?: string): Promise<void>;
  broadcast(type: WindowMessageType, payload: unknown): Promise<void>;
}
```

**Message Flow**:
```
┌─────────────┐      ┌─────────────┐      ┌─────────────┐
│ Main Window │ ◄──► │ Tauri Core  │ ◄──► │  Terminal   │
│   (Chat)    │      │   (IPC)     │      │   Window    │
└─────────────┘      └─────────────┘      └─────────────┘
                           ▲
                           │
                     ┌─────┴─────┐
                     │   Agent   │
                     │ (Backend) │
                     └───────────┘
```

**Acceptance Criteria**:
- [ ] Messages flow between windows
- [ ] Agent can send commands to terminal
- [ ] Terminal output reaches agent
- [ ] Editor save triggers agent notification
- [ ] Preview reload can be triggered from chat

---

### WP-5.6: Window Positioning & Persistence (3 pts) - P2
**Objective**: Remember window positions between sessions

**Scope**:
- Save window positions to localStorage
- Restore positions on app start
- Add window snapping (optional)
- Implement cascading for new windows

**Files to Create**:
```
src/utils/windowPersistence.ts               # Persistence logic
src/hooks/useWindowPersistence.ts            # React hook
```

**Key Interfaces**:
```typescript
// src/utils/windowPersistence.ts
interface PersistedWindowState {
  windows: {
    id: string;
    type: WindowType;
    position: { x: number; y: number };
    size: { width: number; height: number };
  }[];
  lastUpdated: number;
}

function saveWindowState(state: PersistedWindowState): void;
function loadWindowState(): PersistedWindowState | null;
```

**Acceptance Criteria**:
- [ ] Window positions saved on move
- [ ] Positions restored on app restart
- [ ] New windows cascade from top-left
- [ ] Handles multi-monitor gracefully

---

## Sprint Capacity

| Category | Points | Description |
|----------|--------|-------------|
| P0 Work | 34 | WindowManager, Terminal, Preview, IPC |
| P1 Work | 8 | Code Editor |
| P2 Work | 3 | Window Persistence |
| **Total** | **45** | |

---

## Technical Architecture

### Tauri Window Configuration
```json
// tauri.conf.json (updated)
{
  "app": {
    "windows": [
      {
        "label": "main",
        "title": "Citrate",
        "width": 1200,
        "height": 800,
        "resizable": true,
        "center": true
      }
    ],
    "security": {
      "csp": "default-src 'self'; script-src 'self' 'unsafe-eval'; style-src 'self' 'unsafe-inline'"
    }
  }
}
```

### Window Creation (Dynamic)
```rust
// Create new window dynamically
WebviewWindowBuilder::new(&app, label, WebviewUrl::App(url.into()))
    .title(&title)
    .inner_size(width, height)
    .position(x, y)
    .build()?;
```

### Project Structure After Sprint 5
```
src/
├── components/
│   ├── windows/
│   │   ├── TerminalWindow.tsx
│   │   ├── AppPreviewWindow.tsx
│   │   └── CodeEditorWindow.tsx
│   ├── terminal/
│   │   └── XTerminal.tsx
│   ├── editor/
│   │   ├── MonacoEditor.tsx
│   │   ├── EditorTabs.tsx
│   │   └── EditorToolbar.tsx
│   └── preview/
│       └── PreviewToolbar.tsx
├── contexts/
│   └── WindowContext.tsx
├── hooks/
│   ├── useTerminal.ts
│   ├── useEditor.ts
│   ├── usePreview.ts
│   └── useWindowPersistence.ts
├── services/
│   └── windowIPC.ts
└── types/
    ├── window.ts
    └── ipc.ts

src-tauri/src/
├── windows/
│   ├── mod.rs
│   └── manager.rs
├── terminal/
│   ├── mod.rs
│   ├── pty.rs
│   └── session.rs
├── preview/
│   └── mod.rs
└── ipc/
    ├── mod.rs
    └── events.rs
```

---

## Dependencies

### New Cargo Dependencies
```toml
[dependencies]
portable-pty = "0.8"          # Cross-platform PTY
```

### New NPM Dependencies
```json
{
  "dependencies": {
    "xterm": "^5.3.0",
    "xterm-addon-fit": "^0.8.0",
    "xterm-addon-web-links": "^0.9.0",
    "@monaco-editor/react": "^4.6.0"
  }
}
```

---

## Integration Points

### Agent → Terminal
```typescript
// Agent can execute commands in terminal
await ipc.send('terminal:command', {
  command: 'npm run build',
  sessionId: activeTerminalSession,
});

// Agent receives terminal output
ipc.subscribe('terminal:output', (msg) => {
  agent.processTerminalOutput(msg.payload.data);
});
```

### Agent → Editor
```typescript
// Agent can open files
await ipc.send('editor:open', {
  path: '/project/contracts/Token.sol',
});

// Agent can suggest edits
await ipc.send('editor:suggest', {
  path: '/project/contracts/Token.sol',
  range: { startLine: 10, endLine: 15 },
  suggestion: '// Optimized implementation\n...',
});
```

### Agent → Preview
```typescript
// Agent can trigger preview
await ipc.send('preview:navigate', {
  url: 'http://localhost:3000',
  title: 'My dApp',
});
```

---

## Risk Mitigation

| Risk | Impact | Mitigation |
|------|--------|------------|
| PTY complex on Windows | High | Use ConPTY API, test early |
| xterm.js bundle size | Medium | Code split, lazy load |
| Monaco bundle size | Medium | Dynamic import, minimal languages |
| Window Z-order issues | Low | Use Tauri's built-in management |
| IPC message ordering | Medium | Add sequence numbers, queuing |

---

## Testing Strategy

### Unit Tests
- WindowManager state transitions
- IPC message serialization
- Terminal session lifecycle

### Integration Tests
- Open terminal → type command → see output
- Open preview → navigate → see page
- Main window → send to terminal → receive response

### E2E Tests
- Full multi-window workflow
- Window persistence across restart
- Agent-driven terminal commands

---

## Success Criteria

- [ ] User can open terminal window from chat ("open terminal")
- [ ] User can type commands and see output
- [ ] Agent can run commands in terminal
- [ ] User can preview deployed dApps
- [ ] Code editor opens with syntax highlighting
- [ ] Windows remember their positions

---

## Definition of Done

1. All P0 work packages complete
2. Windows open/close without errors
3. PTY terminal works on macOS/Linux (Windows stretch goal)
4. IPC messages flow correctly
5. Integration tests pass
6. Documentation updated

---

*Created: 2025-12-03*
