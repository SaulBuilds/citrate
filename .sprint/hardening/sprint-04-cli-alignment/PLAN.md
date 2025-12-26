# Sprint 04: CLI Signature Alignment

**Status**: Blocked (waiting for Sprint 03)
**Priority**: P2 Medium
**Duration**: 1-2 days
**Depends On**: Sprint 03

---

## Problem Statement

The CLI (`cli/`) uses **secp256k1/ECDSA** for key generation while the wallet (`wallet/`) uses **ed25519**. Accounts created in one cannot be used in the other.

### Current State
- Wallet CLI: Uses `ed25519-dalek` for signing
- General CLI: Uses `secp256k1` + Keccak256 for address derivation
- Result: Incompatible accounts

### Target State
- Both CLI tools use ed25519
- Accounts portable between wallet and CLI
- Consistent address derivation

---

## Work Breakdown

### Task 1: Update CLI Key Generation

**File**: `cli/src/commands/account.rs`

Replace secp256k1 with ed25519:

```rust
use ed25519_dalek::{SigningKey, VerifyingKey};
use rand::rngs::OsRng;

pub fn create_account(password: &str) -> Result<Account> {
    // Generate ed25519 keypair
    let signing_key = SigningKey::generate(&mut OsRng);
    let verifying_key = signing_key.verifying_key();

    // Derive address from public key (Keccak256 of last 20 bytes)
    let pubkey_bytes = verifying_key.as_bytes();
    let address = derive_address(pubkey_bytes);

    // Encrypt and store
    let encrypted = encrypt_key(&signing_key, password)?;
    store_key(&address, &encrypted)?;

    Ok(Account {
        address,
        public_key: hex::encode(pubkey_bytes),
    })
}

fn derive_address(pubkey: &[u8; 32]) -> String {
    // Check if embedded EVM address (20 bytes + 12 zeros)
    if pubkey[20..].iter().all(|&b| b == 0) && !pubkey[..20].iter().all(|&b| b == 0) {
        return format!("0x{}", hex::encode(&pubkey[..20]));
    }

    // Full pubkey: Keccak256 hash last 20 bytes
    let hash = keccak256(pubkey);
    format!("0x{}", hex::encode(&hash[12..]))
}
```

**Acceptance Criteria**:
- [ ] CLI uses ed25519 for key generation
- [ ] Address derivation matches wallet

---

### Task 2: Update CLI Transaction Signing

**File**: `cli/src/commands/transaction.rs`

```rust
use ed25519_dalek::{Signer, SigningKey};

pub fn sign_transaction(tx: &mut Transaction, key: &SigningKey) -> Result<()> {
    // Serialize transaction for signing
    let signable = tx.signable_bytes();

    // Sign with ed25519
    let signature = key.sign(&signable);

    // Store signature
    tx.signature = Signature::new(signature.to_bytes());

    Ok(())
}
```

**Acceptance Criteria**:
- [ ] Transactions signed with ed25519
- [ ] Signatures validate correctly

---

### Task 3: Implement Stub CLI Commands

**File**: `cli/src/commands/model.rs`

Complete stub implementations:

```rust
pub async fn deploy_model(
    rpc_client: &RpcClient,
    model_path: &str,
    metadata: &ModelMetadata,
) -> Result<String> {
    // Read model file
    let model_data = std::fs::read(model_path)?;

    // Upload to IPFS
    let cid = rpc_client.call("ipfs_add", vec![base64::encode(&model_data)]).await?;

    // Deploy via RPC
    let model_id = rpc_client.call("citrate_deployModel", DeployModelRequest {
        name: metadata.name.clone(),
        version: metadata.version.clone(),
        ipfs_cid: cid,
        metadata: serde_json::to_value(metadata)?,
    }).await?;

    Ok(model_id)
}

pub async fn run_inference(
    rpc_client: &RpcClient,
    model_id: &str,
    input: &str,
) -> Result<InferenceResult> {
    rpc_client.call("citrate_runInference", vec![
        model_id.to_string(),
        input.to_string(),
    ]).await
}

pub async fn list_models(
    rpc_client: &RpcClient,
    owner: Option<&str>,
    limit: u32,
) -> Result<Vec<ModelInfo>> {
    rpc_client.call("citrate_listModels", ListModelsRequest {
        owner: owner.map(|s| s.to_string()),
        limit,
    }).await
}
```

