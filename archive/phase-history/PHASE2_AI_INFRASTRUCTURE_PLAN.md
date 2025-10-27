# Phase 2: AI Infrastructure Development Plan

## Executive Summary

**Goal**: Transform Citrate from a blockchain into an AI-native compute network
**Duration**: 8 weeks (Weeks 5-12 of overall roadmap)
**Outcome**: Functional AI model storage, registry, and inference on blockchain

---

## Phase 2 Overview

### Core Deliverables
1. **IPFS Integration** - Distributed model storage
2. **Model Registry** - On-chain model management
3. **Inference Precompile** - AI execution in EVM
4. **HuggingFace Integration** - Import popular models
5. **MCP Protocol** - Standardized AI interfaces

### Success Metrics
- [ ] 10+ models registered on-chain
- [ ] 100+ inference requests processed
- [ ] <2 second inference latency
- [ ] 5+ model types supported
- [x] IPFS storage functional

---

## Week 5-6: IPFS Integration

### Objectives
- Integrate IPFS daemon with Citrate nodes
- Implement model weight storage and retrieval
- Create pinning incentive mechanism

### Technical Tasks

#### Task 1: IPFS Daemon Integration
```rust
// core/storage/src/ipfs/mod.rs
pub struct IPFSService {
    api_endpoint: String,
    client: IpfsClient,
    pinned_models: HashMap<Cid, ModelMetadata>,
    pinning_manager: PinningManager,
}

impl IPFSService {
    pub async fn store_model(
        &mut self,
        model_data: Vec<u8>,
        metadata: ModelMetadata,
    ) -> Result<Cid> {
        // 1. Add to IPFS
        let cid = self.client.add(model_data).await?;
        
        // 2. Pin locally
        self.client.pin_add(&cid).await?;
        
        // 3. Track in local storage
        self.pinned_models.insert(cid.clone(), metadata.clone());
        self.pinning_manager.record_pin(
            cid.clone(),
            "local-node".into(),
            metadata.clone(),
            metadata.size_bytes,
        );
        
        Ok(cid)
    }
    
    pub async fn retrieve_model(&self, cid: &Cid) -> Result<Vec<u8>> {
        // 1. Check if pinned locally
        if !self.pinned_models.contains_key(cid) {
            // 2. Fetch from network
            self.client.get(cid).await
        } else {
            // 3. Read from local
            self.client.cat(cid).await
        }
    }
}
```

#### Task 2: Model Chunking System
```rust
// core/storage/src/ipfs/chunking.rs
pub struct ModelChunker {
    chunk_size: usize, // e.g., 256MB chunks
}

impl ModelChunker {
    pub fn chunk_model(model: &[u8]) -> Vec<Chunk> {
        model.chunks(CHUNK_SIZE)
            .enumerate()
            .map(|(i, data)| Chunk {
                index: i,
                data: data.to_vec(),
                hash: blake3::hash(data),
            })
            .collect()
    }
    
    pub fn reconstruct(chunks: Vec<Chunk>) -> Vec<u8> {
        // Sort by index and concatenate
        chunks.sort_by_key(|c| c.index);
        chunks.into_iter()
            .flat_map(|c| c.data)
            .collect()
    }
}
```

#### Task 3: Pinning Incentives
```solidity
// contracts/IPFSIncentives.sol
contract IPFSIncentives {
    mapping(address => uint256) public pinnedStorage;
    mapping(string => address[]) public modelPinners;
    
    function reportPinning(string memory cid, uint256 size) external {
        pinnedStorage[msg.sender] += size;
        modelPinners[cid].push(msg.sender);
        
        // Reward calculation
        uint256 reward = calculateReward(size);
        _mint(msg.sender, reward);
    }
    
    function verifyPinning(string memory cid, address pinner) 
        external view returns (bool) {
        // Challenge-response verification
        return IPFSVerifier.verify(cid, pinner);
    }
}
```

### Deliverables Week 5-6
- [x] IPFS client integrated into node
- [x] Model storage and retrieval working
- [x] Chunking for large models (>1GB)
- [x] Basic pinning rewards
- [x] Integration tests

