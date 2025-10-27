/**
 * React hooks for Citrate SDK
 * Optional React integration for web applications
 */

import { useState, useEffect, useCallback } from 'react';
import { CitrateClient, CitrateClientConfig } from '../client/CitrateClient';
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

// Check if React is available
let React: any;
try {
  React = require('react');
} catch {
  // React not available - hooks will not work
}

export interface UseCitrateClientOptions extends CitrateClientConfig {
  autoConnect?: boolean;
}

export interface UseCitrateClientReturn {
  client: CitrateClient | null;
  isConnected: boolean;
  isConnecting: boolean;
  error: Error | null;
  connect: () => Promise<void>;
  disconnect: () => void;
  chainId: number | null;
  address: string | null;
  balance: bigint | null;
}

/**
 * Hook for managing Citrate client connection
 */
export function useCitrateClient(options: UseCitrateClientOptions): UseCitrateClientReturn {
  if (!React) {
    throw new Error('React is required to use Citrate React hooks');
  }

  const [client, setClient] = useState<CitrateClient | null>(null);
  const [isConnected, setIsConnected] = useState(false);
  const [isConnecting, setIsConnecting] = useState(false);
  const [error, setError] = useState<Error | null>(null);
  const [chainId, setChainId] = useState<number | null>(null);
  const [address, setAddress] = useState<string | null>(null);
  const [balance, setBalance] = useState<bigint | null>(null);

  const connect = useCallback(async () => {
    setIsConnecting(true);
    setError(null);

    try {
      const newClient = new CitrateClient(options);
      setClient(newClient);

      // Get connection info
      const chainIdResult = await newClient.getChainId();
      setChainId(chainIdResult);

      const clientAddress = newClient.getAddress();
      setAddress(clientAddress);

      if (clientAddress) {
        const balanceResult = await newClient.getBalance();
        setBalance(balanceResult);
      }

      setIsConnected(true);
    } catch (err) {
      setError(err instanceof Error ? err : new Error('Connection failed'));
    } finally {
      setIsConnecting(false);
    }
  }, [options]);

  const disconnect = useCallback(() => {
    setClient(null);
    setIsConnected(false);
    setChainId(null);
    setAddress(null);
    setBalance(null);
    setError(null);
  }, []);

  useEffect(() => {
    if (options.autoConnect !== false) {
      connect();
    }
  }, [connect, options.autoConnect]);

  return {
    client,
    isConnected,
    isConnecting,
    error,
    connect,
    disconnect,
    chainId,
    address,
    balance
  };
}

export interface UseModelDeploymentReturn {
  deploy: (modelData: ArrayBuffer | Uint8Array, config: ModelConfig) => Promise<ModelDeployment>;
  deployment: ModelDeployment | null;
  isDeploying: boolean;
  error: Error | null;
}

/**
 * Hook for model deployment
 */
export function useModelDeployment(client: CitrateClient | null): UseModelDeploymentReturn {
  if (!React) {
    throw new Error('React is required to use Citrate React hooks');
  }

  const [deployment, setDeployment] = useState<ModelDeployment | null>(null);
  const [isDeploying, setIsDeploying] = useState(false);
  const [error, setError] = useState<Error | null>(null);

  const deploy = useCallback(async (
    modelData: ArrayBuffer | Uint8Array,
    config: ModelConfig
  ): Promise<ModelDeployment> => {
    if (!client) {
      throw new Error('Client not connected');
    }

    setIsDeploying(true);
    setError(null);

    try {
      const result = await client.deployModel(modelData, config);
      setDeployment(result);
      return result;
    } catch (err) {
      const error = err instanceof Error ? err : new Error('Deployment failed');
      setError(error);
      throw error;
    } finally {
      setIsDeploying(false);
    }
  }, [client]);

  return {
    deploy,
    deployment,
    isDeploying,
    error
  };
}

export interface UseInferenceReturn {
  execute: (request: InferenceRequest) => Promise<InferenceResult>;
  result: InferenceResult | null;
  isExecuting: boolean;
  error: Error | null;
}

/**
 * Hook for model inference
 */
export function useInference(client: CitrateClient | null): UseInferenceReturn {
  if (!React) {
    throw new Error('React is required to use Citrate React hooks');
  }

  const [result, setResult] = useState<InferenceResult | null>(null);
  const [isExecuting, setIsExecuting] = useState(false);
  const [error, setError] = useState<Error | null>(null);

  const execute = useCallback(async (request: InferenceRequest): Promise<InferenceResult> => {
    if (!client) {
      throw new Error('Client not connected');
    }

    setIsExecuting(true);
    setError(null);

    try {
      const inferenceResult = await client.inference(request);
      setResult(inferenceResult);
      return inferenceResult;
    } catch (err) {
      const error = err instanceof Error ? err : new Error('Inference failed');
      setError(error);
      throw error;
    } finally {
      setIsExecuting(false);
    }
  }, [client]);

  return {
    execute,
    result,
    isExecuting,
    error
  };
}

export interface UseModelInfoReturn {
  modelInfo: ModelInfo | null;
  isLoading: boolean;
  error: Error | null;
  refetch: () => Promise<void>;
}

/**
 * Hook for fetching model information
 */
export function useModelInfo(
  client: CitrateClient | null,
  modelId: string | null
): UseModelInfoReturn {
  if (!React) {
    throw new Error('React is required to use Citrate React hooks');
  }

  const [modelInfo, setModelInfo] = useState<ModelInfo | null>(null);
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<Error | null>(null);

  const refetch = useCallback(async () => {
    if (!client || !modelId) return;

    setIsLoading(true);
    setError(null);

    try {
      const info = await client.getModelInfo(modelId);
      setModelInfo(info);
    } catch (err) {
      setError(err instanceof Error ? err : new Error('Failed to fetch model info'));
    } finally {
      setIsLoading(false);
    }
  }, [client, modelId]);

  useEffect(() => {
    refetch();
  }, [refetch]);

  return {
    modelInfo,
    isLoading,
    error,
    refetch
  };
}

export interface UseModelListReturn {
  models: ModelInfo[];
  isLoading: boolean;
  error: Error | null;
  refetch: () => Promise<void>;
}

/**
 * Hook for listing models
 */
export function useModelList(
  client: CitrateClient | null,
  owner?: string,
  limit: number = 100
): UseModelListReturn {
  if (!React) {
    throw new Error('React is required to use Citrate React hooks');
  }

  const [models, setModels] = useState<ModelInfo[]>([]);
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<Error | null>(null);

  const refetch = useCallback(async () => {
    if (!client) return;

    setIsLoading(true);
    setError(null);

    try {
      const modelList = await client.listModels(owner, limit);
      setModels(modelList);
    } catch (err) {
      setError(err instanceof Error ? err : new Error('Failed to fetch models'));
    } finally {
      setIsLoading(false);
    }
  }, [client, owner, limit]);

  useEffect(() => {
    refetch();
  }, [refetch]);

  return {
    models,
    isLoading,
    error,
    refetch
  };
}