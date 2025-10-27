/**
 * WebSocket client for real-time Citrate communication
 */

import EventEmitter from 'eventemitter3';
import { WebSocketEvent } from '../types/Client';
import { InferenceResult } from '../types/Inference';
import { CitrateError } from '../errors/CitrateError';

export interface WebSocketClientConfig {
  url: string;
  reconnectAttempts?: number;
  reconnectInterval?: number;
  pingInterval?: number;
  timeout?: number;
}

export interface StreamingInferenceOptions {
  modelId: string;
  inputData: Record<string, any>;
  onPartialResult?: (partial: Partial<InferenceResult>) => void;
  onComplete?: (result: InferenceResult) => void;
  onError?: (error: Error) => void;
  encrypted?: boolean;
  maxTokens?: number;
  temperature?: number;
}

export class WebSocketClient extends EventEmitter {
  private ws: WebSocket | null = null;
  private config: Required<WebSocketClientConfig>;
  private reconnectAttempts = 0;
  private pingTimer?: number;
  private isConnecting = false;
  private messageId = 0;
  private pendingRequests = new Map<string, {
    resolve: (value: any) => void;
    reject: (error: Error) => void;
    timeout: number;
  }>();

  constructor(config: WebSocketClientConfig) {
    super();

    this.config = {
      url: config.url,
      reconnectAttempts: config.reconnectAttempts ?? 5,
      reconnectInterval: config.reconnectInterval ?? 5000,
      pingInterval: config.pingInterval ?? 30000,
      timeout: config.timeout ?? 30000
    };
  }

  /**
   * Connect to WebSocket
   */
  async connect(): Promise<void> {
    if (this.ws?.readyState === WebSocket.OPEN) {
      return;
    }

    if (this.isConnecting) {
      throw new CitrateError('Connection already in progress');
    }

    this.isConnecting = true;

    return new Promise((resolve, reject) => {
      try {
        this.ws = new WebSocket(this.config.url);

        this.ws.onopen = () => {
          this.isConnecting = false;
          this.reconnectAttempts = 0;
          this.startPing();
          this.emit('connection', { type: 'connection', timestamp: Date.now() });
          resolve();
        };

        this.ws.onmessage = (event) => {
          this.handleMessage(event.data);
        };

        this.ws.onclose = (event) => {
          this.isConnecting = false;
          this.stopPing();
          this.emit('disconnection', {
            type: 'disconnection',
            data: { code: event.code, reason: event.reason },
            timestamp: Date.now()
          });

          if (!event.wasClean && this.reconnectAttempts < this.config.reconnectAttempts) {
            this.scheduleReconnect();
          }
        };

        this.ws.onerror = (error) => {
          this.isConnecting = false;
          const citrateError = new CitrateError('WebSocket error');
          this.emit('error', {
            type: 'error',
            data: citrateError,
            timestamp: Date.now()
          });
          reject(citrateError);
        };

        // Timeout handling
        setTimeout(() => {
          if (this.isConnecting) {
            this.isConnecting = false;
            reject(new CitrateError('Connection timeout'));
          }
        }, this.config.timeout);

      } catch (error) {
        this.isConnecting = false;
        reject(error instanceof Error ? error : new CitrateError('Connection failed'));
      }
    });
  }

  /**
   * Disconnect from WebSocket
   */
  disconnect(): void {
    this.stopPing();

    if (this.ws) {
      this.ws.close(1000, 'Client disconnect');
      this.ws = null;
    }

    // Reject all pending requests
    for (const [id, request] of this.pendingRequests) {
      clearTimeout(request.timeout);
      request.reject(new CitrateError('Connection closed'));
    }
    this.pendingRequests.clear();
  }

  /**
   * Check if connected
   */
  isConnected(): boolean {
    return this.ws?.readyState === WebSocket.OPEN;
  }

  /**
   * Send message and wait for response
   */
  async sendMessage<T = any>(method: string, params: any[] = []): Promise<T> {
    if (!this.isConnected()) {
      throw new CitrateError('WebSocket not connected');
    }

    const id = (++this.messageId).toString();
    const message = {
      jsonrpc: '2.0',
      method,
      params,
      id
    };

    return new Promise((resolve, reject) => {
      const timeout = setTimeout(() => {
        this.pendingRequests.delete(id);
        reject(new CitrateError('Request timeout'));
      }, this.config.timeout);

      this.pendingRequests.set(id, { resolve, reject, timeout });

      this.ws!.send(JSON.stringify(message));
    });
  }

