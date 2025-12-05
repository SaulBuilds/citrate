# Sprint 1 - Agent Foundation

## Overview
**Duration**: 2 weeks
**Start Date**: 2025-12-02
**Points**: 47
**Phase**: Phase 1 - Agent Foundation
**Status**: IN PROGRESS

## Objective
Build the core agent infrastructure in Rust that powers the AI-first conversational interface. This creates the foundation for all future agent capabilities.

---

## Sprint Goals

1. Create a modular agent orchestration system in the Tauri backend
2. Implement intent classification (fast patterns + LLM fallback)
3. Build the tool dispatch framework with MCP bindings
4. Add streaming response infrastructure
5. Integrate hybrid LLM support (API + local GGUF)
6. Create Tauri commands for frontend agent interaction
7. Implement conversation context management

---

## Work Packages

### WP-1.1: AgentOrchestrator Module
**Points**: 8 | **Priority**: P0 | **Status**: [ ] Not Started

**Description**:
Create the central agent orchestration system that manages conversations, routes intents to tools, and maintains context.

**Tasks**:
- [ ] Create `src-tauri/src/agent/mod.rs` module structure
- [ ] Implement `AgentOrchestrator` struct with:
  - Session management (conversation state per user)
  - Intent routing to appropriate handlers
  - Context window management
  - Response aggregation
- [ ] Add `AgentConfig` for configurable behavior
- [ ] Implement `AgentSession` for stateful conversations
- [ ] Add error handling and graceful degradation
- [ ] Wire into main lib.rs exports

**Acceptance Criteria**:
- [ ] Agent module compiles and integrates with existing Tauri app
- [ ] Can create and manage multiple conversation sessions
- [ ] Basic message routing works end-to-end
- [ ] Config is loadable from file or defaults

**Files to Create**:
- `gui/citrate-core/src-tauri/src/agent/mod.rs`
- `gui/citrate-core/src-tauri/src/agent/orchestrator.rs`
- `gui/citrate-core/src-tauri/src/agent/session.rs`
- `gui/citrate-core/src-tauri/src/agent/config.rs`

**Dependencies**: None

---

### WP-1.2: IntentClassifier (Fast Patterns + LLM)
**Points**: 8 | **Priority**: P0 | **Status**: [ ] Not Started

**Description**:
Implement a two-tier intent classification system:
1. **Fast path**: Regex/keyword patterns for common intents
2. **LLM fallback**: For complex/ambiguous queries

**Tasks**:
- [ ] Define `Intent` enum with all supported intents:
  - `QueryBalance`, `SendTransaction`, `DeployContract`
  - `CallContract`, `RunInference`, `SearchMarketplace`
  - `GetBlockInfo`, `GetDAGStatus`, `ManageWallet`
  - `GeneralChat`, `Unknown`
- [ ] Implement `FastPatternClassifier`:
  - Regex patterns for each intent
  - Keyword matching with confidence scores
  - Parameter extraction (addresses, amounts, etc.)
- [ ] Implement `LLMClassifier` interface:
  - Abstract trait for API and local models
  - System prompt for intent classification
  - JSON output parsing
- [ ] Create `IntentClassifier` that combines both:
  - Try fast path first
  - Fall back to LLM if confidence < threshold
  - Cache recent classifications

**Acceptance Criteria**:
- [ ] Fast patterns match common queries with >95% accuracy
- [ ] LLM fallback works for ambiguous queries
- [ ] Classification completes in <100ms for patterns
- [ ] Parameters extracted correctly (addresses, amounts)

**Files to Create**:
- `gui/citrate-core/src-tauri/src/agent/intent.rs`
- `gui/citrate-core/src-tauri/src/agent/classifier.rs`

**Dependencies**: None

---

### WP-1.3: ToolDispatcher with MCP Bindings
**Points**: 13 | **Priority**: P0 | **Status**: [ ] Not Started

**Description**:
Create the tool execution framework that routes classified intents to appropriate tools and returns structured results.

**Tasks**:
- [ ] Define `Tool` trait with:
  - `name()` - Tool identifier
  - `description()` - For LLM context
  - `parameters()` - JSON schema
  - `execute()` - Async execution
- [ ] Create `ToolRegistry`:
  - Register tools by name
  - Get tool by intent mapping
  - List all available tools
- [ ] Implement core tools:
  - `ChainQueryTool` - Balance, block, transaction queries
  - `SendTransactionTool` - Transaction creation and signing
  - `ContractTool` - Deploy and call contracts
  - `InferenceTool` - Run model inference
  - `WalletTool` - Account management
  - `DAGTool` - DAG visualization queries
- [ ] Create `ToolDispatcher`:
  - Route intents to tools
  - Handle tool execution errors
  - Format results for display
- [ ] Add MCP-compatible JSON schemas for each tool

**Acceptance Criteria**:
- [ ] All core tools registered and callable
- [ ] Intent-to-tool routing works correctly
- [ ] Tool results formatted for chat display
- [ ] Errors handled gracefully with user-friendly messages

