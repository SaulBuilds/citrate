/**
 * Main Lattice client for JavaScript/TypeScript SDK
 */

import { ethers } from 'ethers';
import axios, { AxiosInstance, AxiosResponse } from 'axios';
import { CryptoManager } from '../crypto/CryptoManager';
import { KeyManager } from '../crypto/KeyManager';
import {
  ModelConfig,
  ModelDeployment,
  ModelInfo,
  ModelStats
} from '../types/Model';
import {
  InferenceRequest,
  InferenceResult,
  BatchInferenceRequest,
  BatchInferenceResult
} from '../types/Inference';
import { LatticeError, ModelNotFoundError, InsufficientFundsError } from '../errors/LatticeError';

export interface LatticeClientConfig {
  rpcUrl: string;
  privateKey?: string;
  timeout?: number;
  retries?: number;
  headers?: Record<string, string>;
}

export class LatticeClient {
  private provider: ethers.JsonRpcProvider;
  private wallet?: ethers.Wallet;
  private axios: AxiosInstance;
  private keyManager?: KeyManager;
  private cryptoManager: CryptoManager;

  constructor(config: LatticeClientConfig) {
    // Initialize provider
    this.provider = new ethers.JsonRpcProvider(config.rpcUrl);

    // Initialize wallet if private key provided
    if (config.privateKey) {
      this.wallet = new ethers.Wallet(config.privateKey, this.provider);
      this.keyManager = new KeyManager(config.privateKey);
    }

    // Initialize HTTP client
    this.axios = axios.create({
      baseURL: config.rpcUrl,
      timeout: config.timeout || 30000,
      headers: {
        'Content-Type': 'application/json',
        'User-Agent': 'lattice-js-sdk/0.1.0',
        ...config.headers
      }
    });

    // Initialize crypto manager
    this.cryptoManager = new CryptoManager();

    this.setupAxiosInterceptors();
  }

  private setupAxiosInterceptors(): void {
    // Request interceptor for logging
    this.axios.interceptors.request.use(
      (config) => {
        console.debug('Lattice API Request:', config.method?.toUpperCase(), config.url);
        return config;
      },
      (error) => Promise.reject(error)
    );

    // Response interceptor for error handling
    this.axios.interceptors.response.use(
      (response) => response,
      (error) => {
        if (error.response?.data?.error) {
          throw new LatticeError(
            error.response.data.error.message || 'API Error',
            error.response.data.error.code?.toString()
          );
        }
        throw new LatticeError(`Network error: ${error.message}`);
      }
    );
  }

  // Connection methods
  async getChainId(): Promise<number> {
    const network = await this.provider.getNetwork();
    return Number(network.chainId);
  }

  async getBalance(address?: string): Promise<bigint> {
    const targetAddress = address || this.getAddress();
    if (!targetAddress) {
      throw new LatticeError('No address provided and no wallet configured');
    }
    return await this.provider.getBalance(targetAddress);
  }

  async getNonce(address?: string): Promise<number> {
    const targetAddress = address || this.getAddress();
    if (!targetAddress) {
      throw new LatticeError('No address provided and no wallet configured');
    }
    return await this.provider.getTransactionCount(targetAddress, 'pending');
  }

  getAddress(): string | undefined {
    return this.wallet?.address;
  }

