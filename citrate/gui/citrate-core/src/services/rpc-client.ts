// RPC Client for connecting to the Citrate blockchain node
import type { TransactionRequest, NodeStatus, DAGData, DAGNode, DAGLink, TipInfo } from '../types';

interface JsonRpcRequest {
  jsonrpc: string;
  method: string;
  params: any[];
  id: number;
}

interface JsonRpcResponse {
  jsonrpc: string;
  result?: any;
  error?: {
    code: number;
    message: string;
    data?: any;
  };
  id: number;
}

class RPCClient {
  private url: string;
  private requestId: number = 1;

  constructor(url: string = 'http://127.0.0.1:8545') {
    this.url = url;
  }

  private async sendRequest(method: string, params: any[] = []): Promise<any> {
    const request: JsonRpcRequest = {
      jsonrpc: '2.0',
      method,
      params,
      id: this.requestId++,
    };

    try {
      const response = await fetch(this.url, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify(request),
      });

      if (!response.ok) {
        throw new Error(`HTTP error! status: ${response.status}`);
      }

      const data: JsonRpcResponse = await response.json();

      if (data.error) {
        throw new Error(data.error.message);
      }

      return data.result;
    } catch (error) {
      console.error('RPC request failed:', error);
      throw error;
    }
  }

  // Node methods
  async getNodeStatus(): Promise<NodeStatus> {
    try {
      const [blockNumber, syncing, peerCount, networkId, clientVersion] = await Promise.all([
        this.sendRequest('eth_blockNumber'),
        this.sendRequest('eth_syncing'),
        this.sendRequest('net_peerCount'),
        this.sendRequest('net_version'),
        this.sendRequest('web3_clientVersion'),
      ]);

      return {
        running: true,
        syncing: syncing !== false,
        blockHeight: parseInt(blockNumber, 16),
        peerCount: parseInt(peerCount, 16),
        networkId: networkId,
        version: clientVersion,
        uptime: 0, // This would need a custom RPC method
      } as NodeStatus;
    } catch (error) {
      return {
        running: false,
        syncing: false,
        blockHeight: 0,
        peerCount: 0,
        networkId: 'unknown',
        version: 'unknown',
        uptime: 0,
      } as NodeStatus;
    }
  }

  // Account methods
  async getAccounts(): Promise<string[]> {
    return this.sendRequest('eth_accounts');
  }

  async getBalance(address: string): Promise<string> {
    const balance = await this.sendRequest('eth_getBalance', [address, 'latest']);
    return parseInt(balance, 16).toString();
  }

  async getTransactionCount(address: string): Promise<number> {
    // Use 'pending' to include mempool txs for better UX
    const count = await this.sendRequest('eth_getTransactionCount', [address, 'pending']);
    return parseInt(count, 16);
  }

  async sendTransaction(tx: TransactionRequest): Promise<string> {
    const params: any = {
      from: tx.from,
      to: tx.to,
      value: '0x' + tx.value.toString(16),
      gas: '0x' + tx.gasLimit.toString(16),
      gasPrice: '0x' + tx.gasPrice.toString(16),
      data: Array.isArray(tx.data)
        ? '0x' + Array.from(tx.data).map(b => Number(b).toString(16).padStart(2, '0')).join('')
        : (tx.data as any),
      nonce: tx.nonce !== undefined ? '0x' + tx.nonce.toString(16) : undefined,
    };

    return this.sendRequest('eth_sendTransaction', [params]);
  }

  async signMessage(message: string, address: string): Promise<string> {
    return this.sendRequest('eth_sign', [address, message]);
  }

  async verifySignature(message: string, signature: string, address: string): Promise<boolean> {
    try {
      // Use the personal_ecRecover method to recover the address from the signature
      // and compare it with the expected address
      const recoveredAddress = await this.sendRequest('personal_ecRecover', [message, signature]);
      return recoveredAddress.toLowerCase() === address.toLowerCase();
    } catch {
      // If personal_ecRecover is not available, try citrate-specific method
      try {
        return await this.sendRequest('citrate_verifySignature', [message, signature, address]);
      } catch {
        // If no verification method is available, we cannot verify
        throw new Error('Signature verification not available via RPC');
      }
    }
  }

  async estimateGas(tx: Partial<TransactionRequest>): Promise<string> {
    const params: any = {
      from: tx.from,
      to: tx.to,
      value: tx.value ? '0x' + BigInt(tx.value).toString(16) : '0x0',
      data: Array.isArray(tx.data)
        ? '0x' + Array.from(tx.data).map(b => Number(b).toString(16).padStart(2, '0')).join('')
        : (tx.data || '0x'),
    };

    const gasEstimate = await this.sendRequest('eth_estimateGas', [params]);
    return parseInt(gasEstimate, 16).toString();
  }

  async getTransactionReceipt(txHash: string): Promise<any> {
    return this.sendRequest('eth_getTransactionReceipt', [txHash]);
  }

  async getTransaction(txHash: string): Promise<any> {
    return this.sendRequest('eth_getTransactionByHash', [txHash]);
  }

  // DAG-specific methods
  async getDAGTips(): Promise<string[]> {
    try {
      return await this.sendRequest('citrate_getTips');
    } catch {
      // Fallback to standard method if custom RPC not available
      return [];
    }
  }

  async getBlueScore(blockHash: string): Promise<number> {
    try {
      const score = await this.sendRequest('citrate_getBlueScore', [blockHash]);
      return parseInt(score, 16);
    } catch {
      return 0;
    }
  }

  async getBlueSet(blockHash: string): Promise<string[]> {
    try {
      return await this.sendRequest('citrate_getBlueSet', [blockHash]);
    } catch {
      return [];
    }
  }

  async getBlockByHash(hash: string): Promise<any> {
    return this.sendRequest('eth_getBlockByHash', [hash, true]);
  }

  async getBlockByNumber(number: number | string): Promise<any> {
    const blockNumber = typeof number === 'number' ? '0x' + number.toString(16) : number;
    return this.sendRequest('eth_getBlockByNumber', [blockNumber, true]);
  }

  // Build minimal DAG data from standard JSON-RPC when custom DAG RPC is not available
  async buildDAGData(limit: number, startHeight?: number): Promise<DAGData> {
    const latestBlock = await this.getBlockByNumber('latest');
    const nodes: DAGNode[] = [];
    const links: DAGLink[] = [];
    const tips: TipInfo[] = [];

    if (latestBlock) {
      const latestHeight = parseInt(latestBlock.number, 16);
      const start = startHeight ?? Math.max(0, latestHeight - limit + 1);
      const end = latestHeight;
      for (let h = start; h <= end; h++) {
        try {
          const block = await this.getBlockByNumber(h);
          if (!block) continue;
          const hash: string = block.hash;
          const parent: string = block.parentHash;
          const node: DAGNode = {
            id: hash,
            hash,
            height: parseInt(block.number, 16),
            timestamp: parseInt(block.timestamp, 16) * 1000,
            isBlue: true,
            blueScore: parseInt(block.number, 16),
            isTip: h === end,
            selectedParent: parent,
            mergeParents: [],
            transactions: (block.transactions || []).length,
            proposer: '0x',
            size: 0,
          };
          nodes.push(node);
          if (parent && !/^0x0+$/.test(parent)) {
            links.push({
              source: parent,
              target: hash,
              isSelected: true,
              linkType: 'SelectedParent',
            } as DAGLink);
          }
        } catch (e) {
          console.warn('Failed to fetch block for DAG:', e);
        }
      }

      if (nodes.length) {
        const tip = nodes[nodes.length - 1];
        tips.push({
          hash: tip.hash,
          height: tip.height,
          timestamp: tip.timestamp,
          blueScore: tip.blueScore,
          cumulativeWeight: BigInt(0),
        } as TipInfo);
      }
    }

    const statistics = {
      totalBlocks: nodes.length,
      blueBlocks: nodes.length,
      redBlocks: 0,
      currentTips: tips.length,
      averageBlueScore: nodes.length ? nodes.reduce((s, n) => s + n.blueScore, 0) / nodes.length : 0,
      maxHeight: nodes.reduce((m, n) => Math.max(m, n.height), 0),
    };

    return { nodes, links, tips, statistics } as DAGData;
  }

  // Chain info
  async getChainId(): Promise<string> {
    const chainId = await this.sendRequest('eth_chainId');
    return parseInt(chainId, 16).toString();
  }

  async getGasPrice(): Promise<string> {
    const gasPrice = await this.sendRequest('eth_gasPrice');
    return parseInt(gasPrice, 16).toString();
  }

  // Custom Citrate methods
  async getModelRegistry(): Promise<any[]> {
    // Prefer new method; fall back to legacy alias if unavailable
    try {
      return await this.sendRequest('citrate_listModels');
    } catch {
      try {
        return await this.sendRequest('citrate_getModels');
      } catch {
        return [];
      }
    }
  }

  async getModelInfo(modelIdHex: string): Promise<any> {
    return this.sendRequest('citrate_getModel', [modelIdHex]);
  }

  async deployModel(modelData: any): Promise<string> {
    try {
      return await this.sendRequest('citrate_deployModel', [modelData]);
    } catch (error) {
      throw new Error(`Failed to deploy model: ${error}`);
    }
  }

  async runInference(request: any): Promise<any> {
    try {
      return await this.sendRequest('citrate_runInference', [request]);
    } catch (error) {
      throw new Error(`Failed to run inference: ${error}`);
    }
  }

  // Health check
  async isConnected(): Promise<boolean> {
    try {
      await this.sendRequest('net_version');
      return true;
    } catch {
      return false;
    }
  }
}

// Export singleton instance
export const rpcClient = new RPCClient();

// Export class for custom instances
export default RPCClient;