**Files to Create**:
- `gui/citrate-core/src-tauri/src/agent/tools/mod.rs`
- `gui/citrate-core/src-tauri/src/agent/tools/chain.rs`
- `gui/citrate-core/src-tauri/src/agent/tools/wallet.rs`
- `gui/citrate-core/src-tauri/src/agent/tools/contract.rs`
- `gui/citrate-core/src-tauri/src/agent/tools/inference.rs`
- `gui/citrate-core/src-tauri/src/agent/tools/dag.rs`
- `gui/citrate-core/src-tauri/src/agent/dispatcher.rs`

**Dependencies**: WP-1.1, WP-1.2

---

### WP-1.4: Streaming Response Infrastructure
**Points**: 5 | **Priority**: P0 | **Status**: [ ] Not Started

**Description**:
Implement token-by-token streaming from the agent to the frontend using Tauri events.

**Tasks**:
- [ ] Create `StreamingResponse` type with:
  - `session_id` - Conversation identifier
  - `message_id` - Unique message ID
  - `content` - Token content
  - `is_complete` - Final token flag
  - `metadata` - Usage stats, tool calls, etc.
- [ ] Implement `ResponseStreamer`:
  - Buffer management for partial tokens
  - Rate limiting to prevent UI flooding
  - Backpressure handling
- [ ] Add Tauri event emission:
  - `agent-token` - Individual token events
  - `agent-complete` - Message completion
  - `agent-error` - Error events
  - `agent-tool-call` - Tool invocation notifications
- [ ] Create `StreamChannel` for managing active streams

**Acceptance Criteria**:
- [ ] Tokens stream to frontend in real-time
- [ ] UI updates smoothly without lag
- [ ] Stream cancellation works
- [ ] Metadata included with final message

**Files to Create**:
- `gui/citrate-core/src-tauri/src/agent/streaming.rs`

**Dependencies**: WP-1.1

---

### WP-1.5: GGUF Engine Integration (Local LLM)
**Points**: 5 | **Priority**: P1 | **Status**: [ ] Not Started

**Description**:
Integrate local GGUF model inference using llama.cpp bindings or candle.

**Tasks**:
- [ ] Evaluate and choose Rust GGUF library:
  - `llama-cpp-rs` (llama.cpp bindings)
  - `candle` (pure Rust, supports GGUF)
  - Performance and compatibility comparison
- [ ] Create `LocalLLMEngine`:
  - Model loading from disk
  - Inference with configurable parameters
  - Memory management for large models
  - Token streaming callback
- [ ] Implement model management:
  - Model discovery in data directory
  - Model metadata (size, quantization, context length)
  - Lazy loading and unloading
- [ ] Add configuration:
  - Default model selection
  - GPU acceleration (Metal/CUDA)
  - Context size limits
  - Temperature, top_p, top_k

**Acceptance Criteria**:
- [ ] Can load and run GGUF models locally
- [ ] Inference works without internet
- [ ] Streaming tokens to frontend
- [ ] Memory usage reasonable for desktop

**Files to Create**:
- `gui/citrate-core/src-tauri/src/agent/llm/mod.rs`
- `gui/citrate-core/src-tauri/src/agent/llm/local.rs`
- `gui/citrate-core/src-tauri/src/agent/llm/gguf.rs`

**Dependencies**: WP-1.4

**Notes**:
- Consider starting with smaller models (Phi-2, TinyLlama) for testing
- Metal acceleration important for macOS performance
- May need to ship model or provide download UI

---

### WP-1.6: Tauri Agent Commands
**Points**: 3 | **Priority**: P0 | **Status**: [ ] Not Started

**Description**:
Create Tauri commands that expose the agent functionality to the React frontend.

**Tasks**:
- [ ] Implement core agent commands:
  - `agent_send_message(session_id, message)` - Send user message
  - `agent_create_session()` - Create new conversation
  - `agent_get_history(session_id)` - Get conversation history
  - `agent_clear_session(session_id)` - Clear conversation
  - `agent_cancel_response(session_id)` - Cancel streaming
- [ ] Implement configuration commands:
  - `agent_get_config()` - Get agent settings
  - `agent_update_config(config)` - Update settings
  - `agent_get_available_models()` - List LLM options
  - `agent_set_model(model_id)` - Set active model
- [ ] Add tool-related commands:
  - `agent_get_tools()` - List available tools
  - `agent_approve_tool_call(call_id)` - Approve pending tool
  - `agent_reject_tool_call(call_id)` - Reject pending tool
- [ ] Wire commands into lib.rs

**Acceptance Criteria**:
- [ ] All commands work from frontend
- [ ] Proper error handling and types
- [ ] State properly managed across calls
- [ ] Events emitted for async operations

**Files to Modify**:
- `gui/citrate-core/src-tauri/src/lib.rs`

