import { EventEmitter } from 'events';
import axios, { AxiosInstance } from 'axios';
import { ModelRegistry } from './model';
import { ContractManager } from './contract';
import { AccountManager } from './account';
import { LatticeConfig, NetworkInfo, BlockInfo, ArtifactProviderStatus, ArtifactPinResult } from './types';

/**
 * Main Citrate SDK class
 */
export class LatticeSDK extends EventEmitter {
  private rpcClient: AxiosInstance;
  private config: LatticeConfig;
  
  public models: ModelRegistry;
  public contracts: ContractManager;
  public accounts: AccountManager;
  
  constructor(config: LatticeConfig) {
    super();
    
    const rpcEndpoint = config.rpcEndpoint ?? 'http://localhost:8545';
    const chainId = config.chainId ?? 1337;
    this.config = { ...config, rpcEndpoint, chainId };
    
    this.rpcClient = axios.create({
      baseURL: this.config.rpcEndpoint,
      headers: {
        'Content-Type': 'application/json'
      }
    });
    
    // Initialize managers
    this.models = new ModelRegistry(this.rpcClient, this.config);
    this.contracts = new ContractManager(this.rpcClient, this.config);
    this.accounts = new AccountManager(this.rpcClient, this.config);
  }
  
  /**
   * Deploy a model to the network
   */
  async deployModel(
    modelData: Buffer | Uint8Array,
    metadata: {
      name: string;
      version: string;
      format: string;
      description?: string;
    }
  ): Promise<string> {
    return this.models.deploy(modelData, metadata);
  }
  
  /**
   * Run inference on a deployed model
   */
  async runInference(
    modelId: string,
    input: any,
    options?: {
      withProof?: boolean;
      timeout?: number;
    }
  ): Promise<{
    output: any;
    executionTime: number;
    proofId?: string;
  }> {
    return this.models.runInference(modelId, input, options);
  }
  
  /**
   * Get proof for an execution
   */
  async getProof(executionId: string): Promise<any> {
    const response = await this.rpcCall('citrate_getProof', [{
      execution_id: executionId
    }]);
    
    return response.proof;
  }
  
  /**
   * Verify a proof
   */
  async verifyProof(proof: any, expectedOutput?: string): Promise<boolean> {
    const response = await this.rpcCall('citrate_verifyProof', [{
      proof,
      output_hash: expectedOutput
    }]);
    
    return response.valid;
  }

  // ======= Artifacts =======
  async pinArtifact(cid: string, replicas: number = 1): Promise<ArtifactPinResult> {
    const response = await this.rpcClient.post('', {
      jsonrpc: '2.0', method: 'citrate_pinArtifact', params: [cid, replicas], id: Date.now()
    });
    if (response.data.error) throw new Error(response.data.error.message);
    return response.data.result;
  }

  async getArtifactStatus(cid: string): Promise<ArtifactProviderStatus[]> {
    const response = await this.rpcClient.post('', {
      jsonrpc: '2.0', method: 'citrate_getArtifactStatus', params: [cid], id: Date.now()
    });
    if (response.data.error) throw new Error(response.data.error.message);
    const result = response.data.result;
    try { return JSON.parse(typeof result === 'string' ? result : JSON.stringify(result)); } catch { return []; }
  }

  async listModelArtifacts(modelIdHex: string): Promise<string[]> {
    const response = await this.rpcClient.post('', {
      jsonrpc: '2.0', method: 'citrate_listModelArtifacts', params: [modelIdHex], id: Date.now()
    });
    if (response.data.error) throw new Error(response.data.error.message);
    const result = response.data.result;
    return Array.isArray(result) ? result : [];
  }
  
