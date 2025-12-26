# Sprint 03: Missing RPC Endpoints

**Status**: Blocked (waiting for Sprint 02)
**Priority**: P1 High
**Duration**: 2-3 days
**Depends On**: Sprint 02

---

## Problem Statement

The JavaScript SDK assumes several RPC endpoints that don't exist in the actual implementation. Model management, artifact pinning, and DAG statistics methods are missing, causing SDK calls to fail.

### Missing Methods (SDK expects these)
- `citrate_deployModel` - Deploy AI model
- `citrate_runInference` - Run model inference
- `citrate_getModel` - Get model info
- `citrate_listModels` - List all models
- `citrate_getDagStats` - Get DAG statistics
- `citrate_pinArtifact` - Pin to IPFS
- `citrate_getArtifactStatus` - Get IPFS pin status
- `citrate_verifyContract` - Verify contract source

---

## Work Breakdown

### Task 1: Model Management RPCs

**File**: `core/api/src/server.rs`

```rust
pub async fn citrate_deploy_model(
    &self,
    request: DeployModelRequest,
) -> Result<String, JsonRpcError> {
    let model_id = self.model_registry
        .deploy(request)
        .await
        .map_err(|e| JsonRpcError::internal_error(e.to_string()))?;
    Ok(format!("0x{}", hex::encode(model_id)))
}

pub async fn citrate_run_inference(
    &self,
    model_id: String,
    input: String,
    max_gas: Option<u64>,
) -> Result<InferenceResult, JsonRpcError> {
    let id = parse_model_id(&model_id)?;
    let result = self.model_executor
        .run_inference(id, input.as_bytes(), max_gas.unwrap_or(1_000_000))
        .await
        .map_err(|e| JsonRpcError::internal_error(e.to_string()))?;
    Ok(result)
}

pub async fn citrate_get_model(
    &self,
    model_id: String,
) -> Result<ModelInfo, JsonRpcError> {
    let id = parse_model_id(&model_id)?;
    self.model_registry
        .get(id)
        .await
        .map_err(|e| JsonRpcError::internal_error(e.to_string()))
}

pub async fn citrate_list_models(
    &self,
    offset: Option<u32>,
    limit: Option<u32>,
) -> Result<Vec<ModelInfo>, JsonRpcError> {
    self.model_registry
        .list(offset.unwrap_or(0), limit.unwrap_or(100))
        .await
        .map_err(|e| JsonRpcError::internal_error(e.to_string()))
}
```

**Acceptance Criteria**:
- [ ] `citrate_deployModel` deploys and returns model ID
- [ ] `citrate_runInference` executes model and returns result
- [ ] `citrate_getModel` returns model metadata
- [ ] `citrate_listModels` returns paginated list

---

### Task 2: DAG Statistics RPC

**File**: `core/api/src/eth_rpc.rs`

```rust
pub async fn citrate_get_dag_stats(&self) -> Result<DagStats, JsonRpcError> {
    let tips = self.dag_store.get_tips().await;
    let height = self.dag_store.get_height().await;
    let blue_score = self.ghostdag.get_virtual_blue_score().await;

    Ok(DagStats {
        tip_count: tips.len(),
        tips: tips.iter().map(|h| format!("0x{}", hex::encode(h))).collect(),
        height,
        blue_score,
        k_parameter: self.ghostdag.params().k,
        max_parents: self.ghostdag.params().max_parents,
        finality_depth: self.ghostdag.params().finality_depth,
    })
}

#[derive(Serialize)]
pub struct DagStats {
    tip_count: usize,
    tips: Vec<String>,
    height: u64,
    blue_score: u64,
    k_parameter: u32,
    max_parents: usize,
    finality_depth: u64,
}
```

**Acceptance Criteria**:
- [ ] Returns current DAG tip count
- [ ] Returns tip hashes
- [ ] Returns GhostDAG parameters
- [ ] Returns current blue score

---

### Task 3: Artifact/IPFS RPCs

