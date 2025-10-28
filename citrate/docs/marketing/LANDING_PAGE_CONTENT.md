# Citrate Landing Page Content

## Hero Section

### Headline
**The First AI-Native Blockchain**
Built for the Age of Decentralized Intelligence

### Subheadline
Citrate combines cutting-edge BlockDAG consensus with standardized AI model orchestration, creating the first blockchain purpose-built for artificial intelligence workloads.

### Call to Action (Primary)
- **Get Started** → `/docs/getting-started`
- **Deploy a Model** → `/docs/deploy-model`
- **Read the Whitepaper** → `/whitepaper.pdf`

### Call to Action (Secondary)
- **Join Discord** → `[Discord Link]`
- **View on GitHub** → `https://github.com/citrate-ai/citrate`

---

## The Problem

### Current Blockchain Limitations

**Slow & Sequential**
Traditional blockchains process transactions one block at a time, creating artificial bottlenecks that limit throughput to 15-100 TPS.

**AI Infrastructure Disconnect**
Existing chains treat AI models as afterthoughts—smart contracts weren't designed for gigabyte-scale neural networks and GPU-intensive inference.

**No Verifiable Computation**
Users have no way to verify that AI inference ran correctly, opening the door to manipulation and censorship.

**Fragmented Standards**
Every AI platform uses different APIs, making it impossible to build interoperable AI applications.

---

## The Solution: Citrate

### Three Core Innovations

#### 1. **GhostDAG Consensus: Parallel Block Production**

Citrate uses GhostDAG—a BlockDAG consensus algorithm that allows **multiple blocks to be produced simultaneously** without conflicts.

**Benefits:**
- **10,000+ TPS** throughput vs. 15 TPS on traditional chains
- **Sub-12 second finality** with BFT checkpoints
- **100+ parallel blocks** in the DAG simultaneously
- **No uncle/orphan blocks**—all valid blocks contribute to consensus

**How It Works:**
```
Traditional Blockchain:  Block 1 → Block 2 → Block 3 → Block 4
                         (Sequential, one at a time)

Citrate BlockDAG:       Block 1 ← Block 3 ← Block 5
                          ↓         ↗
                        Block 2 ← Block 4 ← Block 6
                         (Parallel, multiple simultaneous blocks)
```

#### 2. **EVM-Compatible Execution Layer (LVM)**

Citrate's Lattice Virtual Machine (LVM) is **100% EVM-compatible** while adding native AI operations.

**Benefits:**
- **Deploy existing Solidity contracts** without modification
- **Use MetaMask, Hardhat, Foundry**—all Ethereum tools work
- **AI-specific precompiles** for model registration, inference, and verification
- **Dual signature support**: ECDSA (Ethereum) + Ed25519 (native)

**AI Precompiles:**
```solidity
// Register a model on-chain
function registerModel(
    bytes32 modelHash,
    string memory ipfsCID,
    string memory architecture,
    AccessPolicy policy
) external returns (ModelId);

// Run inference with verifiable results
function runInference(
    ModelId modelId,
    bytes memory inputData,
    uint64 maxGas
) external payable returns (bytes memory output);
```

#### 3. **Model Context Protocol (MCP) Integration**

Citrate is the **first blockchain to natively integrate MCP**—Anthropic's open standard for AI model orchestration.

**Benefits:**
- **Standardized API**: OpenAI-compatible and Anthropic-compatible endpoints
- **Model Discovery**: On-chain registry with metadata, versioning, and provenance
- **Verifiable Inference**: Cryptographic proofs that inference ran correctly
- **Resource Management**: Automatic batching, caching, and GPU allocation

**MCP Endpoints:**
```http
POST /v1/chat/completions          # OpenAI-compatible
POST /v1/messages                  # Anthropic-compatible
POST /v1/embeddings                # Embedding generation
GET  /v1/models                    # Model discovery
POST /v1/jobs                      # Async inference jobs
```

---

## Key Features

### For Developers

#### **Deploy AI Models as First-Class Assets**
```solidity
// Models are tokenized, versioned, and tradeable
contract ModelRegistry {
    function deployModel(
        bytes memory weights,      // Model weights (IPFS/Arweave)
        string memory architecture,
        AccessPolicy policy
    ) external returns (ModelId);

    function updateModel(
        ModelId id,
        bytes memory newWeights,
        string memory changelog
    ) external;
}
```

#### **Build with Familiar Tools**
- **Solidity/Vyper**: Standard smart contract languages
- **Hardhat/Foundry**: Full toolchain support
- **MetaMask**: Works out of the box
- **TypeScript SDK**: `npm install @citrate-ai/sdk`
- **Python SDK**: `pip install citrate-sdk`

#### **AI-Native Precompiles**
```solidity
// Tensor operations at L1
function matmul(Tensor A, Tensor B) returns (Tensor);
function softmax(Tensor x) returns (Tensor);

// ZK proof generation
function generateProof(bytes input) returns (bytes proof);
function verifyProof(bytes proof, bytes publicInputs) returns (bool);

// Federated learning
function submitGradient(JobId job, bytes gradient, bytes proof);
```

