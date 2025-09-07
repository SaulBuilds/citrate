export interface LatticeConfig {
  rpcEndpoint: string;
  chainId: number;
  defaultAccount?: string;
  gasPrice?: string;
  gasLimit?: number;
}

export interface NetworkInfo {
  networkId: number;
  chainId: number;
  blockNumber: number;
  syncing: boolean;
  peerCount: number;
}

export interface BlockInfo {
  number: number;
  hash: string;
  parentHash: string;
  timestamp: number;
  miner: string;
  transactions: any[];
  mergeParents?: string[];
  blueScore?: number;
}

export interface ModelMetadata {
  name: string;
  version: string;
  format: string;
  description?: string;
  author?: string;
  license?: string;
  tags?: string[];
}

export interface ModelInfo {
  id: string;
  owner: string;
  metadata: ModelMetadata;
  dataHash: string;
  timestamp: number;
  permissions: string[];
}

export interface InferenceResult {
  output: any;
  executionTime: number;
  proofId?: string;
  gasUsed?: number;
}

export interface ContractInfo {
  address: string;
  bytecode?: string;
  abi?: any[];
  verified?: boolean;
}