/**
 * Inter-Window Communication (IPC) Types
 *
 * Sprint 5: Multi-Window & Terminal Integration
 */

/** IPC message types */
export type IPCMessageType =
  // Terminal messages
  | 'terminal:output'
  | 'terminal:input'
  | 'terminal:resize'
  | 'terminal:clear'
  | 'terminal:close'
  // Editor messages
  | 'editor:open'
  | 'editor:save'
  | 'editor:close'
  | 'editor:content'
  | 'editor:suggest'
  // Preview messages
  | 'preview:navigate'
  | 'preview:reload'
  | 'preview:back'
  | 'preview:forward'
  // Agent messages
  | 'agent:request'
  | 'agent:response'
  | 'agent:stream'
  // Window messages
  | 'window:focus'
  | 'window:close'
  | 'window:ready';

/** Base IPC message */
export interface IPCMessage<T = unknown> {
  /** Unique message ID */
  id: string;
  /** Message type */
  type: IPCMessageType;
  /** Source window ID */
  source: string;
  /** Target window ID (null for broadcast) */
  target: string | null;
  /** Message payload */
  payload: T;
  /** Message timestamp */
  timestamp: number;
}

// Terminal message payloads
export interface TerminalOutputPayload {
  sessionId: string;
  data: string; // Base64 encoded
}

export interface TerminalInputPayload {
  sessionId: string;
  data: string;
}

export interface TerminalResizePayload {
  sessionId: string;
  cols: number;
  rows: number;
}

// Editor message payloads
export interface EditorOpenPayload {
  path: string;
  content?: string;
  language?: string;
}

export interface EditorSavePayload {
  path: string;
  content: string;
}

export interface EditorContentPayload {
  path: string;
  content: string;
}

export interface EditorSuggestPayload {
  path: string;
  range: {
    startLine: number;
    startColumn: number;
    endLine: number;
    endColumn: number;
  };
  suggestion: string;
  description?: string;
}

// Preview message payloads
export interface PreviewNavigatePayload {
  url: string;
  title?: string;
}

// Agent message payloads
export interface AgentRequestPayload {
  requestId: string;
  action: string;
  params: Record<string, unknown>;
}

export interface AgentResponsePayload {
  requestId: string;
  success: boolean;
  result?: unknown;
  error?: string;
}

export interface AgentStreamPayload {
  requestId: string;
  chunk: string;
  done: boolean;
}

/** IPC event handler */
export type IPCHandler<T = unknown> = (message: IPCMessage<T>) => void | Promise<void>;

/** IPC subscription */
export interface IPCSubscription {
  id: string;
  type: IPCMessageType;
  handler: IPCHandler;
  unsubscribe: () => void;
}

/** IPC service interface */
export interface IPCService {
  /** Subscribe to message type */
  subscribe<T = unknown>(type: IPCMessageType, handler: IPCHandler<T>): IPCSubscription;

  /** Unsubscribe by ID */
  unsubscribe(subscriptionId: string): void;

  /** Send message to specific window */
  send<T = unknown>(type: IPCMessageType, payload: T, target: string): Promise<void>;

  /** Broadcast message to all windows */
  broadcast<T = unknown>(type: IPCMessageType, payload: T): Promise<void>;

  /** Send and wait for response */
  request<T = unknown, R = unknown>(
    type: IPCMessageType,
    payload: T,
    target: string,
    timeout?: number
  ): Promise<R>;

  /** Get current window ID */
  getCurrentWindowId(): string;
}

/** Generate unique message ID */
export function generateMessageId(): string {
  return `msg_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`;
}

/** Create IPC message */
export function createMessage<T>(
  type: IPCMessageType,
  payload: T,
  source: string,
  target: string | null = null
): IPCMessage<T> {
  return {
    id: generateMessageId(),
    type,
    source,
    target,
    payload,
    timestamp: Date.now(),
  };
}
