import type { 
  NodeStatus, 
  NodeConfig, 
  Account, 
  DAGData, 
  DAGNode,
  DAGLink,
  BlockDetails,
  TipInfo,
  ModelDeployment,
  InferenceRequest,
  TrainingConfig,
  ModelInfo,
  TransactionRequest,
  PeerInfoSummary,
  TxActivity
} from '../types';
import { rpcClient } from './rpc-client';

// Check if running in Tauri context
const isTauri = () => {
  return typeof window !== 'undefined' && window.__TAURI__ !== undefined;
};

// Import Tauri APIs synchronously when available
let invoke: any;
let listen: any;

// Initialize Tauri APIs immediately if available
if (isTauri() && window.__TAURI__) {
  invoke = window.__TAURI__.core.invoke;
  listen = window.__TAURI__.event.listen;
} else if (isTauri()) {
  // Fallback to dynamic import if __TAURI__ is not ready
  import('@tauri-apps/api/core').then(module => {
    invoke = module.invoke;
  });
  import('@tauri-apps/api/event').then(module => {
    listen = module.listen;
  });
}

// Store for web accounts (persisted in localStorage)
let webAccounts: Account[] = [];

// Helper to serialize BigInt values
const serializeAccount = (account: Account) => {
  return {
    ...account,
    balance: account.balance.toString(),
  };
};

// Helper to deserialize BigInt values
const deserializeAccount = (account: any): Account => {
  return {
    ...account,
    balance: BigInt(account.balance || 0),
  };
};

// Load web accounts from localStorage
const loadWebAccounts = () => {
  const stored = localStorage.getItem('citrate_web_accounts');
  if (stored) {
    try {
      const parsed = JSON.parse(stored);
      webAccounts = parsed.map(deserializeAccount);
    } catch (e) {
      console.warn('Failed to parse stored accounts:', e);
      webAccounts = [];
    }
  }
  return webAccounts;
};

// Save web accounts to localStorage
const saveWebAccounts = () => {
  const serialized = webAccounts.map(serializeAccount);
  localStorage.setItem('citrate_web_accounts', JSON.stringify(serialized));
};

// Initialize accounts
loadWebAccounts();

// Helpers for shape conversion
const mapNodeStatusFromNative = (raw: any): NodeStatus => ({
  running: !!raw.running,
  syncing: !!raw.syncing,
  blockHeight: Number(raw.block_height ?? raw.blockHeight ?? 0),
  peerCount: Number(raw.peer_count ?? raw.peerCount ?? 0),
  networkId: String(raw.network_id ?? raw.networkId ?? 'unknown'),
  version: String(raw.version ?? 'unknown'),
  uptime: Number(raw.uptime ?? 0),
  dagTips: Number(raw.dag_tips ?? raw.dagTips ?? 0),
  blueScore: Number(raw.blue_score ?? raw.blueScore ?? 0),
  lastBlockHash: raw.last_block_hash ?? raw.lastBlockHash ?? undefined,
  lastBlockTimestamp: (() => {
    const ts = raw.last_block_timestamp ?? raw.lastBlockTimestamp;
    if (ts === undefined || ts === null) return undefined;
    const n = Number(ts);
    return n < 2_000_000_000 ? n * 1000 : n;
  })(),
});

const mapDAGDataFromNative = (raw: any): DAGData => {
  const rawTips = raw.tips || [];
  const tipHashSet = new Set<string>((rawTips as any[]).map((t: any) => String(t?.hash ?? t).toLowerCase()));

  const nodes: DAGNode[] = (raw.nodes || []).map((n: any) => ({
    id: String(n.id),
    hash: String(n.hash),
    height: Number(n.height),
    timestamp: Number(n.timestamp) * (Number(n.timestamp) < 2_000_000_000 ? 1000 : 1),
    isBlue: !!(n.is_blue ?? n.isBlue),
    blueScore: Number(n.blue_score ?? n.blueScore ?? 0),
    isTip: tipHashSet.has(String(n.hash).toLowerCase()),
    selectedParent: String(n.selected_parent ?? n.selectedParent ?? ''),
    mergeParents: (n.merge_parents ?? n.mergeParents ?? []).map((x: any) => String(x)),
    transactions: Number(n.transactions ?? 0),
    proposer: String(n.proposer ?? ''),
    size: Number(n.size ?? 0),
  }));

  const links: DAGLink[] = (raw.links || []).map((l: any) => ({
    source: String(l.source),
    target: String(l.target),
    isSelected: !!(l.is_selected ?? l.isSelected),
    linkType: (l.link_type ?? l.linkType ?? 'SelectedParent') as 'SelectedParent' | 'MergeParent',
  }));

  const tips: TipInfo[] = (raw.tips || []).map((t: any) => ({
    hash: String(t.hash),
    height: Number(t.height),
    timestamp: Number(t.timestamp) * (Number(t.timestamp) < 2_000_000_000 ? 1000 : 1),
    blueScore: Number(t.blue_score ?? t.blueScore ?? 0),
    cumulativeWeight: BigInt(t.cumulative_weight ?? t.cumulativeWeight ?? 0),
  }));

  const stats = raw.statistics || {};
  return {
    nodes,
    links,
    tips,
    statistics: {
      totalBlocks: Number(stats.total_blocks ?? stats.totalBlocks ?? nodes.length),
      blueBlocks: Number(stats.blue_blocks ?? stats.blueBlocks ?? nodes.length),
      redBlocks: Number(stats.red_blocks ?? stats.redBlocks ?? 0),
      currentTips: Number(stats.current_tips ?? stats.currentTips ?? tips.length),
      averageBlueScore: Number(stats.average_blue_score ?? stats.averageBlueScore ?? 0),
      maxHeight: Number(stats.max_height ?? stats.maxHeight ?? (nodes.reduce((m, n) => Math.max(m, n.height), 0))),
    },
  };
};

