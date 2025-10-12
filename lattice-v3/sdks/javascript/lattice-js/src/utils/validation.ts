/**
 * Validation utilities for Lattice JavaScript SDK
 */

import { ModelConfig, ModelType, AccessType } from '../types/Model';
import { InferenceRequest } from '../types/Inference';
import { ValidationError } from '../errors/LatticeError';
import { MODEL_LIMITS } from './constants';

/**
 * Validate Ethereum address format
 */
export function validateAddress(address: string): boolean {
  return /^0x[a-fA-F0-9]{40}$/.test(address);
}

/**
 * Validate private key format
 */
export function validatePrivateKey(privateKey: string): boolean {
  const cleanKey = privateKey.replace(/^0x/, '');
  return /^[a-fA-F0-9]{64}$/.test(cleanKey);
}

/**
 * Validate model configuration
 */
export function validateModelConfig(config: ModelConfig): void {
  // Name validation
  if (!config.name || config.name.trim().length === 0) {
    throw new ValidationError('Model name is required');
  }

  if (config.name.length > MODEL_LIMITS.MAX_NAME_LENGTH) {
    throw new ValidationError(`Model name exceeds maximum length of ${MODEL_LIMITS.MAX_NAME_LENGTH}`);
  }

  // Description validation
  if (config.description && config.description.length > MODEL_LIMITS.MAX_DESCRIPTION_LENGTH) {
    throw new ValidationError(`Description exceeds maximum length of ${MODEL_LIMITS.MAX_DESCRIPTION_LENGTH}`);
  }

  // Model type validation
  if (!Object.values(ModelType).includes(config.modelType)) {
    throw new ValidationError(`Invalid model type: ${config.modelType}`);
  }

  // Access type validation
  if (!Object.values(AccessType).includes(config.accessType)) {
    throw new ValidationError(`Invalid access type: ${config.accessType}`);
  }

  // Access price validation
  if (typeof config.accessPrice !== 'bigint') {
    throw new ValidationError('Access price must be a bigint');
  }

  if (config.accessPrice < 0n) {
    throw new ValidationError('Access price cannot be negative');
  }

  // Access list validation
  if (config.accessList) {
    if (config.accessList.length > 100) {
      throw new ValidationError('Access list cannot exceed 100 addresses');
    }

    for (const address of config.accessList) {
      if (!validateAddress(address)) {
        throw new ValidationError(`Invalid address in access list: ${address}`);
      }
    }
  }

  // Tags validation
  if (config.tags) {
    if (config.tags.length > MODEL_LIMITS.MAX_TAGS) {
      throw new ValidationError(`Cannot exceed ${MODEL_LIMITS.MAX_TAGS} tags`);
    }

    for (const tag of config.tags) {
      if (tag.length > MODEL_LIMITS.MAX_TAG_LENGTH) {
        throw new ValidationError(`Tag exceeds maximum length of ${MODEL_LIMITS.MAX_TAG_LENGTH}: ${tag}`);
      }

      if (!/^[a-zA-Z0-9_-]+$/.test(tag)) {
        throw new ValidationError(`Invalid tag format: ${tag}. Only alphanumeric, underscore, and dash allowed`);
      }
    }
  }

  // Performance constraints validation
  if (config.maxBatchSize && (config.maxBatchSize < 1 || config.maxBatchSize > MODEL_LIMITS.MAX_BATCH_SIZE)) {
    throw new ValidationError(`Max batch size must be between 1 and ${MODEL_LIMITS.MAX_BATCH_SIZE}`);
  }

  if (config.timeoutSeconds && (config.timeoutSeconds < 1 || config.timeoutSeconds > 3600)) {
    throw new ValidationError('Timeout must be between 1 and 3600 seconds');
  }

  if (config.memoryLimitMb && (config.memoryLimitMb < 128 || config.memoryLimitMb > 8192)) {
    throw new ValidationError('Memory limit must be between 128MB and 8GB');
  }

  // Revenue shares validation
  if (config.revenueShares) {
    let totalPercentage = 0;

    for (const [address, percentage] of Object.entries(config.revenueShares)) {
      if (!validateAddress(address)) {
        throw new ValidationError(`Invalid address in revenue shares: ${address}`);
      }

      if (percentage < 0 || percentage > 1) {
        throw new ValidationError(`Revenue share percentage must be between 0 and 1: ${percentage}`);
      }

      totalPercentage += percentage;
    }

    if (Math.abs(totalPercentage - 1) > 0.001) {
      throw new ValidationError(`Revenue shares must sum to 1.0, got: ${totalPercentage}`);
    }
  }

  // Encryption config validation
  if (config.encrypted && config.encryptionConfig) {
    const encConfig = config.encryptionConfig;

    if (encConfig.thresholdShares < 0 || encConfig.totalShares < 0) {
      throw new ValidationError('Threshold shares and total shares must be non-negative');
    }

    if (encConfig.thresholdShares > encConfig.totalShares) {
      throw new ValidationError('Threshold shares cannot exceed total shares');
    }

    if (encConfig.totalShares > 255) {
      throw new ValidationError('Total shares cannot exceed 255');
    }
  }
}

