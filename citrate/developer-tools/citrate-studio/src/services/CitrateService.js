import axios from 'axios';
import Web3 from 'web3';

export class CitrateService {
  constructor() {
    this.rpcUrl = process.env.REACT_APP_LATTICE_RPC || 'http://localhost:8545';
    this.web3 = new Web3(this.rpcUrl);
    this.client = axios.create({
      baseURL: this.rpcUrl,
      timeout: 10000,
      headers: {
        'Content-Type': 'application/json',
      },
    });
  }

  async rpcCall(method, params = []) {
    try {
      const response = await this.client.post('/', {
        jsonrpc: '2.0',
        method,
        params,
        id: Date.now(),
      });

      if (response.data.error) {
        throw new Error(response.data.error.message);
      }

      return response.data.result;
    } catch (error) {
      console.error(`RPC call failed: ${method}`, error);
      throw error;
    }
  }

  // Network Status and Information
  async getNetworkStatus() {
    try {
      const chainId = await this.rpcCall('eth_chainId');
      const blockNumber = await this.rpcCall('eth_blockNumber');
      const peerCount = await this.rpcCall('net_peerCount');

      return {
        connected: true,
        chainId: parseInt(chainId, 16),
        blockNumber: parseInt(blockNumber, 16),
        peerCount: parseInt(peerCount, 16),
        timestamp: new Date().toISOString(),
      };
    } catch (error) {
      return null;
    }
  }

  async getChainInfo() {
    try {
      const [chainId, blockNumber, gasPrice, peerCount] = await Promise.all([
        this.rpcCall('eth_chainId'),
        this.rpcCall('eth_blockNumber'),
        this.rpcCall('eth_gasPrice'),
        this.rpcCall('net_peerCount'),
      ]);

      return {
        chainId: parseInt(chainId, 16),
        latestBlock: parseInt(blockNumber, 16),
        gasPrice: parseInt(gasPrice, 16),
        peerCount: parseInt(peerCount, 16),
      };
    } catch (error) {
      throw new Error(`Failed to get chain info: ${error.message}`);
    }
  }

  // Block and Transaction Information
  async getBlock(blockNumber = 'latest') {
    return await this.rpcCall('eth_getBlockByNumber', [
      typeof blockNumber === 'number' ? `0x${blockNumber.toString(16)}` : blockNumber,
      true
    ]);
  }

  async getTransaction(txHash) {
    return await this.rpcCall('eth_getTransactionByHash', [txHash]);
  }

  async getTransactionReceipt(txHash) {
    return await this.rpcCall('eth_getTransactionReceipt', [txHash]);
  }

  // Account Management
  async getBalance(address) {
    const balance = await this.rpcCall('eth_getBalance', [address, 'latest']);
    return this.web3.utils.fromWei(balance, 'ether');
  }

  async getAccounts() {
    return await this.rpcCall('eth_accounts');
  }

  // Model Management (Lattice-specific)
  async getModels() {
    try {
      return await this.rpcCall('citrate_listModels');
    } catch (_) {
      try {
        return await this.rpcCall('citrate_getModels');
      } catch (error) {
        return [];
      }
    }
  }

  async getModelInfo(modelId) {
    try {
      return await this.rpcCall('citrate_getModel', [modelId]);
    } catch (error) {
      throw new Error(`Failed to get model info: ${error.message}`);
    }
  }

  async deployModel(modelData) {
    try {
      return await this.rpcCall('citrate_deployModel', [modelData]);
    } catch (error) {
      throw new Error(`Failed to deploy model: ${error.message}`);
    }
  }

  async runInference(modelId, inputData) {
    try {
      const payload = [{ model_id: modelId, input: inputData, max_gas: 1000000, with_proof: false }];
      return await this.rpcCall('citrate_runInference', payload);
    } catch (error) {
      throw new Error(`Failed to run inference: ${error.message}`);
    }
  }

  // Network Topology (Lattice-specific)
  async getPeers() {
    try {
      return await this.rpcCall('citrate_getPeers');
    } catch (error) {
      // Fallback for standard node
      return [];
    }
  }

  async getNetworkTopology() {
    try {
      return await this.rpcCall('citrate_getNetworkTopology');
    } catch (error) {
      // Fallback - construct basic topology from available data
      const peers = await this.getPeers();
      return {
        nodes: peers.map((peer, index) => ({
          id: peer.id || `peer_${index}`,
          address: peer.address,
          type: 'peer',
        })),
        edges: [],
      };
    }
  }

  // Smart Contract Interaction
  async sendTransaction(transaction) {
    return await this.rpcCall('eth_sendTransaction', [transaction]);
  }

  async call(transaction) {
    return await this.rpcCall('eth_call', [transaction, 'latest']);
  }

  async estimateGas(transaction) {
    const gas = await this.rpcCall('eth_estimateGas', [transaction]);
    return parseInt(gas, 16);
  }

  // Debug and Monitoring
  async getTransactionTrace(txHash) {
    try {
      return await this.rpcCall('debug_traceTransaction', [txHash]);
    } catch (error) {
      // Debug methods might not be available
      return null;
    }
  }

  async getMempool() {
    try {
      return await this.rpcCall('txpool_content');
    } catch (error) {
      return { pending: {}, queued: {} };
    }
  }

  // Model Analytics (Lattice-specific)
  async getModelStats(modelId, timeRange = '24h') {
    try {
      return await this.rpcCall('citrate_getModelStats', [modelId, timeRange]);
    } catch (error) {
      // Return mock data if not available
      return {
        totalInferences: 0,
        avgExecutionTime: 0,
        totalRevenue: '0',
        successRate: 100,
      };
    }
  }

  // File and Storage Operations
  async uploadToIPFS(data) {
    try {
      // This would connect to IPFS node
      const formData = new FormData();
      formData.append('file', new Blob([data]), 'model_data');

      const response = await axios.post('http://localhost:5001/api/v0/add', formData, {
        headers: { 'Content-Type': 'multipart/form-data' },
      });

      return response.data.Hash;
    } catch (error) {
      throw new Error(`Failed to upload to IPFS: ${error.message}`);
    }
  }

  // Real-time Updates
  subscribeToBlocks(callback) {
    // WebSocket subscription for real-time block updates
    const ws = new WebSocket(this.rpcUrl.replace('http', 'ws'));

    ws.onopen = () => {
      ws.send(JSON.stringify({
        jsonrpc: '2.0',
        method: 'eth_subscribe',
        params: ['newHeads'],
        id: 1,
      }));
    };

    ws.onmessage = (event) => {
      const data = JSON.parse(event.data);
      if (data.method === 'eth_subscription') {
        callback(data.params.result);
      }
    };

    return () => ws.close();
  }

  // Utility Functions
  formatAddress(address) {
    if (!address) return '';
    return `${address.slice(0, 6)}...${address.slice(-4)}`;
  }

  formatHash(hash) {
    if (!hash) return '';
    return `${hash.slice(0, 10)}...${hash.slice(-8)}`;
  }

  formatWei(wei) {
    return this.web3.utils.fromWei(wei.toString(), 'ether');
  }

  formatGwei(wei) {
    return this.web3.utils.fromWei(wei.toString(), 'gwei');
  }
}