// Web implementations using RPC client
const webImplementations = {
  // Node
  start_node: async () => {
    const connected = await rpcClient.isConnected();
    if (connected) {
      return 'Connected to external node at http://127.0.0.1:8545';
    } else {
      throw new Error('Cannot connect to node at http://127.0.0.1:8545. Make sure the Citrate node is running.');
    }
  },
  
  stop_node: async () => {
    return 'Disconnected from external node';
  },
  
  get_node_status: async () => {
    try {
      const status = await rpcClient.getNodeStatus();
      return status;
    } catch (error) {
      return {
        running: false,
        syncing: false,
        blockHeight: 0,
        peerCount: 0,
        networkId: 'unknown',
        version: 'unknown',
        uptime: 0,
      };
    }
  },
  
  update_node_config: async (args: { config: NodeConfig }) => {
    console.log('Config update requested (not available in web mode):', args.config);
    return 'Config updates not available in web mode';
  },
  
  // Wallet
  create_account: async (args: { label: string, password: string }) => {
    // Generate random private key for demo purposes
    const privateKey = '0x' + Array.from(crypto.getRandomValues(new Uint8Array(32)))
      .map(b => b.toString(16).padStart(2, '0'))
      .join('');
    
    const account: Account = {
      address: '0x' + privateKey.substr(26, 40), // Simplified address derivation
      label: args.label,
      publicKey: '0x' + Array.from(crypto.getRandomValues(new Uint8Array(32)))
        .map(b => b.toString(16).padStart(2, '0'))
        .join(''),
      balance: 0n,
      nonce: 0,
      createdAt: Date.now(),
    };
    
    // Try to get balance from RPC
    try {
      const balance = await rpcClient.getBalance(account.address);
      account.balance = BigInt(balance);
      const nonce = await rpcClient.getTransactionCount(account.address);
      account.nonce = nonce;
    } catch (error) {
      console.warn('Could not fetch account balance from RPC:', error);
    }
    
    webAccounts.push(account);
    saveWebAccounts();
    return account;
  },
  
  import_account: async (args: { privateKey: string, label: string, password: string }) => {
    // Derive address from private key (simplified)
    const cleanKey = args.privateKey.replace('0x', '');
    const address = '0x' + cleanKey.substr(cleanKey.length - 40);
    
    // Check if already exists
    if (webAccounts.find(a => a.address.toLowerCase() === address.toLowerCase())) {
      throw new Error('Account already exists');
    }
    
    const account: Account = {
      address,
      label: args.label,
      publicKey: '0x' + Array.from(crypto.getRandomValues(new Uint8Array(32)))
        .map(b => b.toString(16).padStart(2, '0'))
        .join(''),
      balance: 0n,
      nonce: 0,
      createdAt: Date.now(),
    };
    
    // Try to get balance from RPC
    try {
      const balance = await rpcClient.getBalance(account.address);
      account.balance = BigInt(balance);
      const nonce = await rpcClient.getTransactionCount(account.address);
      account.nonce = nonce;
    } catch (error) {
      console.warn('Could not fetch account balance from RPC:', error);
    }
    
    webAccounts.push(account);
    saveWebAccounts();
    return account;
  },
  
  get_accounts: async () => {
    loadWebAccounts();
    
    // Update balances from RPC
    for (const account of webAccounts) {
      try {
        const balance = await rpcClient.getBalance(account.address);
        account.balance = BigInt(balance);
        const nonce = await rpcClient.getTransactionCount(account.address);
        account.nonce = nonce;
      } catch (error) {
        console.warn(`Could not update balance for ${account.address}:`, error);
      }
    }
    
    saveWebAccounts();
    return webAccounts;
  },
  
  get_account: async (args: { address: string }) => {
    loadWebAccounts();
    const account = webAccounts.find(a => a.address.toLowerCase() === args.address.toLowerCase());
    
    if (account) {
      // Update balance from RPC
      try {
        const balance = await rpcClient.getBalance(account.address);
        account.balance = BigInt(balance);
        const nonce = await rpcClient.getTransactionCount(account.address);
        account.nonce = nonce;
        saveWebAccounts();
      } catch (error) {
        console.warn(`Could not update balance for ${args.address}:`, error);
      }
    }
    
    return account || null;
  },
  
  send_transaction: async (args: { request: TransactionRequest, password: string }) => {
    try {
      // Web mode RPC expects hex numbers; keep value/gasPrice as BigInt here
      const req = {
        ...args.request,
        value: BigInt(args.request.value as any),
        gasPrice: BigInt(args.request.gasPrice as any),
        data: Array.isArray(args.request.data)
          ? '0x' + Array.from(args.request.data).map(b => Number(b).toString(16).padStart(2, '0')).join('')
          : (args.request.data as any),
      } as any;
      const txHash = await rpcClient.sendTransaction(req);
      
      // Update local account nonce
      const account = webAccounts.find(a => a.address.toLowerCase() === args.request.from.toLowerCase());
      if (account) {
        account.nonce += 1;
        saveWebAccounts();
      }
      
      return txHash;
    } catch (error) {
      throw new Error(`Failed to send transaction: ${error}`);
    }
  },
  
  sign_message: async (args: { message: string, address: string, password: string }) => {
    // SECURITY: No mock signatures allowed - signing failures must propagate
    // This ensures users always know when their transactions are not properly signed
    try {
      const signature = await rpcClient.signMessage(args.message, args.address);
      if (!signature || signature.length < 130) {
        throw new Error('Invalid signature received from RPC');
      }
      return signature;
    } catch (error) {
      const errorMessage = error instanceof Error ? error.message : String(error);
      throw new Error(`Message signing failed: ${errorMessage}. Ensure your wallet is connected and the account is unlocked.`);
    }
  },
  
  verify_signature: async (args: { message: string, signature: string, address: string }) => {
    // SECURITY: Proper signature verification
    // Check signature format first
    if (!args.signature || !args.signature.startsWith('0x')) {
      return false;
    }

    // Signatures should be 65 bytes (130 hex chars) + 0x prefix = 132 chars
    // or 64 bytes (128 hex chars) + 0x prefix = 130 chars for some schemes
    const sigLen = args.signature.length;
    if (sigLen !== 130 && sigLen !== 132) {
      return false;
    }

    // Validate hex characters
    const hexPart = args.signature.slice(2);
    if (!/^[0-9a-fA-F]+$/.test(hexPart)) {
      return false;
    }

    // Try to verify via RPC if available
    try {
      const verified = await rpcClient.verifySignature(args.message, args.signature, args.address);
      return !!verified;
    } catch (error) {
      // If RPC verification fails, we cannot confirm the signature
      console.warn('Signature verification via RPC failed:', error);
      // Return false rather than assuming valid - security first
      return false;
    }
  },
  
  // DAG
  get_dag_data: async (args: { limit: number, startHeight?: number }) => {
    try {
      // Prefer custom DAG RPC if available; otherwise build minimal DAG
      try {
        const data = await (rpcClient as any).sendRequest?.('citrate_getDagData', [args.limit, args.startHeight])
          .catch(() => null);
        if (data) return mapDAGDataFromNative(data);
      } catch {}

      // Fallback path
      const built = await rpcClient.buildDAGData(args.limit, args.startHeight);
      return built;
    } catch (error) {
      console.error('Failed to get DAG data:', error);
      return { nodes: [], links: [], tips: [], statistics: {
        totalBlocks: 0, blueBlocks: 0, redBlocks: 0, currentTips: 0, averageBlueScore: 0, maxHeight: 0,
      }} as DAGData;
    }
  },
  
  get_block_details: async (args: { hash: string }) => {
    try {
      const block = await rpcClient.getBlockByHash(args.hash);
      const blueScore = await rpcClient.getBlueScore(args.hash);
      
      return {
        hash: block.hash,
        selectedParent: block.parentHash,
        mergeParents: [],
        height: parseInt(block.number, 16),
        timestamp: parseInt(block.timestamp, 16) * 1000,
        blueScore: blueScore,
        isBlue: true,
        transactions: block.transactions || [],
        stateRoot: block.stateRoot,
        proposer: block.miner,
      };
    } catch (error) {
      throw new Error(`Failed to get block details: ${error}`);
    }
  },
  
  get_blue_set: async (args: { blockHash: string }) => {
    return await rpcClient.getBlueSet(args.blockHash);
  },
  
  get_current_tips: async () => {
    const tips = await rpcClient.getDAGTips();
    const tipInfos: TipInfo[] = [];
    
    for (const tip of tips) {
      try {
        const block = await rpcClient.getBlockByHash(tip);
        const blueScore = await rpcClient.getBlueScore(tip);
        
        tipInfos.push({
          hash: tip,
          height: parseInt(block.number, 16),
          blueScore: blueScore,
          timestamp: parseInt(block.timestamp, 16) * 1000,
          cumulativeWeight: 0n,
        });
      } catch (error) {
        console.warn(`Could not fetch tip info for ${tip}:`, error);
      }
    }
    
    // If no DAG tips, use latest block
    if (tipInfos.length === 0) {
      try {
        const latest = await rpcClient.getBlockByNumber('latest');
        if (latest) {
          tipInfos.push({
            hash: latest.hash,
            height: parseInt(latest.number, 16),
            blueScore: parseInt(latest.number, 16),
            timestamp: parseInt(latest.timestamp, 16) * 1000,
            cumulativeWeight: 0n,
          });
        }
      } catch (error) {
        console.warn('Could not fetch latest block:', error);
      }
    }
    
    return tipInfos;
  },
  
  calculate_blue_score: async (args: { blockHash: string }) => {
    return await rpcClient.getBlueScore(args.blockHash);
  },
  
  get_block_path: async (args: { blockHash: string }) => {
    const path: string[] = [];
    let currentHash = args.blockHash;
    
    try {
      while (currentHash && currentHash !== '0x0000000000000000000000000000000000000000000000000000000000000000') {
        path.push(currentHash);
        const block = await rpcClient.getBlockByHash(currentHash);
        currentHash = block.parentHash;
      }
    } catch (error) {
      console.warn('Could not complete block path:', error);
    }
    
    return path.reverse();
  },
  
  // Model
  deploy_model: async (args: { deployment: ModelDeployment }) => {
    return await rpcClient.deployModel(args.deployment);
  },
  
  run_inference: async (args: { request: InferenceRequest }) => {
    return await rpcClient.runInference(args.request);
  },
  
  start_training: async (args: { config: TrainingConfig }) => {
    // Training jobs would need custom implementation
    void args; // mark parameter as used
    return { jobId: 'job_' + Math.random().toString(36).substr(2, 9) };
  },
  
  get_model_info: async (args: { modelId: string }) => {
    try {
      const info = await rpcClient.getModelInfo(args.modelId);
      if (!info || !info.metadata) return null;
      const model: ModelInfo = {
        id: args.modelId,
        name: String(info.metadata.name || 'Unnamed Model'),
        architecture: String(info.metadata.framework || 'unknown'),
        version: String(info.metadata.version || '1.0.0'),
        owner: String(info.owner || '0x'),
        weightsCid: String(info.latest_artifact || ''),
        deploymentTime: Number(info.metadata.created_at || 0),
        lastUpdated: Number(info.usage_stats?.last_used || 0),
        totalInferences: Number(info.usage_stats?.total_inferences || 0),
        status: 'Active',
      };
      return model;
    } catch (e) {
      console.warn('get_model_info failed:', e);
      return null;
    }
  },
  
  list_models: async () => {
    return await rpcClient.getModelRegistry();
  },
  
  update_model: async (args: { modelId: string, weightsCid: string, version: string }) => {
    // Would need custom RPC implementation
    return `Updated model ${args.modelId} to version ${args.version}`;
  },
};

