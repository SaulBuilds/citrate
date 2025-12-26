# Citrate Test Plan & User Stories

**Generated**: 2025-12-25
**Version**: 1.0.0
**Reference**: [FEATURE_INVENTORY.md](./FEATURE_INVENTORY.md)

---

## Overview

This test plan validates all features identified in the Feature Inventory. Tests are organized by user journey and priority level.

### Test Categories

| Category | Tests | Priority |
|----------|-------|----------|
| Core Blockchain | 15 | P0 (Critical) |
| Wallet Operations | 12 | P0 (Critical) |
| Smart Contracts | 10 | P0 (Critical) |
| Agent & Tools | 14 | P1 (High) |
| AI/Model Operations | 8 | P1 (High) |
| SDK Integration | 10 | P1 (High) |
| CLI Validation | 8 | P2 (Medium) |
| Security Features | 10 | P0 (Critical) |

---

## 1. Core Blockchain Tests (P0)

### 1.1 Node Startup & Sync

**User Story**: As a user, I want to start the node and have it ready for transactions.

| Test ID | Test Case | Steps | Expected Result |
|---------|-----------|-------|-----------------|
| CB-001 | Node starts successfully | 1. Run `cargo run --bin citrate-node -- devnet` | Node starts, RPC available on :8545 |
| CB-002 | RPC responds to eth_chainId | 1. Start node 2. `curl -X POST localhost:8545 -d '{"jsonrpc":"2.0","method":"eth_chainId","id":1}'` | Returns "0xa455" (42069) |
| CB-003 | Block number increments | 1. Start node 2. Query eth_blockNumber 3. Wait 2s 4. Query again | Block number increases |
| CB-004 | DAG tips available | 1. Start node 2. Query `chain_getTips` | Returns at least 1 tip |
| CB-005 | Mempool accessible | 1. Start node 2. Query `citrate_getMempoolSnapshot` | Returns empty or pending list |

### 1.2 Transaction Types

**User Story**: As a developer, I want to send all transaction types (Legacy, EIP-2930, EIP-1559).

| Test ID | Test Case | Steps | Expected Result |
|---------|-----------|-------|-----------------|
| CB-006 | Legacy transaction | 1. Create legacy tx (v=27/28) 2. Send via eth_sendRawTransaction | Tx included in block |
| CB-007 | EIP-155 transaction | 1. Create EIP-155 tx (v = chainId*2+35) 2. Send | Tx included, correct signer |
| CB-008 | EIP-2930 transaction | 1. Create type 0x01 tx with access list 2. Send | Tx included, access list parsed |
| CB-009 | EIP-1559 transaction | 1. Create type 0x02 tx with maxFeePerGas 2. Send | Tx included, fees calculated |
| CB-010 | Pending nonce support | 1. Send tx1 2. Query nonce with "pending" 3. Send tx2 | Nonces increment correctly |

### 1.3 GhostDAG Consensus

**User Story**: As a validator, I want blocks to be ordered correctly via GhostDAG.

| Test ID | Test Case | Steps | Expected Result |
|---------|-----------|-------|-----------------|
| CB-011 | Blue set calculation | 1. Query `get_blue_set(blockHash)` | Returns blue set with score |
| CB-012 | Multiple tips merge | 1. Create parallel blocks 2. Merge in next block | Merge parents included |
| CB-013 | Total ordering deterministic | 1. Query ordering from multiple nodes | Same order on all nodes |
| CB-014 | Finality at depth 100 | 1. Produce 101 blocks 2. Check finality of block 1 | Block 1 is final |
| CB-015 | Tip selection by score | 1. Get tips 2. Verify highest blue score selected | Correct parent selected |

---

## 2. Wallet Operations Tests (P0)

### 2.1 Account Management

**User Story**: As a user, I want to create and manage wallet accounts.