**Dependencies**: WP-1.1, WP-1.3, WP-1.4

---

### WP-1.7: Context Management
**Points**: 5 | **Priority**: P1 | **Status**: [ ] Not Started

**Description**:
Implement conversation history and context window management for maintaining coherent multi-turn conversations.

**Tasks**:
- [ ] Create `ConversationHistory`:
  - Message storage (user, assistant, system, tool)
  - Timestamp and metadata
  - Serialization for persistence
- [ ] Implement `ContextWindow`:
  - Token counting for messages
  - Sliding window management
  - System prompt injection
  - Tool result summarization
- [ ] Add context strategies:
  - `TruncateOldest` - Remove old messages first
  - `Summarize` - Summarize old context
  - `Hybrid` - Keep recent + summary of older
- [ ] Implement persistence:
  - Save conversations to disk
  - Load previous conversations
  - Export/import functionality

**Acceptance Criteria**:
- [ ] Conversations maintain context across turns
- [ ] Context window respected for model limits
- [ ] Previous conversations restorable
- [ ] System prompts properly injected

**Files to Create**:
- `gui/citrate-core/src-tauri/src/agent/context.rs`
- `gui/citrate-core/src-tauri/src/agent/history.rs`

**Dependencies**: WP-1.1

---

## Architecture Overview

```
┌─────────────────────────────────────────────────────────────────┐
│                         React Frontend                          │
│  ┌─────────────────────────────────────────────────────────┐   │
│  │                     ChatInterface                         │   │
│  │   - Message display    - Input handling                   │   │
│  │   - Stream rendering   - Tool approval                    │   │
│  └─────────────────────────────────────────────────────────┘   │
│                              │                                  │
│                              ▼                                  │
│  ┌─────────────────────────────────────────────────────────┐   │
│  │                   Tauri Commands                          │   │
│  │   agent_send_message, agent_create_session, etc.          │   │
│  └─────────────────────────────────────────────────────────┘   │
└────────────────────────────────┼────────────────────────────────┘
                                 │
                                 ▼
┌─────────────────────────────────────────────────────────────────┐
│                       Rust Agent Layer                          │
│                                                                 │
│  ┌──────────────┐    ┌────────────────┐    ┌──────────────┐   │
│  │   Orchestr-  │    │    Intent      │    │    Tool      │   │
│  │   ator       │───▶│  Classifier    │───▶│  Dispatcher  │   │
│  └──────────────┘    └────────────────┘    └──────────────┘   │
│         │                    │                     │           │
│         ▼                    ▼                     ▼           │
│  ┌──────────────┐    ┌────────────────┐    ┌──────────────┐   │
│  │   Context    │    │   LLM Engine   │    │    Tools     │   │
│  │   Manager    │    │  (API + Local) │    │  (MCP-based) │   │
│  └──────────────┘    └────────────────┘    └──────────────┘   │
│                              │                     │           │
│                              ▼                     ▼           │
│  ┌─────────────────────────────────────────────────────────┐   │
│  │                  Streaming Response                       │   │
│  │              (Tauri Events → Frontend)                    │   │
│  └─────────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────────┘
```

---

## API Contract

### LLM Backend Interface

```rust
#[async_trait]
pub trait LLMBackend: Send + Sync {
    /// Generate a completion with streaming
    async fn complete(
        &self,
        messages: Vec<Message>,
        config: CompletionConfig,
        callback: Box<dyn Fn(StreamToken) + Send>,
    ) -> Result<CompletionResult, LLMError>;

    /// Get model information
    fn model_info(&self) -> ModelInfo;

    /// Check if model supports function calling
    fn supports_tools(&self) -> bool;
}
```

### Tool Interface

```rust
#[async_trait]
pub trait Tool: Send + Sync {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn parameters(&self) -> serde_json::Value; // JSON Schema

    async fn execute(
        &self,
        params: serde_json::Value,
        context: &ToolContext,
    ) -> Result<ToolResult, ToolError>;

    fn requires_approval(&self) -> bool;
}
```

---

## Success Criteria

- [ ] Agent responds to "What's my balance?" with actual balance
- [ ] Agent can send transactions (with approval flow)
- [ ] Intent classification >90% accuracy on test queries
- [ ] Streaming responses visible in UI
- [ ] Local LLM inference functional (if model available)
- [ ] Conversations persist across app restarts
- [ ] All 7 work packages completed

---

## Risk Mitigation

| Risk | Mitigation |
|------|------------|
| GGUF library compatibility | Start with API-only, add local as P1 |
| Tool execution complexity | Start with chain query, add others incrementally |
| Context window overflow | Implement truncation first, summarization later |
| Streaming performance | Rate limit tokens, batch UI updates |

---

## Dependencies

- Sprint 0 complete (consensus, execution working)
- Existing Tauri app structure
- React frontend with chat UI (ChatBot.tsx)
- Wallet and node managers

---

*Last Updated: 2025-12-02*
