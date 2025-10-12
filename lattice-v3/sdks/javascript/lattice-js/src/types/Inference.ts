/**
 * Inference-related type definitions
 */

export interface InferenceRequest {
  modelId: string;
  inputData: Record<string, any>;
  encrypted?: boolean;
  batchSize?: number;
  timeout?: number;
  timestamp?: number;
}

export interface InferenceResult {
  modelId: string;
  outputData: Record<string, any>;
  gasUsed: bigint;
  executionTime: number; // milliseconds
  txHash: string;
  confidence?: number;
  metadata?: Record<string, any>;
}

export interface StreamingInferenceConfig {
  modelId: string;
  inputData: Record<string, any>;
  onPartialResult?: (partial: Partial<InferenceResult>) => void;
  onComplete?: (result: InferenceResult) => void;
  onError?: (error: Error) => void;
  encrypted?: boolean;
  maxTokens?: number;
  temperature?: number;
}

export interface InferenceJob {
  jobId: string;
  modelId: string;
  status: 'pending' | 'running' | 'completed' | 'failed';
  inputData: Record<string, any>;
  outputData?: Record<string, any>;
  progress: number; // 0-100
  createdAt: number;
  updatedAt: number;
  estimatedCompletion?: number;
  gasUsed?: bigint;
  error?: string;
}

export interface BatchInferenceRequest {
  modelId: string;
  inputs: Array<Record<string, any>>;
  batchSize?: number;
  parallel?: boolean;
  onProgress?: (completed: number, total: number) => void;
}

export interface BatchInferenceResult {
  results: InferenceResult[];
  totalGasUsed: bigint;
  totalExecutionTime: number;
  successCount: number;
  failureCount: number;
  errors: string[];
}