| Test ID | Test Case | Steps | Expected Result |
|---------|-----------|-------|-----------------|
| WA-001 | Create account via GUI | 1. Open GUI 2. Navigate to Wallet 3. Click Create Account 4. Enter password | Account created, address shown |
| WA-002 | Import from private key | 1. Open wallet 2. Import from hex key | Account imported, balance shown |
| WA-003 | Import from mnemonic | 1. Open wallet 2. Import from 12-word phrase | Account imported correctly |
| WA-004 | Export private key | 1. Select account 2. Export with password | Hex key displayed |
| WA-005 | Delete account | 1. Select account 2. Delete with password | Account removed from list |

### 2.2 Balance & Transactions

**User Story**: As a user, I want to check balances and send funds.

| Test ID | Test Case | Steps | Expected Result |
|---------|-----------|-------|-----------------|
| WA-006 | Check balance | 1. Create account 2. Request from faucet 3. Refresh balance | Balance updated |
| WA-007 | Send transaction | 1. Fund account 2. Enter recipient 3. Enter amount 4. Enter password | Tx sent, balance reduced |
| WA-008 | View transaction history | 1. Send multiple txs 2. View history | All txs listed with status |
| WA-009 | Gas estimation | 1. Fill tx form 2. Check estimated gas | Reasonable gas estimate |

### 2.3 Signing & Verification

**User Story**: As a developer, I want to sign and verify messages.

| Test ID | Test Case | Steps | Expected Result |
|---------|-----------|-------|-----------------|
| WA-010 | Sign message | 1. Enter message 2. Sign with password | Signature hex returned |
| WA-011 | Verify signature | 1. Get signature 2. Verify with address | Returns true for valid |
| WA-012 | Invalid signature rejected | 1. Modify signature 2. Verify | Returns false |

---

## 3. Smart Contract Tests (P0)

### 3.1 Contract Deployment

**User Story**: As a developer, I want to deploy smart contracts.

| Test ID | Test Case | Steps | Expected Result |
|---------|-----------|-------|-----------------|
| SC-001 | Deploy via GUI | 1. Open ContractDeployer 2. Paste bytecode 3. Deploy with password | Contract address returned |
| SC-002 | Deploy via agent | 1. Chat: "Deploy contract with bytecode 0x..." 2. Approve | Contract deployed |
| SC-003 | Constructor args | 1. Deploy with constructor params | Contract initialized correctly |
| SC-004 | Contract address calculation | 1. Deploy contract 2. Verify address = keccak256(rlp(sender, nonce))[12:] | Address matches |

### 3.2 Contract Interaction

**User Story**: As a developer, I want to interact with deployed contracts.

| Test ID | Test Case | Steps | Expected Result |
|---------|-----------|-------|-----------------|
| SC-005 | Read function (eth_call) | 1. Deploy ERC20 2. Call `balanceOf(address)` | Returns correct balance |
| SC-006 | Write function | 1. Deploy ERC20 2. Call `transfer(to, amount)` | Transfer successful |
| SC-007 | Event logs | 1. Execute transfer 2. Get receipt 3. Parse logs | Transfer event emitted |
| SC-008 | Revert handling | 1. Call function that reverts | Revert reason returned |

### 3.3 Precompiles

**User Story**: As a developer, I want precompiles to work for cryptographic operations.

| Test ID | Test Case | Steps | Expected Result |
|---------|-----------|-------|-----------------|
| SC-009 | ECRECOVER (0x01) | 1. Deploy contract using ecrecover 2. Call with signature | Correct address recovered |
| SC-010 | SHA256 (0x02) | 1. Call SHA256 precompile | Correct hash returned |

---

## 4. Agent & Tools Tests (P1)

### 4.1 Agent Initialization

**User Story**: As a user, I want the AI agent to be ready after onboarding.