### For AI Researchers

#### **Decentralized Model Marketplace**
- **Publish models** with automatic versioning and IPFS storage
- **Set access policies**: Public, private, or pay-per-inference
- **Earn revenue** from model usage
- **Track provenance**: Full lineage from base model to fine-tuned variants

#### **Verifiable Training & Inference**
- **Training logs**: Immutable record of training data, hyperparameters, and loss curves
- **Inference proofs**: Cryptographic verification that computation ran correctly
- **Reproducibility**: Pin exact model versions, weights, and runtime environments

#### **LoRA Fine-Tuning Factory**
```typescript
// Deploy a fine-tuned adapter
const lora = await loraFactory.createLoRA({
    baseModel: "llama-3-70b",
    dataset: "ipfs://Qm...",
    rank: 16,
    alpha: 32,
    targetModules: ["q_proj", "v_proj"]
});

// Inference combines base + adapter
const output = await model.infer(lora.id, "Translate to French: Hello");
```

### For Validators

#### **Enhanced Economics**
Validators earn rewards for:
- **Block production**: 10 LATT base reward
- **Inference computation**: Bonus for GPU-based model execution
- **Storage provision**: IPFS pinning and artifact hosting
- **Congestion bonuses**: Dynamic rewards during high network load

#### **Flexible Staking**
- **Minimum stake**: 10,000 LATT
- **Delegation**: Users can delegate to validators
- **Slashing**: Penalties for downtime or malicious behavior
- **Governance**: Vote on protocol upgrades and parameter changes

---

## Technical Architecture

### Layer 1: GhostDAG Consensus

```
┌─────────────────────────────────────────────────────────┐
│  GhostDAG Consensus Layer                               │
│  - BlockDAG structure with parallel block production    │
│  - k-cluster rule for blue set selection                │
│  - BFT checkpoints for finality                         │
│  - Selected parent chain for total ordering             │
└─────────────────────────────────────────────────────────┘
```

**Key Parameters:**
- `k = 18`: Maximum cluster size for blue set
- `max_parents = 10`: Maximum parent blocks per block
- `block_time = 1-2s`: Target block production interval
- `finality = 12s`: BFT checkpoint finality

### Layer 2: Execution Layer (LVM)

```
┌─────────────────────────────────────────────────────────┐
│  Lattice Virtual Machine (LVM)                          │
│  - EVM-compatible bytecode execution                    │
│  - AI precompiles for model operations                  │
│  - Merkle Patricia Trie state storage                   │
│  - Gas metering for AI workloads                        │
└─────────────────────────────────────────────────────────┘
```

**AI Precompiles:**
- `0x1000`: Model registration
- `0x1001`: Inference execution
- `0x1002`: Tensor operations
- `0x1003`: ZK proof generation
- `0x1004`: ZK proof verification

### Layer 3: MCP Orchestration

```
┌─────────────────────────────────────────────────────────┐
│  Model Context Protocol Layer                           │
│  - OpenAI/Anthropic-compatible APIs                     │
│  - Model discovery and routing                          │
│  - Resource allocation and batching                     │
│  - Inference verification                               │
└─────────────────────────────────────────────────────────┘
```

**MCP Services:**
- **Registry**: On-chain model metadata and discovery
- **Router**: Intelligent request routing to available models
- **Scheduler**: GPU resource allocation and queueing
- **Verifier**: Proof generation and validation

---

## Use Cases

### 1. **Decentralized LLM Hosting**
Deploy and monetize AI models without centralized platforms.

**Example:**
```typescript
// Deploy Llama 3 70B
const model = await registry.registerModel({
    name: "Llama 3 70B Instruct",
    weights: "ipfs://Qm...",
    architecture: "transformer",
    accessPolicy: {
        type: "pay-per-inference",
        pricePerToken: 0.0001 // 0.0001 LATT per token
    }
});

// Users pay directly to use your model
const response = await model.infer("Explain quantum computing");
// You earn 0.0001 LATT * num_tokens
```

### 2. **Verifiable AI in DeFi**
AI-powered trading strategies with on-chain verification.

**Example:**
```solidity
// AI oracle for price prediction
contract TradingBot {
    function executeTrade() external {
        // Get AI prediction with proof
        (bytes memory prediction, bytes memory proof) =
            aiOracle.predict(marketData);

        // Verify proof on-chain
        require(verifyProof(proof, prediction), "Invalid prediction");

        // Execute trade based on verified AI output
        dex.swap(tokenIn, tokenOut, amountIn);
    }
}
```

### 3. **Federated Learning Networks**
Train models collaboratively without sharing private data.

**Example:**
```typescript
// Create federated learning job
const job = await federatedLearning.createJob({
    baseModel: "gpt-3.5-turbo",
    task: "medical-diagnosis",
    minContributors: 100,
    reward: 1000 // LATT per contributor
});

// Contributors train locally and submit gradients
await job.submitGradient({
    gradient: localGradient,
    proof: zkProof // Proves training ran correctly
});

// Aggregated model is published on-chain
const improvedModel = await job.finalizeModel();
```