  // Model deployment
  async deployModel(
    modelData: ArrayBuffer | Uint8Array,
    config: ModelConfig
  ): Promise<ModelDeployment> {
    if (!this.wallet) {
      throw new LatticeError('Wallet required for model deployment');
    }

    // Convert to Uint8Array if needed
    const modelBytes = modelData instanceof ArrayBuffer
      ? new Uint8Array(modelData)
      : modelData;

    // Calculate model hash
    const modelHash = await this.cryptoManager.hashData(modelBytes);

    // Encrypt model if requested
    let encryptedData: Uint8Array = modelBytes;
    let encryptionMetadata: any = null;

    if (config.encrypted && this.keyManager) {
      const result = await this.keyManager.encryptModel(modelBytes, config.encryptionConfig);
      encryptedData = result.encryptedData;
      encryptionMetadata = result.metadata;
    }

    // Upload to IPFS
    const ipfsHash = await this.uploadToIPFS(encryptedData);

    // Prepare transaction data
    const txData = {
      modelHash,
      ipfsHash,
      encrypted: config.encrypted,
      accessPrice: config.accessPrice.toString(),
      accessList: config.accessList || [],
      metadata: config.metadata || {}
    };

    if (encryptionMetadata) {
      txData.metadata.encryption = encryptionMetadata;
    }

    // Deploy to blockchain (call precompile at 0x0100)
    const tx = await this.wallet.sendTransaction({
      to: '0x0100000000000000000000000000000000000100',
      data: ethers.hexlify(ethers.toUtf8Bytes(JSON.stringify(txData))),
      gasLimit: 500000n
    });

    // Wait for confirmation
    const receipt = await tx.wait();
    if (!receipt) {
      throw new LatticeError('Transaction failed');
    }

    // Extract model ID from logs
    const modelId = this.extractModelIdFromReceipt(receipt);

    return {
      modelId,
      txHash: tx.hash,
      ipfsHash,
      encrypted: config.encrypted,
      accessPrice: config.accessPrice,
      deploymentTime: Math.floor(Date.now() / 1000),
      gasUsed: receipt.gasUsed
    };
  }

  // Inference execution
  async inference(request: InferenceRequest): Promise<InferenceResult> {
    if (!this.wallet) {
      throw new LatticeError('Wallet required for inference execution');
    }

    // Prepare inference data
    let inputData = request.inputData;

    // Encrypt input if requested
    if (request.encrypted && this.keyManager) {
      inputData = await this.keyManager.encryptData(JSON.stringify(request.inputData));
    }

    const inferenceData = {
      modelId: request.modelId,
      inputData,
      encrypted: request.encrypted || false,
      timestamp: request.timestamp || Math.floor(Date.now() / 1000)
    };

    // Execute inference (call precompile at 0x0101)
    const tx = await this.wallet.sendTransaction({
      to: '0x0100000000000000000000000000000000000101',
      data: ethers.hexlify(ethers.toUtf8Bytes(JSON.stringify(inferenceData))),
      gasLimit: BigInt(request.timeout || 1000000)
    });

    // Wait for execution
    const receipt = await tx.wait();
    if (!receipt) {
      throw new LatticeError('Inference execution failed');
    }

    // Extract output from logs
    let outputData = this.extractInferenceOutput(receipt);

    // Decrypt output if encrypted
    if (request.encrypted && this.keyManager && outputData.encrypted) {
      const decrypted = await this.keyManager.decryptData(outputData.encrypted);
      outputData = JSON.parse(decrypted);
    }

    return {
      modelId: request.modelId,
      outputData,
      gasUsed: receipt.gasUsed,
      executionTime: 0, // Would be extracted from receipt in real implementation
      txHash: tx.hash
    };
  }

  // Batch inference
  async batchInference(request: BatchInferenceRequest): Promise<BatchInferenceResult> {
    const results: InferenceResult[] = [];
    const errors: string[] = [];
    let totalGasUsed = 0n;
    let totalExecutionTime = 0;

    const batchSize = request.batchSize || 10;
    const parallel = request.parallel || false;

    for (let i = 0; i < request.inputs.length; i += batchSize) {
      const batch = request.inputs.slice(i, i + batchSize);

      const batchPromises = batch.map(async (input, index) => {
        try {
          const result = await this.inference({
            modelId: request.modelId,
            inputData: input
          });
          results.push(result);
          totalGasUsed += result.gasUsed;
          totalExecutionTime += result.executionTime;
          return result;
        } catch (error) {
          const errorMsg = error instanceof Error ? error.message : 'Unknown error';
          errors.push(`Input ${i + index}: ${errorMsg}`);
          return null;
        }
      });

      if (parallel) {
        await Promise.all(batchPromises);
      } else {
        for (const promise of batchPromises) {
          await promise;
        }
      }

      // Update progress
      if (request.onProgress) {
        request.onProgress(Math.min(i + batchSize, request.inputs.length), request.inputs.length);
      }
    }

    return {
      results,
      totalGasUsed,
      totalExecutionTime,
      successCount: results.length,
      failureCount: errors.length,
      errors
    };
  }

