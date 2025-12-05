# Sprint 3: Contract Tooling & IPFS Integration

## Sprint Info
- **Duration**: Week 3
- **Sprint Goal**: Implement real contract compilation and IPFS integration
- **Phase**: Audit Fixes - Tooling
- **Depends On**: Sprint 1 (Core), Sprint 2 (RPC working)

---

## Sprint Objectives

1. Implement real Solidity compiler (solc-js WASM + Foundry CLI)
2. Replace mock IPFS with real pinning
3. Fix marketplace mock data and contract integration
4. Implement proper codegen → deploy → interact flow

---

## Work Breakdown Structure (WBS)

### WP-3.1: solc-js WASM Integration
**Points**: 8 | **Priority**: P0 | **Status**: [ ] Not Started

**Description**:
Contract compiler is placeholder/mock. Integrate solc-js as WASM for browser-based compilation (works offline).

**Tasks**:
- [ ] Add solc-js dependency (via CDN or bundled WASM)
- [ ] Create compilation worker for non-blocking compile
- [ ] Implement version selection (0.8.x range)
- [ ] Handle optimizer settings
- [ ] Parse ABI and bytecode from output
- [ ] Handle compilation errors with line numbers
- [ ] Add progress indication for large contracts
- [ ] Cache compiled artifacts by hash

**Acceptance Criteria**:
- [ ] Simple contracts compile successfully
- [ ] Complex contracts with imports compile (remappings)
- [ ] Compilation errors show file and line
- [ ] Compiled output matches forge output

**Files to Modify**:
- `gui/citrate-core/src/utils/contractCompiler.ts`
- `gui/citrate-core/package.json` (add solc dependency)

**Dependencies**: None (can start immediately)

---

### WP-3.2: Foundry CLI Integration
**Points**: 5 | **Priority**: P1 | **Status**: [ ] Not Started

**Description**:
Per user request: support both solc-js and Foundry. Shell out to forge for users who have it installed.

**Tasks**:
- [ ] Detect if forge is installed (which forge)
- [ ] Create Tauri command to invoke forge build
- [ ] Parse forge output JSON
- [ ] Handle foundry.toml configuration
- [ ] Support forge remappings
- [ ] Add setting to choose compiler (solc-js vs Foundry)
- [ ] Handle forge not found gracefully

**Acceptance Criteria**:
- [ ] forge build works via Tauri
- [ ] User can choose compiler in settings
- [ ] Falls back to solc-js if forge unavailable

**Files to Modify**:
- `gui/citrate-core/src-tauri/src/lib.rs` (new command)
- `gui/citrate-core/src/utils/contractCompiler.ts`
- `gui/citrate-core/src/components/Settings.tsx`

**Dependencies**: WP-3.1 (needs solc-js as fallback)

---

### WP-3.3: Real IPFS Pinning (Agent Tools)
**Points**: 5 | **Priority**: P0 | **Status**: [ ] Not Started

**Description**:
Agent IPFS upload returns hash-based mock CID. Need real IPFS integration.

**Tasks**:
- [ ] Add IPFS HTTP client (kubo/js-ipfs-http-client)
- [ ] Implement add/pin operations
- [ ] Add IPFS node connectivity check
- [ ] Return real CID from uploads
- [ ] Handle IPFS offline: fail with error, not mock
- [ ] Add upload progress for large files
- [ ] Support remote IPFS gateway (Infura, Pinata)

**Acceptance Criteria**:
- [ ] Files are actually pinned to IPFS
- [ ] CID is valid and content retrievable
- [ ] Offline mode fails with clear error
- [ ] Gateway option works for users without local node

**Files to Modify**:
- `gui/citrate-core/src-tauri/src/agent/tools/storage.rs:148-174`
- `gui/citrate-core/src-tauri/src/ipfs/mod.rs`
- `gui/citrate-core/src-tauri/Cargo.toml` (add ipfs-api)

**Dependencies**: None

---

### WP-3.4: IPFS React Component Fix
**Points**: 3 | **Priority**: P1 | **Status**: [ ] Not Started

**Description**:
React IPFS component is fully mocked (lines 80-227). Need real integration.

**Tasks**:
- [ ] Replace mock file listing with real IPFS ls
- [ ] Implement real file upload via backend
- [ ] Implement real file download/view
- [ ] Add pin/unpin functionality
- [ ] Show actual IPFS node status
- [ ] Handle connection errors gracefully

**Acceptance Criteria**:
- [ ] Component shows real IPFS content
- [ ] Upload creates real pins
- [ ] Downloads work for pinned content

**Files to Modify**:
- `gui/citrate-core/src/components/IPFS.tsx:80-227`
- `gui/citrate-core/src/services/tauri.ts` (IPFS commands)

**Dependencies**: WP-3.3

---

