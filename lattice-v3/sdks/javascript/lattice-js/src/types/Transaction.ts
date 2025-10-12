/**
 * Transaction-related type definitions
 */

export interface TransactionRequest {
  to?: string;
  from?: string;
  value?: bigint;
  data?: string;
  gasLimit?: bigint;
  gasPrice?: bigint;
  maxFeePerGas?: bigint;
  maxPriorityFeePerGas?: bigint;
  nonce?: number;
  type?: number;
  chainId?: number;
}

export interface TransactionResponse {
  hash: string;
  blockNumber?: number;
  blockHash?: string;
  from: string;
  to?: string;
  value: bigint;
  gasLimit: bigint;
  gasPrice?: bigint;
  maxFeePerGas?: bigint;
  maxPriorityFeePerGas?: bigint;
  nonce: number;
  data: string;
  type?: number;
  chainId: number;
}

export interface TransactionReceipt {
  transactionHash: string;
  transactionIndex: number;
  blockHash: string;
  blockNumber: number;
  from: string;
  to?: string;
  gasUsed: bigint;
  cumulativeGasUsed: bigint;
  effectiveGasPrice?: bigint;
  status?: number;
  logs: LogEntry[];
  contractAddress?: string;
}

export interface LogEntry {
  address: string;
  topics: string[];
  data: string;
  blockNumber: number;
  transactionHash: string;
  transactionIndex: number;
  blockHash: string;
  logIndex: number;
  removed?: boolean;
}

export interface PaymentInfo {
  modelId: string;
  pricePerInference: bigint;
  paymentToken: string;
  paymentAddress: string;
  revenueSharing?: Record<string, number>;
}