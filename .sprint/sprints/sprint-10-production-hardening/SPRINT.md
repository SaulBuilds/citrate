# Sprint 10: Production Hardening & AI Infrastructure

**Sprint Duration:** 4 weeks
**Priority:** P0 - Critical for Production Release
**Status:** Planning

---

## Executive Summary

This sprint addresses critical gaps identified in the Citrate platform before production release:
1. Security vulnerabilities in wallet/onboarding flow
2. Non-functional API key integrations (OpenAI, Anthropic)
3. Agentic tool debugging and end-to-end testing
4. AI model ecosystem (LoRA training, Hugging Face, model registry)
5. Foundation for distributed GPU compute network

---

## Current State Analysis

### Inventory: Agentic Tools (18 Total)

| Category | Tool | Status | Issues |
|----------|------|--------|--------|
| **Blockchain** | NodeStatusTool | Implemented | Needs E2E testing |
| | BlockInfoTool | Implemented | Needs E2E testing |
| | DAGStatusTool | Implemented | Needs E2E testing |
| | TransactionInfoTool | Implemented | Needs E2E testing |
| | AccountInfoTool | Implemented | Needs E2E testing |
| **Wallet** | BalanceTool | Implemented | Needs E2E testing |
| | SendTransactionTool | Implemented | Needs E2E testing |
| | TransactionHistoryTool | Implemented | Needs E2E testing |
| **Contracts** | DeployContractTool | Implemented | **Not working - needs debug** |
| | CallContractTool | Implemented | **Not working - needs debug** |
| | WriteContractTool | Implemented | **Not working - needs debug** |
| **Models** | ListModelsTool | Implemented | Needs registry connection |
| | RunInferenceTool | Implemented | Local works, on-chain untested |
| | DeployModelTool | Implemented | **Not working - needs debug** |
| | GetModelInfoTool | Implemented | Needs registry connection |
| **Marketplace** | SearchMarketplaceTool | Implemented | Needs E2E testing |
| | GetListingTool | Implemented | Needs E2E testing |
| | BrowseCategoryTool | Implemented | Needs E2E testing |
| **Terminal** | ExecuteCommandTool | Implemented | Needs E2E testing |
| **IPFS** | UploadIPFSTool | Implemented | Needs E2E testing |
| **Image** | GenerateImageTool | Implemented | **No image model bundled** |

### Inventory: Smart Contracts (13 Total)

| Contract | Purpose | Deployment Status |
|----------|---------|-------------------|
| ModelRegistry | AI model registration & access | **Needs verification** |
| ModelMarketplace | Buy/sell model access (2.5% fee) | **Needs verification** |
| InferenceRouter | Route inference to providers | **Needs verification** |
| LoRAFactory | Create LoRA fine-tunes | **Needs verification** |
| ModelAccessControl | Fine-grained model permissions | **Needs verification** |
| IPFSIncentives | Reward storage providers | **Needs verification** |
| ColorCirclesNFT | Example ERC721 | Deployed (test) |
| Counter | Test contract | Deployed (test) |

### Inventory: API Endpoints

**Ethereum-Compatible (27+ methods):** All implemented
**Citrate Extensions:** `citrate_getMempoolSnapshot`, `citrate_getTransactionStatus`, `citrate_chatCompletion`, `citrate_getTextEmbedding`
**REST/MCP:** `/v1/chat/completions`, `/v1/embeddings`, `/v1/models`, `/v1/images/generations`

### Critical Issues Identified

#### P0 - Security Critical
1. **Hardcoded default password** in wallet setup (`secure_default_password_2024`)
2. **Non-standard key derivation** - Uses Keccak256(mnemonic) instead of BIP32
3. **API key validation failing** - OpenAI/Anthropic connections not working

#### P1 - Functional Critical
4. **Contract deployment tools not working** via agent
5. **Model registry not connected** to GUI
6. **No LoRA training interface** in GUI
7. **Hugging Face integration incomplete** - OAuth not configured

#### P2 - Experience Critical
8. **Onboarding doesn't require password setup**
9. **No mnemonic backup verification**
10. **System prompts need Citrate-specific training**

---

## Work Packages

### WP-10.1: Security Hardening (P0)
**Estimated Effort:** 1 week

#### 10.1.1 Fix Wallet Security
- [ ] Remove hardcoded default password from `lib.rs:~330`
- [ ] Add password setup step to onboarding flow (before wallet creation)
- [ ] Implement password strength requirements (12+ chars, mixed case, numbers, symbols)
- [ ] Add password confirmation field
- [ ] Implement proper BIP32/BIP44 key derivation for mnemonic recovery
- [ ] Add mnemonic backup verification step (show 3 words, user confirms)