#### Week 5-6 Status Update (current iteration)
- Added `PinningManager` incentive accounting (`core/storage/src/ipfs/pinning.rs`) and wired summaries into `IPFSService`.
- Extended the node storage pipeline to record local replicas and expose external pin reporting (`core/storage/src/ipfs/mod.rs`).
- Deployed a dedicated pinning incentives smart contract with native reward payouts (`contracts/src/IPFSIncentives.sol`) and corresponding Foundry coverage.
- Added unit coverage to validate reward multipliers and replica tracking in Rust plus new Solidity integration tests.

---

## Week 7-8: Model Registry Smart Contract

### Objectives
- Create on-chain model registry
- Implement model versioning
- Add access control and licensing

### Technical Tasks

#### Task 1: Core Registry Contract
```solidity
// contracts/ModelRegistry.sol
contract ModelRegistry {
    struct Model {
        string name;
        string ipfsCID;
        address owner;
        uint256 version;
        ModelType modelType;
        uint256 size;
        string framework; // tensorflow, pytorch, etc
        LicenseType license;
        uint256 usageCount;
        uint256 totalRewards;
    }
    
    enum ModelType {
        LANGUAGE,
        VISION,
        AUDIO,
        MULTIMODAL,
        CUSTOM
    }
    
    enum LicenseType {
        OPEN_SOURCE,
        COMMERCIAL,
        RESEARCH_ONLY,
        CUSTOM
    }
    
    mapping(bytes32 => Model) public models;
    mapping(address => bytes32[]) public ownerModels;
    mapping(string => bytes32[]) public modelVersions;
    
    event ModelRegistered(
        bytes32 indexed modelId,
        string name,
        address owner,
        string ipfsCID
    );
    
    function registerModel(
        string memory name,
        string memory ipfsCID,
        ModelType modelType,
        string memory framework,
        uint256 size,
        LicenseType license
    ) external returns (bytes32) {
        bytes32 modelId = keccak256(
            abi.encodePacked(name, block.timestamp, msg.sender)
        );
        
        models[modelId] = Model({
            name: name,
            ipfsCID: ipfsCID,
            owner: msg.sender,
            version: 1,
            modelType: modelType,
            size: size,
            framework: framework,
            license: license,
            usageCount: 0,
            totalRewards: 0
        });
        
        ownerModels[msg.sender].push(modelId);
        modelVersions[name].push(modelId);
        
        emit ModelRegistered(modelId, name, msg.sender, ipfsCID);
        return modelId;
    }
}
```

#### Task 2: Model Metadata Extensions
```solidity
// contracts/ModelMetadata.sol
contract ModelMetadata {
    struct DetailedMetadata {
        string description;
        string[] inputShape;
        string[] outputShape;
        uint256 parameters; // number of parameters
        uint256 flops; // FLOPs required
        uint256 minMemory; // minimum GPU memory
        string[] tags;
        string documentation;
    }
    
    mapping(bytes32 => DetailedMetadata) public metadata;
    
    function setMetadata(
        bytes32 modelId,
        DetailedMetadata memory meta
    ) external onlyModelOwner(modelId) {
        metadata[modelId] = meta;
    }
}
```

#### Task 3: Access Control & Payments
```solidity
// contracts/ModelAccess.sol
contract ModelAccess {
    struct AccessTier {
        uint256 price;
        uint256 duration;
        uint256 maxUses;
    }
    
    mapping(bytes32 => AccessTier[]) public modelTiers;
    mapping(address => mapping(bytes32 => uint256)) public userAccess;
    
    function purchaseAccess(
        bytes32 modelId,
        uint256 tierId
    ) external payable {
        AccessTier memory tier = modelTiers[modelId][tierId];
        require(msg.value >= tier.price, "Insufficient payment");
        
        userAccess[msg.sender][modelId] = block.timestamp + tier.duration;
        
        // Transfer payment to model owner
        Model memory model = ModelRegistry(registry).getModel(modelId);
        payable(model.owner).transfer(msg.value * 90 / 100);
        payable(treasury).transfer(msg.value * 10 / 100);
    }
}
```

### Deliverables Week 7-8
- [ ] ModelRegistry.sol deployed
- [ ] Model versioning system
- [ ] Access control implementation
- [ ] Payment/licensing system
- [ ] Web3 integration tests

