# Sprint 3: Testing Checklist

**Sprint Goal:** Verify all blockchain integration features work correctly

---

## Unit Tests

### Contract Utilities
- [ ] `contractCompiler.ts`
  - [ ] Compiles valid Solidity contract
  - [ ] Returns compilation errors for invalid code
  - [ ] Estimates gas correctly
  - [ ] Warns for contracts >24KB
  
- [ ] `abiParser.ts`
  - [ ] Parses function ABIs correctly
  - [ ] Extracts events and errors
  - [ ] Handles all Solidity types
  - [ ] Formats return values properly

- [ ] `transactionManager.ts`
  - [ ] Builds valid EIP-1559 transactions
  - [ ] Auto-fills nonce correctly
  - [ ] Estimates gas limits
  - [ ] Signs transactions properly

---

## Integration Tests

### Contract Deployment Flow
- [ ] Deploy ERC-20 token contract
  - [ ] Compile contract source
  - [ ] Fill constructor parameters (name, symbol, supply)
  - [ ] Deploy to testnet
  - [ ] Verify contract address
  - [ ] Save to "My Contracts"
  
- [ ] Deploy contract with errors
  - [ ] Show compilation error
  - [ ] Show deployment failure (out of gas)
  - [ ] Handle transaction revert

### Contract Interaction Flow
- [ ] Call read functions
  - [ ] Load deployed ERC-20 contract
  - [ ] Call `balanceOf(address)`
  - [ ] Display result correctly
  - [ ] Call `totalSupply()`
  
- [ ] Call write functions
  - [ ] Call `transfer(to, amount)`
  - [ ] Show transaction preview
  - [ ] Track transaction status
  - [ ] Verify balance changed

### Transaction Management
- [ ] Create and send transaction
  - [ ] Build simple transfer transaction
  - [ ] Estimate gas
  - [ ] Send transaction
  - [ ] Get transaction hash immediately
  
- [ ] Track transaction status
  - [ ] Show pending status
  - [ ] Update to confirmed
  - [ ] Display receipt
  - [ ] Show gas used

### DAG Visualization
- [ ] Live block updates
  - [ ] Connect WebSocket
  - [ ] Receive new block event
  - [ ] Add block to visualization
  - [ ] Maintain performance
  
- [ ] Filtering and search
  - [ ] Filter by blue/red blocks
  - [ ] Search by hash
  - [ ] Jump to block
  - [ ] Clear filters

---

## End-to-End Tests

### Test Scenario 1: Deploy and Interact with Smart Contract
**Duration:** 5 minutes

1. [ ] Open Contracts tab
2. [ ] Select ERC-20 example contract
3. [ ] Compile contract (should succeed)
4. [ ] Enter constructor params:
   - Name: "Test Token"
   - Symbol: "TEST"
   - Supply: "1000000"
5. [ ] Deploy contract
6. [ ] Wait for confirmation (~12s)
7. [ ] Copy contract address
8. [ ] Switch to "Interact" tab
9. [ ] Load contract by address
10. [ ] Call `balanceOf(deployer_address)`
11. [ ] Verify result shows 1,000,000
12. [ ] Call `transfer(recipient, 100)`
13. [ ] Wait for transaction
14. [ ] Call `balanceOf(recipient)`
15. [ ] Verify result shows 100

**Expected Result:** All steps complete without errors

---

### Test Scenario 2: Transaction Management
**Duration:** 3 minutes

1. [ ] Open Wallet tab
2. [ ] Click "Send Transaction"
3. [ ] Enter recipient address
4. [ ] Enter amount: 1.0 CTRT
5. [ ] Click "Estimate Gas"
6. [ ] Verify gas estimate appears
7. [ ] Click "Send"
8. [ ] Verify transaction appears in queue
9. [ ] Wait for confirmation
10. [ ] Check receipt shows success
11. [ ] Verify balance updated

**Expected Result:** Transaction confirms successfully

---

### Test Scenario 3: Live DAG Updates
**Duration:** 2 minutes