#### 10.1.2 Secure Key Operations
- [ ] Add rate limiting on `export_private_key` endpoint
- [ ] Require password re-entry for private key export
- [ ] Consider removing `create_account_extended` endpoint
- [ ] Set explicit 0600 permissions on fallback key files
- [ ] Add 2FA option for high-value operations (optional)

#### 10.1.3 Fix API Key Storage & Validation
- [ ] Debug OpenAI API key connection failure
- [ ] Debug Anthropic API key connection failure
- [ ] Add proper connection test that validates key works
- [ ] Store API keys in OS keychain (not config file)
- [ ] Add visual feedback for connection status

**Acceptance Criteria:**
- [ ] New users must set password during onboarding
- [ ] Password meets minimum strength requirements
- [ ] Mnemonic can be recovered with standard BIP39/BIP44 wallets
- [ ] API keys connect successfully when valid key provided
- [ ] API key connection status visible in UI

---

### WP-10.2: Agentic Tools Debugging (P1)
**Estimated Effort:** 1 week

#### 10.2.1 Contract Tool Debugging
- [ ] Debug DeployContractTool - identify why deployment fails
- [ ] Debug CallContractTool - test against deployed contracts
- [ ] Debug WriteContractTool - test state modifications
- [ ] Add detailed error messages for contract failures
- [ ] Test Solidity compilation integration

#### 10.2.2 Model Tool Debugging
- [ ] Connect GUI to on-chain ModelRegistry
- [ ] Verify DeployModelTool registers to chain
- [ ] Test RunInferenceTool with registered models
- [ ] Add fallback to local model if chain unavailable

#### 10.2.3 Create E2E Test Suite
- [ ] Create test harness for agentic tools
- [ ] Implement automated test runner
- [ ] Test each tool category:
  - [ ] Blockchain tools (5)
  - [ ] Wallet tools (3)
  - [ ] Contract tools (3)
  - [ ] Model tools (4)
  - [ ] Marketplace tools (3)
- [ ] Generate test report with pass/fail status
- [ ] Add CI integration for tool tests

**Test Framework Structure:**
```
tests/
├── e2e/
│   ├── agent_tools/
│   │   ├── blockchain_tools_test.rs
│   │   ├── wallet_tools_test.rs
│   │   ├── contract_tools_test.rs
│   │   ├── model_tools_test.rs
│   │   └── marketplace_tools_test.rs
│   ├── fixtures/
│   │   ├── test_contracts/
│   │   └── test_models/
│   └── harness.rs
└── integration/
    └── full_flow_test.rs
```

**Acceptance Criteria:**
- [ ] All 18 agentic tools pass E2E tests
- [ ] Contract deployment via agent works
- [ ] Model deployment via agent works
- [ ] Test suite runs in CI on every PR

---

### WP-10.3: AI Infrastructure Enhancement (P1)
**Estimated Effort:** 1.5 weeks

#### 10.3.1 System Prompt Enhancement
- [ ] Create Citrate-specific system prompt with:
  - Detailed knowledge of Citrate architecture
  - Smart contract deployment guidance
  - Model registry interaction patterns
  - DAG/GhostDAG explanation capability
  - Available tool descriptions
- [ ] Inject real-time context:
  - Current block height
  - Network status
  - Wallet balance
  - Available models
- [ ] Create prompt templates for different user skill levels

#### 10.3.2 Hugging Face Integration
- [ ] Configure OAuth client_id for HuggingFace
- [ ] Implement proper OAuth PKCE flow
- [ ] Add model browser UI in GUI
- [ ] Filter for GGUF-compatible models
- [ ] Download progress indicator
- [ ] Auto-detect downloaded models
- [ ] Model conversion tools (if needed)

#### 10.3.3 Model Registry UI
- [ ] Display on-chain registered models in GUI
- [ ] Show model metadata (owner, pricing, usage stats)
- [ ] Enable model registration from GUI
- [ ] Add model access purchase flow
- [ ] Display inference pricing

#### 10.3.4 LoRA Training Interface
- [ ] Design LoRA training UI component
- [ ] Integrate with LoRAFactory contract
- [ ] Training configuration panel:
  - Base model selection
  - Training data upload (IPFS)
  - Hyperparameters (rank, alpha, dropout)
  - Training epochs
- [ ] Training progress monitoring
- [ ] LoRA registration on-chain
- [ ] Merge strategies (Linear, SVD, Task-Arithmetic)