#### Week 7-8 Status Update (current iteration)
- Executor now persists on-chain registrations into persistent storage and MCP via adapter bridges, including IPFS CIDs for weights (`core/execution/src/executor.rs:1151`).
- MCP registry tracks weight artifacts and `ModelExecutor` fetches model bytes from IPFS or manifests before inference (`core/mcp/src/registry.rs:74`, `core/mcp/src/execution.rs:26`).
- Node runtime wires new storage & registry adapters so registerModel events hydrate StorageManager and MCP (`node/src/adapters.rs:1`, `node/src/main.rs:283`).
- Model inference pipeline can retrieve weight blobs through IPFS, satisfying core registry/execution wiring ahead of Phase 2 inference milestones.
- CLI + RPC pipeline now handles metadata-aware model registration by uploading artifacts to IPFS and passing access policy data to the executor (`cli/src/commands/model.rs`, `core/api/src/server.rs:1060`).

---

## Week 9-10: Inference Precompile

### Objectives
- Create AI inference precompile for EVM
- Implement proof of inference
- Add result verification

### Technical Tasks

#### Task 1: Inference Precompile Implementation
```rust
// core/execution/src/precompiles/inference.rs
pub struct InferencePrecompile {
    model_cache: Arc<ModelCache>,
    executor: Arc<InferenceExecutor>,
}

impl Precompile for InferencePrecompile {
    fn execute(
        &self,
        input: &[u8],
        target_gas: u64,
    ) -> PrecompileResult {
        // 1. Decode input
        let request = InferenceRequest::decode(input)?;
        
        // 2. Load model from cache or IPFS
        let model = self.model_cache
            .get_or_load(&request.model_id)
            .await?;
        
        // 3. Verify user has access
        self.verify_access(
            &request.caller,
            &request.model_id
        )?;
        
        // 4. Execute inference
        let result = self.executor.infer(
            &model,
            &request.input_data,
            request.parameters,
        )?;
        
        // 5. Generate proof
        let proof = self.generate_proof(
            &request,
            &result,
            target_gas,
        )?;
        
        // 6. Return result + proof
        Ok(PrecompileOutput {
            output: result.encode(),
            gas_used: calculate_gas(&model, &request),
            proof: Some(proof),
        })
    }
}
```

#### Task 2: Inference Executor
```rust
// core/execution/src/inference/executor.rs
pub struct InferenceExecutor {
    runtime: ONNXRuntime,
    cache: LruCache<ModelId, LoadedModel>,
}

impl InferenceExecutor {
    pub fn infer(
        &self,
        model: &Model,
        input: &[u8],
        params: InferenceParams,
    ) -> Result<InferenceResult> {
        // 1. Load model into runtime
        let session = self.runtime.create_session(&model.weights)?;
        
        // 2. Prepare input tensors
        let input_tensor = self.prepare_input(input, &model.input_shape)?;
        
        // 3. Run inference
        let output = session.run(
            vec![input_tensor],
            &params,
        )?;
        
        // 4. Post-process output
        let result = self.post_process(
            output,
            &model.output_shape,
            params.output_format,
        )?;
        
        Ok(InferenceResult {
            output: result,
            model_id: model.id,
            timestamp: now(),
            gas_used: self.calculate_gas(model, input),
        })
    }
}
```

#### Task 3: Proof of Inference
```rust
// core/execution/src/inference/proof.rs
pub struct InferenceProof {
    model_hash: Hash,
    input_hash: Hash,
    output_hash: Hash,
    execution_trace: Vec<TraceStep>,
    provider: Address,
    signature: Signature,
}

impl InferenceProof {
    pub fn generate(
        model: &Model,
        input: &[u8],
        output: &[u8],
        trace: Vec<TraceStep>,
    ) -> Self {
        let proof = Self {
            model_hash: hash(&model.weights),
            input_hash: hash(input),
            output_hash: hash(output),
            execution_trace: trace,
            provider: get_node_address(),
            signature: Signature::default(),
        };
        
        // Sign the proof
        proof.signature = sign_proof(&proof);
        proof
    }
    
    pub fn verify(&self) -> bool {
        // 1. Verify signature
        if !verify_signature(&self.signature, &self.provider) {
            return false;
        }
        
        // 2. Verify execution trace
        if !verify_trace(&self.execution_trace) {
            return false;
        }
        
        // 3. Verify hashes match
        true
    }
}
```