  /**
   * Get network information
   */
  async getNetworkInfo(): Promise<NetworkInfo> {
    const [networkId, blockNumber, syncing, peerCount] = await Promise.all([
      this.rpcCall('net_version'),
      this.rpcCall('eth_blockNumber'),
      this.rpcCall('eth_syncing'),
      this.rpcCall('net_peerCount')
    ]);
    
    return {
      networkId: parseInt(networkId),
      chainId: this.config.chainId,
      blockNumber: parseInt(blockNumber, 16),
      syncing: typeof syncing === 'object',
      peerCount: parseInt(peerCount, 16)
    };
  }
  
  /**
   * Get block information
   */
  async getBlock(blockNumber: number | 'latest'): Promise<BlockInfo> {
    const block = await this.rpcCall('eth_getBlockByNumber', [
      blockNumber === 'latest' ? 'latest' : `0x${blockNumber.toString(16)}`,
      true
    ]);
    
    return {
      number: parseInt(block.number, 16),
      hash: block.hash,
      parentHash: block.parentHash,
      timestamp: parseInt(block.timestamp, 16),
      miner: block.miner,
      transactions: block.transactions,
      // GhostDAG specific
      mergeParents: block.mergeParents || [],
      blueScore: block.blueScore ? parseInt(block.blueScore, 16) : undefined
    };
  }
  
  /**
   * Get DAG statistics
   */
  async getDagStats(): Promise<{
    totalBlocks: number;
    blueBlocks: number;
    redBlocks: number;
    tipsCount: number;
    maxBlueScore: number;
    currentTips: string[];
  }> {
    const stats = await this.rpcCall('citrate_getDagStats');
    
    return {
      totalBlocks: stats.totalBlocks,
      blueBlocks: stats.blueBlocks,
      redBlocks: stats.redBlocks,
      tipsCount: stats.tipsCount,
      maxBlueScore: stats.maxBlueScore,
      currentTips: stats.currentTips || []
    };
  }
  
  /**
   * Subscribe to new blocks
   */
  subscribeToBlocks(callback: (block: BlockInfo) => void): () => void {
    let intervalId: NodeJS.Timeout;
    let lastBlockNumber = 0;
    
    const checkNewBlock = async () => {
      try {
        const block = await this.getBlock('latest');
        if (block.number > lastBlockNumber) {
          lastBlockNumber = block.number;
          callback(block);
          this.emit('block', block);
        }
      } catch (error) {
        this.emit('error', error);
      }
    };
    
    // Poll for new blocks
    intervalId = setInterval(checkNewBlock, 2000);
    checkNewBlock(); // Check immediately
    
    // Return unsubscribe function
    return () => {
      clearInterval(intervalId);
    };
  }
  
  /**
   * Make an RPC call
   */
  async rpcCall(method: string, params: any = []): Promise<any> {
    const response = await this.rpcClient.post('', {
      jsonrpc: '2.0',
      method,
      params: Array.isArray(params) ? params : [params],
      id: Date.now()
    });
    
    if (response.data.error) {
      throw new Error(response.data.error.message);
    }
    
    return response.data.result;
  }
  
  /**
   * Wait for a transaction to be mined
   */
  async waitForTransaction(
    txHash: string,
    confirmations: number = 1
  ): Promise<any> {
    let receipt = null;
    let attempts = 0;
    const maxAttempts = 60;
    
    while (!receipt && attempts < maxAttempts) {
      attempts++;
      
      try {
        receipt = await this.rpcCall('eth_getTransactionReceipt', [txHash]);
        
        if (receipt && confirmations > 1) {
          // Wait for additional confirmations
          const currentBlock = await this.getBlock('latest');
          const txBlock = parseInt(receipt.blockNumber, 16);
          
          if (currentBlock.number - txBlock < confirmations - 1) {
            receipt = null; // Keep waiting
          }
        }
      } catch (error) {
        // Transaction not yet mined
      }
      
      if (!receipt) {
        await new Promise(resolve => setTimeout(resolve, 2000));
      }
    }
    
    if (!receipt) {
      throw new Error('Transaction timeout');
    }
    
    return receipt;
  }
}