**File**: `core/api/src/server.rs`

```rust
pub async fn citrate_pin_artifact(
    &self,
    cid: String,
    replicas: Option<u32>,
) -> Result<ArtifactPinResult, JsonRpcError> {
    self.ipfs_client
        .pin(&cid, replicas.unwrap_or(3))
        .await
        .map_err(|e| JsonRpcError::internal_error(e.to_string()))
}

pub async fn citrate_get_artifact_status(
    &self,
    cid: String,
) -> Result<ArtifactStatus, JsonRpcError> {
    self.ipfs_client
        .get_status(&cid)
        .await
        .map_err(|e| JsonRpcError::internal_error(e.to_string()))
}

pub async fn citrate_list_model_artifacts(
    &self,
    model_id: String,
) -> Result<Vec<String>, JsonRpcError> {
    let id = parse_model_id(&model_id)?;
    self.artifact_store
        .list_for_model(id)
        .await
        .map_err(|e| JsonRpcError::internal_error(e.to_string()))
}
```

**Acceptance Criteria**:
- [ ] `citrate_pinArtifact` pins CID to IPFS
- [ ] `citrate_getArtifactStatus` returns pin status
- [ ] `citrate_listModelArtifacts` returns CIDs for model

---

### Task 4: Contract Verification RPC

**File**: `core/api/src/server.rs`

```rust
pub async fn citrate_verify_contract(
    &self,
    address: String,
    source_code: String,
    compiler_version: String,
    optimization_enabled: bool,
) -> Result<VerificationResult, JsonRpcError> {
    let addr = parse_address(&address)?;

    // Get deployed bytecode
    let deployed = self.executor.get_code(&addr);

    // Compile source with specified version
    let compiled = self.solc_compiler
        .compile(&source_code, &compiler_version, optimization_enabled)
        .map_err(|e| JsonRpcError::internal_error(e.to_string()))?;

    // Compare bytecode (ignoring metadata)
    let matches = bytecode_matches(&deployed, &compiled.bytecode);

    if matches {
        // Store verification
        self.verification_store.store(VerificationRecord {
            address: addr,
            source_code: source_code.clone(),
            compiler_version: compiler_version.clone(),
            optimization_enabled,
            verified_at: chrono::Utc::now(),
        })?;
    }

    Ok(VerificationResult {
        verified: matches,
        address,
        compiler_version,
    })
}
```

**Acceptance Criteria**:
- [ ] Compiles source code
- [ ] Compares with deployed bytecode
- [ ] Stores verification record
- [ ] Returns verification status

---

### Task 5: Update SDK to Use New Endpoints

**File**: `sdk/javascript/src/model.ts`

Verify SDK methods now work:

```typescript
// These should now work
const modelId = await sdk.models.deploy(modelData, metadata);
const result = await sdk.models.runInference(modelId, input);
const info = await sdk.models.getModel(modelId);
const list = await sdk.models.listModels();
```

**Acceptance Criteria**:
- [ ] SDK model methods work end-to-end
- [ ] SDK tests pass

---

## Testing Checklist

### RPC Tests
```bash
# Test model RPCs
curl -X POST localhost:8545 -d '{"jsonrpc":"2.0","method":"citrate_listModels","params":[],"id":1}'

# Test DAG stats
curl -X POST localhost:8545 -d '{"jsonrpc":"2.0","method":"citrate_getDagStats","params":[],"id":1}'
```

### SDK Tests
```bash
cd sdk/javascript && npm test
```

---

## Files Modified

| File | Changes |
|------|---------|
| `core/api/src/server.rs` | Add 7 RPC methods |
| `core/api/src/eth_rpc.rs` | Add citrate_getDagStats |
| `sdk/javascript/tests/` | Verify tests pass |

---

## Definition of Done

- [ ] All 5 tasks completed
- [ ] All RPC endpoints respond correctly
- [ ] SDK tests pass
- [ ] Git commit: "Sprint 03: Missing RPC Endpoints"
