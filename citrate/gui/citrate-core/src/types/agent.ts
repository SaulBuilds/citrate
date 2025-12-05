/**
 * Agent Type Definitions
 *
 * Types for the AI agent conversation interface.
 */

// =============================================================================
// Message Types
// =============================================================================

export type MessageRole = 'user' | 'assistant' | 'system' | 'tool';

export interface Message {
  id: string;
  role: MessageRole;
  content: string;
  timestamp: number;
  isStreaming?: boolean;
  intent?: IntentInfo;
  toolResult?: ToolResultInfo;
  metadata?: MessageMetadata;
}

export interface MessageMetadata {
  model?: string;
  tokens?: number;
  duration?: number;
}

export interface IntentInfo {
  intent: string;
  confidence: number;
  params?: Record<string, unknown>;
}

export interface ToolResultInfo {
  toolName: string;
  success: boolean;
  data?: unknown;
  error?: string;
}

// =============================================================================
// Session Types
// =============================================================================

export interface AgentSession {
  id: string;
  state: SessionState;
  messageCount: number;
  createdAt: number;
  lastActivity: number;
}

export type SessionState =
  | 'active'
  | 'processing'
  | 'waiting_for_input'
  | 'paused'
  | 'closed';

// =============================================================================
// Streaming Types
// =============================================================================

export interface StreamToken {
  sessionId: string;
  messageId: string;
  content: string;
  isComplete: boolean;
  metadata?: StreamMetadata;
}

export interface StreamMetadata {
  tokensGenerated?: number;
  totalTokens?: number;
  finishReason?: string;
}

// =============================================================================
// Tool Types
// =============================================================================

export interface PendingToolCall {
  id: string;
  toolName: string;
  description: string;
  highRisk: boolean;
  params: Record<string, unknown>;
}

export interface ToolApprovalStatus {
  id: string;
  approved: boolean;
  timestamp: number;
}

// =============================================================================
// Transaction Types (for TransactionCard)
// =============================================================================

export interface PendingTransaction {
  id: string;
  type: 'send' | 'deploy' | 'call';
  from: string;
  to?: string;
  value?: string;
  gas?: string;
  data?: string;
  contractName?: string;
  methodName?: string;
  methodArgs?: unknown[];
  simulation?: TransactionSimulation;
  timeoutAt?: number;
}

export interface TransactionSimulation {
  success: boolean;
  gasUsed?: string;
  returnValue?: string;
  logs?: TransactionLog[];
  error?: string;
}

export interface TransactionLog {
  address: string;
  topics: string[];
  data: string;
}

// =============================================================================
// Chain Result Types (for ChainResultCard)
// =============================================================================

export type ChainResultType =
  | 'balance'
  | 'block'
  | 'transaction'
  | 'receipt'
  | 'account'
  | 'contract';

export interface ChainResult {
  type: ChainResultType;
  data: BalanceResult | BlockResult | TransactionResult | ReceiptResult | AccountResult | ContractResult;
  timestamp: number;
}

export interface BalanceResult {
  address: string;
  balance: string;
  formatted: string;
  tokenSymbol: string;
}

export interface BlockResult {
  hash: string;
  number: number;
  timestamp: number;
  parentHash: string;
  transactionCount: number;
  gasUsed: string;
  gasLimit: string;
  miner?: string;
}

export interface TransactionResult {
  hash: string;
  from: string;
  to?: string;
  value: string;
  gas: string;
  gasPrice: string;
  nonce: number;
  blockHash?: string;
  blockNumber?: number;
  status?: 'pending' | 'success' | 'failed';
}

export interface ReceiptResult {
  transactionHash: string;
  blockNumber: number;
  status: boolean;
  gasUsed: string;
  logs: TransactionLog[];
  contractAddress?: string;
}

export interface AccountResult {
  address: string;
  balance: string;
  nonce: number;
  codeHash?: string;
  hasCode: boolean;
}

export interface ContractResult {
  address: string;
  bytecode?: string;
  abi?: unknown[];
  name?: string;
}

// =============================================================================
// Agent Config Types
// =============================================================================

export interface AgentConfig {
  enabled: boolean;
  llmBackend: 'OpenAI' | 'Anthropic' | 'LocalGGUF' | 'Auto';
  model: string;
  streamingEnabled: boolean;
  localModelPath?: string;
}

export interface LocalModelInfo {
  name: string;
  size: string;
  quantization: string;
  path: string;
}

// =============================================================================
// Agent Status Types
// =============================================================================

export interface AgentStatus {
  initialized: boolean;
  enabled: boolean;
  llmBackend?: string;
  activeSessions?: number;
  streamingEnabled?: boolean;
}

// =============================================================================
// Agent Context State
// =============================================================================

export interface AgentContextState {
  // Connection state
  isInitialized: boolean;
  isConnected: boolean;

  // Session state
  currentSessionId: string | null;
  sessions: AgentSession[];

  // Message state
  messages: Message[];
  isStreaming: boolean;
  streamingMessageId: string | null;

  // Tool state
  pendingTools: PendingToolCall[];

  // Config
  config: AgentConfig | null;

  // Status
  status: AgentStatus | null;
}

// =============================================================================
// Tauri Event Payloads
// =============================================================================

export interface AgentTokenEvent {
  session_id: string;
  message_id: string;
  content: string;
  is_complete: boolean;
}

export interface AgentCompleteEvent {
  session_id: string;
  message_id: string;
  total_tokens?: number;
  finish_reason?: string;
}

export interface AgentErrorEvent {
  session_id: string;
  error: string;
  code?: string;
}

export interface AgentToolCallEvent {
  session_id: string;
  tool_id: string;
  tool_name: string;
  description: string;
  high_risk: boolean;
  params: Record<string, unknown>;
}