/**
 * Validate model data
 */
export function validateModelData(data: ArrayBuffer | Uint8Array): void {
  const size = data instanceof ArrayBuffer ? data.byteLength : data.length;

  if (size === 0) {
    throw new ValidationError('Model data cannot be empty');
  }

  if (size > MODEL_LIMITS.MAX_MODEL_SIZE) {
    throw new ValidationError(`Model size exceeds maximum of ${MODEL_LIMITS.MAX_MODEL_SIZE} bytes`);
  }
}

/**
 * Validate inference request
 */
export function validateInferenceRequest(request: InferenceRequest): void {
  // Model ID validation
  if (!request.modelId || request.modelId.trim().length === 0) {
    throw new ValidationError('Model ID is required');
  }

  if (!/^[a-zA-Z0-9_-]+$/.test(request.modelId)) {
    throw new ValidationError('Model ID contains invalid characters');
  }

  // Input data validation
  if (!request.inputData) {
    throw new ValidationError('Input data is required');
  }

  const inputSize = JSON.stringify(request.inputData).length;
  if (inputSize > MODEL_LIMITS.MAX_INPUT_SIZE) {
    throw new ValidationError(`Input data exceeds maximum size of ${MODEL_LIMITS.MAX_INPUT_SIZE} bytes`);
  }

  // Batch size validation
  if (request.batchSize && (request.batchSize < 1 || request.batchSize > MODEL_LIMITS.MAX_BATCH_SIZE)) {
    throw new ValidationError(`Batch size must be between 1 and ${MODEL_LIMITS.MAX_BATCH_SIZE}`);
  }

  // Timeout validation
  if (request.timeout && (request.timeout < 1000 || request.timeout > 300000)) {
    throw new ValidationError('Timeout must be between 1 second and 5 minutes (in milliseconds)');
  }
}

/**
 * Validate RPC URL
 */
export function validateRpcUrl(url: string): boolean {
  try {
    const parsed = new URL(url);
    return ['http:', 'https:', 'ws:', 'wss:'].includes(parsed.protocol);
  } catch {
    return false;
  }
}

/**
 * Validate chain ID
 */
export function validateChainId(chainId: number): boolean {
  return Number.isInteger(chainId) && chainId > 0 && chainId < 2 ** 32;
}

/**
 * Validate gas limit
 */
export function validateGasLimit(gasLimit: bigint): boolean {
  return gasLimit > 0n && gasLimit <= 15000000n; // 15M gas limit
}

/**
 * Validate gas price
 */
export function validateGasPrice(gasPrice: bigint): boolean {
  return gasPrice >= 0n && gasPrice <= 1000000000000n; // 1000 gwei max
}

/**
 * Validate nonce
 */
export function validateNonce(nonce: number): boolean {
  return Number.isInteger(nonce) && nonce >= 0 && nonce < 2 ** 32;
}

/**
 * Validate transaction value
 */
export function validateTransactionValue(value: bigint): boolean {
  return value >= 0n && value <= 10n ** 30n; // Reasonable upper bound
}

/**
 * Validate hex string
 */
export function validateHexString(hex: string, length?: number): boolean {
  const cleanHex = hex.replace(/^0x/, '');

  if (!/^[a-fA-F0-9]*$/.test(cleanHex)) {
    return false;
  }

  if (length !== undefined && cleanHex.length !== length * 2) {
    return false;
  }

  return true;
}

/**
 * Validate JSON RPC method
 */
export function validateRpcMethod(method: string): boolean {
  return /^[a-zA-Z][a-zA-Z0-9_]*$/.test(method);
}

/**
 * Sanitize string input
 */
export function sanitizeString(input: string, maxLength?: number): string {
  let sanitized = input.trim();

  if (maxLength && sanitized.length > maxLength) {
    sanitized = sanitized.substring(0, maxLength);
  }

  // Remove any potentially dangerous characters
  sanitized = sanitized.replace(/[<>\"'&]/g, '');

  return sanitized;
}

/**
 * Validate file extension for model type
 */
export function validateModelFileExtension(filename: string, modelType: ModelType): boolean {
  const extension = filename.toLowerCase().split('.').pop();
  if (!extension) return false;

  const validExtensions: Record<ModelType, string[]> = {
    [ModelType.COREML]: ['mlpackage', 'mlmodel'],
    [ModelType.ONNX]: ['onnx'],
    [ModelType.TENSORFLOW]: ['pb', 'savedmodel'],
    [ModelType.PYTORCH]: ['pt', 'pth', 'pkl'],
    [ModelType.CUSTOM]: ['json', 'bin', 'dat']
  };

  return validExtensions[modelType].includes(extension);
}