| Test ID | Test Case | Steps | Expected Result |
|---------|-----------|-------|-----------------|
| AG-001 | Agent ready after onboarding | 1. Complete onboarding 2. Open ChatDashboard | Agent responds to messages |
| AG-002 | Model auto-download | 1. Fresh install 2. Complete onboarding | Qwen2.5-7B downloaded |
| AG-003 | Session persists | 1. Send messages 2. Close/reopen | Conversation preserved |

### 4.2 Blockchain Tools

**User Story**: As a user, I want to use agent for blockchain queries.

| Test ID | Test Case | Steps | Expected Result |
|---------|-----------|-------|-----------------|
| AG-004 | Check balance via chat | 1. "What's my wallet balance?" | Balance displayed |
| AG-005 | DAG status via chat | 1. "Show me the DAG status" | Tips and stats shown |
| AG-006 | Transaction info | 1. "Show transaction 0x..." | Tx details displayed |
| AG-007 | Block info | 1. "Show block 123" | Block details displayed |

### 4.3 Contract Tools

**User Story**: As a developer, I want to deploy contracts via chat.

| Test ID | Test Case | Steps | Expected Result |
|---------|-----------|-------|-----------------|
| AG-008 | Deploy contract via agent | 1. "Deploy this contract: 0x..." 2. Approve | Contract deployed |
| AG-009 | Call contract via agent | 1. "Call balanceOf on 0x..." | Result returned |
| AG-010 | Write contract via agent | 1. "Transfer 10 tokens to 0x..." 2. Approve | Transfer executed |

### 4.4 Confirmation Workflow

**User Story**: As a user, I want to approve high-risk operations.

| Test ID | Test Case | Steps | Expected Result |
|---------|-----------|-------|-----------------|
| AG-011 | Tool requires confirmation | 1. "Send 1 ETH to 0x..." | Approval prompt shown |
| AG-012 | Approve tool execution | 1. Request send 2. Click Approve | Transaction sent |
| AG-013 | Reject tool execution | 1. Request send 2. Click Reject | Operation cancelled |
| AG-014 | Pending tools visible | 1. Request multiple operations 2. Check pending | All pending shown |

---

## 5. AI/Model Operations Tests (P1)

### 5.1 Local Model Inference

**User Story**: As a user, I want to use local AI models.

| Test ID | Test Case | Steps | Expected Result |
|---------|-----------|-------|-----------------|
| AI-001 | Local model loads | 1. Complete onboarding 2. Check model status | Model loaded and ready |
| AI-002 | Chat completion | 1. Send message to agent | Coherent response |
| AI-003 | ReAct reasoning | 1. Ask complex question 2. Check thinking | Shows Thought/Action/Observation |
| AI-004 | Tool selection | 1. "Check my balance" 2. Observe tool call | Correct tool selected |

### 5.2 Model Management

**User Story**: As a developer, I want to manage AI models.

| Test ID | Test Case | Steps | Expected Result |
|---------|-----------|-------|-----------------|
| AI-005 | Scan local models | 1. Call `hf_scan_local_models` | Returns model list |
| AI-006 | Download HF model | 1. Search for GGUF model 2. Download | Model downloaded |
| AI-007 | Switch active model | 1. Load different model | New model active |
| AI-008 | GPU detection | 1. Call `gpu_get_devices` | GPU(s) detected |

---

## 6. SDK Integration Tests (P1)

### 6.1 JavaScript SDK

**User Story**: As a developer, I want to use the JavaScript SDK.

| Test ID | Test Case | Steps | Expected Result |
|---------|-----------|-------|-----------------|
| JS-001 | SDK connects to node | 1. `new CitrateSDK({ rpcEndpoint })` 2. Call getNetworkInfo() | Network info returned |
| JS-002 | Create account | 1. `sdk.accounts.createAccount()` | Address and key returned |
| JS-003 | Get balance | 1. `sdk.accounts.getBalance(addr)` | Balance returned |
| JS-004 | Send transaction | 1. `sdk.accounts.sendTransaction(tx)` | Tx hash returned |
| JS-005 | Deploy contract | 1. `sdk.contracts.deploy(bytecode)` | Contract address returned |