// Helper to convert account from backend format
const convertAccount = (account: any): Account => {
  return {
    ...account,
    balance: BigInt(account.balance || 0),
    publicKey: account.public_key || account.publicKey,
    createdAt: account.created_at || account.createdAt,
  };
};

// Ensure invoke is available or use web implementation
const safeInvoke = async <T>(cmd: string, args?: any): Promise<T> => {
  console.log(`safeInvoke called: ${cmd}`, args);
  
  if (!isTauri()) {
    // Use web implementations with RPC
    console.log(`Web mode - using RPC for ${cmd}`);
    const webFn = (webImplementations as any)[cmd];
    if (webFn) {
      return await webFn(args);
    }
    console.warn(`No web implementation for ${cmd}`);
    return {} as T;
  }
  
  console.log(`Native mode - using Tauri invoke for ${cmd}`);
  
  // Try to use window.__TAURI__ directly if available
  if (window.__TAURI__ && window.__TAURI__.core && window.__TAURI__.core.invoke) {
    try {
      console.log(`Using window.__TAURI__.core.invoke for ${cmd} with args:`, args);
      const result = await window.__TAURI__.core.invoke(cmd, args) as any;
      console.log(`Tauri invoke result for ${cmd}:`, result);
      
      // Convert accounts to proper format
      if (cmd === 'get_accounts' && Array.isArray(result)) {
        return result.map(convertAccount) as T;
      }
      if ((cmd === 'create_account' || cmd === 'import_account' || cmd === 'get_account') && result) {
        return convertAccount(result) as T;
      }

      if (cmd === 'get_node_status' && result) {
        return mapNodeStatusFromNative(result) as T;
      }
      if (cmd === 'get_dag_data' && result) {
        return mapDAGDataFromNative(result) as T;
      }
      
      return result as T;
    } catch (error: any) {
      console.error(`Failed to invoke ${cmd}:`, error);
      throw new Error(error?.message || error?.toString() || `Failed to invoke ${cmd}`);
    }
  }
  
  // Fallback to the imported invoke function
  if (!invoke) {
    // Wait a bit for dynamic import to complete
    await new Promise(resolve => setTimeout(resolve, 100));
    if (!invoke) {
      console.error('Tauri API is still loading');
      throw new Error('Tauri API is still loading. Please try again.');
    }
  }
  
  try {
    console.log(`Calling Tauri invoke for ${cmd} with args:`, args);
    const result = await invoke(cmd, args) as any;
    console.log(`Tauri invoke result for ${cmd}:`, result);
    
    // Convert accounts to proper format
    if (cmd === 'get_accounts' && Array.isArray(result)) {
      return result.map(convertAccount) as T;
    }
    if ((cmd === 'create_account' || cmd === 'import_account' || cmd === 'get_account') && result) {
      return convertAccount(result) as T;
    }
    if (cmd === 'get_node_status' && result) {
      return mapNodeStatusFromNative(result) as T;
    }
    if (cmd === 'get_dag_data' && result) {
      return mapDAGDataFromNative(result) as T;
    }
    
    return result as T;
  } catch (error: any) {
    console.error(`Failed to invoke ${cmd}:`, error);
    throw new Error(error?.message || error?.toString() || `Failed to invoke ${cmd}`);
  }
};

