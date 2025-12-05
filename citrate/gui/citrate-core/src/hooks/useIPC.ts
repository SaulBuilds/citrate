/**
 * IPC React Hooks
 *
 * Sprint 5: Multi-Window & Terminal Integration
 *
 * React hooks for inter-window communication.
 */

import { useEffect, useCallback, useRef, useState } from 'react';
import {
  IPCMessage,
  IPCMessageType,
  IPCHandler,
  TerminalOutputPayload,
  TerminalInputPayload,
  TerminalResizePayload,
  EditorOpenPayload,
  EditorSavePayload,
  EditorSuggestPayload,
  PreviewNavigatePayload,
  AgentRequestPayload,
  AgentResponsePayload,
  AgentStreamPayload,
} from '../types/ipc';
import { getIPCService, initializeIPC } from '../services/ipc';

/**
 * Hook to initialize IPC service
 * Should be called once at app root
 */
export function useIPCInit(): boolean {
  const [ready, setReady] = useState(false);

  useEffect(() => {
    initializeIPC()
      .then(() => setReady(true))
      .catch((err) => {
        console.error('Failed to initialize IPC:', err);
      });
  }, []);

  return ready;
}

/**
 * Hook to subscribe to IPC messages
 */
export function useIPCSubscription<T = unknown>(
  type: IPCMessageType,
  handler: IPCHandler<T>,
  deps: React.DependencyList = []
): void {
  const handlerRef = useRef(handler);
  handlerRef.current = handler;

  useEffect(() => {
    const service = getIPCService();
    const subscription = service.subscribe(type, (message: IPCMessage<T>) => {
      handlerRef.current(message);
    });

    return () => {
      subscription.unsubscribe();
    };
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [type, ...deps]);
}

/**
 * Hook to send IPC messages
 */
export function useIPCSend() {
  const send = useCallback(
    async <T = unknown>(type: IPCMessageType, payload: T, target: string): Promise<void> => {
      const service = getIPCService();
      await service.send(type, payload, target);
    },
    []
  );

  const broadcast = useCallback(
    async <T = unknown>(type: IPCMessageType, payload: T): Promise<void> => {
      const service = getIPCService();
      await service.broadcast(type, payload);
    },
    []
  );

  const request = useCallback(
    async <T = unknown, R = unknown>(
      type: IPCMessageType,
      payload: T,
      target: string,
      timeout?: number
    ): Promise<R> => {
      const service = getIPCService();
      return service.request(type, payload, target, timeout);
    },
    []
  );

  return { send, broadcast, request };
}

/**
 * Hook to get current window ID
 */
export function useWindowId(): string {
  const service = getIPCService();
  return service.getCurrentWindowId();
}

// ========== Terminal-specific hooks ==========

/**
 * Hook for terminal output subscription
 */
export function useTerminalOutput(
  sessionId: string,
  onOutput: (data: string) => void
): void {
  useIPCSubscription<TerminalOutputPayload>(
    'terminal:output',
    (message) => {
      if (message.payload.sessionId === sessionId) {
        // Decode base64 data
        const decoded = atob(message.payload.data);
        onOutput(decoded);
      }
    },
    [sessionId, onOutput]
  );
}

/**
 * Hook to send terminal input
 */
export function useTerminalInput() {
  const { send } = useIPCSend();

  return useCallback(
    async (sessionId: string, data: string, targetWindow?: string): Promise<void> => {
      const payload: TerminalInputPayload = { sessionId, data };
      if (targetWindow) {
        await send('terminal:input', payload, targetWindow);
      } else {
        const service = getIPCService();
        await service.broadcast('terminal:input', payload);
      }
    },
    [send]
  );
}

/**
 * Hook to send terminal resize
 */
export function useTerminalResize() {
  const { broadcast } = useIPCSend();

  return useCallback(
    async (sessionId: string, cols: number, rows: number): Promise<void> => {
      const payload: TerminalResizePayload = { sessionId, cols, rows };
      await broadcast('terminal:resize', payload);
    },
    [broadcast]
  );
}

// ========== Editor-specific hooks ==========

/**
 * Hook for editor events
 */
export function useEditorEvents(handlers: {
  onOpen?: (payload: EditorOpenPayload) => void;
  onSave?: (payload: EditorSavePayload) => void;
  onSuggest?: (payload: EditorSuggestPayload) => void;
}): void {
  useIPCSubscription<EditorOpenPayload>('editor:open', (message) => {
    handlers.onOpen?.(message.payload);
  }, [handlers.onOpen]);

  useIPCSubscription<EditorSavePayload>('editor:save', (message) => {
    handlers.onSave?.(message.payload);
  }, [handlers.onSave]);

  useIPCSubscription<EditorSuggestPayload>('editor:suggest', (message) => {
    handlers.onSuggest?.(message.payload);
  }, [handlers.onSuggest]);
}

/**
 * Hook to request file opening in editor
 */
export function useOpenInEditor() {
  const { broadcast } = useIPCSend();

  return useCallback(
    async (path: string, content?: string, language?: string): Promise<void> => {
      const payload: EditorOpenPayload = { path, content, language };
      await broadcast('editor:open', payload);
    },
    [broadcast]
  );
}

/**
 * Hook to suggest code changes in editor
 */
export function useSuggestEdit() {
  const { send } = useIPCSend();

  return useCallback(
    async (
      editorWindowId: string,
      path: string,
      range: EditorSuggestPayload['range'],
      suggestion: string,
      description?: string
    ): Promise<void> => {
      const payload: EditorSuggestPayload = { path, range, suggestion, description };
      await send('editor:suggest', payload, editorWindowId);
    },
    [send]
  );
}

// ========== Preview-specific hooks ==========

/**
 * Hook for preview navigation events
 */
export function usePreviewNavigation(
  onNavigate: (url: string, title?: string) => void
): void {
  useIPCSubscription<PreviewNavigatePayload>('preview:navigate', (message) => {
    onNavigate(message.payload.url, message.payload.title);
  }, [onNavigate]);
}

/**
 * Hook to navigate preview window
 */
export function useNavigatePreview() {
  const { send, broadcast } = useIPCSend();

  const navigate = useCallback(
    async (url: string, title?: string, targetWindow?: string): Promise<void> => {
      const payload: PreviewNavigatePayload = { url, title };
      if (targetWindow) {
        await send('preview:navigate', payload, targetWindow);
      } else {
        await broadcast('preview:navigate', payload);
      }
    },
    [send, broadcast]
  );

  const reload = useCallback(
    async (targetWindow?: string): Promise<void> => {
      if (targetWindow) {
        await send('preview:reload', {}, targetWindow);
      } else {
        await broadcast('preview:reload', {});
      }
    },
    [send, broadcast]
  );

  return { navigate, reload };
}

// ========== Agent-specific hooks ==========

/**
 * Hook for agent communication
 */
export function useAgentIPC(handlers: {
  onRequest?: (payload: AgentRequestPayload) => void;
  onResponse?: (payload: AgentResponsePayload) => void;
  onStream?: (payload: AgentStreamPayload) => void;
}): void {
  useIPCSubscription<AgentRequestPayload>('agent:request', (message) => {
    handlers.onRequest?.(message.payload);
  }, [handlers.onRequest]);

  useIPCSubscription<AgentResponsePayload>('agent:response', (message) => {
    handlers.onResponse?.(message.payload);
  }, [handlers.onResponse]);

  useIPCSubscription<AgentStreamPayload>('agent:stream', (message) => {
    handlers.onStream?.(message.payload);
  }, [handlers.onStream]);
}

/**
 * Hook to send agent requests from child windows
 */
export function useAgentRequest() {
  const { send, request } = useIPCSend();

  const sendRequest = useCallback(
    async (
      action: string,
      params: Record<string, unknown>,
      mainWindowId: string = 'main'
    ): Promise<unknown> => {
      const requestId = `req_${Date.now()}_${Math.random().toString(36).slice(2, 9)}`;
      const payload: AgentRequestPayload = { requestId, action, params };

      return request<AgentRequestPayload, AgentResponsePayload>(
        'agent:request',
        payload,
        mainWindowId,
        60000 // 60 second timeout for agent requests
      );
    },
    [request]
  );

  const streamResponse = useCallback(
    async (
      requestId: string,
      chunk: string,
      done: boolean,
      targetWindow: string
    ): Promise<void> => {
      const payload: AgentStreamPayload = { requestId, chunk, done };
      await send('agent:stream', payload, targetWindow);
    },
    [send]
  );

  return { sendRequest, streamResponse };
}

// ========== Window coordination hooks ==========

/**
 * Hook to signal window ready state
 */
export function useWindowReady(): () => Promise<void> {
  const { broadcast } = useIPCSend();
  const windowId = useWindowId();

  return useCallback(async (): Promise<void> => {
    await broadcast('window:ready', { windowId });
  }, [broadcast, windowId]);
}

/**
 * Hook to listen for window ready events
 */
export function useOnWindowReady(
  onReady: (windowId: string) => void
): void {
  useIPCSubscription<{ windowId: string }>('window:ready', (message) => {
    onReady(message.payload.windowId);
  }, [onReady]);
}