### 6.2 Python SDK

**User Story**: As a developer, I want to use the Python SDK.

| Test ID | Test Case | Steps | Expected Result |
|---------|-----------|-------|-----------------|
| PY-001 | SDK connects | 1. `CitrateClient(rpc_url)` | Client created |
| PY-002 | Get chain ID | 1. `client.get_chain_id()` | Returns 42069 |
| PY-003 | Get balance | 1. `client.get_balance(addr)` | Balance returned |
| PY-004 | Sign transaction | 1. `key_manager.sign_transaction(tx)` | Signed tx returned |
| PY-005 | Send transaction | 1. `client._send_transaction(...)` | Tx hash returned |

---

## 7. CLI Validation Tests (P2)

### 7.1 Wallet CLI

**User Story**: As a user, I want to use the wallet CLI.

| Test ID | Test Case | Steps | Expected Result |
|---------|-----------|-------|-----------------|
| CLI-001 | Create wallet | 1. `wallet new` | Account created |
| CLI-002 | List accounts | 1. `wallet list` | Accounts shown |
| CLI-003 | Check balance | 1. `wallet balance 0` | Balance displayed |
| CLI-004 | Send funds | 1. `wallet send --from 0 --to ... --amount 1` | Tx sent |

### 7.2 General CLI

**User Story**: As a developer, I want to use the general CLI.

| Test ID | Test Case | Steps | Expected Result |
|---------|-----------|-------|-----------------|
| CLI-005 | Network status | 1. `citrate network status` | Status shown |
| CLI-006 | Get block | 1. `citrate network block latest` | Block shown |
| CLI-007 | Gas price | 1. `citrate network gas-price` | Price shown |
| CLI-008 | Peers | 1. `citrate network peers` | Peer list shown |

---

## 8. Security Tests (P0)

### 8.1 Wallet Security

**User Story**: As a user, I want my wallet to be secure.

| Test ID | Test Case | Steps | Expected Result |
|---------|-----------|-------|-----------------|
| SEC-001 | Password required for tx | 1. Try send without password | Error: password required |
| SEC-002 | Wrong password rejected | 1. Enter wrong password | Error: invalid password |
| SEC-003 | Rate limiting active | 1. Enter wrong password 6 times | Account locked 5 min |
| SEC-004 | Private key encrypted | 1. Check keystore file | Key is encrypted (AES-256-GCM) |
| SEC-005 | Mnemonic verification | 1. Complete onboarding 2. Verify 3 random words | Mnemonic verified |

### 8.2 Session Management (BLOCKED - Not Exposed)

**User Story**: As a user, I want session-based authentication.

| Test ID | Test Case | Steps | Expected Result |
|---------|-----------|-------|-----------------|
| SEC-006 | Session created on auth | 1. Enter password 2. Check session | Session active |
| SEC-007 | Session expires after 15min | 1. Wait 16 minutes 2. Try operation | Password required again |
| SEC-008 | Session touch on activity | 1. Perform operations 2. Check timeout | Timeout extended |
| SEC-009 | Manual lock wallet | 1. Click Lock 2. Try operation | Password required |
| SEC-010 | High-value tx requires reauth | 1. Send >= 10 SALT | Password required regardless |

**Note**: Tests SEC-006 to SEC-010 are BLOCKED because session management is not exposed to the frontend.

---

## 9. Integration Test Scenarios

### 9.1 Full User Journey: First-Time Setup

```
1. Install Citrate GUI
2. Complete onboarding (set password, backup mnemonic)
3. Wait for model download
4. Create first account
5. Request testnet tokens from faucet
6. Check balance updates
7. Send transaction to another address
8. Verify transaction in history
9. Use agent to check balance
10. Deploy simple contract via agent
```

**Pass Criteria**: All steps complete without errors

