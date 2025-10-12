/**
 * Model-related type definitions
 */

export enum ModelType {
  COREML = 'coreml',
  ONNX = 'onnx',
  TENSORFLOW = 'tensorflow',
  PYTORCH = 'pytorch',
  CUSTOM = 'custom'
}

export enum AccessType {
  PUBLIC = 'public',
  PRIVATE = 'private',
  PAID = 'paid',
  WHITELIST = 'whitelist'
}

export interface ModelConfig {
  name: string;
  description?: string;
  modelType: ModelType;
  version?: string;
  accessType: AccessType;
  accessPrice: bigint; // Price in wei per inference
  accessList?: string[]; // Whitelist addresses
  encrypted: boolean;
  encryptionConfig?: EncryptionConfig;
  metadata?: Record<string, any>;
  tags?: string[];
  maxBatchSize?: number;
  timeoutSeconds?: number;
  memoryLimitMb?: number;
  revenueShares?: Record<string, number>; // address -> percentage
}

export interface EncryptionConfig {
  algorithm: string;
  keyDerivation: string;
  accessControl: boolean;
  thresholdShares: number;
  totalShares: number;
}

export interface ModelDeployment {
  modelId: string;
  txHash: string;
  ipfsHash: string;
  encrypted: boolean;
  accessPrice: bigint;
  deploymentTime: number;
  gasUsed?: bigint;
  deploymentCost?: bigint;
}

export interface ModelInfo {
  modelId: string;
  name: string;
  description: string;
  owner: string;
  modelType: ModelType;
  accessType: AccessType;
  accessPrice: bigint;
  encrypted: boolean;
  ipfsHash: string;
  deploymentTime: number;
  totalInferences: number;
  totalRevenue: bigint;
  metadata: Record<string, any>;
  tags: string[];
}

export interface ModelStats {
  modelId: string;
  totalInferences: number;
  totalRevenue: bigint;
  averageExecutionTime: number;
  averageGasCost: bigint;
  uniqueUsers: number;
  lastInferenceTime: number;
}

export interface ModelVersion {
  modelId: string;
  version: string;
  ipfsHash: string;
  deploymentTime: number;
  changes: string;
  deprecated: boolean;
}