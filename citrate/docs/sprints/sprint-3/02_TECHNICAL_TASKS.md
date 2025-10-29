# Sprint 3: Technical Tasks - Advanced Features & Blockchain Integration

**Sprint Goal:** Implement core blockchain functionality, smart contract interactions, and AI model integration

---

## Day 1: Smart Contract Foundation (6 hours)

### Task 1.1: Create Contracts Component Structure
**Estimated Time:** 2 hours
**Assigned To:** Developer
**Priority:** P0

#### Implementation Steps
1. Create `src/components/Contracts.tsx`
2. Add tab-based navigation:
   - Deploy Contract
   - Interact with Contract
   - My Contracts
3. Integrate with App.tsx navigation
4. Add icon to sidebar (Code/FileCode from lucide-react)

#### Code Structure
```typescript
// src/components/Contracts.tsx
export const Contracts: React.FC = () => {
  const [activeTab, setActiveTab] = useState<'deploy' | 'interact' | 'my-contracts'>('deploy');

  return (
    <div className="contracts">
      <div className="contracts-tabs">
        {/* Tab navigation */}
      </div>
      <div className="contracts-content">
        {activeTab === 'deploy' && <ContractDeployer />}
        {activeTab === 'interact' && <ContractInteraction />}
        {activeTab === 'my-contracts' && <MyContracts />}
      </div>
    </div>
  );
};
```

#### Acceptance Criteria
- [ ] Contracts tab appears in App sidebar
- [ ] Three sub-tabs working with proper routing
- [ ] Responsive layout matches other components
- [ ] Dark mode styles applied

---

### Task 1.2: Contract Editor with Monaco
**Estimated Time:** 2 hours
**Assigned To:** Developer
**Priority:** P0

#### Implementation Steps
1. Install `@monaco-editor/react`
   ```bash
   npm install @monaco-editor/react
   ```
2. Create `src/components/ContractEditor.tsx`
3. Configure Monaco with Solidity syntax highlighting
4. Add example contracts library
5. Implement auto-save to localStorage
6. Add import/export buttons

#### Code Structure
```typescript
// src/components/ContractEditor.tsx
import Editor from '@monaco-editor/react';

export const ContractEditor: React.FC<ContractEditorProps> = ({
  value,
  onChange,
  readonly = false
}) => {
  const { currentTheme } = useTheme();

  return (
    <Editor
      height="500px"
      defaultLanguage="sol"
      theme={currentTheme === 'dark' ? 'vs-dark' : 'vs-light'}
      value={value}
      onChange={onChange}
      options={{
        minimap: { enabled: false },
        fontSize: 14,
        readOnly: readonly,
      }}
    />
  );
};
```

#### Example Contracts
```solidity
// ERC-20 Token
// Simple Storage
// Counter Contract
// Voting Contract
```

#### Acceptance Criteria
- [ ] Monaco editor loads and displays Solidity code
- [ ] Syntax highlighting works correctly
- [ ] Code auto-saves every 30 seconds
- [ ] Theme switches between light/dark
- [ ] Can load example contracts
- [ ] Import .sol files from disk

---

### Task 1.3: Contract Compilation Utility
**Estimated Time:** 2 hours
**Assigned To:** Developer
**Priority:** P0

#### Implementation Steps
1. Create `src/utils/contractCompiler.ts`
2. Option A: Use solc-js (browser-compatible Solidity compiler)
3. Option B: Call Foundry CLI via Tauri backend
4. Parse compilation output (bytecode, ABI, errors)
5. Validate contract size (<24KB after deployment)
6. Estimate deployment gas

