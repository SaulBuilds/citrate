# ModelMarketplace Testing Guide

## Overview

This document provides comprehensive testing procedures for the ModelMarketplace smart contract and its GUI integration.

## Contract Deployment

### Prerequisites
- Running Citrate testnet node on localhost:8545
- Private key with testnet funds
- Forge/Foundry installed

### Deploy Contracts
```bash
cd contracts
source .env.testnet

# Deploy all contracts
forge script script/Deploy.s.sol:DeployScript \
  --rpc-url http://localhost:8545 \
  --broadcast \
  --legacy

# Note the deployed addresses:
# - ModelRegistry: 0x...
# - ModelMarketplace: 0x...
```

### Populate with Test Data
```bash
# Update addresses in PopulateMarketplace.s.sol first!

forge script script/PopulateMarketplace.s.sol:PopulateMarketplace \
  --rpc-url http://localhost:8545 \
  --broadcast \
  --legacy
```

## Testing Strategy

### 1. Smart Contract Tests (Foundry)

```bash
cd contracts
forge test -vv

# Specific marketplace tests
forge test --match-contract ModelMarketplaceTest -vvv

# Gas reporting
forge test --gas-report
```

**Key Test Cases**:
- ✅ List model with valid parameters
- ✅ Purchase access with correct payment
- ✅ Update pricing and category
- ✅ Feature model (requires 1 ETH fee)
- ✅ Add reviews and ratings
- ✅ Get marketplace statistics
- ✅ Query by category and featured status

### 2. Backend RPC Integration Tests

**Test eth_call Implementation**:
```bash
# Via curl
curl -X POST http://localhost:8545 \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "method": "eth_call",
    "params": [{
      "to": "0xC9F9A1e0C2822663e31c0fCdF46aF0dc10081423",
      "data": "0x..."
    }, "latest"],
    "id": 1
  }'
```

**Test via Tauri Command**:
- Open GUI developer console
- Execute: `window.__TAURI__.invoke('eth_call', { request: { to: '0x...', data: '0x...' } })`
- Verify response format

### 3. Frontend Integration Tests

**MarketplaceService Tests**:
```typescript
// In browser console
import { initMarketplaceService } from './utils/marketplaceService';

const service = await initMarketplaceService();

// Test getListing
const listing = await service.getListing('0x...');
console.log('Listing:', listing);

// Test getMarketplaceStats
const stats = await service.getMarketplaceStats();
console.log('Stats:', stats);

// Test getFeaturedModels
const featured = await service.getFeaturedModels();
console.log('Featured:', featured);
```

**UI Component Tests**:
1. Navigate to Marketplace tab in GUI
2. Verify models display correctly
3. Test search and filter functionality
4. Click on model card → verify modal opens
5. Test purchase flow (with test wallet)

### 4. End-to-End Flow Test

**Complete User Journey**:

```
1. Start GUI with testnet connection
   ├─ Verify wallet has funds
   └─ Confirm network: Chain ID 1337

2. Navigate to Marketplace
   ├─ Check loading state displays
   ├─ Verify models load (or mock data with warning)
   └─ Test category filters

3. Browse Model Details
   ├─ Click on model card
   ├─ Verify modal displays all metadata
   ├─ Check pricing information
   └─ Review ratings and downloads

4. Purchase Model Access
   ├─ Click "Purchase Access" button
   ├─ Enter wallet password when prompted
   ├─ Wait for transaction confirmation
   ├─ Verify purchase recorded on-chain
   └─ Check wallet balance decreased

5. List Own Model (Advanced)
   ├─ Navigate to "Publish Model" section
   ├─ Fill in model details
   ├─ Set pricing and category
   ├─ Upload to IPFS (or provide CID)
   ├─ Submit listing transaction
   └─ Verify model appears in marketplace
```

## Known Issues & Workarounds

### Issue: Contract State Not Persisting

**Symptom**: Contracts deploy successfully but aren't available for subsequent calls

