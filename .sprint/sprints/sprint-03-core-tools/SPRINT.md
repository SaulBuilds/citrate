# Sprint 3 - Core Tools

**Phase**: Phase 3 (Tool Implementation - Part 1)
**Duration**: 2 weeks
**Status**: IN PROGRESS
**Started**: 2025-12-02

---

## Sprint Goal

Implement the core MCP tools that power the agent's blockchain interaction capabilities:
1. Chain query tool for balance, blocks, transactions
2. Send transaction tool for token transfers
3. Deploy contract tool for smart contract deployment
4. Call contract tool for read/write operations
5. Run inference tool for model execution
6. Tool result formatting for chat display

---

## Work Packages

### WP-3.1: CHAIN_QUERY Tool (5 points) - P0

**Description**: Implement tool for querying blockchain data.

**Acceptance Criteria**:
- [ ] Query account balance by address
- [ ] Query block by number or hash
- [ ] Query transaction by hash
- [ ] Query transaction receipt
- [ ] Query account info (nonce, code)
- [ ] Format results for agent consumption

**Files**:
- `src-tauri/src/agent/tools/chain_query.rs`

---

### WP-3.2: SEND_TRANSACTION Tool (5 points) - P0

**Description**: Implement tool for sending token transfers.

**Acceptance Criteria**:
- [ ] Parse natural language amount ("1.5 CIT", "500000 wei")
- [ ] Validate recipient address
- [ ] Estimate gas before sending
- [ ] Create transaction and add to mempool
- [ ] Return transaction hash
- [ ] Handle insufficient balance gracefully

**Files**:
- `src-tauri/src/agent/tools/send_transaction.rs`

---

### WP-3.3: DEPLOY_CONTRACT Tool (8 points) - P0

**Description**: Implement tool for deploying smart contracts.

**Acceptance Criteria**:
- [ ] Accept Solidity source code or bytecode
- [ ] Compile Solidity if source provided (via solc)
- [ ] Encode constructor arguments
- [ ] Deploy contract transaction
- [ ] Return deployed contract address
- [ ] Store ABI for future calls

**Files**:
- `src-tauri/src/agent/tools/deploy_contract.rs`
- `src-tauri/src/agent/tools/solidity_compiler.rs` (helper)

---

### WP-3.4: CALL_CONTRACT Tool (5 points) - P0

**Description**: Implement tool for contract interactions.

**Acceptance Criteria**:
- [ ] Support read-only calls (view/pure functions)
- [ ] Support state-changing calls (transactions)
- [ ] ABI encoding/decoding of function calls
- [ ] Parse function signatures ("transfer(address,uint256)")
- [ ] Return decoded results

**Files**:
- `src-tauri/src/agent/tools/call_contract.rs`
- `src-tauri/src/agent/tools/abi_utils.rs` (helper)

---

### WP-3.5: RUN_INFERENCE Tool (8 points) - P0

**Description**: Implement tool for model inference execution.

**Acceptance Criteria**:
- [ ] List available models (local GGUF + on-chain)
- [ ] Load model by name or ID
- [ ] Run text generation inference
- [ ] Run embedding generation
- [ ] Stream inference results
- [ ] Handle model loading errors

**Files**:
- `src-tauri/src/agent/tools/run_inference.rs`

---

### WP-3.6: Tool Result Formatting (5 points) - P1

**Description**: Format tool outputs for chat display.

**Acceptance Criteria**:
- [ ] Format balance results as currency
- [ ] Format block info as card data
- [ ] Format transaction details clearly
- [ ] Format contract call results
- [ ] Create error message formatting
- [ ] Support markdown in results

**Files**:
- `src-tauri/src/agent/tools/formatter.rs`
- Update `src/types/agent.ts` with new result types

---

## Sprint Summary

| Metric | Value |
|--------|-------|
| **Total Points** | 36 |
| **P0 Points** | 31 |
| **P1 Points** | 5 |
| **Work Packages** | 6 |

---

## Dependencies

- **Sprint 1** (Agent Foundation): AgentOrchestrator, ToolDispatcher ✅
- **Sprint 2** (Frontend): ChainResultCard, TransactionCard ✅
- **Existing**: RPC client, wallet manager, mempool access

---

## Technical Notes

### Tool Registration Pattern

Tools should register with the ToolDispatcher using a consistent interface:

```rust
pub trait AgentTool: Send + Sync {
    fn name(&self) -> &'static str;
    fn description(&self) -> &'static str;
    fn parameters(&self) -> Vec<ToolParameter>;
    async fn execute(&self, params: serde_json::Value) -> Result<ToolResult, ToolError>;
}
```

### Integration Points

1. **RPC Client**: Use existing `eth_*` endpoints for chain queries
2. **Wallet Manager**: Access to signing keys for transactions
3. **Mempool**: Direct access for transaction submission
4. **MCP Layer**: Model registry and inference endpoints

### Error Handling

All tools should return structured errors:
```rust
pub enum ToolError {
    InvalidParameters(String),
    ChainError(String),
    InsufficientBalance,
    ContractError(String),
    ModelError(String),
    Timeout,
}
```

---

## Success Criteria

- [ ] User can query balance via chat: "What's my balance?"
- [ ] User can send tokens via chat: "Send 1 CIT to 0x..."
- [ ] User can deploy contracts via chat (with approval)
- [ ] User can call contract functions via chat
- [ ] User can run model inference via chat
- [ ] All tool results display correctly in chat UI

---

## Risks

| Risk | Mitigation |
|------|------------|
| Solidity compilation complexity | Use solc binary or skip to bytecode-only |
| Model loading slow | Implement progress indicators |
| Gas estimation inaccurate | Add safety margin, allow user override |

---

## Definition of Done

- [ ] All tools implemented and registered
- [ ] Unit tests for each tool
- [ ] Integration tests with agent
- [ ] Tools display results in chat correctly
- [ ] Error cases handled gracefully
- [ ] Documentation updated

---

*Created: 2025-12-02*