**Acceptance Criteria:**
- [ ] Users can browse/download Hugging Face models
- [ ] On-chain model registry visible in GUI
- [ ] Basic LoRA training workflow functional
- [ ] System prompt provides accurate Citrate guidance

---

### WP-10.4: Onboarding Experience (P2)
**Estimated Effort:** 0.5 weeks

#### 10.4.1 Enhanced Onboarding Flow
```
New Flow:
1. Welcome Screen
2. Security Setup (NEW)
   ├── Create strong password
   ├── Confirm password
   └── Show strength indicator
3. Wallet Creation
   ├── Generate mnemonic
   ├── Display mnemonic with copy button
   ├── Backup verification (select 3 words)
   └── Confirm backup saved
4. AI Configuration (Optional)
   ├── Local model (bundled)
   ├── API keys (OpenAI/Anthropic)
   └── Hugging Face login
5. Skill Assessment (Optional)
6. Personalized Guidance
7. Complete
```

#### 10.4.2 Recovery Flow
- [ ] Add "Recover Wallet" button on welcome screen
- [ ] Implement mnemonic input UI (12 words)
- [ ] Validate mnemonic format
- [ ] Derive keys using BIP32 standard
- [ ] Import recovered wallet

**Acceptance Criteria:**
- [ ] New users set secure password before wallet creation
- [ ] Mnemonic backup verified before completion
- [ ] Existing users can recover via mnemonic
- [ ] Clear security guidance during setup

---

### WP-10.5: Distributed GPU Compute Foundation (P2)
**Estimated Effort:** 1 week (Design + Foundation)

#### 10.5.1 Architecture Design
```
GPU Compute Network Architecture:

┌─────────────────────────────────────────────────────────────┐
│                    Citrate Network                          │
├─────────────────────────────────────────────────────────────┤
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐         │
│  │  User Node  │  │  User Node  │  │  User Node  │         │
│  │  (GPU: 20%) │  │  (GPU: 50%) │  │  (GPU: 80%) │         │
│  └──────┬──────┘  └──────┬──────┘  └──────┬──────┘         │
│         │                │                │                 │
│         └────────────────┼────────────────┘                 │
│                          │                                  │
│                    ┌─────▼─────┐                            │
│                    │  Compute  │                            │
│                    │  Router   │                            │
│                    └─────┬─────┘                            │
│                          │                                  │
│         ┌────────────────┼────────────────┐                 │
│         │                │                │                 │
│    ┌────▼────┐     ┌─────▼────┐    ┌─────▼────┐            │
│    │ Inference│     │ Training │    │  LoRA    │            │
│    │  Jobs    │     │   Jobs   │    │  Jobs    │            │
│    └─────────┘     └──────────┘    └──────────┘            │
├─────────────────────────────────────────────────────────────┤
│                    Economics Layer                          │
│  ┌─────────────────────────────────────────────────────┐   │
│  │  Job Payment → Stake Rewards → Provider Earnings    │   │
│  │  SALT Token ←── Compute Credits ←── GPU Contribution │   │
│  └─────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────┘
```

#### 10.5.2 GPU Resource Manager Design
```rust
// Proposed structure for GPU allocation
pub struct GPUResourceManager {
    /// User-configurable allocation (0-100%)
    allocation_percent: u8,

    /// Currently allocated memory
    allocated_memory: u64,

    /// Available GPU memory
    total_memory: u64,

    /// Active compute jobs
    active_jobs: Vec<ComputeJob>,

    /// Safety limits
    max_temperature: u8,  // Default 80°C
    max_power_draw: u32,  // Watts

    /// Earnings tracking
    tokens_earned: u128,
    jobs_completed: u64,
}

pub struct ComputeJob {
    job_id: Hash,
    job_type: JobType,  // Inference, Training, LoRA
    requester: Address,
    payment: u128,
    gpu_requirement: GPURequirement,
    status: JobStatus,
}
```

#### 10.5.3 Safety & Security Considerations
- [ ] Sandboxed execution environment
- [ ] Memory isolation between jobs
- [ ] Temperature/power monitoring
- [ ] Automatic throttling if limits exceeded
- [ ] Job verification (ZKP for inference correctness)
- [ ] Reputation system for compute providers
- [ ] Slashing for malicious behavior

