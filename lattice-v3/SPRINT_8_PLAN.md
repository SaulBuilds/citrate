# Sprint 8: Integration & Testing

## Overview
Sprint 8 focuses on integration testing, security hardening, and preparing for devnet launch.

## Audit Findings Addressed (from Sprint 7 audit)

### Critical Fixes Applied ✅
1. **GhostDAG blue-set recursion** - Fixed recursive calculation via DagStore
2. **JSON-RPC wiring** - All methods now registered in IoHandler
3. **Monorepo structure** - Fixed Git layout, removed nested repos

### Remaining Security Gaps to Address

#### High Priority
1. **Signature Verification** 
   - Current: Placeholder signatures ([1; 64])
   - Need: Ed25519 real cryptographic signatures
   - Files: `validator.rs`, `mempool.rs`, `transaction.rs`

2. **Node Crate Compatibility**
   - Current: Uses old API (storage.chain, storage.dag)
   - Need: Update to new StorageManager API
   - Files: `node/src/genesis.rs`, `node/src/producer.rs`

3. **Network Transport**
   - Current: In-memory channels only
   - Need: Real TCP/QUIC transport
   - Files: `network/src/peer.rs`, `network/src/protocol.rs`

#### Medium Priority
4. **Transaction Encoding**
   - Standardize canonical format for signing
   - Add RLP or bincode serialization

5. **API Security**
   - Add rate limiting
   - Input validation on all endpoints
   - CORS configuration

## Sprint 8 Objectives

### Phase 1: Security Hardening (Days 1-3)
- [ ] Implement Ed25519 signature verification
- [ ] Add transaction signature tests
- [ ] Update mempool validation
- [ ] Secure API endpoints

### Phase 2: Node Integration (Days 4-5)
- [ ] Update node crate to new APIs
- [ ] Fix genesis block creation
- [ ] Update block producer
- [ ] Add devnet configuration

### Phase 3: Integration Testing (Days 6-7)
- [ ] End-to-end transaction flow
- [ ] Block production test
- [ ] State transitions test
- [ ] API integration tests

### Phase 4: Devnet Preparation (Days 8-10)
- [ ] Single-node devnet launch
- [ ] Basic block production
- [ ] Transaction submission via RPC
- [ ] State queries working

## Implementation Details

### 1. Signature Verification

```rust
// Add to core/consensus/src/crypto.rs
use ed25519_dalek::{Keypair, PublicKey, Signature, Verifier};

pub fn verify_transaction(tx: &Transaction) -> Result<bool, CryptoError> {
    let message = canonical_tx_bytes(tx)?;
    let public_key = PublicKey::from_bytes(&tx.from.0)?;
    let signature = Signature::from_bytes(&tx.signature.0)?;
    Ok(public_key.verify(&message, &signature).is_ok())
}

fn canonical_tx_bytes(tx: &Transaction) -> Vec<u8> {
    // Serialize without signature field
    let mut data = Vec::new();
    data.extend_from_slice(&tx.nonce.to_le_bytes());
    data.extend_from_slice(&tx.from.0);
    if let Some(to) = &tx.to {
        data.extend_from_slice(&to.0);
    }
    data.extend_from_slice(&tx.value.to_le_bytes());
    data.extend_from_slice(&tx.gas_limit.to_le_bytes());
    data.extend_from_slice(&tx.gas_price.to_le_bytes());
    data.extend_from_slice(&tx.data);
    data
}
```

### 2. Node Crate Updates

```rust
// Update node/src/producer.rs
impl BlockProducer {
    async fn produce_block(&self) -> Result<Hash> {
        // Use new StorageManager API
        let last_block = self.storage.blocks
            .get_latest_block()?
            .unwrap_or_else(|| self.create_genesis());
        
        // Use new header format
        let header = BlockHeader {
            version: 1,
            block_hash: Hash::default(),
            parent_hashes: vec![last_block.header.block_hash],
            height: last_block.header.height + 1,
            // ... rest of fields
        };
        
        // Store via new API
        self.storage.blocks.put_block(&block)?;
    }
}
```

### 3. Testing Matrix

| Component | Test Type | Coverage Target |
|-----------|-----------|----------------|
| Consensus | Unit | 90% |
| Mempool | Integration | 85% |
| Execution | Unit + Integration | 80% |
| Storage | Unit | 90% |
| API | Integration | 75% |
| Node | E2E | 70% |

### 4. Devnet Launch Checklist

- [ ] Genesis block creation working
- [ ] Block producer loop running
- [ ] RPC server accessible
- [ ] Can submit transactions
- [ ] Can query state
- [ ] Blocks being produced every 2s
- [ ] Transaction included in blocks
- [ ] State transitions applied

## Success Criteria

1. **Security**: All signatures verified, no placeholder crypto
2. **Integration**: All components work together
3. **Testing**: 80+ tests passing, including integration tests
4. **Devnet**: Single node producing blocks with transactions
5. **API**: All RPC methods functional

## Risks & Mitigations

| Risk | Impact | Mitigation |
|------|--------|------------|
| Signature integration breaks existing tests | High | Add backward compat flag |
| Node crate requires major refactor | Medium | Focus on minimal viable producer |
| Network transport complex | Low | Defer to Sprint 9 |
| Performance regression | Medium | Profile and optimize hot paths |

## Next Sprint Preview (Sprint 9)

- Multi-node networking
- P2P protocol implementation  
- Consensus with multiple validators
- Load testing and optimization