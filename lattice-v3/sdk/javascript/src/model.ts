import { AxiosInstance } from 'axios';
import { LatticeConfig, ModelMetadata, ModelInfo, InferenceResult } from './types';

export class ModelRegistry {
  constructor(
    private rpcClient: AxiosInstance,
    private config: LatticeConfig
  ) {}
  
  /**
   * Deploy a model to the network
   */
  async deploy(
    modelData: Buffer | Uint8Array,
    metadata: ModelMetadata
  ): Promise<string> {
    const base64Data = Buffer.from(modelData).toString('base64');
    
    const response = await this.rpcCall('lattice_deployModel', {
      from: this.config.defaultAccount,
      model_data: base64Data,
      metadata,
      gas_price: this.config.gasPrice,
      gas_limit: this.config.gasLimit
    });
    
    return response.model_id;
  }
  
  /**
   * Run inference on a model
   */
  async runInference(
    modelId: string,
    input: any,
    options?: {
      withProof?: boolean;
      timeout?: number;
    }
  ): Promise<InferenceResult> {
    const response = await this.rpcCall('lattice_runInference', {
      model_id: modelId,
      input,
      with_proof: options?.withProof || false,
      timeout: options?.timeout
    });
    
    return {
      output: response.output,
      executionTime: response.execution_time_ms,
      proofId: response.proof,
      gasUsed: response.gas_used
    };
  }
  
  /**
   * Get model information
   */
  async getModel(modelId: string): Promise<ModelInfo> {
    const response = await this.rpcCall('lattice_getModel', {
      model_id: modelId
    });
    
    return response.model;
  }
  
  /**
   * List models
   */
  async listModels(options?: {
    owner?: string;
    type?: string;
    limit?: number;
    offset?: number;
  }): Promise<ModelInfo[]> {
    const response = await this.rpcCall('lattice_listModels', {
      owner: options?.owner,
      type: options?.type,
      limit: options?.limit || 10,
      offset: options?.offset || 0
    });
    
    return response.models;
  }
  
  /**
   * Update model metadata
   */
  async updateMetadata(
    modelId: string,
    metadata: Partial<ModelMetadata>
  ): Promise<void> {
    await this.rpcCall('lattice_updateModel', {
      model_id: modelId,
      metadata,
      from: this.config.defaultAccount
    });
  }
  
  /**
   * Grant permissions for a model
   */
  async grantPermission(
    modelId: string,
    address: string,
    permission: 'read' | 'execute' | 'update'
  ): Promise<void> {
    await this.rpcCall('lattice_grantModelPermission', {
      model_id: modelId,
      address,
      permission,
      from: this.config.defaultAccount
    });
  }
  
  /**
   * Revoke permissions for a model
   */
  async revokePermission(
    modelId: string,
    address: string,
    permission: 'read' | 'execute' | 'update'
  ): Promise<void> {
    await this.rpcCall('lattice_revokeModelPermission', {
      model_id: modelId,
      address,
      permission,
      from: this.config.defaultAccount
    });
  }
  
  private async rpcCall(method: string, params: any): Promise<any> {
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