### WP-3.5: Metadata Uploader Fix
**Points**: 2 | **Priority**: P2 | **Status**: [ ] Not Started

**Description**:
IPFS metadata uploader has TODO at line 288 for update functionality.

**Tasks**:
- [ ] Implement metadata update (re-pin with new data)
- [ ] Add version tracking for metadata
- [ ] Handle metadata validation
- [ ] Return new CID on update

**Acceptance Criteria**:
- [ ] Metadata can be updated
- [ ] Previous versions tracked
- [ ] Invalid metadata rejected

**Files to Modify**:
- `gui/citrate-core/src/utils/metadata/ipfsUploader.ts:288`

**Dependencies**: WP-3.3

---

### WP-3.6: Marketplace Contract Integration
**Points**: 5 | **Priority**: P1 | **Status**: [ ] Not Started

**Description**:
Marketplace falls back to mock data when contracts absent. Need real contract integration.

**Tasks**:
- [ ] Deploy marketplace contracts (if not deployed)
- [ ] Implement real contract calls for model listing
- [ ] Implement real contract calls for model purchase
- [ ] Remove mock data fallback
- [ ] Add contract deployment check on startup
- [ ] Cache contract addresses in config

**Acceptance Criteria**:
- [ ] Models listed from contract state
- [ ] Purchases call real contract
- [ ] No mock data in marketplace

**Files to Modify**:
- `gui/citrate-core/src/components/Marketplace.tsx`
- `gui/citrate-core/src/utils/marketplaceHelpers.ts:99,124,132,170`

**Dependencies**: Sprint 2 WP-2.3 (RPC working)

---

### WP-3.7: Search Index Fix
**Points**: 2 | **Priority**: P2 | **Status**: [ ] Not Started

**Description**:
Search index has totalInferences stub at line 252.

**Tasks**:
- [ ] Implement real inference count tracking
- [ ] Query on-chain events for inference count
- [ ] Add caching for performance
- [ ] Update index incrementally

**Acceptance Criteria**:
- [ ] Search shows real inference counts
- [ ] Counts update as inference events occur

**Files to Modify**:
- `gui/citrate-core/src/utils/search/searchIndexBuilder.ts:252`

**Dependencies**: WP-3.6

---

### WP-3.8: MCP Connection Testing
**Points**: 3 | **Priority**: P1 | **Status**: [ ] Not Started

**Description**:
MCP connectivity needs proper testing and validation.

**Tasks**:
- [ ] Add MCP health check endpoint
- [ ] Implement connection test on startup
- [ ] Add reconnection logic
- [ ] Show MCP status in GUI
- [ ] Add MCP error codes and messages
- [ ] Write integration tests for MCP

**Acceptance Criteria**:
- [ ] MCP connection validated before use
- [ ] Connection failures shown to user
- [ ] Reconnection works after network issues

**Files to Modify**:
- `gui/citrate-core/src-tauri/src/agent/mcp/mod.rs`
- `gui/citrate-core/src/components/ChatBot.tsx`

**Dependencies**: Sprint 2 WP-2.5

---

## Sprint Backlog Summary

| WP | Title | Points | Priority | Status |
|----|-------|--------|----------|--------|
| WP-3.1 | solc-js WASM Integration | 8 | P0 | [ ] |
| WP-3.2 | Foundry CLI Integration | 5 | P1 | [ ] |
| WP-3.3 | Real IPFS Pinning | 5 | P0 | [ ] |
| WP-3.4 | IPFS React Component Fix | 3 | P1 | [ ] |
| WP-3.5 | Metadata Uploader Fix | 2 | P2 | [ ] |
| WP-3.6 | Marketplace Contract Integration | 5 | P1 | [ ] |
| WP-3.7 | Search Index Fix | 2 | P2 | [ ] |
| WP-3.8 | MCP Connection Testing | 3 | P1 | [ ] |

**Total Points**: 33
**Committed Points**: 29 (excluding P2)
**Buffer**: 4 points (P2 items)

---

## Definition of Done

- [ ] Contracts compile via solc-js or Foundry
- [ ] IPFS operations use real pinning
- [ ] Marketplace reads from contracts
- [ ] MCP connection validated
- [ ] No mock data in production paths

---

## Risks & Blockers

| Risk | Impact | Mitigation |
|------|--------|------------|
| solc-js WASM size (~40MB) | Med | Lazy-load, offer Foundry-only mode |
| IPFS gateway rate limits | Med | Support multiple gateways |
| Marketplace contracts not deployed | High | Include deploy script in sprint |

---

## Notes

- WP-3.1 and WP-3.3 can be worked in parallel
- Foundry integration (WP-3.2) is bonus if time permits
- Marketplace work depends on RPC being fixed in Sprint 2

---

*Created: 2025-12-04*
*Last Updated: 2025-12-04*