#### Code Structure
```typescript
// src/utils/contractCompiler.ts
export interface CompilationResult {
  success: boolean;
  bytecode?: string;
  abi?: any[];
  errors?: CompilationError[];
  warnings?: string[];
  gasEstimate?: number;
  contractSize?: number;
}

export async function compileContract(
  source: string,
  contractName: string
): Promise<CompilationResult> {
  try {
    // Option A: Use solc-js
    const solc = await import('solc');
    const input = {
      language: 'Solidity',
      sources: {
        'Contract.sol': { content: source }
      },
      settings: {
        outputSelection: {
          '*': {
            '*': ['abi', 'evm.bytecode']
          }
        }
      }
    };

    const output = JSON.parse(solc.compile(JSON.stringify(input)));

    // Option B: Call Tauri backend
    // const output = await invoke('compile_contract', { source, contractName });

    return {
      success: !output.errors,
      bytecode: output.contracts['Contract.sol'][contractName].evm.bytecode.object,
      abi: output.contracts['Contract.sol'][contractName].abi,
      errors: output.errors || [],
      gasEstimate: estimateDeploymentGas(bytecode),
      contractSize: bytecode.length / 2, // bytes
    };
  } catch (error) {
    return {
      success: false,
      errors: [{ message: error.message }],
    };
  }
}

function estimateDeploymentGas(bytecode: string): number {
  // Gas estimation:
  // - 32,000 gas base cost
  // - 200 gas per byte of bytecode
  const byteCount = bytecode.length / 2;
  return 32000 + (byteCount * 200);
}
```

#### Acceptance Criteria
- [ ] Compiles valid Solidity contracts
- [ ] Returns bytecode and ABI
- [ ] Shows compilation errors with line numbers
- [ ] Warns if contract >24KB
- [ ] Estimates deployment gas accurately
- [ ] Handles syntax errors gracefully

---

## Day 2: Contract Deployment & Interaction (7 hours)

### Task 2.1: Contract Deployer UI
**Estimated Time:** 2 hours
**Assigned To:** Developer
**Priority:** P0

#### Implementation Steps
1. Create `src/components/ContractDeployer.tsx`
2. Add source editor (using ContractEditor component)
3. Compile button with loading state
4. Constructor parameter form (dynamic based on ABI)
5. Deployment transaction preview
6. Deploy button with confirmation

#### Code Structure
```typescript
// src/components/ContractDeployer.tsx
export const ContractDeployer: React.FC = () => {
  const [sourceCode, setSourceCode] = useState('');
  const [compilationResult, setCompilationResult] = useState<CompilationResult | null>(null);
  const [constructorParams, setConstructorParams] = useState<any[]>([]);
  const [deploying, setDeploying] = useState(false);

  const handleCompile = async () => {
    const result = await compileContract(sourceCode, 'MyContract');
    setCompilationResult(result);
  };

  const handleDeploy = async () => {
    setDeploying(true);
    try {
      const tx = await deployContract(
        compilationResult.bytecode,
        compilationResult.abi,
        constructorParams
      );
      // Track deployment
    } finally {
      setDeploying(false);
    }
  };

  return (
    <div className="contract-deployer">
      <ContractEditor value={sourceCode} onChange={setSourceCode} />
      <button onClick={handleCompile}>Compile</button>

      {compilationResult?.success && (
        <>
          <ConstructorParamsForm
            abi={compilationResult.abi}
            values={constructorParams}
            onChange={setConstructorParams}
          />
          <button onClick={handleDeploy} disabled={deploying}>
            {deploying ? 'Deploying...' : 'Deploy Contract'}
          </button>
        </>
      )}
    </div>
  );
};
```

#### Acceptance Criteria
- [ ] Can compile contract from editor
- [ ] Shows compilation errors clearly
- [ ] Constructor params form generates dynamically
- [ ] Validates all constructor inputs
- [ ] Shows deployment cost estimate
- [ ] Confirms before deploying

---

### Task 2.2: Contract Deployment Integration
**Estimated Time:** 1.5 hours
**Assigned To:** Developer
**Priority:** P0

#### Implementation Steps
1. Create `src/utils/contractDeployer.ts`
2. Use ethers.js ContractFactory
3. Submit deployment transaction via wallet
4. Track transaction status
5. Get deployed contract address from receipt
6. Save to "My Contracts" list in localStorage

#### Code Structure
```typescript
// src/utils/contractDeployer.ts
import { ethers } from 'ethers';
import { CitrateClient } from 'citrate-js';

export interface DeploymentOptions {
  bytecode: string;
  abi: any[];
  constructorArgs?: any[];
  from: string;
  gasLimit?: bigint;
}

export async function deployContract(
  options: DeploymentOptions
): Promise<DeploymentResult> {
  const client = CitrateClient.getInstance();
  const signer = client.getSigner(options.from);

  // Create contract factory
  const factory = new ethers.ContractFactory(
    options.abi,
    options.bytecode,
    signer
  );

  // Deploy contract
  const contract = await factory.deploy(...(options.constructorArgs || []), {
    gasLimit: options.gasLimit,
  });

  // Wait for deployment
  await contract.waitForDeployment();

  const address = await contract.getAddress();

  // Save to storage
  saveDeployedContract({
    address,
    name: 'MyContract',
    abi: options.abi,
    deployer: options.from,
    timestamp: Date.now(),
  });

  return {
    address,
    transactionHash: contract.deploymentTransaction().hash,
  };
}
```