1. [ ] Open DAG Explorer tab
2. [ ] Note current block height
3. [ ] Wait for new block (~1-2 seconds)
4. [ ] Verify new block appears
5. [ ] Click on new block
6. [ ] Verify details panel shows
7. [ ] Search for block by hash
8. [ ] Verify block highlights
9. [ ] Filter to show only blue blocks
10. [ ] Verify red blocks hidden

**Expected Result:** DAG updates in real-time

---

## Performance Tests

### Contract Deployment
- [ ] Deploy small contract (<5KB): <3 seconds
- [ ] Deploy large contract (20KB): <8 seconds
- [ ] Compile complex contract: <5 seconds

### Function Calls
- [ ] Read function (simple): <300ms
- [ ] Read function (complex): <500ms
- [ ] Write function submission: <200ms

### DAG Rendering
- [ ] Render 100 blocks: <500ms
- [ ] Render 1,000 blocks: <2 seconds
- [ ] Render 10,000 blocks: <10 seconds
- [ ] Maintain 60fps during zoom/pan

### WebSocket
- [ ] Connection establishes: <1 second
- [ ] Block update latency: <200ms
- [ ] Reconnection: <1 second

---

## Security Tests

### Input Validation
- [ ] Contract address validation
  - [ ] Reject invalid checksums
  - [ ] Reject non-address strings
  - [ ] Accept valid addresses
  
- [ ] Amount validation
  - [ ] Reject negative amounts
  - [ ] Reject non-numeric input
  - [ ] Handle decimals correctly
  
- [ ] Gas limit validation
  - [ ] Reject unreasonably low gas
  - [ ] Reject unreasonably high gas
  - [ ] Accept valid range

### Transaction Security
- [ ] Confirm before sending
  - [ ] Show preview dialog
  - [ ] Display total cost
  - [ ] Require user confirmation
  
- [ ] Private key handling
  - [ ] Never log private keys
  - [ ] Never send keys over network
  - [ ] Use secure storage only

---

## Accessibility Tests

### Keyboard Navigation
- [ ] Tab through all contract fields
- [ ] Submit forms with Enter key
- [ ] Cancel with Escape key
- [ ] Navigate DAG with arrow keys

### Screen Reader
- [ ] All buttons have labels
- [ ] Form errors announced
- [ ] Transaction status announced
- [ ] Loading states announced

### Visual
- [ ] Focus indicators visible
- [ ] Sufficient color contrast
- [ ] Text readable at 200% zoom
- [ ] Works in high contrast mode

---

## Cross-Platform Tests

### Platforms to Test
- [ ] macOS (Intel)
- [ ] macOS (Apple Silicon)
- [ ] Windows 10/11
- [ ] Linux (Ubuntu 22.04)

### Browsers (Web Mode)
- [ ] Chrome/Chromium
- [ ] Firefox
- [ ] Safari
- [ ] Edge

---

## Regression Tests

### Sprint 1 Features
- [ ] Password validation still works
- [ ] Input validation still works
- [ ] Error boundaries catch errors
- [ ] Loading skeletons display

### Sprint 2 Features
- [ ] State persistence works
- [ ] Dark mode toggles correctly
- [ ] Keyboard shortcuts work
- [ ] Accessibility features intact

---

## Bug Tracker

| ID | Description | Severity | Status | Fixed In |
|----|-------------|----------|--------|----------|
| S3-001 | | | | |

---

## Test Environment

### Testnet Configuration
- **Network:** Citrate Testnet
- **Chain ID:** 100001
- **RPC URL:** http://localhost:8545
- **WebSocket:** ws://localhost:8546

### Test Accounts
- **Deployer:** 0x... (funded with 100 CTRT)
- **Recipient:** 0x... (funded with 10 CTRT)

---

## Sign-Off

- [ ] All unit tests passing
- [ ] All integration tests passing
- [ ] All E2E scenarios complete
- [ ] Performance targets met
- [ ] Security audit passed
- [ ] Accessibility compliance verified
- [ ] Cross-platform tested
- [ ] No blocking bugs

**Tested By:** _______________
**Date:** _______________
**Build Version:** v3.1.0
**Status:** ⬜ Pass  ⬜ Fail

---

**Document Version:** 1.0
**Last Updated:** February 11, 2026
