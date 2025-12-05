/**
 * useAvailableModels Hook
 *
 * Fetches available AI models from the ModelRegistry smart contract
 * and combines with genesis-embedded models for the chat interface.
 */

import { useState, useEffect, useCallback } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { callContractFunction } from '../utils/contractInteraction';

export interface AvailableModel {
  id: string;  // Model hash or identifier
  name: string;
  provider: 'citrate' | 'genesis' | 'ipfs';
  type: 'text' | 'embedding' | 'vision' | 'code';
  framework: string;
  inferencePrice: string; // In LATT wei
  available: boolean; // Whether model is pinned/ready
  isPinned: boolean;
  metadata?: {
    version?: string;
    description?: string;
    maxTokens?: number;
  };
}

interface ModelRegistryConfig {
  contractAddress: string;
  enabled: boolean;
}

const GENESIS_MODELS: AvailableModel[] = [
  {
    id: 'bge-m3',
    name: 'BGE-M3 Embeddings',
    provider: 'genesis',
    type: 'embedding',
    framework: 'onnx',
    inferencePrice: '0',
    available: true,
    isPinned: true,
    metadata: {
      version: '1.0.0',
      description: 'Genesis-embedded BGE-M3 model for text embeddings (1024 dimensions)',
      maxTokens: 512,
    },
  },
];

const IPFS_MODELS: AvailableModel[] = [
  {
    id: 'mistral-7b-instruct-v0.3',
    name: 'Mistral 7B Instruct v0.3',
    provider: 'ipfs',
    type: 'text',
    framework: 'gguf',
    inferencePrice: '0',
    available: true,
    isPinned: true,
    metadata: {
      version: '0.3',
      description: 'IPFS-pinned Mistral 7B model for text generation',
      maxTokens: 8192,
    },
  },
  {
    id: 'qwen2-0.5b',
    name: 'Qwen2 0.5B (Q4)',
    provider: 'ipfs',
    type: 'text',
    framework: 'gguf',
    inferencePrice: '0',
    available: true,
    isPinned: true,
    metadata: {
      version: '2.0',
      description: 'IPFS-pinned Qwen2 0.5B model - fast, lightweight inference',
      maxTokens: 4096,
    },
  },
];