#### Acceptance Criteria
- [ ] Deploys contract successfully
- [ ] Returns contract address
- [ ] Saves to "My Contracts" list
- [ ] Handles deployment failures
- [ ] Shows deployment progress

---

### Task 2.3: Contract Interaction UI
**Estimated Time:** 2 hours
**Assigned To:** Developer
**Priority:** P0

#### Implementation Steps
1. Create `src/components/ContractInteraction.tsx`
2. Contract address input with validation
3. Load ABI (from storage or manual paste)
4. Display functions grouped by type
5. Function call forms (dynamic parameter inputs)
6. Result display area

#### Code Structure
```typescript
// src/components/ContractInteraction.tsx
export const ContractInteraction: React.FC = () => {
  const [contractAddress, setContractAddress] = useState('');
  const [abi, setAbi] = useState<any[] | null>(null);
  const [selectedFunction, setSelectedFunction] = useState<AbiFunction | null>(null);

  const readFunctions = abi?.filter(f => f.stateMutability === 'view' || f.stateMutability === 'pure');
  const writeFunctions = abi?.filter(f => f.stateMutability !== 'view' && f.stateMutability !== 'pure');

  return (
    <div className="contract-interaction">
      <input
        type="text"
        value={contractAddress}
        onChange={(e) => setContractAddress(e.target.value)}
        placeholder="Contract Address (0x...)"
      />

      <div className="functions-list">
        <h3>Read Functions</h3>
        {readFunctions?.map(fn => (
          <FunctionButton key={fn.name} func={fn} onClick={() => setSelectedFunction(fn)} />
        ))}

        <h3>Write Functions</h3>
        {writeFunctions?.map(fn => (
          <FunctionButton key={fn.name} func={fn} onClick={() => setSelectedFunction(fn)} />
        ))}
      </div>

      {selectedFunction && (
        <FunctionCallForm
          contractAddress={contractAddress}
          abi={abi}
          func={selectedFunction}
        />
      )}
    </div>
  );
};
```

#### Acceptance Criteria
- [ ] Loads contract by address
- [ ] Displays all public functions
- [ ] Groups read/write functions
- [ ] Shows function signatures
- [ ] Loads ABI from storage or manual input

---

### Task 2.4: ABI Parser and Function Caller
**Estimated Time:** 1.5 hours
**Assigned To:** Developer
**Priority:** P0

#### Implementation Steps
1. Create `src/utils/abiParser.ts`
2. Parse ABI and extract functions, events, errors
3. Generate parameter input forms
4. Call read functions (no transaction)
5. Call write functions (create transaction)
6. Decode return values

#### Code Structure
```typescript
// src/utils/abiParser.ts
export interface AbiFunction {
  name: string;
  inputs: AbiParameter[];
  outputs: AbiParameter[];
  stateMutability: 'view' | 'pure' | 'nonpayable' | 'payable';
}

export function parseAbi(abi: any[]): {
  functions: AbiFunction[];
  events: AbiEvent[];
  errors: AbiError[];
} {
  return {
    functions: abi.filter(item => item.type === 'function'),
    events: abi.filter(item => item.type === 'event'),
    errors: abi.filter(item => item.type === 'error'),
  };
}

export async function callReadFunction(
  contractAddress: string,
  abi: any[],
  functionName: string,
  args: any[]
): Promise<any> {
  const client = CitrateClient.getInstance();
  const contract = new ethers.Contract(contractAddress, abi, client.getProvider());

  const result = await contract[functionName](...args);
  return formatReturnValue(result);
}

export async function callWriteFunction(
  contractAddress: string,
  abi: any[],
  functionName: string,
  args: any[],
  from: string,
  value?: bigint
): Promise<string> {
  const client = CitrateClient.getInstance();
  const signer = client.getSigner(from);
  const contract = new ethers.Contract(contractAddress, abi, signer);

  const tx = await contract[functionName](...args, { value });
  return tx.hash;
}
```