### 9.2 Full User Journey: Developer Workflow

```
1. Start node via GUI
2. Connect with JavaScript SDK
3. Create account via SDK
4. Fund account from faucet
5. Write Solidity contract
6. Compile with Forge
7. Deploy via SDK
8. Call contract functions
9. Verify events emitted
10. Query via agent
```

**Pass Criteria**: Contract deployed and functional

### 9.3 Full User Journey: AI Agent Workflow

```
1. Open ChatDashboard
2. Ask "What's my balance?"
3. Ask "Show DAG status"
4. Ask "Deploy this contract: [bytecode]"
5. Approve deployment
6. Ask "Call balanceOf on [address]"
7. Ask "Transfer 10 tokens to [address]"
8. Approve transfer
9. Ask "Show transaction history"
10. Verify all operations succeeded
```

**Pass Criteria**: All agent operations complete

---

## 10. Test Execution Checklist

### Pre-Test Setup

- [ ] Clean build: `cargo clean && cargo build --release`
- [ ] Fresh chain data: `rm -rf ~/.citrate-devnet`
- [ ] Node running: `cargo run --bin citrate-node -- devnet`
- [ ] GUI built: `cd gui/citrate-core && npm run build`
- [ ] Models available: Check `~/.cache/citrate/models/`

### Test Execution Order

1. **P0 Critical** (Must pass before release)
   - [ ] Core Blockchain (CB-001 to CB-015)
   - [ ] Wallet Operations (WA-001 to WA-012)
   - [ ] Smart Contracts (SC-001 to SC-010)
   - [ ] Security (SEC-001 to SEC-005)

2. **P1 High** (Should pass for feature-complete)
   - [ ] Agent & Tools (AG-001 to AG-014)
   - [ ] AI/Model Operations (AI-001 to AI-008)
   - [ ] SDK Integration (JS-001 to JS-005, PY-001 to PY-005)

3. **P2 Medium** (Nice to have)
   - [ ] CLI Validation (CLI-001 to CLI-008)

### Blockers

| Test | Blocker | Ticket |
|------|---------|--------|
| SEC-006 to SEC-010 | Session management not exposed | HIGH PRIORITY |
| PY-003 to PY-005 | Precompile addresses not implemented | MEDIUM |
| CLI-005 to CLI-008 | CLI stubs not implemented | LOW |

---

## 11. Automated Test Commands

### Unit Tests
```bash
# Core tests
cargo test -p citrate-consensus
cargo test -p citrate-execution
cargo test -p citrate-api

# GUI backend tests
cd gui/citrate-core && cargo test

# Contract tests
cd contracts && forge test
```

### Integration Tests
```bash
# Start node in background
cargo run --bin citrate-node -- devnet &

# Run SDK tests
cd sdk/javascript && npm test

# Run Python tests
cd sdks/python && pytest
```

### E2E Tests
```bash
# Start testnet
./scripts/start_testnet.sh

# Run smoke tests
./scripts/smoke_inference.sh

# Cluster tests
./scripts/cluster_smoke.sh
```

---

## Appendix: Test Data

### Sample Bytecode (Simple Storage)
```
0x608060405234801561001057600080fd5b5060f78061001f6000396000f3fe6080604052348015600f57600080fd5b5060043610603c5760003560e01c80632a1afcd914604157806360fe47b114605b575b600080fd5b60476073565b604051605291906098565b60405180910390f35b6071600480360381019060009190819080359060200190929190505050607c565b005b60005481565b8060008190555050565b6000819050919050565b609281609883565b82525050565b600060208201905060ab6000830184608b565b9291505056fea264697066735822122000
```

### Sample ERC20 Bytecode
```
[Use standard OpenZeppelin ERC20 bytecode]
```

### Test Mnemonic (DO NOT USE IN PRODUCTION)
```
abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about
```

### Test Private Key (DO NOT USE IN PRODUCTION)
```
0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80
```