### 4. **AI-Powered NFTs**
NFTs that generate dynamic content using on-chain AI.

**Example:**
```solidity
contract DynamicNFT is ERC721 {
    function tokenURI(uint256 tokenId) public view returns (string) {
        // Generate image from AI based on token properties
        bytes memory image = aiPrecompile.generateImage(
            "cyberpunk cityscape",
            tokenProperties[tokenId]
        );

        return encodeJSON(image);
    }
}
```

---

## Roadmap

### ✅ Phase 1: Foundation (Completed)
- GhostDAG consensus implementation
- EVM-compatible execution layer
- Genesis block and testnet launch

### ✅ Phase 2: Core Infrastructure (Completed)
- Model registry smart contracts
- IPFS/Arweave integration
- Basic MCP endpoints

### ✅ Phase 3: Developer Tools (Completed)
- TypeScript SDK
- Python SDK
- CLI wallet
- Block explorer

### 🚧 Phase 4: Model Marketplace (In Progress - Week 3)
- Model marketplace contracts
- Discovery and search engine
- Rating and review system
- Payment processing

### 📅 Phase 5: Advanced Features (Q1 2025)
- Federated learning framework
- ZK proof integration
- LoRA fine-tuning at scale
- Cross-chain bridges

### 📅 Phase 6: Mainnet Launch (Q2 2025)
- Security audits
- Mainnet genesis
- Validator onboarding
- Token distribution

---

## Tokenomics

### LATT Token Distribution

**Total Supply:** 1,000,000,000 LATT

| Allocation | Amount | Vesting | Purpose |
|------------|--------|---------|---------|
| **Mining Rewards** | 500M (50%) | 10 years | Block production, inference, storage |
| **Ecosystem Fund** | 250M (25%) | 4 years | Grants, partnerships, development |
| **Treasury** | 100M (10%) | 4 years | Protocol development and operations |
| **Team** | 150M (15%) | 4 years (1yr cliff) | Core contributors and advisors |

### Reward Mechanisms

**Block Rewards:**
- Base: 10 LATT per block
- Halving: Every 2.1M blocks (~4 years)
- Inference bonus: +1% per model execution in block
- Congestion bonus: Dynamic adjustment based on network load

**Staking Rewards:**
- Validators: 10,000 LATT minimum stake
- Delegators: Earn proportional share of validator rewards
- APY: 8-15% depending on network participation

---

## Security

### Audits
- **Consensus Layer**: Audited by [Security Firm]
- **Execution Layer**: Audited by [Security Firm]
- **Smart Contracts**: Audited by [Security Firm]

### Bug Bounty
- **Critical**: Up to $100,000
- **High**: Up to $50,000
- **Medium**: Up to $10,000
- **Low**: Up to $1,000

### Responsible Disclosure
Report vulnerabilities to: security@citrate.ai

---

## Community

### Get Involved

**Developers**
- GitHub: https://github.com/citrate-ai/citrate
- Documentation: https://docs.citrate.ai
- Discord: [Developer Channel]

**Validators**
- Testnet: https://testnet.citrate.ai
- Validator Guide: https://docs.citrate.ai/validators
- Telegram: [Validator Channel]

**Researchers**
- Research Forum: https://research.citrate.ai
- Grants Program: https://grants.citrate.ai
- Twitter: @CitrateAI

---

## FAQ

**Q: Is Citrate EVM-compatible?**
A: Yes, 100%. You can deploy any Solidity contract from Ethereum without modification.

**Q: What makes Citrate different from other AI blockchains?**
A: Three things: (1) GhostDAG consensus for 10,000+ TPS, (2) Native MCP integration for standardized AI APIs, (3) AI models as first-class on-chain assets.

**Q: How do I deploy an AI model?**
A: Upload weights to IPFS, then call `ModelRegistry.registerModel()` with the CID. See our [deployment guide](https://docs.citrate.ai/deploy-model).

**Q: What's the gas cost for inference?**
A: Dynamic based on model size and compute. Approximately 0.0001-0.001 LATT per 1K tokens for LLMs.

**Q: Can I run a validator?**
A: Yes! Minimum stake is 10,000 LATT. See our [validator guide](https://docs.citrate.ai/validators).

---

## Call to Action (Footer)

### Ready to Build the Future of AI?

**For Developers:**
```bash
npm install @citrate-ai/sdk
```

**For Validators:**
[Join the Testnet →](https://testnet.citrate.ai)

**For Researchers:**
[Apply for a Grant →](https://grants.citrate.ai)

---

## Legal

© 2024 Citrate Foundation. All rights reserved.

[Privacy Policy](#) | [Terms of Service](#) | [Cookie Policy](#)

Chain ID: 1337 (Testnet) | 1 (Mainnet)
RPC Endpoint: https://rpc.citrate.ai
Explorer: https://explorer.citrate.ai