#### Acceptance Criteria
- [ ] Parses ABI correctly
- [ ] Calls read functions without gas
- [ ] Formats return values properly
- [ ] Creates transactions for write functions
- [ ] Handles function reverts

---

## Day 3: Transaction Management (6.5 hours)

### Task 3.1: Transaction Builder UI
**Estimated Time:** 2 hours
**Assigned To:** Developer
**Priority:** P0

#### Implementation Steps
1. Create `src/components/TransactionBuilder.tsx`
2. Transaction type selector (transfer/contract call/deploy)
3. Dynamic form fields based on type
4. Gas controls (EIP-1559)
5. Transaction preview
6. Integrate with Wallet component

#### Code Structure
```typescript
// src/components/TransactionBuilder.tsx
export const TransactionBuilder: React.FC = () => {
  const [txType, setTxType] = useState<'transfer' | 'call' | 'deploy'>('transfer');
  const [to, setTo] = useState('');
  const [value, setValue] = useState('');
  const [data, setData] = useState('');
  const [maxFeePerGas, setMaxFeePerGas] = useState('');
  const [maxPriorityFee, setMaxPriorityFee] = useState('');
  const [gasLimit, setGasLimit] = useState('');
  const [nonce, setNonce] = useState<number | null>(null);

  const estimateGas = async () => {
    // Estimate gas limit
  };

  return (
    <div className="transaction-builder">
      <select value={txType} onChange={(e) => setTxType(e.target.value)}>
        <option value="transfer">Simple Transfer</option>
        <option value="call">Contract Call</option>
        <option value="deploy">Contract Deploy</option>
      </select>

      {txType !== 'deploy' && (
        <input type="text" value={to} onChange={(e) => setTo(e.target.value)} placeholder="To Address" />
      )}

      <input type="text" value={value} onChange={(e) => setValue(e.target.value)} placeholder="Amount (ETH)" />

      {/* Gas controls */}
      <GasControls
        maxFeePerGas={maxFeePerGas}
        maxPriorityFee={maxPriorityFee}
        gasLimit={gasLimit}
        onMaxFeeChange={setMaxFeePerGas}
        onPriorityFeeChange={setMaxPriorityFee}
        onGasLimitChange={setGasLimit}
      />

      <button onClick={estimateGas}>Estimate Gas</button>
      <button onClick={sendTransaction}>Send Transaction</button>
    </div>
  );
};
```

#### Acceptance Criteria
- [ ] Three transaction types supported
- [ ] All fields validate correctly
- [ ] Gas estimation works
- [ ] Nonce auto-fills from network
- [ ] Preview shows all details

---

### Task 3.2: Transaction Signing and Broadcasting
**Estimated Time:** 1.5 hours
**Assigned To:** Developer
**Priority:** P0

#### Implementation Steps
1. Create `src/utils/transactionManager.ts`
2. Build transaction object
3. Sign with wallet (via citrate-js)
4. Broadcast to network
5. Return transaction hash

#### Code Structure
```typescript
// src/utils/transactionManager.ts
export interface TransactionParams {
  to?: string;
  value?: bigint;
  data?: string;
  maxFeePerGas: bigint;
  maxPriorityFeePerGas: bigint;
  gasLimit: bigint;
  nonce?: number;
}

export async function buildAndSendTransaction(
  from: string,
  params: TransactionParams
): Promise<string> {
  const client = CitrateClient.getInstance();

  // Auto-fill nonce if not provided
  if (params.nonce === undefined) {
    params.nonce = await client.getTransactionCount(from, 'pending');
  }

  const tx = {
    type: 2, // EIP-1559
    chainId: await client.getChainId(),
    from,
    to: params.to || null, // null for contract deployment
    value: params.value || 0n,
    data: params.data || '0x',
    maxFeePerGas: params.maxFeePerGas,
    maxPriorityFeePerGas: params.maxPriorityFeePerGas,
    gasLimit: params.gasLimit,
    nonce: params.nonce,
  };

  const signer = client.getSigner(from);
  const signedTx = await signer.signTransaction(tx);
  const txHash = await client.sendRawTransaction(signedTx);

  // Add to transaction queue
  addToTransactionQueue({
    hash: txHash,
    from,
    to: params.to,
    value: params.value,
    status: 'pending',
    timestamp: Date.now(),
  });

  return txHash;
}
```