export function useAvailableModels() {
  const [models, setModels] = useState<AvailableModel[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [registryConfig, setRegistryConfig] = useState<ModelRegistryConfig | null>(null);

  // Fetch registry configuration from config
  const fetchRegistryConfig = useCallback(async () => {
    try {
      const config = await invoke<any>('get_config');
      if (config?.modelRegistry) {
        setRegistryConfig(config.modelRegistry);
      } else {
        // Use default/fallback config
        setRegistryConfig({
          contractAddress: '0x0000000000000000000000000000000000000000', // Will be deployed
          enabled: false, // Disable until contract is deployed
        });
      }
    } catch (err) {
      console.warn('[useAvailableModels] Failed to fetch registry config:', err);
      setRegistryConfig({
        contractAddress: '0x0000000000000000000000000000000000000000',
        enabled: false,
      });
    }
  }, []);

  // Fetch models from ModelRegistry contract
  const fetchRegistryModels = useCallback(async (contractAddress: string): Promise<AvailableModel[]> => {
    try {
      console.log('[useAvailableModels] Fetching models from registry:', contractAddress);

      // Call getAllModelHashes
      const hashesResult = await callContractFunction({
        contractAddress,
        functionName: 'getAllModelHashes',
        inputs: [],
        outputs: [{ name: 'hashes', type: 'bytes32[]' }],
        args: [],
      });

      const modelHashes = hashesResult.outputs[0] as string[];

      if (!modelHashes || modelHashes.length === 0) {
        console.log('[useAvailableModels] No models in registry');
        return [];
      }

      console.log('[useAvailableModels] Found', modelHashes.length, 'models');

      // Fetch info for all models
      const infoResult = await callContractFunction({
        contractAddress,
        functionName: 'getModelsInfo',
        inputs: [{ name: 'modelHashes', type: 'bytes32[]' }],
        outputs: [
          { name: 'names', type: 'string[]' },
          { name: 'frameworks', type: 'string[]' },
          { name: 'prices', type: 'uint256[]' },
          { name: 'activeStates', type: 'bool[]' },
        ],
        args: [modelHashes],
      });

      const [names, frameworks, prices, activeStates] = infoResult.outputs;

      // Convert to AvailableModel format
      const registryModels: AvailableModel[] = modelHashes.map((hash, i) => ({
        id: hash,
        name: names[i] || `Model ${i}`,
        provider: 'citrate',
        type: 'text', // Default type, could be extended
        framework: frameworks[i] || 'unknown',
        inferencePrice: prices[i]?.toString() || '0',
        available: activeStates[i] ?? false,
        isPinned: activeStates[i] ?? false,
        metadata: {
          version: '1.0.0',
          description: `Registry model: ${names[i]}`,
        },
      }));

      console.log('[useAvailableModels] Loaded', registryModels.length, 'registry models');
      return registryModels;
    } catch (err) {
      console.error('[useAvailableModels] Error fetching registry models:', err);
      return [];
    }
  }, []);

  // Fetch locally downloaded GGUF models
  const fetchLocalModels = useCallback(async (): Promise<AvailableModel[]> => {
    try {
      // Scan for local models using agent command
      const localModels = await invoke<any[]>('agent_scan_local_models', { directory: null });

      if (!localModels || localModels.length === 0) {
        console.log('[useAvailableModels] No local models found');
        return [];
      }

      console.log('[useAvailableModels] Found', localModels.length, 'local models');

      return localModels.map((model: any) => ({
        id: model.path, // Use full path as ID for local models
        name: model.name,
        provider: 'ipfs' as const,
        type: model.name.toLowerCase().includes('embed') ? 'embedding' as const : 'text' as const,
        framework: 'gguf',
        inferencePrice: '0',
        available: true,
        isPinned: true,
        metadata: {
          description: `Local GGUF model: ${model.name}`,
          version: model.quantization,
        },
      }));
    } catch (err) {
      console.warn('[useAvailableModels] Error scanning local models:', err);
      return [];
    }
  }, []);

  // Fetch HuggingFace downloaded models
  const fetchHFModels = useCallback(async (): Promise<AvailableModel[]> => {
    try {
      const hfModels = await invoke<string[]>('hf_get_local_models');

      if (!hfModels || hfModels.length === 0) {
        console.log('[useAvailableModels] No HuggingFace models found');
        return [];
      }

      console.log('[useAvailableModels] Found', hfModels.length, 'HuggingFace models');

      return hfModels.map((modelPath: string) => {
        const name = modelPath.split('/').pop() || modelPath;
        return {
          id: modelPath,
          name: name.replace('.gguf', ''),
          provider: 'ipfs' as const,
          type: name.toLowerCase().includes('embed') ? 'embedding' as const : 'text' as const,
          framework: 'gguf',
          inferencePrice: '0',
          available: true,
          isPinned: true,
          metadata: {
            description: `HuggingFace model: ${name}`,
          },
        };
      });
    } catch (err) {
      console.warn('[useAvailableModels] Error fetching HuggingFace models:', err);
      return [];
    }
  }, []);

  // Main fetch function
  const fetchModels = useCallback(async () => {
    setLoading(true);
    setError(null);

    try {
      let allModels = [...GENESIS_MODELS, ...IPFS_MODELS];

      // Fetch models from registry if enabled
      if (registryConfig?.enabled && registryConfig.contractAddress) {
        const registryModels = await fetchRegistryModels(registryConfig.contractAddress);
        allModels = [...allModels, ...registryModels];
      }

      // Fetch local GGUF models
      const localModels = await fetchLocalModels();

      // Fetch HuggingFace downloaded models
      const hfModels = await fetchHFModels();

      // Merge models, avoiding duplicates by path/id
      const existingIds = new Set(allModels.map(m => m.id));
      for (const model of [...localModels, ...hfModels]) {
        if (!existingIds.has(model.id)) {
          allModels.push(model);
          existingIds.add(model.id);
        }
      }

      setModels(allModels);
    } catch (err) {
      console.error('[useAvailableModels] Error fetching models:', err);
      setError(err instanceof Error ? err.message : 'Failed to fetch models');
      // Fallback to genesis + IPFS models
      setModels([...GENESIS_MODELS, ...IPFS_MODELS]);
    } finally {
      setLoading(false);
    }
  }, [registryConfig, fetchRegistryModels, fetchLocalModels, fetchHFModels]);

  // Initialize
  useEffect(() => {
    fetchRegistryConfig();
  }, [fetchRegistryConfig]);

  // Fetch models when config is ready
  useEffect(() => {
    if (registryConfig !== null) {
      fetchModels();
    }
  }, [registryConfig, fetchModels]);

  // Refresh function
  const refresh = useCallback(() => {
    fetchModels();
  }, [fetchModels]);

  // Get model by ID
  const getModelById = useCallback((id: string) => {
    return models.find(m => m.id === id);
  }, [models]);

  // Filter models by type
  const getModelsByType = useCallback((type: AvailableModel['type']) => {
    return models.filter(m => m.type === type);
  }, [models]);

  // Filter available/pinned models
  const getAvailableModels = useCallback(() => {
    return models.filter(m => m.available);
  }, [models]);

  return {
    models,
    loading,
    error,
    refresh,
    getModelById,
    getModelsByType,
    getAvailableModels,
  };
}