**Root Cause**: Testnet node may not have persistence enabled between restarts

**Workaround**:
1. Deploy contracts in same session as testing
2. Use persistent node configuration:
   ```toml
   [storage]
   persist = true
   data_dir = ".citrate-testnet"
   ```
3. Or use mock data for development

### Issue: IPFS Metadata Not Loading

**Symptom**: Model metadata shows placeholder values

**Root Cause**: `fetchModelMetadata()` is stubbed (returns null)

**Solution**:
```typescript
// Implement in marketplaceHelpers.ts
export async function fetchModelMetadata(metadataURI: string) {
  const cid = extractIPFSCid(metadataURI);
  const response = await fetch(`https://ipfs.io/ipfs/${cid}`);
  return await response.json();
}
```

### Issue: Transaction Nonce Conflicts

**Symptom**: "Nonce too low" or "already known" errors

**Solution**:
- Use "pending" tag for `eth_getTransactionCount`
- Clear mempool between tests
- Wait for transaction confirmation before next tx

## Performance Benchmarks

**Target Metrics**:
- Contract deployment: < 30 seconds
- Model listing: < 5 seconds
- Purchase transaction: < 3 seconds
- Read operations (getListing): < 500ms
- UI load time: < 2 seconds

**Gas Usage**:
```
ModelRegistry.registerModel:  ~200,000 gas
ModelMarketplace.listModel:   ~150,000 gas
ModelMarketplace.purchaseAccess: ~100,000 gas
ModelMarketplace.addReview:   ~80,000 gas
```

## Test Data

**Sample Model Listings**:

1. **GPT-4 Fine-tuned**
   - Category: Language Models (0)
   - Price: 0.05 ETH
   - IPFS: QmXoYpizjW3WknFiJnKLwHCnL72vedxjQkDDP1mXWo6uco

2. **Stable Diffusion v3**
   - Category: Image Generation (1)
   - Price: 0.15 ETH
   - IPFS: QmYoZpizjW3WknFiJnKLwHCnL72vedxjQkDDP1mXWo6use

3. **Whisper ASR**
   - Category: Audio Processing (6)
   - Price: 0.02 ETH
   - IPFS: QmZoApizjW3WknFiJnKLwHCnL72vedxjQkDDP1mXWo6usr

## Security Checklist

- [x] Price validation (MIN_PRICE, MAX_PRICE)
- [x] Ownership verification
- [x] Reentrancy protection
- [x] Integer overflow protection (Solidity 0.8+)
- [x] Access control (onlyModelOwner)
- [x] Fee calculation (2.5% marketplace fee)
- [ ] Rate limiting (TODO: implement on frontend)
- [ ] Input sanitization (TODO: validate IPFS CIDs)

## Troubleshooting

**Contract not found**:
```bash
# Check contract exists
curl -X POST http://localhost:8545 \
  -d '{"jsonrpc":"2.0","method":"eth_getCode","params":["0xCONTRACT_ADDRESS", "latest"],"id":1}'

# Should return bytecode, not "0x"
```

**Transaction reverts**:
```bash
# Check transaction receipt
curl -X POST http://localhost:8545 \
  -d '{"jsonrpc":"2.0","method":"eth_getTransactionReceipt","params":["0xTX_HASH"],"id":1}'

# Look for "status": "0x0" (failed)
```

**GUI not connecting**:
1. Verify node is running: `curl http://localhost:8545`
2. Check browser console for errors
3. Confirm contract address in marketplaceService.ts
4. Try connecting to external RPC in GUI settings

## Next Steps

After successful testing:

1. **Mainnet Preparation**
   - Security audit
   - Gas optimization
   - Frontend hardening

2. **Feature Additions**
   - IPFS integration (pin metadata)
   - Advanced search/filters
   - Model versioning
   - Bulk purchases with discounts

3. **Monitoring**
   - Set up Grafana dashboards
   - Track marketplace metrics
   - Monitor gas usage
   - Alert on anomalies
