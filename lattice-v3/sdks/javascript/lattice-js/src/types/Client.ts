/**
 * Client-related type definitions
 */

export interface ClientConfig {
  rpcUrl: string;
  privateKey?: string;
  timeout?: number;
  retries?: number;
  headers?: Record<string, string>;
  enableWebSocket?: boolean;
  webSocketUrl?: string;
}

export interface ConnectionInfo {
  chainId: number;
  blockNumber: number;
  networkName: string;
  isConnected: boolean;
  latency: number;
}

export interface ClientMetrics {
  totalRequests: number;
  successfulRequests: number;
  failedRequests: number;
  averageResponseTime: number;
  uptime: number;
}

export interface RpcRequest {
  jsonrpc: string;
  method: string;
  params: any[];
  id: number | string;
}

export interface RpcResponse<T = any> {
  jsonrpc: string;
  result?: T;
  error?: {
    code: number;
    message: string;
    data?: any;
  };
  id: number | string;
}

export interface WebSocketEvent {
  type: 'connection' | 'disconnection' | 'message' | 'error';
  data?: any;
  timestamp: number;
}