**Acceptance Criteria**:
- [ ] `model deploy` works end-to-end
- [ ] `model inference` works end-to-end
- [ ] `model list` works end-to-end

---

### Task 4: Implement Contract CLI Commands

**File**: `cli/src/commands/contract.rs`

```rust
pub async fn deploy_contract(
    rpc_client: &RpcClient,
    bytecode_path: &str,
    constructor_args: Option<&str>,
    account: &Account,
    password: &str,
) -> Result<String> {
    // Read bytecode
    let bytecode = std::fs::read_to_string(bytecode_path)?;

    // Parse constructor args if provided
    let data = if let Some(args) = constructor_args {
        let args: Vec<Value> = serde_json::from_str(args)?;
        encode_constructor(bytecode, args)?
    } else {
        hex::decode(bytecode.trim_start_matches("0x"))?
    };

    // Build transaction
    let nonce = rpc_client.call("eth_getTransactionCount", vec![
        account.address.clone(),
        "pending".to_string(),
    ]).await?;

    let tx = Transaction {
        nonce: parse_hex_u64(&nonce)?,
        to: None, // Contract creation
        value: 0,
        data,
        gas_limit: 3_000_000,
        gas_price: 1_000_000_000, // 1 gwei
        ..Default::default()
    };

    // Sign and send
    let signed = sign_transaction(tx, account, password)?;
    let tx_hash = rpc_client.call("eth_sendRawTransaction", vec![
        hex::encode(&signed),
    ]).await?;

    // Wait for receipt
    let receipt = wait_for_receipt(rpc_client, &tx_hash).await?;

    Ok(receipt.contract_address.unwrap())
}
```

**Acceptance Criteria**:
- [ ] `contract deploy` works end-to-end
- [ ] `contract call` works end-to-end
- [ ] `contract read` works end-to-end

---

### Task 5: Update Cargo.toml Dependencies

**File**: `cli/Cargo.toml`

Uncomment and align dependencies:

```toml
[dependencies]
ed25519-dalek = "2.0"
rand = "0.8"
sha3 = "0.10"
# Remove or feature-gate secp256k1
# secp256k1 = { version = "0.27", optional = true }

# Uncomment local dependencies
citrate-primitives = { path = "../core/primitives" }
citrate-api = { path = "../core/api" }
```

**Acceptance Criteria**:
- [ ] CLI builds with ed25519
- [ ] No secp256k1 dependency in default features

---

## Testing Checklist

### Cross-Compatibility Test
```bash
# Create account in wallet
./wallet new --alias test-account

# Import in CLI
./citrate account import --key <exported-key>

# Send transaction via CLI
./citrate send --from test-account --to <addr> --amount 1

# Verify in wallet
./wallet balance test-account
```

### CLI Command Tests
```bash
# Account commands
./citrate account create
./citrate account list
./citrate account balance <addr>

# Network commands
./citrate network status
./citrate network block latest

# Model commands (if RPC available)
./citrate model list
```

---

## Files Modified

| File | Changes |
|------|---------|
| `cli/Cargo.toml` | Update dependencies |
| `cli/src/commands/account.rs` | Use ed25519 |
| `cli/src/commands/transaction.rs` | Use ed25519 signing |
| `cli/src/commands/model.rs` | Complete implementations |
| `cli/src/commands/contract.rs` | Complete implementations |

---

## Definition of Done

- [ ] CLI uses ed25519 consistently
- [ ] Wallet accounts work in CLI
- [ ] CLI accounts work in wallet
- [ ] All CLI commands functional
- [ ] Git commit: "Sprint 04: CLI Signature Alignment"