  /**
   * Start streaming inference
   */
  async startStreamingInference(options: StreamingInferenceOptions): Promise<void> {
    if (!this.isConnected()) {
      throw new CitrateError('WebSocket not connected');
    }

    const streamId = `stream_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`;

    // Set up event listeners for this stream
    const handlePartialResult = (data: any) => {
      if (data.streamId === streamId && options.onPartialResult) {
        options.onPartialResult(data.partial);
      }
    };

    const handleComplete = (data: any) => {
      if (data.streamId === streamId) {
        this.off('inferencePartial', handlePartialResult);
        this.off('inferenceComplete', handleComplete);
        this.off('inferenceError', handleError);

        if (options.onComplete) {
          options.onComplete(data.result);
        }
      }
    };

    const handleError = (data: any) => {
      if (data.streamId === streamId) {
        this.off('inferencePartial', handlePartialResult);
        this.off('inferenceComplete', handleComplete);
        this.off('inferenceError', handleError);

        if (options.onError) {
          options.onError(new CitrateError(data.error));
        }
      }
    };

    this.on('inferencePartial', handlePartialResult);
    this.on('inferenceComplete', handleComplete);
    this.on('inferenceError', handleError);

    // Start streaming inference
    await this.sendMessage('citrate_startStreamingInference', [{
      streamId,
      modelId: options.modelId,
      inputData: options.inputData,
      encrypted: options.encrypted || false,
      maxTokens: options.maxTokens,
      temperature: options.temperature
    }]);
  }

  /**
   * Subscribe to model events
   */
  async subscribeToModel(modelId: string): Promise<void> {
    await this.sendMessage('citrate_subscribeModel', [modelId]);
  }

  /**
   * Unsubscribe from model events
   */
  async unsubscribeFromModel(modelId: string): Promise<void> {
    await this.sendMessage('citrate_unsubscribeModel', [modelId]);
  }

  /**
   * Subscribe to marketplace events
   */
  async subscribeToMarketplace(): Promise<void> {
    await this.sendMessage('citrate_subscribeMarketplace', []);
  }

  /**
   * Handle incoming messages
   */
  private handleMessage(data: string): void {
    try {
      const message = JSON.parse(data);

      // Handle RPC responses
      if (message.id && this.pendingRequests.has(message.id)) {
        const request = this.pendingRequests.get(message.id)!;
        this.pendingRequests.delete(message.id);
        clearTimeout(request.timeout);

        if (message.error) {
          request.reject(new CitrateError(message.error.message));
        } else {
          request.resolve(message.result);
        }
        return;
      }

      // Handle notifications/events
      if (message.method) {
        this.handleNotification(message.method, message.params);
      }

    } catch (error) {
      this.emit('error', {
        type: 'error',
        data: new CitrateError('Failed to parse WebSocket message'),
        timestamp: Date.now()
      });
    }
  }

  /**
   * Handle WebSocket notifications
   */
  private handleNotification(method: string, params: any): void {
    switch (method) {
      case 'citrate_inferencePartial':
        this.emit('inferencePartial', params);
        break;

      case 'citrate_inferenceComplete':
        this.emit('inferenceComplete', params);
        break;

      case 'citrate_inferenceError':
        this.emit('inferenceError', params);
        break;

      case 'citrate_modelDeployed':
        this.emit('modelDeployed', params);
        break;

      case 'citrate_modelUpdated':
        this.emit('modelUpdated', params);
        break;

      case 'citrate_marketplaceSale':
        this.emit('marketplaceSale', params);
        break;

      default:
        this.emit('message', {
          type: 'message',
          data: { method, params },
          timestamp: Date.now()
        });
    }
  }

  /**
   * Start ping timer
   */
  private startPing(): void {
    this.pingTimer = setInterval(() => {
      if (this.isConnected()) {
        this.ws!.ping();
      }
    }, this.config.pingInterval) as any;
  }

  /**
   * Stop ping timer
   */
  private stopPing(): void {
    if (this.pingTimer) {
      clearInterval(this.pingTimer);
      this.pingTimer = undefined;
    }
  }

  /**
   * Schedule reconnection attempt
   */
  private scheduleReconnect(): void {
    this.reconnectAttempts++;

    setTimeout(() => {
      if (this.reconnectAttempts <= this.config.reconnectAttempts) {
        this.connect().catch(() => {
          // Reconnection failed, will try again if attempts remain
        });
      }
    }, this.config.reconnectInterval);
  }
}