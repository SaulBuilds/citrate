# Citrate v3 Deployment Action Plan

## Phase 1: Fix Explorer & RPC Connection (Immediate)

### 1.1 Fix Explorer Connection to Local Node
```bash
# Current issue: Explorer expecting standard Ethereum RPC at port 8545
# Citrate node running custom RPC implementation
```

**Actions:**
- [ ] Update Citrate node RPC to implement required Ethereum methods:
  - `eth_blockNumber` - Get latest block number
  - `eth_getBlockByNumber` - Get block by number
  - `eth_getBlockByHash` - Get block by hash
  - `eth_getTransactionByHash` - Get transaction details
  - `eth_getTransactionReceipt` - Get transaction receipt
  - `eth_chainId` - Return chain ID (1337 for local)
  - `eth_syncing` - Return sync status
  - `net_peerCount` - Return peer count

**Files to modify:**
- `core/api/src/rpc/eth.rs` - Implement Ethereum-compatible RPC methods
- `core/api/src/rpc/mod.rs` - Register RPC methods
- `explorer/.env` - Ensure RPC_ENDPOINT=http://localhost:8545

### 1.2 Start Services Properly
```bash
# Terminal 1: Start Citrate node
cd citrate
cargo build --release -p citrate-node
./target/release/lattice devnet --rpc-port 8545

# Terminal 2: Start PostgreSQL for explorer
docker run -d --name lattice-db \
  -p 5432:5432 \
  -e POSTGRES_PASSWORD=password \
  -e POSTGRES_DB=citrate_explorer \
  postgres:15-alpine

# Terminal 3: Run migrations and start indexer
cd explorer
npx prisma migrate deploy
npm run indexer:dev

# Terminal 4: Start explorer frontend
npm run dev
```

## Phase 2: Genesis Block & Model Integration

### 2.1 Create Genesis Block with Embedded Model
**Location:** `core/genesis/src/lib.rs`

```rust
pub struct GenesisBlock {
    pub block: Block,
    pub model: GenesisModel,
    pub initial_accounts: Vec<(Address, U256)>,
}

pub struct GenesisModel {
    pub model_id: H256,
    pub architecture: "bert-tiny",
    pub weights: Vec<u8>, // Serialized weights (~45MB)
    pub metadata: ModelMetadata,
}
```

**Actions:**
- [ ] Train/obtain BERT-tiny model on clean dataset
- [ ] Serialize model weights to binary format
- [ ] Embed in genesis block
- [ ] Register in ModelRegistry at block 0

### 2.2 Deploy Core Smart Contracts
**Contracts needed:**
```solidity
// contracts/ModelRegistry.sol
contract ModelRegistry {
    mapping(bytes32 => Model) public models;
    
    struct Model {
        address owner;
        string ipfsHash;
        uint256 version;
        ModelType modelType;
        uint256 inferenceCount;
    }
}

// contracts/InferenceRouter.sol
contract InferenceRouter {
    function requestInference(
        bytes32 modelId,
        bytes calldata input
    ) external payable returns (bytes32 requestId);
}

// contracts/LatticeToken.sol
contract CitrateToken is ERC20 {
    uint256 public constant BLOCK_REWARD = 10 ether;
    
    function mintBlockReward(address validator) external {
        _mint(validator, BLOCK_REWARD);
    }
}
```

## Phase 3: Block Rewards & Economics

### 3.1 Implement Block Reward System
**Location:** `core/consensus/src/rewards.rs`

```rust
pub struct BlockReward {
    pub base_reward: U256,        // 10 LATT
    pub inference_bonus: U256,    // 0.1 LATT per inference
    pub model_deployment_bonus: U256, // 1 LATT per model
}

impl BlockReward {
    pub fn calculate_reward(&self, block: &Block) -> U256 {
        let mut total = self.base_reward;
        
        // Add inference bonuses
        total += self.inference_bonus * block.inference_count();
        
        // Add model deployment bonus
        if block.has_model_deployment() {
            total += self.model_deployment_bonus;
        }
        
        total
    }
}
```

**Actions:**
- [ ] Define tokenomics (supply, emission schedule)
- [ ] Implement reward calculation in block production
- [ ] Add reward distribution to validators
- [ ] Create treasury allocation mechanism