#### Acceptance Criteria
- [ ] Builds valid EIP-1559 transactions
- [ ] Signs with wallet key
- [ ] Broadcasts successfully
- [ ] Returns transaction hash
- [ ] Adds to queue automatically

---

### Task 3.3: Transaction Queue with Status Tracking
**Estimated Time:** 2 hours
**Assigned To:** Developer
**Priority:** P0

#### Implementation Steps
1. Create `src/components/TransactionQueue.tsx`
2. Display pending transactions
3. Poll for transaction status
4. Update status indicators
5. Show confirmation count
6. Cancel/speed up transactions

#### Code Structure
```typescript
// src/components/TransactionQueue.tsx
export const TransactionQueue: React.FC = () => {
  const [transactions, setTransactions] = useState<Transaction[]>([]);

  useEffect(() => {
    const pollInterval = setInterval(async () => {
      for (const tx of transactions.filter(t => t.status === 'pending')) {
        const receipt = await client.getTransactionReceipt(tx.hash);
        if (receipt) {
          updateTransactionStatus(tx.hash, receipt.status === 1 ? 'confirmed' : 'failed');
        }
      }
    }, 2000); // Poll every 2 seconds

    return () => clearInterval(pollInterval);
  }, [transactions]);

  return (
    <div className="transaction-queue">
      <h3>Pending Transactions</h3>
      {transactions.map(tx => (
        <div key={tx.hash} className="transaction-item">
          <TransactionStatus status={tx.status} />
          <span>{formatHash(tx.hash)}</span>
          <button onClick={() => speedUpTransaction(tx.hash)}>Speed Up</button>
        </div>
      ))}
    </div>
  );
};
```

#### Acceptance Criteria
- [ ] Shows all pending transactions
- [ ] Updates status automatically
- [ ] Displays confirmation count
- [ ] Speed up works (replaces with higher gas)
- [ ] Cancel works (if possible)

---

### Task 3.4: Transaction Receipts and History
**Estimated Time:** 1 hour
**Assigned To:** Developer
**Priority:** P1

#### Implementation Steps
1. Create `src/components/TransactionReceipt.tsx`
2. Display receipt details
3. Show gas used vs estimated
4. Link to block explorer
5. Transaction history with filtering

#### Acceptance Criteria
- [ ] Receipts show all details
- [ ] History paginated
- [ ] Filter by status/type/date
- [ ] Export to CSV

---

## Day 4: DAG Enhancements (6 hours)

### Task 4.1: WebSocket Live Block Updates
**Estimated Time:** 1.5 hours
**Assigned To:** Developer
**Priority:** P0

#### Implementation Steps
1. Add WebSocket subscription in Tauri backend
2. Subscribe to `new_block` events
3. Update DAG visualization on new blocks
4. Handle reconnection logic

#### Code Structure
```rust
// src-tauri/src/websocket.rs
pub async fn subscribe_to_blocks(window: Window) -> Result<()> {
    let ws = connect_websocket().await?;

    while let Some(msg) = ws.next().await {
        if let Ok(block) = parse_block_event(msg) {
            window.emit("new_block", block)?;
        }
    }

    Ok(())
}
```

```typescript
// src/components/DAGVisualization.tsx
useEffect(() => {
  const unlisten = listen('new_block', (event) => {
    addBlockToDAG(event.payload);
  });

  return () => {
    unlisten.then(fn => fn());
  };
}, []);
```

#### Acceptance Criteria
- [ ] WebSocket connects on mount
- [ ] New blocks added in real-time
- [ ] Reconnects on disconnect
- [ ] No duplicate blocks

---

### Task 4.2: Block Filtering and Search
**Estimated Time:** 1.5 hours
**Assigned To:** Developer
**Priority:** P1

#### Implementation Steps
1. Create `src/components/DAGFilters.tsx`
2. Filter by block type (blue/red/all)
3. Search by hash or height
4. Apply filters to visualization

#### Acceptance Criteria
- [ ] Filters work correctly
- [ ] Search highlights results
- [ ] Performance remains good with filters

