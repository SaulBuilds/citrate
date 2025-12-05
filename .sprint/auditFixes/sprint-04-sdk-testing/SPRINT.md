# Sprint 4: SDK Integration & Testing

## Sprint Info
- **Duration**: Weeks 4-5 (larger sprint for comprehensive testing)
- **Sprint Goal**: Comprehensive SDK testing against embedded node and testnet
- **Phase**: Audit Fixes - SDK Validation
- **Depends On**: Sprints 1-3 (Core, RPC, Contract tooling)

---

## Sprint Objectives

1. Add integration tests for all SDKs against embedded node
2. Add integration tests against testnet endpoint
3. Verify dual address derivation across all components
4. Test EIP-1559 and EIP-2930 transaction types
5. Validate dist bundles match current API

---

## Work Breakdown Structure (WBS)

### WP-4.1: JavaScript SDK Embedded Node Tests
**Points**: 8 | **Priority**: P0 | **Status**: [ ] Not Started

**Description**:
JS SDK (`sdk/javascript`) lacks integration tests against live RPC. Need to spin up embedded node and test all methods.

**Tasks**:
- [ ] Create test harness that starts embedded node
- [ ] Test eth_chainId returns configured value
- [ ] Test eth_blockNumber returns current height
- [ ] Test eth_getBalance for accounts
- [ ] Test eth_call for contract reads
- [ ] Test eth_sendRawTransaction end-to-end
- [ ] Test eth_getTransactionReceipt polling
- [ ] Test eth_feeHistory returns real data
- [ ] Test eth_estimateGas accuracy
- [ ] Test transaction with EIP-1559 fields
- [ ] Test transaction with EIP-2930 access list
- [ ] Add CI workflow for SDK tests

**Acceptance Criteria**:
- [ ] All eth_* methods tested against embedded node
- [ ] Tests run in CI on every PR
- [ ] >80% code coverage for SDK

**Files to Modify**:
- `sdk/javascript/tests/integration/` (new)
- `sdk/javascript/package.json` (test scripts)
- `.github/workflows/sdk-tests.yml` (new)

**Dependencies**: Sprints 1-3 complete

---

### WP-4.2: JavaScript SDK Testnet Tests
**Points**: 5 | **Priority**: P1 | **Status**: [ ] Not Started

**Description**:
Per user request: test against both embedded and testnet. Add E2E tests against running testnet.

**Tasks**:
- [ ] Create testnet configuration
- [ ] Add env-based endpoint selection
- [ ] Test transaction lifecycle on testnet
- [ ] Test block finality confirmation
- [ ] Test gas price fluctuations
- [ ] Add testnet faucet integration for test accounts
- [ ] Document testnet test requirements

**Acceptance Criteria**:
- [ ] SDK works against real testnet
- [ ] Tests can run with testnet URL env var
- [ ] Faucet provides test tokens

**Files to Modify**:
- `sdk/javascript/tests/e2e/` (new)
- `sdk/javascript/tests/config.ts`

**Dependencies**: WP-4.1, testnet running

---

### WP-4.3: Address Derivation Tests
**Points**: 5 | **Priority**: P0 | **Status**: [ ] Not Started

**Description**:
Dual address handling (EVM 20-byte vs native 32-byte) needs comprehensive testing across all components.

**Tasks**:
- [ ] Test SDK address derivation from private key
- [ ] Test wallet address derivation matches SDK
- [ ] Test RPC accepts both address formats
- [ ] Test executor handles both formats consistently
- [ ] Test GUI wallet address matches expected
- [ ] Add collision resistance tests
- [ ] Document address format specification

**Acceptance Criteria**:
- [ ] Same private key produces same address in all components
- [ ] Both 20-byte and 32-byte formats work everywhere
- [ ] No address collisions in test suite

**Files to Modify**:
- `sdk/javascript/src/address.ts`
- `sdk/javascript/tests/address.test.ts`
- `core/execution/src/types.rs` (document)

**Dependencies**: None

---

### WP-4.4: EIP-1559/2930 Transaction Tests
**Points**: 5 | **Priority**: P0 | **Status**: [ ] Not Started

**Description**:
Transaction types (legacy, EIP-2930, EIP-1559) need comprehensive testing.

**Tasks**:
- [ ] Test legacy transaction (type 0) end-to-end
- [ ] Test EIP-2930 access list transaction (type 1)
- [ ] Test EIP-1559 dynamic fee transaction (type 2)
- [ ] Verify RLP encoding matches expected
- [ ] Test gas pricing for each type
- [ ] Test signature recovery for each type
- [ ] Add negative tests for malformed transactions

**Acceptance Criteria**:
- [ ] All three transaction types work
- [ ] RLP encoding validated against ethers.js
- [ ] Signature verification passes for all types

**Files to Modify**:
- `sdk/javascript/tests/transactions.test.ts`
- `core/api/src/eth_tx_decoder.rs` (verify)

**Dependencies**: WP-4.1

---

### WP-4.5: Python SDK Integration Tests
**Points**: 5 | **Priority**: P1 | **Status**: [ ] Not Started

**Description**:
Python SDK (`sdks/python`) needs RPC integration tests similar to JS SDK.

**Tasks**:
- [ ] Create pytest fixtures for embedded node
- [ ] Test eth_call, sendRawTx, feeHistory
- [ ] Test address derivation matches JS SDK
- [ ] Test transaction signing
- [ ] Add CI workflow for Python tests
- [ ] Ensure examples work with embedded node

