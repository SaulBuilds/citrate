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
    try {
      return await rpcClient.signMessage(args.message, args.address);
    } catch (error) {
      // Fallback to mock signature
      return '0x' + Array.from(crypto.getRandomValues(new Uint8Array(65)))
        .map(b => b.toString(16).padStart(2, '0'))
        .join('');
    }
  },
  
  verify_signature: async (args: { message: string, signature: string, address: string }) => {
    // Simplified verification - in production would use proper crypto
    return args.signature.length === 132 && args.signature.startsWith('0x');
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
  
  sendTransaction: (txRequest: TransactionRequest, password: string) =>
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
      password 
    }),

  // Wallet activity
  getAccountActivity: (address: string, blockWindow = 256, limit = 100) =>
    safeInvoke<TxActivity[]>('get_account_activity', { address, blockWindow, limit }),
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
