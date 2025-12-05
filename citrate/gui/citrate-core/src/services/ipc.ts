/**
 * Inter-Window Communication Service
 *
 * Sprint 5: Multi-Window & Terminal Integration
 *
 * Provides message-based communication between windows using Tauri events.
 */

import { listen, emit, UnlistenFn } from '@tauri-apps/api/event';
import { getCurrentWindow } from '@tauri-apps/api/window';
import {
  IPCMessage,
  IPCMessageType,
  IPCHandler,
  IPCSubscription,
  IPCService,
  generateMessageId,
  createMessage,
} from '../types/ipc';

/**
 * IPC Service Implementation
 *
 * Singleton service that manages inter-window communication.
 */
class IPCServiceImpl implements IPCService {
  private subscriptions: Map<string, IPCSubscription> = new Map();
  private pendingRequests: Map<string, {
    resolve: (value: unknown) => void;
    reject: (reason: unknown) => void;
    timeout: ReturnType<typeof setTimeout>;
  }> = new Map();
  private windowId: string = 'main';
  private initialized: boolean = false;
  private messageUnlisten: UnlistenFn | null = null;

  /**
   * Initialize the IPC service
   * Must be called once during app startup
   */
  async initialize(): Promise<void> {
    if (this.initialized) return;

    try {
      // Get current window label as our ID
      const currentWindow = getCurrentWindow();
      this.windowId = currentWindow.label;
    } catch {
      // Fallback for non-Tauri environment (web dev)
      this.windowId = 'main';
    }

    // Listen for all IPC messages
    this.messageUnlisten = await listen<IPCMessage>('ipc-message', (event) => {
      this.handleMessage(event.payload);
    });

    // Listen for response messages
    await listen<IPCMessage>('ipc-response', (event) => {
      this.handleResponse(event.payload);
    });

    this.initialized = true;
    console.log(`IPC Service initialized for window: ${this.windowId}`);
  }

  /**
   * Clean up resources
   */
  async destroy(): Promise<void> {
    // Unsubscribe from all subscriptions
    this.subscriptions.forEach((sub) => sub.unsubscribe());
    this.subscriptions.clear();

    // Cancel pending requests
    this.pendingRequests.forEach(({ reject, timeout }) => {
      clearTimeout(timeout);
      reject(new Error('IPC service destroyed'));
    });
    this.pendingRequests.clear();

    // Unlisten from global events
    if (this.messageUnlisten) {
      this.messageUnlisten();
      this.messageUnlisten = null;
    }

    this.initialized = false;
  }

  /**
   * Subscribe to a specific message type
   */
  subscribe<T = unknown>(type: IPCMessageType, handler: IPCHandler<T>): IPCSubscription {
    const id = `sub_${generateMessageId()}`;

    const subscription: IPCSubscription = {
      id,
      type,
      handler: handler as IPCHandler,
      unsubscribe: () => {
        this.subscriptions.delete(id);
      },
    };

    this.subscriptions.set(id, subscription);
    return subscription;
  }

  /**
   * Unsubscribe by subscription ID
   */
  unsubscribe(subscriptionId: string): void {
    const sub = this.subscriptions.get(subscriptionId);
    if (sub) {
      sub.unsubscribe();
    }
  }

  /**
   * Send a message to a specific window
   */
  async send<T = unknown>(type: IPCMessageType, payload: T, target: string): Promise<void> {
    const message = createMessage(type, payload, this.windowId, target);
    await emit('ipc-message', message);
  }

  /**
   * Broadcast a message to all windows
   */
  async broadcast<T = unknown>(type: IPCMessageType, payload: T): Promise<void> {
    const message = createMessage(type, payload, this.windowId, null);
    await emit('ipc-message', message);
  }

  /**
   * Send a message and wait for a response
   */
  async request<T = unknown, R = unknown>(
    type: IPCMessageType,
    payload: T,
    target: string,
    timeout: number = 30000
  ): Promise<R> {
    const message = createMessage(type, payload, this.windowId, target);

    return new Promise<R>((resolve, reject) => {
      const timeoutHandle = setTimeout(() => {
        this.pendingRequests.delete(message.id);
        reject(new Error(`IPC request timeout for ${type}`));
      }, timeout);

      this.pendingRequests.set(message.id, {
        resolve: resolve as (value: unknown) => void,
        reject,
        timeout: timeoutHandle,
      });

      emit('ipc-message', message).catch((err) => {
        this.pendingRequests.delete(message.id);
        clearTimeout(timeoutHandle);
        reject(err);
      });
    });
  }

  /**
   * Get the current window ID
   */
  getCurrentWindowId(): string {
    return this.windowId;
  }

  /**
   * Handle incoming messages
   */
  private handleMessage(message: IPCMessage): void {
    // Ignore messages from self
    if (message.source === this.windowId) return;

    // Check if message is targeted at another window
    if (message.target && message.target !== this.windowId) return;

    // Find and call all matching handlers
    this.subscriptions.forEach((sub) => {
      if (sub.type === message.type) {
        try {
          const result = sub.handler(message);
          // If handler returns a promise, handle errors
          if (result instanceof Promise) {
            result.catch((err) => {
              console.error(`IPC handler error for ${message.type}:`, err);
            });
          }
        } catch (err) {
          console.error(`IPC handler error for ${message.type}:`, err);
        }
      }
    });
  }

  /**
   * Handle response messages
   */
  private handleResponse(message: IPCMessage): void {
    // Check if this is a response to one of our requests
    const requestId = (message.payload as { requestId?: string })?.requestId;
    if (!requestId) return;

    const pending = this.pendingRequests.get(requestId);
    if (pending) {
      clearTimeout(pending.timeout);
      this.pendingRequests.delete(requestId);

      const response = message.payload as { success?: boolean; result?: unknown; error?: string };
      if (response.success) {
        pending.resolve(response.result);
      } else {
        pending.reject(new Error(response.error || 'Unknown error'));
      }
    }
  }

  /**
   * Send a response to a request
   */
  async respond<T = unknown>(
    originalMessage: IPCMessage,
    success: boolean,
    result?: T,
    error?: string
  ): Promise<void> {
    const response = createMessage(
      originalMessage.type,
      {
        requestId: originalMessage.id,
        success,
        result,
        error,
      },
      this.windowId,
      originalMessage.source
    );

    await emit('ipc-response', response);
  }
}

// Singleton instance
let ipcService: IPCServiceImpl | null = null;

/**
 * Get the IPC service singleton
 */
export function getIPCService(): IPCService {
  if (!ipcService) {
    ipcService = new IPCServiceImpl();
  }
  return ipcService;
}

/**
 * Initialize the IPC service
 * Call this during app initialization
 */
export async function initializeIPC(): Promise<void> {
  const service = getIPCService() as IPCServiceImpl;
  await service.initialize();
}

/**
 * Destroy the IPC service
 * Call this during app cleanup
 */
export async function destroyIPC(): Promise<void> {
  if (ipcService) {
    await ipcService.destroy();
    ipcService = null;
  }
}

/**
 * Helper hook for React components
 */
export { getIPCService as useIPC };
