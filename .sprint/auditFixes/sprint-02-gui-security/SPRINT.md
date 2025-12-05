# Sprint 2: GUI Security & Mock Removal

## Sprint Info
- **Duration**: Week 2
- **Sprint Goal**: Remove all security-critical mocks and fix GUI/Tauri integration issues
- **Phase**: Audit Fixes - Security Hardening
- **Depends On**: Sprint 1 (Core Infrastructure)

---

## Sprint Objectives

1. Remove mock signature fallback in tauri.ts (HARD FAIL only)
2. Fix RPC mempool type mismatch so embedded RPC server starts
3. Fix agent tool fallbacks (image gen, LLM, MCP)
4. Implement proper signature verification
5. Add receipt polling and gas estimation

---

## Work Breakdown Structure (WBS)

### WP-2.1: Remove Mock Signature Fallback
**Points**: 5 | **Priority**: P0 | **Status**: [ ] Not Started

**Description**:
`gui/citrate-core/src/services/tauri.ts` falls back to random mock signature on failure. Per user decision: hard-fail only, no mock fallback ever.

**Tasks**:
- [ ] Remove mock signature generation code entirely
- [ ] Update signTransaction to throw on any error
- [ ] Update signMessage to throw on any error
- [ ] Add proper error types for signing failures
- [ ] Update all callers to handle signing errors
- [ ] Add user-facing error messages for signing failures
- [ ] Write tests for signing failure scenarios

**Acceptance Criteria**:
- [ ] No mock signature code exists in codebase
- [ ] Signing failure results in error, never fake signature
- [ ] User sees clear error message when signing fails
- [ ] Tests verify no mock path exists

**Files to Modify**:
- `gui/citrate-core/src/services/tauri.ts`
- `gui/citrate-core/src/hooks/useWallet.ts`
- `gui/citrate-core/src/components/TransactionModal.tsx` (if exists)

**Dependencies**: None

---

### WP-2.2: Fix Signature Verification
**Points**: 3 | **Priority**: P0 | **Status**: [ ] Not Started

**Description**:
Signature verification is currently simplified (length-only check). Need proper cryptographic verification.

**Tasks**:
- [ ] Implement ed25519 signature verification
- [ ] Implement ECDSA/secp256k1 verification (for EVM compatibility)
- [ ] Add signature type detection
- [ ] Verify recovery ID handling for EVM signatures
- [ ] Add verification to transaction receipt
- [ ] Write comprehensive signature verification tests

**Acceptance Criteria**:
- [ ] Both ed25519 and ECDSA signatures properly verified
- [ ] Invalid signatures rejected with clear error
- [ ] Recovery ID (v) correctly handled

**Files to Modify**:
- `gui/citrate-core/src-tauri/src/wallet/mod.rs`
- `core/execution/src/signature.rs` (if exists)

**Dependencies**: WP-2.1

---

### WP-2.3: Fix RPC Mempool Type Mismatch
**Points**: 5 | **Priority**: P0 | **Status**: [ ] Not Started

**Description**:
RPC server is commented out due to mempool Arc type mismatch. Need to resolve type issues so embedded RPC server can start.

**Tasks**:
- [ ] Identify exact type mismatch between node and RPC mempool
- [ ] Create shared mempool interface/trait
- [ ] Update RPC server to use correct mempool type
- [ ] Update node to expose mempool with correct type
- [ ] Uncomment RPC server initialization
- [ ] Test RPC server starts and responds
- [ ] Verify eth_sendRawTransaction works

**Acceptance Criteria**:
- [ ] RPC server starts without errors
- [ ] eth_sendRawTransaction accepts transactions
- [ ] eth_getTransactionReceipt returns data
- [ ] Mempool contents visible via RPC

**Files to Modify**:
- `gui/citrate-core/src-tauri/src/node/mod.rs`
- `gui/citrate-core/src-tauri/Cargo.toml`
- `core/api/src/eth_rpc.rs`

**Dependencies**: Sprint 1 complete

---

### WP-2.4: Image Generation Tool Fix
**Points**: 3 | **Priority**: P1 | **Status**: [ ] Not Started

**Description**:
Agent image generation returns mock payload on failure (lines 145-171). Should fail with error, not pretend success.

**Tasks**:
- [ ] Remove mock image generation fallback
- [ ] Add proper error handling for image gen failures
- [ ] Add connectivity check before attempting generation
- [ ] Return clear error message to user
- [ ] Add retry logic with backoff for transient failures
- [ ] Log image generation attempts for debugging

**Acceptance Criteria**:
- [ ] Image gen failure returns error, not mock
- [ ] User sees clear message when image gen unavailable
- [ ] Transient failures are retried

**Files to Modify**:
- `gui/citrate-core/src-tauri/src/agent/tools/generation.rs:145-171`

**Dependencies**: None

---

### WP-2.5: LLM/MCP Mock Removal
**Points**: 5 | **Priority**: P1 | **Status**: [ ] Not Started

**Description**:
LLM and MCP connections fall back to mock (llm/mod.rs:140-177,207, ChatBot.tsx:110-113). Need real connection or fail.