// Node Management
export const nodeService = {
  start: () => safeInvoke<string>('start_node'),
  stop: () => safeInvoke<string>('stop_node'),
  getStatus: () => safeInvoke<NodeStatus>('get_node_status'),
  updateConfig: (config: NodeConfig) => safeInvoke<string>('update_node_config', { config }),
  getConfig: () => safeInvoke<NodeConfig>('get_node_config'),
  getTxOverview: () => safeInvoke<{ pending: number; last_block: number }>('get_tx_overview'),
  getMempoolPending: (limit = 50) => safeInvoke<any[]>('get_mempool_pending', { limit }),
  joinTestnet: (args: {
    chainId?: number,
    dataDir?: string,
    rpcPort?: number,
    wsPort?: number,
    p2pPort?: number,
    restPort?: number,
    bootnodes?: string[],
    clearChain?: boolean,
    seedFrom?: string,
  }) => safeInvoke<string>('join_testnet', {
    // Tauri commands with a single struct parameter expect a key matching the param name
    args: {
      chain_id: args.chainId,
      data_dir: args.dataDir,
      rpc_port: args.rpcPort,
      ws_port: args.wsPort,
      p2p_port: args.p2pPort,
      rest_port: args.restPort,
      bootnodes: args.bootnodes,
      clear_chain: args.clearChain,
      seed_from: args.seedFrom,
    },
  }),
  autoAddBootnodes: () => safeInvoke<string[]>('auto_add_bootnodes'),
  // Network/Bootnodes
  getBootnodes: () => safeInvoke<string[]>('get_bootnodes'),
  addBootnode: (entry: string) => safeInvoke<string>('add_bootnode', { entry }),
  removeBootnode: (entry: string) => safeInvoke<string>('remove_bootnode', { entry }),
  connectBootnodes: () => safeInvoke<number>('connect_bootnodes'),
  connectPeer: (entry: string) => safeInvoke<string>('connect_peer', { entry }),
  disconnectPeer: (peerId: string) => safeInvoke<void>('disconnect_peer', { peerId }),
  getPeers: async () => {
    const raw = await safeInvoke<any[]>('get_peers');
    return (raw || []).map((p: any) => ({
      id: String(p.id),
      addr: String(p.addr),
      direction: (p.direction as string) as 'inbound' | 'outbound',
      state: (p.state as string) as 'connecting' | 'handshaking' | 'connected' | 'disconnecting' | 'disconnected',
      score: Number(p.score ?? 0),
      lastSeenSecs: Number(p.last_seen_secs ?? p.lastSeenSecs ?? 0),
    })) as PeerInfoSummary[];
  },
  setRewardAddress: (address: string) => safeInvoke<string>('set_reward_address', { address }),
  getRewardAddress: () => safeInvoke<string | null>('get_reward_address'),
  
  // Listen to status updates
  onStatusUpdate: (callback: (status: NodeStatus) => void) => {
    if (!isTauri() || !listen) {
      // Return a dummy unsubscribe function
      return Promise.resolve(() => {});
    }
    return listen('node-status', (event: any) => {
      try {
        callback(mapNodeStatusFromNative(event.payload));
      } catch {
        callback(event.payload as NodeStatus);
      }
    });
  }
};