---

### Task 4.3: Block Details Panel
**Estimated Time:** 1 hour
**Assigned To:** Developer
**Priority:** P1

#### Implementation Steps
1. Create `src/components/BlockDetails.tsx`
2. Show on block click
3. Display all block fields
4. Navigate to parents/children

#### Acceptance Criteria
- [ ] Panel shows all details
- [ ] Links work correctly
- [ ] Copy hash to clipboard

---

### Task 4.4: Optimize DAG Rendering
**Estimated Time:** 2 hours
**Assigned To:** Developer
**Priority:** P0

#### Implementation Steps
1. Switch to Canvas rendering for large DAGs
2. Implement virtual rendering (only visible blocks)
3. Use Web Worker for layout calculations
4. Add zoom/pan controls

#### Acceptance Criteria
- [ ] Handles 10,000+ blocks
- [ ] Maintains 60fps
- [ ] Smooth zoom and pan

---

## Day 5: AI Integration & Polish (8 hours)

### Task 5.1: Model Browser UI
**Estimated Time:** 1.5 hours
**Assigned To:** Developer
**Priority:** P2

#### Implementation Steps
1. Create `src/components/ModelBrowser.tsx`
2. Query ModelRegistry contract
3. Display model list with metadata
4. Search and filter

#### Acceptance Criteria
- [ ] Lists all models
- [ ] Shows metadata
- [ ] Search works
- [ ] Pagination implemented

---

### Task 5.2: Inference Runner
**Estimated Time:** 1.5 hours
**Assigned To:** Developer
**Priority:** P2

#### Implementation Steps
1. Create `src/components/InferenceRunner.tsx`
2. Input form for model parameters
3. Submit inference request
4. Display results

#### Acceptance Criteria
- [ ] Can submit inference
- [ ] Shows results
- [ ] Displays cost

---

### Task 5.3: Integration Testing
**Estimated Time:** 2 hours
**Assigned To:** Developer
**Priority:** P0

#### Test Cases
- [ ] Deploy ERC-20 contract
- [ ] Call transfer function
- [ ] Track transaction to finality
- [ ] View live DAG updates
- [ ] Load and query AI model

---

### Task 5.4: Bug Fixes and Polish
**Estimated Time:** 2 hours
**Assigned To:** Developer
**Priority:** P0

#### Focus Areas
- [ ] Fix any discovered bugs
- [ ] Improve error messages
- [ ] Polish UI/UX
- [ ] Performance tuning

---

### Task 5.5: Documentation
**Estimated Time:** 1 hour
**Assigned To:** Developer
**Priority:** P1

#### Deliverables
- [ ] Update README with new features
- [ ] Document contract deployment flow
- [ ] Add screenshots to docs
- [ ] Update CHANGELOG

---

## Technical Dependencies

### NPM Packages
```bash
npm install @monaco-editor/react  # Code editor
npm install solc                   # Solidity compiler (optional)
npm install d3                     # DAG visualization
npm install recharts               # Charts (optional)
```

### Tauri Commands Needed
```rust
// src-tauri/src/lib.rs
#[tauri::command]
async fn compile_contract(source: String, name: String) -> Result<CompilationResult>;

#[tauri::command]
async fn subscribe_to_blocks() -> Result<()>;
```

---

## Performance Targets

| Operation | Target | Measurement |
|-----------|--------|-------------|
| Contract Compilation | <3s | Time from click to result |
| Contract Deployment | <5s | Time to get contract address |
| Function Call (read) | <500ms | Time to display result |
| Transaction Send | <200ms | Time to get tx hash |
| DAG Update (new block) | <100ms | Time from event to render |
| Model Inference | <2s | Depends on model size |

---

## Code Quality Checklist

- [ ] All components have TypeScript types
- [ ] Error boundaries wrap async operations
- [ ] Loading states for all async actions
- [ ] Input validation on all forms
- [ ] Accessibility labels on all inputs
- [ ] Dark mode styles applied
- [ ] Responsive design for mobile
- [ ] Unit tests for utilities
- [ ] Integration tests for flows
- [ ] No console warnings
- [ ] ESLint passes
- [ ] Build succeeds

---

**Document Version:** 1.0
**Last Updated:** February 11, 2026
**Status:** ✅ Ready for Implementation