**Tasks**:
- [ ] Remove mock model fallback in llm/mod.rs
- [ ] Add MCP connection validation in ChatBot.tsx
- [ ] Implement health check for LLM providers
- [ ] Add provider selection UI with status indicators
- [ ] Handle offline mode gracefully (show status, don't fake)
- [ ] Add connection retry with exponential backoff
- [ ] Add speech recognition error handling (TODOs at 241,244)

**Acceptance Criteria**:
- [ ] No mock LLM responses in production
- [ ] MCP connection validated before use
- [ ] Offline status clearly shown to user
- [ ] Speech recognition errors handled

**Files to Modify**:
- `gui/citrate-core/src/agent/llm/mod.rs:140-177,207`
- `gui/citrate-core/src/components/ChatBot.tsx:110-113,241,244`

**Dependencies**: None

---

### WP-2.6: Receipt Polling Implementation
**Points**: 3 | **Priority**: P1 | **Status**: [ ] Not Started

**Description**:
Contract deployment lacks receipt polling. Need to poll for transaction confirmation.

**Tasks**:
- [ ] Implement receipt polling with configurable timeout
- [ ] Add polling interval configuration
- [ ] Handle dropped transactions (not in mempool, no receipt)
- [ ] Add confirmation count tracking
- [ ] Update deployment helpers to use polling
- [ ] Add progress indicator for pending transactions

**Acceptance Criteria**:
- [ ] Transactions are tracked until confirmed or timeout
- [ ] Dropped transactions detected and reported
- [ ] User sees confirmation status

**Files to Modify**:
- `gui/citrate-core/src/utils/contractDeployment.ts:113-128,147`
- `gui/citrate-core/src/hooks/useTransaction.ts` (if exists)

**Dependencies**: WP-2.3 (RPC must work)

---

### WP-2.7: Gas Estimation Implementation
**Points**: 3 | **Priority**: P1 | **Status**: [ ] Not Started

**Description**:
Gas estimation is missing in deployment helpers. Need to call eth_estimateGas.

**Tasks**:
- [ ] Implement eth_estimateGas call in tauri backend
- [ ] Add gas estimation to deployment flow
- [ ] Add gas buffer configuration (e.g., +20%)
- [ ] Handle estimation failures gracefully
- [ ] Show estimated gas cost to user before send
- [ ] Cache gas estimates for similar transactions

**Acceptance Criteria**:
- [ ] Gas is estimated before transaction send
- [ ] User sees estimated cost
- [ ] Estimation failures handled with fallback limit

**Files to Modify**:
- `gui/citrate-core/src-tauri/src/lib.rs`
- `gui/citrate-core/src/utils/contractDeployment.ts:172`
- `gui/citrate-core/src/services/tauri.ts`

**Dependencies**: WP-2.3

---

### WP-2.8: Sync Stub Removal
**Points**: 3 | **Priority**: P1 | **Status**: [ ] Not Started

**Description**:
GUI sync is stubbed, preventing block catch-up. Need to wire real sync from Sprint 1.

**Tasks**:
- [ ] Connect GUI sync to core efficient_sync implementation
- [ ] Add sync progress indicator in GUI
- [ ] Handle sync errors and retries
- [ ] Show current block height vs network height
- [ ] Add "syncing" status to node status display

**Acceptance Criteria**:
- [ ] GUI node syncs with network
- [ ] Sync progress visible to user
- [ ] Sync errors shown clearly

**Files to Modify**:
- `gui/citrate-core/src-tauri/src/sync/mod.rs`
- `gui/citrate-core/src/components/NodeStatus.tsx` (if exists)

**Dependencies**: Sprint 1 WP-1.2 (efficient sync)

---

## Sprint Backlog Summary

| WP | Title | Points | Priority | Status |
|----|-------|--------|----------|--------|
| WP-2.1 | Remove Mock Signature Fallback | 5 | P0 | [ ] |
| WP-2.2 | Fix Signature Verification | 3 | P0 | [ ] |
| WP-2.3 | Fix RPC Mempool Type Mismatch | 5 | P0 | [ ] |
| WP-2.4 | Image Generation Tool Fix | 3 | P1 | [ ] |
| WP-2.5 | LLM/MCP Mock Removal | 5 | P1 | [ ] |
| WP-2.6 | Receipt Polling Implementation | 3 | P1 | [ ] |
| WP-2.7 | Gas Estimation Implementation | 3 | P1 | [ ] |
| WP-2.8 | Sync Stub Removal | 3 | P1 | [ ] |

**Total Points**: 30
**Committed Points**: 30 (all items critical for testnet)
**Buffer**: 0 points (may need to carry items to Sprint 3)

---

## Definition of Done

- [ ] No mock signatures anywhere in codebase
- [ ] RPC server starts and handles transactions
- [ ] All agent tools fail with errors, not mocks
- [ ] Gas estimation works for all transactions
- [ ] Receipt polling tracks confirmations
- [ ] GUI syncs with network

---

## Risks & Blockers

| Risk | Impact | Mitigation |
|------|--------|------------|
| Mempool type mismatch complex | High | Review core mempool design first |
| LLM providers offline | Med | Add local model fallback option |
| Sprint 1 incomplete | High | Can work on WP-2.1, 2.4, 2.5 independently |

---

## Notes

- WP-2.1 and WP-2.2 are security critical and should be done first
- WP-2.3 unblocks many other GUI features
- Per user request: hard-fail only for signatures, no dev flag option

---

*Created: 2025-12-04*
*Last Updated: 2025-12-04*