**Acceptance Criteria**:
- [ ] Python SDK tests pass against embedded node
- [ ] Address derivation matches JS SDK
- [ ] CI runs Python tests

**Files to Modify**:
- `sdks/python/tests/integration/` (new)
- `sdks/python/pytest.ini`
- `.github/workflows/python-sdk-tests.yml` (new)

**Dependencies**: WP-4.1

---

### WP-4.6: citrate-js Alternative SDK Tests
**Points**: 3 | **Priority**: P2 | **Status**: [ ] Not Started

**Description**:
Alternative JS SDK (`sdks/javascript/citrate-js`) needs similar integration tests.

**Tasks**:
- [ ] Mirror test suite from official SDK
- [ ] Verify API compatibility with official SDK
- [ ] Add deprecation notice if needed
- [ ] Document differences from official SDK

**Acceptance Criteria**:
- [ ] citrate-js passes same integration tests
- [ ] API surface documented

**Files to Modify**:
- `sdks/javascript/citrate-js/tests/`

**Dependencies**: WP-4.1

---

### WP-4.7: SDK Dist Bundle Validation
**Points**: 3 | **Priority**: P1 | **Status**: [ ] Not Started

**Description**:
Dist bundles need validation against current API surface.

**Tasks**:
- [ ] Build dist bundles fresh
- [ ] Compare against committed bundles
- [ ] Verify exports match TypeScript definitions
- [ ] Test bundle in browser context
- [ ] Test bundle in Node context
- [ ] Add bundle size tracking

**Acceptance Criteria**:
- [ ] Dist bundles up-to-date with source
- [ ] Bundles work in browser and Node
- [ ] Bundle size reasonable (<500KB gzipped)

**Files to Modify**:
- `sdk/javascript/rollup.config.js` (or equivalent)
- `sdk/javascript/package.json`

**Dependencies**: None

---

### WP-4.8: Chain ID Consistency Tests
**Points**: 2 | **Priority**: P1 | **Status**: [ ] Not Started

**Description**:
Verify chain ID is consistent across all components and matches network.

**Tasks**:
- [ ] Test SDK chain ID matches node config
- [ ] Test wallet uses correct chain ID for signing
- [ ] Test RPC returns configured chain ID
- [ ] Test transactions with wrong chain ID are rejected
- [ ] Document chain ID configuration

**Acceptance Criteria**:
- [ ] Chain ID consistent everywhere
- [ ] Wrong chain ID transactions rejected

**Files to Modify**:
- `sdk/javascript/tests/chainId.test.ts`
- Verification across components

**Dependencies**: Sprint 1 WP-1.1

---

### WP-4.9: CLI Wallet Verification
**Points**: 3 | **Priority**: P1 | **Status**: [ ] Not Started

**Description**:
CLI wallet needs verification for chain ID, address derivation, and signing paths.

**Tasks**:
- [ ] Test CLI wallet address derivation
- [ ] Test ed25519 signing path
- [ ] Test ECDSA/secp256k1 signing path
- [ ] Verify chain ID used in signatures
- [ ] Add integration test suite for CLI
- [ ] Document CLI usage

**Acceptance Criteria**:
- [ ] CLI wallet works with both signature schemes
- [ ] Chain ID configurable via flag or env
- [ ] Address matches SDK/GUI derivation

**Files to Modify**:
- `wallet/tests/` (integration tests)
- `wallet/src/lib.rs`

**Dependencies**: Sprint 1 WP-1.1

---

## Sprint Backlog Summary

| WP | Title | Points | Priority | Status |
|----|-------|--------|----------|--------|
| WP-4.1 | JS SDK Embedded Node Tests | 8 | P0 | [ ] |
| WP-4.2 | JS SDK Testnet Tests | 5 | P1 | [ ] |
| WP-4.3 | Address Derivation Tests | 5 | P0 | [ ] |
| WP-4.4 | EIP-1559/2930 Transaction Tests | 5 | P0 | [ ] |
| WP-4.5 | Python SDK Integration Tests | 5 | P1 | [ ] |
| WP-4.6 | citrate-js Alternative SDK Tests | 3 | P2 | [ ] |
| WP-4.7 | SDK Dist Bundle Validation | 3 | P1 | [ ] |
| WP-4.8 | Chain ID Consistency Tests | 2 | P1 | [ ] |
| WP-4.9 | CLI Wallet Verification | 3 | P1 | [ ] |

**Total Points**: 39
**Committed Points**: 36 (excluding P2)
**Buffer**: 3 points

---

## Definition of Done

- [ ] All SDKs have integration tests
- [ ] Tests run against embedded node
- [ ] Tests run against testnet
- [ ] Address derivation consistent across all components
- [ ] All transaction types tested
- [ ] CI runs all SDK tests
- [ ] >80% code coverage

---

## Risks & Blockers

| Risk | Impact | Mitigation |
|------|--------|------------|
| Embedded node unstable for tests | High | Fix in earlier sprints first |
| Testnet not available | Med | Can defer testnet tests |
| Address derivation mismatch | High | Test early in sprint |

---

## Notes

- This is a larger sprint (2 weeks) due to comprehensive testing scope
- WP-4.1 and WP-4.3 should be prioritized first
- Testnet tests (WP-4.2) can be done after testnet launch

---

*Created: 2025-12-04*
*Last Updated: 2025-12-04*