#### 10.5.4 Economic Model Design
```
Token Flow:
1. Job Requester stakes SALT for compute job
2. Compute Router selects provider(s) based on:
   - Available capacity
   - Reputation score
   - Historical latency
   - Price bid
3. Provider executes job with GPU allocation
4. Result verified (hash check or ZKP)
5. Payment released:
   - 95% to compute provider
   - 3% to protocol treasury
   - 2% to routing incentives
6. Provider reputation updated

Staking Requirements:
- Minimum stake to become provider: 1,000 SALT
- Stake locked during active jobs
- Slashing for failed/incorrect jobs
- Bonus rewards for consistent uptime
```

#### 10.5.5 GUI Integration Design
- [ ] GPU allocation slider in Settings (0-100%)
- [ ] Real-time GPU usage display
- [ ] Temperature/power monitoring
- [ ] Earnings dashboard
- [ ] Job history
- [ ] Provider reputation display
- [ ] Safety warnings for high allocation

**Acceptance Criteria (Design Phase):**
- [ ] Architecture document complete
- [ ] Smart contract interfaces defined
- [ ] Economic model documented
- [ ] Security threat model documented
- [ ] GUI mockups created
- [ ] Foundation code stubs in place

---

### WP-10.6: Image Model & Training Interface (P2)
**Estimated Effort:** 1 week

#### 10.6.1 Bundle Basic Image Model
- [ ] Select appropriate small image model (Stable Diffusion variant)
- [ ] Convert to GGUF/optimized format if needed
- [ ] Bundle with GUI distribution (~1-2GB)
- [ ] Integrate with GenerateImageTool

#### 10.6.2 LoRA Training UI for Images
- [ ] Image dataset upload interface
- [ ] Training configuration:
  - Base model selection
  - LoRA rank
  - Learning rate
  - Training steps
  - Sample prompts
- [ ] Training progress visualization
- [ ] Sample image generation during training
- [ ] LoRA export and registration

#### 10.6.3 Model Training Backend
- [ ] Local training execution (Metal GPU)
- [ ] Progress streaming to frontend
- [ ] Checkpoint saving
- [ ] Training resumption
- [ ] Export to GGUF/safetensors

**Acceptance Criteria:**
- [ ] Basic image generation works in GUI
- [ ] Users can train LoRA on custom images
- [ ] Trained LoRA can be registered on-chain

---

## Testing Strategy

### Automated E2E Testing Framework

```yaml
# .github/workflows/e2e-tests.yml
name: E2E Agent Tool Tests

on:
  push:
    branches: [main]
  pull_request:

jobs:
  agent-tools:
    runs-on: macos-latest
    steps:
      - uses: actions/checkout@v4

      - name: Start Test Node
        run: |
          cargo build --release -p citrate-node
          ./target/release/citrate --data-dir .test-node devnet &
          sleep 10

      - name: Run Tool Tests
        run: |
          cargo test --package citrate-core --test agent_tools -- --nocapture

      - name: Generate Report
        run: |
          cargo test --package citrate-core --test agent_tools -- --format json > test-results.json

      - name: Upload Results
        uses: actions/upload-artifact@v4
        with:
          name: test-results
          path: test-results.json
```

### Manual Testing Checklist

```markdown
## Pre-Release Testing Checklist

### Wallet Security
- [ ] Create new wallet - requires password setup
- [ ] Password strength meter works
- [ ] Mnemonic backup verification required
- [ ] Recover wallet from mnemonic (standard wallet compatible)
- [ ] API keys save and connect successfully

### Agent Tools
- [ ] "What is my balance?" - returns correct balance
- [ ] "Deploy this contract: <solidity>" - deploys successfully
- [ ] "Send 1 SALT to 0x..." - transaction succeeds
- [ ] "List available models" - shows on-chain and local
- [ ] "Run inference with model X" - returns result

### Model Operations
- [ ] Browse Hugging Face models
- [ ] Download GGUF model
- [ ] Register model on-chain
- [ ] Run inference via registered model
- [ ] View model in marketplace

### Onboarding
- [ ] New user sees password setup first
- [ ] Mnemonic shown and verified
- [ ] Skill assessment works
- [ ] Personalized guidance displayed
- [ ] Can skip to advanced setup
```

---

## Success Metrics

| Metric | Target | Measurement |
|--------|--------|-------------|
| Wallet security vulnerabilities | 0 P0 issues | Security audit |
| API key connection success rate | >95% | User telemetry |
| Agent tool E2E test pass rate | 100% | CI/CD |
| Onboarding completion rate | >80% | Analytics |
| Time to first transaction | <5 minutes | User testing |
| GPU allocation UI usability | >4/5 rating | User feedback |

---

## Dependencies & Risks

### Dependencies
- Hugging Face OAuth app registration
- Image model selection and licensing
- GPU driver compatibility testing