// Wallet Management
export const walletService = {
  createAccount: (label: string, password: string) => 
    safeInvoke<Account>('create_account', { label, password }),
  createAccountExtended: (label: string, password: string) =>
    safeInvoke<[Account, string, string]>('create_account_extended', { label, password }),
  
  importAccount: (privateKey: string, label: string, password: string) =>
    safeInvoke<Account>('import_account', { privateKey, label, password }),
  importAccountFromMnemonic: (mnemonic: string, label: string, password: string) =>
    safeInvoke<Account>('import_account_from_mnemonic', { mnemonic, label, password }),
  
  getAccounts: () => safeInvoke<Account[]>('get_accounts'),

  deleteAccount: (address: string) =>
    safeInvoke<void>('delete_account', { address }),

  getAccount: (address: string) =>
    safeInvoke<Account | null>('get_account', { address }),
  
  signMessage: (message: string, address: string, password: string) =>
    safeInvoke<string>('sign_message', { message, address, password }),
  
  verifySignature: (message: string, signature: string, address: string) =>
    safeInvoke<boolean>('verify_signature', { message, signature, address }),
  exportPrivateKey: (address: string, password: string) =>
    safeInvoke<string>('export_private_key', { address, password }),
  getObservedBalance: (address: string, blockWindow = 256) =>
    safeInvoke<string>('get_address_observed_balance', { address, blockWindow }),
  
  // Check if password is required for a transaction (session-based signing support)
  checkPasswordRequired: (address: string, value: string) =>
    safeInvoke<boolean>('check_password_required', { address, value }),

  // Send transaction - password is optional if session is active
  sendTransaction: (txRequest: TransactionRequest, password?: string) =>
    safeInvoke<string>('send_transaction', {
      request: {
        ...txRequest,
        // Serialize BigInt fields to strings to avoid JSON errors and preserve precision
        value: (txRequest.value as unknown as bigint).toString(),
        gasPrice: (txRequest.gasPrice as unknown as bigint).toString(),
        data: Array.isArray(txRequest.data)
          ? '0x' + Array.from(txRequest.data).map(b => Number(b).toString(16).padStart(2, '0')).join('')
          : (txRequest.data as any),
      },
      password: password || null
    }),

  // Wallet activity
  getAccountActivity: (address: string, blockWindow = 256, limit = 100) =>
    safeInvoke<TxActivity[]>('get_account_activity', { address, blockWindow, limit }),

  // Tracked addresses (persisted in backend)
  getTrackedAddresses: () =>
    safeInvoke<string[]>('get_tracked_addresses'),

  saveTrackedAddresses: (addresses: string[]) =>
    safeInvoke<void>('save_tracked_addresses', { addresses }),
};

// DAG Management
export const dagService = {
  getData: (limit: number, startHeight?: number) =>
    safeInvoke<DAGData>('get_dag_data', { limit, startHeight }),
  
  getBlockDetails: (hash: string) =>
    safeInvoke<BlockDetails>('get_block_details', { hash }),
  
  getBlueSet: (blockHash: string) =>
    safeInvoke<string[]>('get_blue_set', { blockHash }),
  
  getCurrentTips: () =>
    safeInvoke<TipInfo[]>('get_current_tips'),
  
  calculateBlueScore: (blockHash: string) =>
    safeInvoke<number>('calculate_blue_score', { blockHash }),
  
  getBlockPath: (blockHash: string) =>
    safeInvoke<string[]>('get_block_path', { blockHash })
};

// Model Management
export const modelService = {
  deploy: (deployment: ModelDeployment) =>
    safeInvoke<any>('deploy_model', { deployment }),
  
  runInference: (request: InferenceRequest) =>
    safeInvoke<any>('run_inference', { request }),
  
  startTraining: (config: TrainingConfig) =>
    safeInvoke<any>('start_training', { config }),
  
  getInfo: (modelId: string) =>
    safeInvoke<ModelInfo>('get_model_info', { modelId }),
  
  list: () =>
    // Web/Tauri bridge: assemble ModelInfo objects from RPC registry
    safeInvoke<ModelInfo[]>('list_models').then(async (res) => {
      // If running in web mode, the web implementation returns via RPC
      // Otherwise, res may already be in the correct shape
      if (Array.isArray(res) && res.length && typeof res[0] === 'string') {
        const ids = res as unknown as string[];
        const out: ModelInfo[] = [];
        for (const id of ids) {
          try {
            const info = await rpcClient.getModelInfo(id);
            if (info && info.metadata) {
              out.push({
                id,
                name: String(info.metadata.name || 'Unnamed Model'),
                architecture: String(info.metadata.framework || 'unknown'),
                version: String(info.metadata.version || '1.0.0'),
                owner: String(info.owner || '0x'),
                weightsCid: String(info.latest_artifact || ''),
                deploymentTime: Number(info.metadata.created_at || 0),
                lastUpdated: Number(info.usage_stats?.last_used || 0),
                totalInferences: Number(info.usage_stats?.total_inferences || 0),
                status: 'Active',
              });
            }
          } catch (_) {
            // Ignore individual failures and continue
          }
        }
        return out;
      }
      return res as ModelInfo[];
    }),
  
  update: (modelId: string, weightsCid: string, version: string) =>
    safeInvoke<string>('update_model', { modelId, weightsCid, version })
};

// IPFS Types
export interface IpfsStatus {
  running: boolean;
  peer_id?: string;
  addresses: string[];
  repo_size?: number;
  num_objects?: number;
  version?: string;
}

export interface IpfsConfig {
  binary_path?: string;
  repo_path: string;
  api_port: number;
  gateway_port: number;
  swarm_port: number;
  external_gateways: string[];
  enable_pubsub: boolean;
  bootstrap_peers: string[];
}

export interface IpfsAddResult {
  cid: string;
  size: number;
  name: string;
  gateway_url: string;
}

export interface IpfsContent {
  cid: string;
  size: number;
  content_type?: string;
  data?: number[];  // byte array
  gateway_url: string;
}