### Deliverables Week 9-10
- [ ] Inference precompile implemented
- [ ] ONNX runtime integrated
- [ ] Proof of inference system
- [ ] Gas calculation for AI operations
- [ ] Integration with ModelRegistry

---

## Week 11-12: HuggingFace Integration

### Objectives
- Import models from HuggingFace
- Support multiple model formats
- Create model conversion pipeline

### Technical Tasks

#### Task 1: HuggingFace Importer
```python
# tools/huggingface_import.py
import torch
import onnx
from transformers import AutoModel, AutoTokenizer
import ipfsclient
import web3

class HuggingFaceImporter:
    def __init__(self, ipfs_api, registry_address, web3_provider):
        self.ipfs = ipfsclient.Client(ipfs_api)
        self.registry = ModelRegistry(registry_address, web3_provider)
    
    def import_model(self, model_name: str, model_type: str):
        """Import model from HuggingFace to Citrate"""
        
        # 1. Download from HuggingFace
        print(f"Downloading {model_name}...")
        model = AutoModel.from_pretrained(model_name)
        tokenizer = AutoTokenizer.from_pretrained(model_name)
        
        # 2. Convert to ONNX
        print("Converting to ONNX...")
        onnx_model = self.convert_to_onnx(model, tokenizer)
        
        # 3. Optimize for inference
        print("Optimizing model...")
        optimized = self.optimize_model(onnx_model)
        
        # 4. Upload to IPFS
        print("Uploading to IPFS...")
        model_bytes = optimized.SerializeToString()
        result = self.ipfs.add(model_bytes)
        cid = result['Hash']
        
        # 5. Register on-chain
        print(f"Registering on-chain with CID: {cid}")
        tx_hash = self.registry.register_model(
            name=model_name,
            ipfs_cid=cid,
            model_type=model_type,
            framework="onnx",
            size=len(model_bytes),
            license="apache-2.0"
        )
        
        print(f"Model registered! TX: {tx_hash}")
        return cid, tx_hash
    
    def convert_to_onnx(self, model, tokenizer):
        """Convert PyTorch model to ONNX"""
        # Create dummy input
        dummy_input = tokenizer(
            "Sample text",
            return_tensors="pt",
            padding=True,
            truncation=True,
            max_length=512
        )
        
        # Export to ONNX
        torch.onnx.export(
            model,
            tuple(dummy_input.values()),
            "temp.onnx",
            export_params=True,
            opset_version=13,
            input_names=['input_ids', 'attention_mask'],
            output_names=['output'],
            dynamic_axes={
                'input_ids': {0: 'batch_size', 1: 'sequence'},
                'attention_mask': {0: 'batch_size', 1: 'sequence'},
                'output': {0: 'batch_size', 1: 'sequence'}
            }
        )
        
        return onnx.load("temp.onnx")
```

#### Task 2: Supported Models List
```python
# Initial models to support
SUPPORTED_MODELS = [
    # Language Models
    {
        "name": "gpt2",
        "type": "language",
        "size": "124M",
        "use_case": "Text generation"
    },
    {
        "name": "bert-base-uncased",
        "type": "language",
        "size": "110M",
        "use_case": "Classification, NER"
    },
    {
        "name": "t5-small",
        "type": "language",
        "size": "60M",
        "use_case": "Translation, summarization"
    },
    
    # Vision Models
    {
        "name": "stable-diffusion-v1-5",
        "type": "vision",
        "size": "4GB",
        "use_case": "Image generation"
    },
    {
        "name": "clip-vit-base-patch32",
        "type": "multimodal",
        "size": "150M",
        "use_case": "Image-text matching"
    },
    
    # More to be added...
]
```