### Risks
| Risk | Likelihood | Impact | Mitigation |
|------|------------|--------|------------|
| BIP32 migration breaks existing wallets | Medium | High | Provide migration tool |
| Image model too large for bundle | Medium | Medium | Use smaller model or download |
| GPU allocation causes system instability | Low | High | Conservative defaults, extensive testing |

---

## Timeline

```
Week 1: WP-10.1 Security Hardening
        - Remove hardcoded password
        - Add password setup to onboarding
        - Fix API key connections

Week 2: WP-10.2 Tool Debugging + Testing
        - Debug contract tools
        - Debug model tools
        - Create E2E test suite

Week 3: WP-10.3 AI Infrastructure
        - Hugging Face integration
        - Model registry UI
        - System prompt enhancement

Week 4: WP-10.4 + WP-10.5 + WP-10.6
        - Onboarding refinement
        - GPU compute design doc
        - Image model foundation
```

---

## Appendix A: Current System Prompts

**Default System Prompt (to be enhanced):**
```
You are Citrate AI, an intelligent assistant for the Citrate blockchain. You help users:
- Query wallet balance and transaction history
- Send transactions and interact with smart contracts
- Deploy and interact with AI models on-chain
- Explore DAG structure and block information
- Understand blockchain concepts and Citrate-specific features
```

**Proposed Enhanced System Prompt:**
```
You are Citrate AI, the native AI assistant for the Citrate blockchain - an AI-native Layer-1 BlockDAG with integrated machine learning capabilities.

## Your Capabilities
You can help users with:
1. **Wallet Operations**: Check balances, send transactions, view history
2. **Smart Contracts**: Deploy Solidity contracts, call functions, write state
3. **AI Models**: List, deploy, and run inference on registered models
4. **DAG Exploration**: Query blocks, transactions, DAG structure
5. **Development**: Scaffold dApps, execute terminal commands, manage IPFS

## Citrate Architecture
- **Consensus**: GhostDAG protocol with k=18 cluster tolerance
- **Execution**: EVM-compatible LVM with AI precompiles
- **Storage**: RocksDB + IPFS for model weights
- **Token**: SALT (18 decimals)

## Available Tools
When users request actions, you can use these tools:
- Blockchain: NodeStatus, BlockInfo, DAGStatus, TransactionInfo, AccountInfo
- Wallet: Balance, SendTransaction, TransactionHistory
- Contracts: DeployContract, CallContract, WriteContract
- Models: ListModels, RunInference, DeployModel, GetModelInfo
- Marketplace: SearchMarketplace, GetListing, BrowseCategory
- Terminal: ExecuteCommand (git, npm, cargo, python)
- IPFS: Upload, Get, Pin

## Current Context
- Network: {network_name}
- Block Height: {block_height}
- Wallet: {wallet_address}
- Balance: {wallet_balance} SALT
- Node Status: {node_status}

Always be helpful, accurate, and guide users through complex operations step by step.
```

---

## Appendix B: Smart Contract Addresses (Testnet)

| Contract | Address | ABI Location |
|----------|---------|--------------|
| ModelRegistry | TBD | `contracts/out/ModelRegistry.sol/ModelRegistry.json` |
| ModelMarketplace | TBD | `contracts/out/ModelMarketplace.sol/ModelMarketplace.json` |
| LoRAFactory | TBD | `contracts/out/LoRAFactory.sol/LoRAFactory.json` |
| InferenceRouter | TBD | `contracts/out/InferenceRouter.sol/InferenceRouter.json` |

---

## Appendix C: File Reference

| Component | Path |
|-----------|------|
| Wallet Manager | `gui/citrate-core/src-tauri/src/wallet/mod.rs` |
| Onboarding Modal | `gui/citrate-core/src/components/OnboardingModal.tsx` |
| Onboarding Backend | `gui/citrate-core/src-tauri/src/agent/onboarding.rs` |
| LLM Config | `gui/citrate-core/src-tauri/src/agent/llm/mod.rs` |
| API Backends | `gui/citrate-core/src-tauri/src/agent/llm/api.rs` |
| Agent Config | `gui/citrate-core/src-tauri/src/agent/config.rs` |
| Tool Handlers | `gui/citrate-core/src-tauri/src/agent/tools/` |
| Orchestrator | `gui/citrate-core/src-tauri/src/agent/orchestrator.rs` |
| Model Registry | `core/mcp/src/registry.rs` |
| HuggingFace | `gui/citrate-core/src-tauri/src/huggingface/mod.rs` |
| Tauri Commands | `gui/citrate-core/src-tauri/src/lib.rs` |