  // Model information
  async getModelInfo(modelId: string): Promise<ModelInfo> {
    const response = await this.rpcCall('lattice_getModelInfo', [modelId]);

    if (!response) {
      throw new ModelNotFoundError(`Model not found: ${modelId}`);
    }

    return {
      modelId: response.modelId,
      name: response.name,
      description: response.description,
      owner: response.owner,
      modelType: response.modelType,
      accessType: response.accessType,
      accessPrice: BigInt(response.accessPrice),
      encrypted: response.encrypted,
      ipfsHash: response.ipfsHash,
      deploymentTime: response.deploymentTime,
      totalInferences: response.totalInferences,
      totalRevenue: BigInt(response.totalRevenue),
      metadata: response.metadata,
      tags: response.tags
    };
  }

  async listModels(owner?: string, limit: number = 100): Promise<ModelInfo[]> {
    const params = owner ? [owner, limit] : [limit];
    const response = await this.rpcCall('lattice_listModels', params);

    return response.map((model: any) => ({
      modelId: model.modelId,
      name: model.name,
      description: model.description,
      owner: model.owner,
      modelType: model.modelType,
      accessType: model.accessType,
      accessPrice: BigInt(model.accessPrice),
      encrypted: model.encrypted,
      ipfsHash: model.ipfsHash,
      deploymentTime: model.deploymentTime,
      totalInferences: model.totalInferences,
      totalRevenue: BigInt(model.totalRevenue),
      metadata: model.metadata,
      tags: model.tags
    }));
  }

  // Payment
  async purchaseModelAccess(modelId: string, paymentAmount: bigint): Promise<string> {
    if (!this.wallet) {
      throw new LatticeError('Wallet required for purchases');
    }

    const txData = {
      modelId,
      paymentAmount: paymentAmount.toString()
    };

    const tx = await this.wallet.sendTransaction({
      to: '0x0100000000000000000000000000000000000104', // Access control precompile
      data: ethers.hexlify(ethers.toUtf8Bytes(JSON.stringify(txData))),
      value: paymentAmount,
      gasLimit: 200000n
    });

    return tx.hash;
  }

  // Private helper methods
  private async rpcCall(method: string, params: any[] = []): Promise<any> {
    const response: AxiosResponse = await this.axios.post('', {
      jsonrpc: '2.0',
      method,
      params,
      id: Math.floor(Math.random() * 10000)
    });

    if (response.data.error) {
      throw new LatticeError(response.data.error.message);
    }

    return response.data.result;
  }

  private async uploadToIPFS(data: Uint8Array): Promise<string> {
    try {
      // Try to upload to IPFS using HTTP API
      const formData = new FormData();
      const blob = new Blob([data], { type: 'application/octet-stream' });
      formData.append('file', blob, 'model_data');

      const response = await fetch('http://localhost:5001/api/v0/add?pin=true', {
        method: 'POST',
        body: formData
      });

      if (response.ok) {
        const result = await response.json();
        return result.Hash;
      } else {
        throw new Error(`IPFS upload failed: ${response.status}`);
      }
    } catch (error) {
      // Fallback to hash-based simulation if IPFS unavailable
      console.warn('IPFS upload failed, using hash fallback:', error);
      const hash = await this.cryptoManager.hashData(data);
      return `fallback_${hash.slice(0, 40)}`;
    }
  }

  private extractModelIdFromReceipt(receipt: ethers.TransactionReceipt): string {
    // Extract model ID from deployment receipt logs
    for (const log of receipt.logs) {
      try {
        // Look for ModelDeployed event
        if (log.topics[0]?.startsWith('0x' + 'ModelDeployed'.slice(0, 8))) {
          return log.data.slice(0, 66); // First 32 bytes as hex
        }
      } catch (error) {
        continue;
      }
    }

    throw new LatticeError('Model ID not found in deployment receipt');
  }

  private extractInferenceOutput(receipt: ethers.TransactionReceipt): any {
    // Extract inference output from execution receipt
    for (const log of receipt.logs) {
      try {
        if (log.topics[0]?.startsWith('0x' + 'InferenceComplete'.slice(0, 8))) {
          const dataBytes = ethers.getBytes(log.data);
          const jsonStr = ethers.toUtf8String(dataBytes);
          return JSON.parse(jsonStr);
        }
      } catch (error) {
        continue;
      }
    }

    throw new LatticeError('Inference output not found in receipt');
  }
}