// IPFS Management
export const ipfsService = {
  start: () => safeInvoke<IpfsStatus>('ipfs_start'),
  stop: () => safeInvoke<void>('ipfs_stop'),
  getStatus: () => safeInvoke<IpfsStatus>('ipfs_status'),
  getConfig: () => safeInvoke<IpfsConfig>('ipfs_get_config'),
  updateConfig: (config: IpfsConfig) => safeInvoke<void>('ipfs_update_config', { config }),

  // Content operations
  add: async (data: Uint8Array | string, name?: string) => {
    // Convert to base64 for transfer
    let base64Data: string;
    if (typeof data === 'string') {
      base64Data = btoa(data);
    } else {
      base64Data = btoa(String.fromCharCode(...data));
    }
    return safeInvoke<IpfsAddResult>('ipfs_add', { data: base64Data, name });
  },

  addFile: (path: string) => safeInvoke<IpfsAddResult>('ipfs_add_file', { path }),

  get: async (cid: string): Promise<IpfsContent> => {
    const result = await safeInvoke<IpfsContent>('ipfs_get', { cid });
    return result;
  },

  // Content as string (for text files)
  getText: async (cid: string): Promise<string> => {
    const result = await safeInvoke<IpfsContent>('ipfs_get', { cid });
    if (result.data) {
      return String.fromCharCode(...result.data);
    }
    return '';
  },

  // Pinning
  pin: (cid: string) => safeInvoke<void>('ipfs_pin', { cid }),
  unpin: (cid: string) => safeInvoke<void>('ipfs_unpin', { cid }),
  listPins: () => safeInvoke<string[]>('ipfs_list_pins'),

  // Network
  getPeers: () => safeInvoke<string[]>('ipfs_get_peers'),
};

// HuggingFace Types
export interface HFAuthState {
  authenticated: boolean;
  username?: string;
  email?: string;
  avatar_url?: string;
  token_type?: string;
  expires_at?: number;
}

export interface HFConfig {
  models_dir: string;
  cache_dir: string;
  default_quantization: string;
  auto_download: boolean;
  max_concurrent_downloads: number;
}

export interface HFModelInfo {
  id: string;
  author?: string;
  sha?: string;
  last_modified?: string;
  private: boolean;
  gated?: boolean;
  disabled?: boolean;
  downloads: number;
  likes: number;
  tags: string[];
  pipeline_tag?: string;
  library_name?: string;
  model_index?: any;
  config?: any;
  card_data?: any;
  siblings?: HFModelFile[];
}

export interface HFModelFile {
  rfilename: string;
  size?: number;
  blob_id?: string;
  lfs?: {
    sha256: string;
    size: number;
    pointer_size: number;
  };
}

export interface ModelSearchParams {
  search?: string;
  author?: string;
  filter?: string;
  sort?: string;
  direction?: string;
  limit?: number;
  full?: boolean;
  config?: boolean;
}

export interface DownloadProgress {
  id: string;
  model_id: string;
  filename: string;
  total_bytes: number;
  downloaded_bytes: number;
  status: 'pending' | 'downloading' | 'completed' | 'failed' | 'cancelled';
  error?: string;
  started_at: number;
  completed_at?: number;
}

// Enhanced GGUF-specific types from backend
export interface GGUFModelInfo {
  model_id: string;
  name: string;
  author: string;
  files: GGUFFileInfo[];
  downloads: number;
  likes: number;
  last_modified?: string;
  description?: string;
  tags: string[];
  gated: boolean;
}

export interface GGUFFileInfo {
  filename: string;
  size: number;
  quantization?: string;
  recommended: boolean;
  download_url: string;
}

export interface LocalModelInfo {
  model_id: string;
  path: string;
  size: number;
  quantization?: string;
  loaded: boolean;
}

export interface RecommendedModel {
  model_id: string;
  name: string;
  description: string;
}

// HuggingFace Management
export const huggingFaceService = {
  // Authentication - Enhanced with PKCE support
  startAuthFlow: () => safeInvoke<{ url: string; state: string }>('hf_start_auth_flow'),
  getAuthUrl: () => safeInvoke<string>('hf_get_auth_url'),
  exchangeCode: (code: string) =>
    safeInvoke<HFAuthState>('hf_exchange_code', { code }),
  exchangeCodeWithPkce: (code: string, state: string) =>
    safeInvoke<HFAuthState>('hf_exchange_code_with_pkce', { code, state }),
  setToken: (token: string) => safeInvoke<void>('hf_set_token', { token }),
  setApiToken: (token: string) => safeInvoke<void>('hf_set_api_token', { token }),
  getAuthState: () => safeInvoke<HFAuthState>('hf_get_auth_state'),
  logout: () => safeInvoke<void>('hf_logout'),

  // Configuration
  getConfig: () => safeInvoke<HFConfig>('hf_get_config'),
  updateConfig: (config: HFConfig) => safeInvoke<void>('hf_update_config', { config }),

  // Model Discovery - Legacy API
  searchModels: (params: ModelSearchParams) =>
    safeInvoke<HFModelInfo[]>('hf_search_models', { params }),
  getModelInfo: (modelId: string) =>
    safeInvoke<HFModelInfo>('hf_get_model_info', { modelId }),
  getModelFiles: (modelId: string) =>
    safeInvoke<HFModelFile[]>('hf_get_model_files', { modelId }),

  // GGUF-specific Model Discovery (Enhanced)
  searchGGUFModels: (query: string, limit: number = 20) =>
    safeInvoke<GGUFModelInfo[]>('hf_search_gguf_models', { query, limit }),
  getGGUFModel: (modelId: string) =>
    safeInvoke<GGUFModelInfo>('hf_get_gguf_model', { modelId }),
  getRecommendedModels: () =>
    safeInvoke<RecommendedModel[]>('hf_get_recommended_models'),

  // Downloads - Legacy
  downloadFile: (modelId: string, filename: string) =>
    safeInvoke<string>('hf_download_file', { modelId, filename }),
  getDownloads: () => safeInvoke<DownloadProgress[]>('hf_get_downloads'),
  cancelDownload: (modelId: string, filename: string) =>
    safeInvoke<void>('hf_cancel_download', { modelId, filename }),

  // Downloads - Enhanced with resume support
  downloadFileResumable: (modelId: string, filename: string) =>
    safeInvoke<string>('hf_download_file_resumable', { modelId, filename }),
  cancelDownloadResumable: (modelId: string, filename: string) =>
    safeInvoke<void>('hf_cancel_download_resumable', { modelId, filename }),
  getDownloadStats: () =>
    safeInvoke<{ active: number; completed: number; totalBytes: number }>('hf_get_download_stats'),

  // Local Models - Legacy
  getLocalModels: () => safeInvoke<string[]>('hf_get_local_models'),
  getModelsDir: () => safeInvoke<string>('hf_get_models_dir'),

  // Local Models - Enhanced
  scanLocalModels: () => safeInvoke<LocalModelInfo[]>('hf_scan_local_models'),
  autoSelectModel: () => safeInvoke<LocalModelInfo | null>('hf_auto_select_model'),
  deleteLocalModel: (path: string) => safeInvoke<void>('hf_delete_local_model', { path }),

  // Listen to download progress events
  onDownloadProgress: (callback: (progress: DownloadProgress) => void) => {
    if (!isTauri() || !listen) {
      return Promise.resolve(() => {});
    }
    return listen('hf-download-progress', (event: any) => {
      callback(event.payload as DownloadProgress);
    });
  },
};