#### Task 3: Model Testing Framework
```rust
// core/tests/model_tests.rs
#[tokio::test]
async fn test_gpt2_inference() {
    let model_id = "gpt2_model_hash";
    let input = "The future of AI is";
    
    // Load model
    let model = load_model_from_ipfs(model_id).await.unwrap();
    
    // Run inference
    let result = executor.infer(
        &model,
        input.as_bytes(),
        InferenceParams::default(),
    ).await.unwrap();
    
    // Verify output
    assert!(!result.output.is_empty());
    assert!(result.gas_used > 0);
}

#[tokio::test]
async fn test_bert_classification() {
    let model_id = "bert_model_hash";
    let input = "This movie is amazing!";
    
    let result = executor.infer(
        &load_model(model_id).await.unwrap(),
        input.as_bytes(),
        InferenceParams {
            task: "sentiment",
            ..Default::default()
        },
    ).await.unwrap();
    
    let sentiment = parse_classification(&result.output);
    assert_eq!(sentiment, "positive");
}
```

### Deliverables Week 11-12
- [ ] HuggingFace import tool
- [ ] 5+ models imported and tested
- [ ] ONNX conversion pipeline
- [ ] Model optimization tools
- [ ] End-to-end inference tests

---

## Integration & Testing Plan

### Integration Tests Required

1. **IPFS + Registry**
   - Upload model to IPFS
   - Register CID on-chain
   - Retrieve and verify

2. **Registry + Inference**
   - Register model
   - Request inference
   - Verify access control

3. **Full Pipeline**
   - Import from HuggingFace
   - Store in IPFS
   - Register on-chain
   - Execute inference
   - Verify proof

### Performance Benchmarks

| Model | Size | Load Time | Inference Time | Memory |
|-------|------|-----------|----------------|--------|
| GPT-2 | 124M | <1s | <100ms | 500MB |
| BERT | 110M | <1s | <50ms | 450MB |
| T5-small | 60M | <500ms | <75ms | 250MB |
| CLIP | 150M | <1s | <150ms | 600MB |

---

## Risk Mitigation

### Technical Risks

1. **IPFS Reliability**
   - Risk: IPFS gateway downtime
   - Mitigation: Multiple gateway fallbacks
   - Backup: Direct HTTP model serving

2. **Model Size Limits**
   - Risk: Large models (>1GB) slow to load
   - Mitigation: Chunking and streaming
   - Optimization: Model quantization

3. **Inference Performance**
   - Risk: Slow inference blocking chain
   - Mitigation: Async execution
   - Solution: Off-chain compute with proofs

### Security Risks

1. **Malicious Models**
   - Risk: Models with backdoors
   - Mitigation: Model scanning
   - Verification: Community audits

2. **Data Privacy**
   - Risk: Inference data leakage
   - Mitigation: Encryption
   - Option: TEE execution

---

## Success Criteria

### Must Have (Week 12)
- [ ] IPFS integration working
- [ ] Model registry deployed
- [ ] 5+ models available
- [ ] Basic inference functional
- [ ] Proof system implemented

### Should Have
- [ ] 10+ models imported
- [ ] GUI for model browsing
- [ ] Automated testing suite
- [ ] Performance monitoring

### Could Have
- [ ] Model marketplace UI
- [ ] Advanced optimizations
- [ ] Multi-language SDKs

---

## Timeline Summary

```
Week 5-6:  IPFS Integration
           ├─ IPFS daemon setup
           ├─ Model storage/retrieval
           └─ Pinning incentives

Week 7-8:  Model Registry
           ├─ Smart contract development
           ├─ Access control system
           └─ Payment integration

Week 9-10: Inference Precompile
           ├─ EVM precompile
           ├─ ONNX runtime
           └─ Proof generation

Week 11-12: HuggingFace Integration
            ├─ Import pipeline
            ├─ Model conversion
            └─ Testing & validation
```

---

## Next Steps After Phase 2

### Phase 3 Preview: Distributed Compute (Weeks 13-20)
- GPU node registration
- Compute job marketplace
- Distributed training
- Proof of compute
- Incentive mechanisms

### Phase 4 Preview: Production (Weeks 21-24)
- Public testnet launch
- Website and documentation
- SDK releases
- Community onboarding
- Marketing launch

---

## Conclusion

Phase 2 will transform Citrate into the first truly AI-native blockchain with:
- Decentralized model storage via IPFS
- On-chain model registry and governance
- Native AI inference in smart contracts
- Support for popular ML frameworks
- Verifiable compute with proof system

This foundation enables Phase 3's distributed GPU compute marketplace.

---

*Phase 2 Start Date: Week 5 (after Phase 1 completion)*
*Estimated Completion: Week 12*
*Total Duration: 8 weeks*
