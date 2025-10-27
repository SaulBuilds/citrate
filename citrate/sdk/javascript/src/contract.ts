import { AxiosInstance } from 'axios';
import { ethers } from 'ethers';
import { CitrateConfig, ContractInfo } from './types';

export class ContractManager {
  constructor(
    private rpcClient: AxiosInstance,
    private config: CitrateConfig
  ) {}
  
  /**
   * Deploy a contract
   */
  async deploy(
    bytecode: string,
    abi?: any[],
    constructorArgs?: any[],
    value?: string
  ): Promise<string> {
    const data = bytecode.startsWith('0x') ? bytecode : `0x${bytecode}`;
    
    // If constructor args provided, encode them
    let deployData = data;
    if (abi && constructorArgs && constructorArgs.length > 0) {
      const iface = new ethers.Interface(abi);
      const constructor = abi.find(item => item.type === 'constructor');
      if (constructor) {
        const encoded = iface.encodeDeploy(constructorArgs);
        deployData = ethers.concat([data, encoded]);
      }
    }
    
    const response = await this.rpcCall('eth_sendTransaction', [{
      from: this.config.defaultAccount,
      data: deployData,
      value: value || '0x0',
      gas: `0x${(this.config.gasLimit || 3000000).toString(16)}`,
      gasPrice: this.config.gasPrice || '0x3b9aca00' // 1 gwei
    }]);
    
    // Wait for transaction receipt
    const receipt = await this.waitForReceipt(response);
    
    if (!receipt.contractAddress) {
      throw new Error('Contract deployment failed');
    }
    
    return receipt.contractAddress;
  }
  
  /**
   * Call a contract method (transaction)
   */
  async call(
    address: string,
    abi: any[],
    method: string,
    args: any[] = [],
    value?: string
  ): Promise<any> {
    const iface = new ethers.Interface(abi);
    const data = iface.encodeFunctionData(method, args);
    
    const response = await this.rpcCall('eth_sendTransaction', [{
      from: this.config.defaultAccount,
      to: address,
      data,
      value: value || '0x0',
      gas: `0x${(this.config.gasLimit || 100000).toString(16)}`,
      gasPrice: this.config.gasPrice || '0x3b9aca00'
    }]);
    
    const receipt = await this.waitForReceipt(response);
    
    // Decode logs if any
    const logs = receipt.logs.map((log: any) => {
      try {
        return iface.parseLog({
          topics: log.topics,
          data: log.data
        });
      } catch {
        return log;
      }
    });
    
    return { receipt, logs };
  }
  
  /**
   * Read contract state (call without transaction)
   */
  async read(
    address: string,
    abi: any[],
    method: string,
    args: any[] = []
  ): Promise<any> {
    const iface = new ethers.Interface(abi);
    const data = iface.encodeFunctionData(method, args);
    
    const response = await this.rpcCall('eth_call', [{
      to: address,
      data
    }, 'latest']);
    
    // Decode the result
    const result = iface.decodeFunctionResult(method, response);
    
    // If single return value, return it directly
    if (result.length === 1) {
      return result[0];
    }
    
    return result;
  }
  
  /**
   * Get contract code
   */
  async getCode(address: string): Promise<string> {
    return await this.rpcCall('eth_getCode', [address, 'latest']);
  }
  
  /**
   * Get contract info
   */
  async getContractInfo(address: string): Promise<ContractInfo> {
    const code = await this.getCode(address);
    
    return {
      address,
      bytecode: code,
      verified: false // Would check verification status
    };
  }
  
  /**
   * Verify contract source code
   */
  async verify(
    address: string,
    sourceCode: string,
    compilerVersion: string,
    optimizationEnabled: boolean = false
  ): Promise<string> {
    const response = await this.rpcCall('citrate_verifyContract', [{
      address,
      source_code: sourceCode,
      compiler_version: compilerVersion,
      optimization_enabled: optimizationEnabled
    }]);
    
    return response.verification_id;
  }
  
  /**
   * Create contract instance with ABI
   */
  createInstance(address: string, abi: any[]): ContractInstance {
    return new ContractInstance(address, abi, this);
  }
  
  private async waitForReceipt(txHash: string): Promise<any> {
    let receipt = null;
    let attempts = 0;
    
    while (!receipt && attempts < 30) {
      attempts++;
      await new Promise(resolve => setTimeout(resolve, 2000));
      
      try {
        receipt = await this.rpcCall('eth_getTransactionReceipt', [txHash]);
      } catch {
        // Not yet mined
      }
    }
    
    if (!receipt) {
      throw new Error('Transaction timeout');
    }
    
    if (receipt.status === '0x0') {
      throw new Error('Transaction failed');
    }
    
    return receipt;
  }
  
  async rpcCall(method: string, params: any[]): Promise<any> {
    const response = await this.rpcClient.post('', {
      jsonrpc: '2.0',
      method,
      params,
      id: Date.now()
    });
    
    if (response.data.error) {
      throw new Error(response.data.error.message);
    }
    
    return response.data.result;
  }
}

/**
 * Contract instance helper class
 */
export class ContractInstance {
  private iface: ethers.Interface;
  
  constructor(
    public address: string,
    public abi: any[],
    private manager: ContractManager
  ) {
    this.iface = new ethers.Interface(abi);
  }
  
  /**
   * Call a method (transaction)
   */
  async send(method: string, args: any[] = [], value?: string): Promise<any> {
    return this.manager.call(this.address, this.abi, method, args, value);
  }
  
  /**
   * Read state (no transaction)
   */
  async call(method: string, args: any[] = []): Promise<any> {
    return this.manager.read(this.address, this.abi, method, args);
  }
  
  /**
   * Encode method call data
   */
  encodeCall(method: string, args: any[] = []): string {
    return this.iface.encodeFunctionData(method, args);
  }
  
  /**
   * Decode method result
   */
  decodeResult(method: string, data: string): any {
    return this.iface.decodeFunctionResult(method, data);
  }
  
  /**
   * Parse event logs
   */
  parseLogs(logs: any[]): any[] {
    return logs.map(log => {
      try {
        return this.iface.parseLog({
          topics: log.topics,
          data: log.data
        });
      } catch {
        return log;
      }
    });
  }
}