// ==========================================
// LoRA Training Types and Service
// ==========================================

// Dataset formats for LoRA training
export type DatasetFormat = 'Jsonl' | 'Csv' | 'Parquet' | 'Alpaca' | 'ShareGPT' | 'Custom';

// LoRA bias types
export type LoRaBias = 'None' | 'All' | 'LoraOnly';

// Task types for LoRA training
export type LoRaTaskType =
  | 'CausalLM'
  | 'SequenceClassification'
  | 'TokenClassification'
  | 'QuestionAnswering'
  | 'FeatureExtraction';

// Learning rate scheduler types
export type LRSchedulerType = 'Cosine' | 'Linear' | 'Constant' | 'ConstantWithWarmup' | 'Polynomial';

// Evaluation strategy
export type EvalStrategy = 'No' | 'Steps' | 'Epoch';

// Job status enum
export type JobStatus = 'Queued' | 'Running' | 'Completed' | 'Failed' | 'Cancelled';

// LoRA-specific configuration
export interface LoraConfig {
  rank: number;
  alpha: number;
  dropout: number;
  target_modules: string[];
  bias: LoRaBias;
  task_type: LoRaTaskType;
}

// Training hyperparameters configuration
export interface LoraTrainingConfig {
  epochs: number;
  batch_size: number;
  gradient_accumulation_steps: number;
  learning_rate: number;
  lr_scheduler: LRSchedulerType;
  warmup_ratio: number;
  weight_decay: number;
  max_seq_length: number;
  gradient_checkpointing: boolean;
  eval_strategy: EvalStrategy;
  eval_steps?: number;
  save_steps: number;
  logging_steps: number;
  max_grad_norm: number;
  seed: number;
  fp16: boolean;
  bf16: boolean;
  num_threads: number;
  use_gpu: boolean;
  n_gpu_layers: number;
}

// Training metrics point for progress tracking
export interface TrainingMetricsPoint {
  step: number;
  epoch: number;
  train_loss: number;
  val_loss?: number;
  learning_rate: number;
  timestamp: number;
}

// LoRA training job
export interface LoraTrainingJob {
  id: string;
  base_model_path: string;
  base_model_name: string;
  dataset_path: string;
  dataset_format: DatasetFormat;
  output_dir: string;
  lora_config: LoraConfig;
  training_config: LoraTrainingConfig;
  status: JobStatus;
  progress: number;
  current_epoch: number;
  current_step: number;
  total_steps: number;
  train_loss: number;
  val_loss?: number;
  metrics_history: TrainingMetricsPoint[];
  error_message?: string;
  created_at: number;
  started_at?: number;
  completed_at?: number;
}

// LoRA adapter info for saved adapters
export interface LoraAdapterInfo {
  id: string;
  name: string;
  base_model: string;
  path: string;
  size_bytes: number;
  rank: number;
  alpha: number;
  target_modules: string[];
  created_at: number;
  training_job_id?: string;
  description?: string;
  tags: string[];
}

// Dataset validation result
export interface DatasetValidation {
  valid: boolean;
  total_samples: number;
  valid_samples: number;
  errors: string[];
  estimated_tokens: number;
}

// LoRA training preset
export interface LoraPreset {
  name: string;
  description: string;
  lora_config: LoraConfig;
  training_config: LoraTrainingConfig;
  recommended_vram_gb: number;
}

// LoRA Training Service
export const loraTrainingService = {
  // Job Management
  createJob: (
    base_model_path: string,
    base_model_name: string,
    dataset_path: string,
    dataset_format: DatasetFormat,
    output_dir: string,
    lora_config?: LoraConfig,
    training_config?: LoraTrainingConfig
  ) =>
    safeInvoke<LoraTrainingJob>('create_lora_job', {
      base_model_path,
      base_model_name,
      dataset_path,
      dataset_format,
      output_dir,
      lora_config,
      training_config,
    }),

  startTraining: (job_id: string) =>
    safeInvoke<void>('start_lora_training', { job_id }),

  getJob: (job_id: string) =>
    safeInvoke<LoraTrainingJob | null>('get_lora_job', { job_id }),

  getJobs: () => safeInvoke<LoraTrainingJob[]>('get_lora_jobs'),

  cancelJob: (job_id: string) =>
    safeInvoke<void>('cancel_lora_job', { job_id }),

  deleteJob: (job_id: string) =>
    safeInvoke<void>('delete_lora_job', { job_id }),

  // Adapter Management
  getAdapters: () => safeInvoke<LoraAdapterInfo[]>('get_lora_adapters'),

  deleteAdapter: (adapter_id: string) =>
    safeInvoke<void>('delete_lora_adapter', { adapter_id }),

  // Inference with LoRA
  runInferenceWithLora: (
    model_path: string,
    adapter_path: string,
    prompt: string,
    max_tokens?: number,
    temperature?: number
  ) =>
    safeInvoke<string>('run_inference_with_lora', {
      model_path,
      adapter_path,
      prompt,
      max_tokens,
      temperature,
    }),

  // Dataset Validation
  validateDataset: (path: string, format: DatasetFormat) =>
    safeInvoke<DatasetValidation>('validate_dataset', { path, format }),

  // Presets
  getPresets: () => safeInvoke<LoraPreset[]>('get_lora_presets'),

  // Helper: Default LoRA config
  getDefaultLoraConfig: (): LoraConfig => ({
    rank: 8,
    alpha: 16.0,
    dropout: 0.05,
    target_modules: ['q_proj', 'v_proj', 'k_proj', 'o_proj'],
    bias: 'None',
    task_type: 'CausalLM',
  }),

  // Helper: Default training config
  getDefaultTrainingConfig: (): LoraTrainingConfig => ({
    epochs: 3,
    batch_size: 4,
    gradient_accumulation_steps: 4,
    learning_rate: 2e-4,
    lr_scheduler: 'Cosine',
    warmup_ratio: 0.03,
    weight_decay: 0.001,
    max_seq_length: 2048,
    gradient_checkpointing: true,
    eval_strategy: 'Epoch',
    eval_steps: undefined,
    save_steps: 100,
    logging_steps: 10,
    max_grad_norm: 1.0,
    seed: 42,
    fp16: false,
    bf16: false,
    num_threads: navigator.hardwareConcurrency || 4,
    use_gpu: false,
    n_gpu_layers: 0,
  }),
};