### 3.2 Native Token Configuration
```toml
# config/tokenomics.toml
[token]
name = "Lattice"
symbol = "LATT"
decimals = 18
total_supply = "1000000000" # 1 billion

[rewards]
block_reward = "10"
halving_interval = 2100000  # blocks
inference_fee = "0.01"      # per inference
model_registration_fee = "100"

[distribution]
genesis_allocation = "100000000"  # 10% at genesis
team = "150000000"                # 15% team (vested)
ecosystem = "250000000"           # 25% ecosystem
mining_rewards = "500000000"      # 50% mining
```

## Phase 4: Wallet Application

### 4.1 Build Web Wallet
**Tech Stack:** Next.js + ethers.js + MetaMask integration

```typescript
// wallet/src/features/wallet.ts
interface CitrateWallet {
  // Key management
  generateMnemonic(): string;
  importMnemonic(mnemonic: string): void;
  
  // Account operations
  getBalance(address: string): Promise<BigNumber>;
  getInferenceCredits(address: string): Promise<number>;
  
  // Transactions
  sendTransaction(tx: TransactionRequest): Promise<TransactionResponse>;
  requestInference(modelId: string, input: any): Promise<InferenceResult>;
  
  // Model operations
  deployModel(model: ModelData): Promise<string>;
  updateModel(modelId: string, update: ModelUpdate): Promise<void>;
}
```

**Features needed:**
- [ ] HD wallet generation (BIP39/BIP44)
- [ ] Private key encryption & storage
- [ ] Transaction signing
- [ ] Balance display
- [ ] Transaction history
- [ ] Model deployment interface
- [ ] Inference request UI
- [ ] QR code for addresses
- [ ] Export/Import functionality

### 4.2 Mobile Wallet (React Native)
```javascript
// mobile-wallet/src/App.tsx
const CitrateWalletApp = () => {
  // Biometric authentication
  // Secure keychain storage
  // Push notifications for transactions
  // Camera for QR scanning
};
```

## Phase 5: Inference dApp

### 5.1 Frontend Application
**Location:** `apps/inference-dapp/`

```typescript
// Main features
interface InferenceDApp {
  // Model discovery
  browseModels(): ModelListing[];
  searchModels(query: string): ModelListing[];
  
  // Inference requests
  submitInference(request: InferenceRequest): Promise<InferenceResult>;
  trackInference(id: string): InferenceStatus;
  
  // Results visualization
  displayResults(result: InferenceResult): void;
  exportResults(format: 'json' | 'csv'): void;
}
```

**UI Components:**
- [ ] Model marketplace/browser
- [ ] Inference request form
- [ ] Real-time inference status
- [ ] Result visualization
- [ ] Payment/credit management
- [ ] History & analytics

### 5.2 Backend API
```typescript
// apps/inference-api/src/routes/inference.ts
POST /api/inference/request
GET  /api/inference/:id/status
GET  /api/models
GET  /api/models/:id
POST /api/models/:id/inference
```

## Phase 6: Mempool & Transaction Flow

### 6.1 Implement Full Mempool
**Location:** `core/mempool/src/lib.rs`

```rust
pub struct Mempool {
    pending: BTreeMap<U256, Transaction>, // sorted by gas price
    queued: HashMap<Address, Vec<Transaction>>,
    
    // Validation rules
    min_gas_price: U256,
    max_pool_size: usize,
    max_account_slots: usize,
}

impl Mempool {
    pub fn add_transaction(&mut self, tx: Transaction) -> Result<()> {
        // Validate signature
        self.validate_signature(&tx)?;
        
        // Check nonce
        self.validate_nonce(&tx)?;
        
        // Verify balance for gas
        self.validate_balance(&tx)?;
        
        // Add to pool
        self.insert_transaction(tx);
        
        Ok(())
    }
}
```

### 6.2 Transaction Types
```rust
pub enum TransactionType {
    Transfer(TransferTx),
    Deploy(DeployTx),
    Call(CallTx),
    ModelDeploy(ModelDeployTx),
    InferenceRequest(InferenceRequestTx),
    ModelUpdate(ModelUpdateTx),
}
```

## Phase 7: Testing Infrastructure

### 7.1 Faucet Service
**Location:** `services/faucet/`

```typescript
// Testnet faucet for token distribution
interface Faucet {
  requestTokens(address: string): Promise<TxHash>;
  getRateLimit(address: string): RateLimit;
  getBalance(): Promise<BigNumber>;
}

// Rate limiting
const FAUCET_AMOUNT = parseEther("100");
const RATE_LIMIT = "1 request per 24 hours";
```

