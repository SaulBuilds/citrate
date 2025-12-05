// Node types
export interface NodeStatus {
  running: boolean;
  syncing: boolean;
  blockHeight: number;
  peerCount: number;
  networkId: string;
  version: string;
  uptime: number;
  dagTips?: number;
  blueScore?: number;
  lastBlockHash?: string | null;
  lastBlockTimestamp?: number | null; // ms
}

// Network / Peers
export interface PeerInfoSummary {
  id: string;
  addr: string;
  direction: 'inbound' | 'outbound';
  state: 'connecting' | 'handshaking' | 'connected' | 'disconnecting' | 'disconnected';
  score: number;
  lastSeenSecs: number;
}

export interface NodeConfig {
  dataDir: string;
  network: string;
  rpcPort: number;
  wsPort: number;
  p2pPort: number;
  restPort: number;
  maxPeers: number;
  bootnodes: string[];
  rewardAddress?: string;
  enableNetwork?: boolean;
  discovery?: boolean;
  mempool: MempoolSettings;
  consensus: ConsensusConfig;
}

export interface ConsensusConfig {
  kParameter: number;
  pruningWindow: number;
  blockTimeSeconds: number;
  finalityDepth: number;
}

export interface MempoolSettings {
  minGasPrice: number;
  maxPerSender: number;
  allowReplacement: boolean;
  chainId: number;
  maxSize: number;
  replacementFactor: number;
  requireValidSignature: boolean;
  txExpirySecs: number;
}

// Wallet types
export interface Account {
  address: string;
  label: string;
  publicKey: string;
  balance: bigint;
  nonce: number;
  createdAt: number;
}

export interface TxActivity {
  hash: string;
  from: string;
  to?: string;
  value: string;
  nonce: number;
  status: 'pending' | 'confirmed';
  blockHash?: string;
  blockHeight?: number;
  timestamp?: number;
}

export interface TransactionRequest {
  from: string;
  to?: string;
  value: bigint;
  gasLimit: number;
  gasPrice: bigint;
  data: number[] | string;
  nonce?: number;
}

// DAG types
export interface DAGData {
  nodes: DAGNode[];
  links: DAGLink[];
  tips: TipInfo[];
  statistics: DAGStatistics;
}

export interface DAGNode {
  id: string;
  hash: string;
  height: number;
  timestamp: number;
  isBlue: boolean;
  blueScore: number;
  // Optional UI hint computed client-side
  isTip?: boolean;
  selectedParent?: string;
  mergeParents: string[];
  transactions: number;
  proposer: string;
  size: number;
}

export interface DAGLink {
  source: string;
  target: string;
  isSelected: boolean;
  linkType: 'SelectedParent' | 'MergeParent';
}

export interface TipInfo {
  hash: string;
  height: number;
  timestamp: number;
  blueScore: number;
  cumulativeWeight: bigint;
}

export interface DAGStatistics {
  totalBlocks: number;
  blueBlocks: number;
  redBlocks: number;
  currentTips: number;
  averageBlueScore: number;
  maxHeight: number;
}

export interface BlockDetails {
  hash: string;
  height: number;
  timestamp: number;
  isBlue: boolean;
  blueScore: number;
  blueSet?: string[];
  redSet?: string[];
  selectedParent?: string;
  mergeParents: string[];
  children: string[];
  transactions: TransactionInfo[];
  stateRoot: string;
  txRoot: string;
  receiptRoot: string;
  artifactRoot?: string;
  proposer: string;
  vrfOutput?: string;
  signature?: string;
}

export interface TransactionInfo {
  hash: string;
  from: string;
  to?: string;
  fromAddr?: string;
  toAddr?: string;
  value: string;
  gasUsed?: number;
  txType?: string;
  status?: boolean;
}

// Model types
export interface ModelDeployment {
  modelId: string;
  name: string;
  description: string;
  architecture: string;
  version: string;
  weightsCid: string;
  metadata: Record<string, any>;
  owner: string;
}

export interface InferenceRequest {
  modelId: string;
  input: any;
  parameters?: Record<string, any>;
}

export interface TrainingConfig {
  modelId: string;
  datasetCid: string;
  epochs: number;
  batchSize: number;
  learningRate: number;
  optimizerConfig?: Record<string, any>;
}

export interface ModelInfo {
  id: string;
  name: string;
  architecture: string;
  version: string;
  owner: string;
  weightsCid: string;
  deploymentTime: number;
  lastUpdated: number;
  totalInferences: number;
  status: 'Active' | 'Training' | 'Updating' | 'Deprecated';
}