// Agent Types
export interface AgentSession {
  id: string;
  state: string;
  message_count: number;
  created_at: number;
  last_activity: number;
}

export interface AgentMessage {
  id: string;
  role: string;
  content: string;
  timestamp: number;
  is_streaming?: boolean;
  intent?: string;
}

export interface AgentMessageResponse {
  session_id: string;
  message_id: string;
  content: string;
  role: string;
  intent?: string;
  intent_confidence?: number;
  tool_invoked: boolean;
  tool_name?: string;
  pending_approval: boolean;
}

export interface PendingTool {
  id: string;
  tool_name: string;
  description: string;
  high_risk: boolean;
  params: Record<string, unknown>;
}

export interface AgentConfigResponse {
  enabled: boolean;
  llm_backend: string;
  model: string;
  streaming_enabled: boolean;
  local_model_path?: string;
}

export interface ActiveModelInfo {
  backend: string;
  model_id: string;
  temperature: number;
  max_tokens: number;
  context_size: number;
  has_api_key: boolean;
}

// Agent-specific local model info (different from HuggingFace LocalModelInfo above)
export interface AgentLocalModelInfo {
  name: string;
  size: string;
  quantization: string;
  path: string;
}

export interface AgentStatusResponse {
  initialized: boolean;
  enabled: boolean;
  llm_backend?: string;
  active_sessions?: number;
  streaming_enabled?: boolean;
}

// Agent Service
export const agentService = {
  // Session management
  createSession: () => safeInvoke<AgentSession>('agent_create_session'),
  getSession: (sessionId: string) =>
    safeInvoke<AgentSession | null>('agent_get_session', { sessionId }),
  listSessions: () => safeInvoke<string[]>('agent_list_sessions'),
  deleteSession: (sessionId: string) =>
    safeInvoke<boolean>('agent_delete_session', { sessionId }),

  // Messages
  sendMessage: (sessionId: string, message: string) =>
    safeInvoke<AgentMessageResponse>('agent_send_message', { sessionId, message }),
  getMessages: (sessionId: string) =>
    safeInvoke<AgentMessage[]>('agent_get_messages', { sessionId }),
  clearHistory: (sessionId: string) =>
    safeInvoke<void>('agent_clear_history', { sessionId }),

  // Tool approval
  getPendingTools: (sessionId: string) =>
    safeInvoke<PendingTool[]>('agent_get_pending_tools', { sessionId }),
  approveTool: (sessionId: string, toolId: string) =>
    safeInvoke<boolean>('agent_approve_tool', { sessionId, toolId }),
  rejectTool: (sessionId: string, toolId: string) =>
    safeInvoke<boolean>('agent_reject_tool', { sessionId, toolId }),

  // Configuration
  getConfig: () => safeInvoke<AgentConfigResponse>('agent_get_config'),
  updateConfig: (params: {
    enabled?: boolean;
    apiKey?: string;
    model?: string;
    streamingEnabled?: boolean;
  }) => safeInvoke<void>('agent_update_config', params),

  // Local models
  scanLocalModels: (directory?: string) =>
    safeInvoke<AgentLocalModelInfo[]>('agent_scan_local_models', { directory }),
  getModelsDir: () => safeInvoke<string | null>('agent_get_models_dir'),
  loadLocalModel: (modelPath: string) =>
    safeInvoke<string>('agent_load_local_model', { modelPath }),
  getActiveModel: () => safeInvoke<ActiveModelInfo>('agent_get_active_model'),
  setApiKey: (provider: string, apiKey: string) =>
    safeInvoke<void>('agent_set_api_key', { provider, apiKey }),
  setAutoMode: () => safeInvoke<void>('agent_set_auto_mode'),

  // Status
  isReady: () => safeInvoke<boolean>('agent_is_ready'),
  getStatus: () => safeInvoke<AgentStatusResponse>('agent_get_status'),

  // Event listeners
  onToken: (callback: (event: { session_id: string; message_id: string; content: string; is_complete: boolean }) => void) => {
    if (!isTauri() || !listen) {
      return Promise.resolve(() => {});
    }
    return listen('agent-token', (event: any) => {
      callback(event.payload);
    });
  },

  onComplete: (callback: (event: { session_id: string; message_id: string; total_tokens?: number; finish_reason?: string }) => void) => {
    if (!isTauri() || !listen) {
      return Promise.resolve(() => {});
    }
    return listen('agent-complete', (event: any) => {
      callback(event.payload);
    });
  },

  onError: (callback: (event: { session_id: string; error: string; code?: string }) => void) => {
    if (!isTauri() || !listen) {
      return Promise.resolve(() => {});
    }
    return listen('agent-error', (event: any) => {
      callback(event.payload);
    });
  },

  onToolCall: (callback: (event: { session_id: string; tool_id: string; tool_name: string; description: string; high_risk: boolean; params: Record<string, unknown> }) => void) => {
    if (!isTauri() || !listen) {
      return Promise.resolve(() => {});
    }
    return listen('agent-tool-call', (event: any) => {
      callback(event.payload);
    });
  },
};