### 7.2 Integration Tests
```rust
// tests/integration/end_to_end.rs
#[tokio::test]
async fn test_full_transaction_flow() {
    // 1. Start local node
    let node = start_devnet().await;
    
    // 2. Create wallet
    let wallet = Wallet::new();
    
    // 3. Get tokens from faucet
    faucet.request_tokens(wallet.address()).await;
    
    // 4. Deploy model
    let model_id = deploy_model(&wallet, model_data).await;
    
    // 5. Request inference
    let result = request_inference(&wallet, model_id, input).await;
    
    // 6. Verify result
    assert!(result.is_valid());
}
```

## Phase 8: Docker Deployment

### 8.1 Docker Compose Setup
```yaml
# docker-compose.yml
version: '3.8'

services:
  citrate-node:
    build: ./core
    ports:
      - "8545:8545"  # RPC
      - "8546:8546"  # WebSocket
      - "30303:30303" # P2P
    environment:
      - CHAIN_ID=1337
      - NETWORK=devnet
    volumes:
      - ./data:/data

  postgres:
    image: postgres:15-alpine
    environment:
      POSTGRES_DB: citrate_explorer
      POSTGRES_PASSWORD: password
    ports:
      - "5432:5432"
    volumes:
      - postgres_data:/var/lib/postgresql/data

  explorer:
    build: ./explorer
    ports:
      - "3000:3000"
    environment:
      - DATABASE_URL=postgresql://postgres:password@postgres:5432/citrate_explorer
      - RPC_ENDPOINT=http://citrate-node:8545
    depends_on:
      - postgres
      - citrate-node

  indexer:
    build: ./explorer
    command: npm run indexer
    environment:
      - DATABASE_URL=postgresql://postgres:password@postgres:5432/citrate_explorer
      - RPC_ENDPOINT=http://citrate-node:8545
    depends_on:
      - postgres
      - citrate-node

  wallet:
    build: ./apps/wallet
    ports:
      - "3001:3001"
    environment:
      - RPC_ENDPOINT=http://citrate-node:8545

  inference-dapp:
    build: ./apps/inference-dapp
    ports:
      - "3002:3002"
    environment:
      - RPC_ENDPOINT=http://citrate-node:8545
      - MODEL_REGISTRY=0x...

  faucet:
    build: ./services/faucet
    ports:
      - "3003:3003"
    environment:
      - RPC_ENDPOINT=http://citrate-node:8545
      - FAUCET_PRIVATE_KEY=${FAUCET_KEY}

volumes:
  postgres_data:
  chain_data:
```

## Immediate Next Steps (Priority Order)

### Day 1-2: Get Explorer Working
1. Implement missing RPC methods in Citrate node
2. Fix explorer database connection
3. Verify indexer can read blocks
4. Test UI components loading

### Day 3-4: Genesis & Contracts
1. Create genesis block with model
2. Deploy core contracts (Token, Registry, Router)
3. Test contract interactions

### Day 5-6: Wallet & Transactions
1. Build basic wallet UI
2. Implement transaction signing
3. Test token transfers
4. Add faucet service

### Day 7-8: Inference dApp
1. Create model browser UI
2. Implement inference request flow
3. Test end-to-end inference
4. Add result visualization

### Day 9-10: Testing & Polish
1. Write integration tests
2. Create Docker setup
3. Document API endpoints
4. Performance testing

## Commands to Run Now

```bash
# 1. Check current RPC methods
curl -X POST http://localhost:8545 \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","method":"eth_blockNumber","params":[],"id":1}'

# 2. If that fails, we need to implement RPC methods first
cd citrate/core/api
cargo build

# 3. Check if blocks are being produced
curl -X POST http://localhost:8545 \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","method":"citrate_getLatestBlock","params":[],"id":1}'

# 4. Deploy contracts (after RPC is working)
cd citrate/contracts
forge build
forge script script/Deploy.s.sol --rpc-url http://localhost:8545 --broadcast

# 5. Start wallet dev server (after creating it)
cd apps/wallet
npm run dev
```

## Success Metrics

- [ ] Explorer shows real-time blocks from local node
- [ ] Can deploy and query AI models via contracts
- [ ] Wallet can send transactions and check balances
- [ ] Inference requests execute and return results
- [ ] Block rewards distributed correctly
- [ ] Full transaction lifecycle works end-to-end
- [ ] Docker compose brings up entire stack

This plan will get you from current state to a fully functional testnet with transaction capability, block rewards, and AI inference features.