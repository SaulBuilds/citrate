/**
 * Agent Context
 *
 * Provides centralized state management for the AI agent conversation interface.
 * Handles sessions, messages, streaming, tool approvals, and Tauri event bindings.
 */

import React, {
  createContext,
  useContext,
  useReducer,
  useEffect,
  useCallback,
  useRef,
  ReactNode,
} from 'react';
import { invoke } from '@tauri-apps/api/core';
import { listen, UnlistenFn } from '@tauri-apps/api/event';
import {
  AgentContextState,
  AgentSession,
  Message,
  PendingToolCall,
  AgentConfig,
  AgentStatus,
  AgentTokenEvent,
  AgentCompleteEvent,
  AgentErrorEvent,
  AgentToolCallEvent,
} from '../types/agent';

// =============================================================================
// Action Types
// =============================================================================

type AgentAction =
  | { type: 'SET_INITIALIZED'; payload: boolean }
  | { type: 'SET_CONNECTED'; payload: boolean }
  | { type: 'SET_SESSION'; payload: string | null }
  | { type: 'SET_SESSIONS'; payload: AgentSession[] }
  | { type: 'ADD_SESSION'; payload: AgentSession }
  | { type: 'REMOVE_SESSION'; payload: string }
  | { type: 'SET_MESSAGES'; payload: Message[] }
  | { type: 'ADD_MESSAGE'; payload: Message }
  | { type: 'UPDATE_MESSAGE'; payload: { id: string; updates: Partial<Message> } }
  | { type: 'APPEND_TO_MESSAGE'; payload: { id: string; content: string } }
  | { type: 'CLEAR_MESSAGES' }
  | { type: 'SET_STREAMING'; payload: { isStreaming: boolean; messageId: string | null } }
  | { type: 'SET_PENDING_TOOLS'; payload: PendingToolCall[] }
  | { type: 'ADD_PENDING_TOOL'; payload: PendingToolCall }
  | { type: 'REMOVE_PENDING_TOOL'; payload: string }
  | { type: 'SET_CONFIG'; payload: AgentConfig | null }
  | { type: 'SET_STATUS'; payload: AgentStatus | null };

// =============================================================================
// Initial State
// =============================================================================

const initialState: AgentContextState = {
  isInitialized: false,
  isConnected: false,
  currentSessionId: null,
  sessions: [],
  messages: [],
  isStreaming: false,
  streamingMessageId: null,
  pendingTools: [],
  config: null,
  status: null,
};

// =============================================================================
// Reducer
// =============================================================================

function agentReducer(state: AgentContextState, action: AgentAction): AgentContextState {
  switch (action.type) {
    case 'SET_INITIALIZED':
      return { ...state, isInitialized: action.payload };

    case 'SET_CONNECTED':
      return { ...state, isConnected: action.payload };

    case 'SET_SESSION':
      return { ...state, currentSessionId: action.payload };

    case 'SET_SESSIONS':
      return { ...state, sessions: action.payload };

    case 'ADD_SESSION':
      return { ...state, sessions: [...state.sessions, action.payload] };

    case 'REMOVE_SESSION':
      return {
        ...state,
        sessions: state.sessions.filter((s) => s.id !== action.payload),
        currentSessionId:
          state.currentSessionId === action.payload ? null : state.currentSessionId,
      };

    case 'SET_MESSAGES':
      return { ...state, messages: action.payload };

    case 'ADD_MESSAGE':
      return { ...state, messages: [...state.messages, action.payload] };

    case 'UPDATE_MESSAGE':
      return {
        ...state,
        messages: state.messages.map((m) =>
          m.id === action.payload.id ? { ...m, ...action.payload.updates } : m
        ),
      };

    case 'APPEND_TO_MESSAGE':
      return {
        ...state,
        messages: state.messages.map((m) =>
          m.id === action.payload.id
            ? { ...m, content: m.content + action.payload.content }
            : m
        ),
      };

    case 'CLEAR_MESSAGES':
      return { ...state, messages: [] };

    case 'SET_STREAMING':
      return {
        ...state,
        isStreaming: action.payload.isStreaming,
        streamingMessageId: action.payload.messageId,
      };

    case 'SET_PENDING_TOOLS':
      return { ...state, pendingTools: action.payload };

    case 'ADD_PENDING_TOOL':
      return { ...state, pendingTools: [...state.pendingTools, action.payload] };

    case 'REMOVE_PENDING_TOOL':
      return {
        ...state,
        pendingTools: state.pendingTools.filter((t) => t.id !== action.payload),
      };

    case 'SET_CONFIG':
      return { ...state, config: action.payload };

    case 'SET_STATUS':
      return { ...state, status: action.payload };

    default:
      return state;
  }
}

// =============================================================================
// Context Interface
// =============================================================================

interface AgentContextValue extends AgentContextState {
  // Session actions
  createSession: () => Promise<AgentSession | null>;
  switchSession: (sessionId: string) => Promise<void>;
  deleteSession: (sessionId: string) => Promise<boolean>;
  refreshSessions: () => Promise<void>;

  // Message actions
  sendMessage: (content: string) => Promise<void>;
  clearHistory: () => Promise<void>;
  cancelStreaming: () => void;

  // Tool actions
  approveTool: (toolId: string) => Promise<boolean>;
  rejectTool: (toolId: string) => Promise<boolean>;
  refreshPendingTools: () => Promise<void>;

  // Config actions
  updateConfig: (updates: Partial<AgentConfig>) => Promise<void>;
  refreshConfig: () => Promise<void>;

  // Status actions
  refreshStatus: () => Promise<void>;
}

// =============================================================================
// Context Creation
// =============================================================================

const AgentContext = createContext<AgentContextValue | undefined>(undefined);

// =============================================================================
// Provider Component
// =============================================================================

interface AgentProviderProps {
  children: ReactNode;
}

export const AgentProvider: React.FC<AgentProviderProps> = ({ children }) => {
  const [state, dispatch] = useReducer(agentReducer, initialState);
  const unlistenRefs = useRef<UnlistenFn[]>([]);

  // ---------------------------------------------------------------------------
  // Tauri Event Listeners
  // ---------------------------------------------------------------------------

  useEffect(() => {
    const setupListeners = async () => {
      try {
        // Listen for streaming tokens
        const unlistenToken = await listen<AgentTokenEvent>('agent-token', (event) => {
          const { session_id, message_id, content, is_complete } = event.payload;

          if (session_id !== state.currentSessionId) return;

          if (is_complete) {
            dispatch({
              type: 'UPDATE_MESSAGE',
              payload: { id: message_id, updates: { isStreaming: false } },
            });
            dispatch({
              type: 'SET_STREAMING',
              payload: { isStreaming: false, messageId: null },
            });
          } else {
            dispatch({
              type: 'APPEND_TO_MESSAGE',
              payload: { id: message_id, content },
            });
          }
        });
        unlistenRefs.current.push(unlistenToken);

        // Listen for message completion
        const unlistenComplete = await listen<AgentCompleteEvent>(
          'agent-complete',
          (event) => {
            const { session_id, message_id } = event.payload;

            if (session_id !== state.currentSessionId) return;

            dispatch({
              type: 'UPDATE_MESSAGE',
              payload: { id: message_id, updates: { isStreaming: false } },
            });
            dispatch({
              type: 'SET_STREAMING',
              payload: { isStreaming: false, messageId: null },
            });
          }
        );
        unlistenRefs.current.push(unlistenComplete);

        // Listen for errors
        const unlistenError = await listen<AgentErrorEvent>('agent-error', (event) => {
          const { session_id, error } = event.payload;

          if (session_id !== state.currentSessionId) return;

          console.error('[AgentContext] Agent error:', error);

          // Add error message to chat
          const errorMessage: Message = {
            id: `error-${Date.now()}`,
            role: 'system',
            content: `Error: ${error}`,
            timestamp: Date.now(),
          };
          dispatch({ type: 'ADD_MESSAGE', payload: errorMessage });
          dispatch({
            type: 'SET_STREAMING',
            payload: { isStreaming: false, messageId: null },
          });
        });
        unlistenRefs.current.push(unlistenError);

        // Listen for tool calls
        const unlistenTool = await listen<AgentToolCallEvent>(
          'agent-tool-call',
          (event) => {
            const { session_id, tool_id, tool_name, description, high_risk, params } =
              event.payload;

            if (session_id !== state.currentSessionId) return;

            const toolCall: PendingToolCall = {
              id: tool_id,
              toolName: tool_name,
              description,
              highRisk: high_risk,
              params,
            };
            dispatch({ type: 'ADD_PENDING_TOOL', payload: toolCall });
          }
        );
        unlistenRefs.current.push(unlistenTool);

        console.log('[AgentContext] Event listeners registered');
      } catch (error) {
        console.error('[AgentContext] Failed to setup listeners:', error);
      }
    };

    setupListeners();

    return () => {
      unlistenRefs.current.forEach((unlisten) => unlisten());
      unlistenRefs.current = [];
    };
  }, [state.currentSessionId]);

  // ---------------------------------------------------------------------------
  // Initialization
  // ---------------------------------------------------------------------------

  useEffect(() => {
    const initialize = async () => {
      try {
        // Check if agent is ready
        const isReady = await invoke<boolean>('agent_is_ready');
        dispatch({ type: 'SET_INITIALIZED', payload: isReady });
        dispatch({ type: 'SET_CONNECTED', payload: isReady });

        if (isReady) {
          // Load status
          const status = await invoke<AgentStatus>('agent_get_status');
          dispatch({ type: 'SET_STATUS', payload: status });

          // Load config
          const config = await invoke<AgentConfig>('agent_get_config');
          dispatch({ type: 'SET_CONFIG', payload: config });

          // Load sessions
          const sessionIds = await invoke<string[]>('agent_list_sessions');
          const sessions: AgentSession[] = [];
          for (const id of sessionIds) {
            const session = await invoke<AgentSession | null>('agent_get_session', {
              sessionId: id,
            });
            if (session) sessions.push(session);
          }
          dispatch({ type: 'SET_SESSIONS', payload: sessions });
        }

        console.log('[AgentContext] Initialized, ready:', isReady);
      } catch (error) {
        console.error('[AgentContext] Initialization failed:', error);
        dispatch({ type: 'SET_INITIALIZED', payload: false });
        dispatch({ type: 'SET_CONNECTED', payload: false });
      }
    };

    initialize();
  }, []);

  // ---------------------------------------------------------------------------
  // Session Actions
  // ---------------------------------------------------------------------------

  const createSession = useCallback(async (): Promise<AgentSession | null> => {
    try {
      const session = await invoke<AgentSession>('agent_create_session');
      dispatch({ type: 'ADD_SESSION', payload: session });
      dispatch({ type: 'SET_SESSION', payload: session.id });
      dispatch({ type: 'CLEAR_MESSAGES' });

      // Add welcome message
      const welcomeMessage: Message = {
        id: 'welcome',
        role: 'system',
        content:
          'Welcome! I can help you with blockchain operations, smart contracts, and AI model management. What would you like to do?',
        timestamp: Date.now(),
      };
      dispatch({ type: 'ADD_MESSAGE', payload: welcomeMessage });

      console.log('[AgentContext] Created session:', session.id);
      return session;
    } catch (error) {
      console.error('[AgentContext] Failed to create session:', error);
      return null;
    }
  }, []);

  const switchSession = useCallback(async (sessionId: string): Promise<void> => {
    try {
      const session = await invoke<AgentSession | null>('agent_get_session', {
        sessionId,
      });

      if (session) {
        dispatch({ type: 'SET_SESSION', payload: sessionId });

        // Load messages for this session
        const messages = await invoke<Message[]>('agent_get_messages', { sessionId });
        dispatch({ type: 'SET_MESSAGES', payload: messages });

        // Load pending tools
        const tools = await invoke<PendingToolCall[]>('agent_get_pending_tools', {
          sessionId,
        });
        dispatch({ type: 'SET_PENDING_TOOLS', payload: tools });

        console.log('[AgentContext] Switched to session:', sessionId);
      }
    } catch (error) {
      console.error('[AgentContext] Failed to switch session:', error);
    }
  }, []);

  const deleteSession = useCallback(async (sessionId: string): Promise<boolean> => {
    try {
      const result = await invoke<boolean>('agent_delete_session', { sessionId });
      if (result) {
        dispatch({ type: 'REMOVE_SESSION', payload: sessionId });
        console.log('[AgentContext] Deleted session:', sessionId);
      }
      return result;
    } catch (error) {
      console.error('[AgentContext] Failed to delete session:', error);
      return false;
    }
  }, []);

  const refreshSessions = useCallback(async (): Promise<void> => {
    try {
      const sessionIds = await invoke<string[]>('agent_list_sessions');
      const sessions: AgentSession[] = [];
      for (const id of sessionIds) {
        const session = await invoke<AgentSession | null>('agent_get_session', {
          sessionId: id,
        });
        if (session) sessions.push(session);
      }
      dispatch({ type: 'SET_SESSIONS', payload: sessions });
    } catch (error) {
      console.error('[AgentContext] Failed to refresh sessions:', error);
    }
  }, []);

  // ---------------------------------------------------------------------------
  // Message Actions
  // ---------------------------------------------------------------------------

  const sendMessage = useCallback(
    async (content: string): Promise<void> => {
      if (!state.currentSessionId) {
        // Create a session first
        const session = await createSession();
        if (!session) return;
      }

      const sessionId = state.currentSessionId!;

      // Add user message immediately
      const userMessage: Message = {
        id: `user-${Date.now()}`,
        role: 'user',
        content,
        timestamp: Date.now(),
      };
      dispatch({ type: 'ADD_MESSAGE', payload: userMessage });

      // Create placeholder for assistant response
      const assistantMessageId = `assistant-${Date.now()}`;
      const assistantMessage: Message = {
        id: assistantMessageId,
        role: 'assistant',
        content: '',
        timestamp: Date.now(),
        isStreaming: true,
      };
      dispatch({ type: 'ADD_MESSAGE', payload: assistantMessage });
      dispatch({
        type: 'SET_STREAMING',
        payload: { isStreaming: true, messageId: assistantMessageId },
      });

      try {
        const response = await invoke<{
          session_id: string;
          message_id: string;
          content: string;
          role: string;
          intent: string | null;
          intent_confidence: number | null;
          tool_invoked: boolean;
          tool_name: string | null;
          pending_approval: boolean;
        }>('agent_send_message', {
          sessionId,
          message: content,
        });

        // Update the assistant message with the response
        dispatch({
          type: 'UPDATE_MESSAGE',
          payload: {
            id: assistantMessageId,
            updates: {
              content: response.content,
              isStreaming: false,
              intent: response.intent
                ? {
                    intent: response.intent,
                    confidence: response.intent_confidence || 0,
                  }
                : undefined,
            },
          },
        });

        dispatch({
          type: 'SET_STREAMING',
          payload: { isStreaming: false, messageId: null },
        });

        // Refresh pending tools if there are any
        if (response.pending_approval) {
          await refreshPendingTools();
        }
      } catch (error) {
        console.error('[AgentContext] Failed to send message:', error);

        // Update with error
        dispatch({
          type: 'UPDATE_MESSAGE',
          payload: {
            id: assistantMessageId,
            updates: {
              content: `Sorry, I encountered an error: ${error}`,
              isStreaming: false,
            },
          },
        });
        dispatch({
          type: 'SET_STREAMING',
          payload: { isStreaming: false, messageId: null },
        });
      }
    },
    [state.currentSessionId, createSession]
  );

  const clearHistory = useCallback(async (): Promise<void> => {
    if (!state.currentSessionId) return;

    try {
      await invoke('agent_clear_history', { sessionId: state.currentSessionId });
      dispatch({ type: 'CLEAR_MESSAGES' });

      // Add fresh welcome message
      const welcomeMessage: Message = {
        id: 'welcome',
        role: 'system',
        content: 'History cleared. How can I help you?',
        timestamp: Date.now(),
      };
      dispatch({ type: 'ADD_MESSAGE', payload: welcomeMessage });

      console.log('[AgentContext] Cleared history');
    } catch (error) {
      console.error('[AgentContext] Failed to clear history:', error);
    }
  }, [state.currentSessionId]);

  const cancelStreaming = useCallback((): void => {
    if (state.streamingMessageId) {
      dispatch({
        type: 'UPDATE_MESSAGE',
        payload: {
          id: state.streamingMessageId,
          updates: { isStreaming: false, content: state.messages.find(m => m.id === state.streamingMessageId)?.content + ' [cancelled]' },
        },
      });
    }
    dispatch({ type: 'SET_STREAMING', payload: { isStreaming: false, messageId: null } });
  }, [state.streamingMessageId, state.messages]);

  // ---------------------------------------------------------------------------
  // Tool Actions
  // ---------------------------------------------------------------------------

  const refreshPendingTools = useCallback(async (): Promise<void> => {
    if (!state.currentSessionId) return;

    try {
      const tools = await invoke<PendingToolCall[]>('agent_get_pending_tools', {
        sessionId: state.currentSessionId,
      });
      dispatch({ type: 'SET_PENDING_TOOLS', payload: tools });
    } catch (error) {
      console.error('[AgentContext] Failed to refresh pending tools:', error);
    }
  }, [state.currentSessionId]);

  const approveTool = useCallback(
    async (toolId: string): Promise<boolean> => {
      if (!state.currentSessionId) return false;

      try {
        const result = await invoke<boolean>('agent_approve_tool', {
          sessionId: state.currentSessionId,
          toolId,
        });
        if (result) {
          dispatch({ type: 'REMOVE_PENDING_TOOL', payload: toolId });
        }
        return result;
      } catch (error) {
        console.error('[AgentContext] Failed to approve tool:', error);
        return false;
      }
    },
    [state.currentSessionId]
  );

  const rejectTool = useCallback(
    async (toolId: string): Promise<boolean> => {
      if (!state.currentSessionId) return false;

      try {
        const result = await invoke<boolean>('agent_reject_tool', {
          sessionId: state.currentSessionId,
          toolId,
        });
        if (result) {
          dispatch({ type: 'REMOVE_PENDING_TOOL', payload: toolId });
        }
        return result;
      } catch (error) {
        console.error('[AgentContext] Failed to reject tool:', error);
        return false;
      }
    },
    [state.currentSessionId]
  );

  // ---------------------------------------------------------------------------
  // Config Actions
  // ---------------------------------------------------------------------------

  const updateConfig = useCallback(
    async (updates: Partial<AgentConfig>): Promise<void> => {
      try {
        await invoke('agent_update_config', updates);
        const newConfig = await invoke<AgentConfig>('agent_get_config');
        dispatch({ type: 'SET_CONFIG', payload: newConfig });
        console.log('[AgentContext] Config updated');
      } catch (error) {
        console.error('[AgentContext] Failed to update config:', error);
      }
    },
    []
  );

  const refreshConfig = useCallback(async (): Promise<void> => {
    try {
      const config = await invoke<AgentConfig>('agent_get_config');
      dispatch({ type: 'SET_CONFIG', payload: config });
    } catch (error) {
      console.error('[AgentContext] Failed to refresh config:', error);
    }
  }, []);

  // ---------------------------------------------------------------------------
  // Status Actions
  // ---------------------------------------------------------------------------

  const refreshStatus = useCallback(async (): Promise<void> => {
    try {
      const status = await invoke<AgentStatus>('agent_get_status');
      dispatch({ type: 'SET_STATUS', payload: status });
    } catch (error) {
      console.error('[AgentContext] Failed to refresh status:', error);
    }
  }, []);

  // ---------------------------------------------------------------------------
  // Context Value
  // ---------------------------------------------------------------------------

  const value: AgentContextValue = {
    ...state,
    createSession,
    switchSession,
    deleteSession,
    refreshSessions,
    sendMessage,
    clearHistory,
    cancelStreaming,
    approveTool,
    rejectTool,
    refreshPendingTools,
    updateConfig,
    refreshConfig,
    refreshStatus,
  };

  return <AgentContext.Provider value={value}>{children}</AgentContext.Provider>;
};

// =============================================================================
// Hooks
// =============================================================================

/**
 * Main hook for accessing the agent context
 */
export function useAgent(): AgentContextValue {
  const context = useContext(AgentContext);
  if (context === undefined) {
    throw new Error('useAgent must be used within an AgentProvider');
  }
  return context;
}

/**
 * Hook for accessing messages only
 */
export function useMessages() {
  const { messages, isStreaming, streamingMessageId, sendMessage, clearHistory } =
    useAgent();
  return { messages, isStreaming, streamingMessageId, sendMessage, clearHistory };
}

/**
 * Hook for accessing streaming state only
 */
export function useStreaming() {
  const { isStreaming, streamingMessageId, cancelStreaming } = useAgent();
  return { isStreaming, streamingMessageId, cancelStreaming };
}

/**
 * Hook for accessing pending tool approvals
 */
export function usePendingTools() {
  const { pendingTools, approveTool, rejectTool, refreshPendingTools } = useAgent();
  return { pendingTools, approveTool, rejectTool, refreshPendingTools };
}

/**
 * Hook for accessing sessions
 */
export function useSessions() {
  const {
    currentSessionId,
    sessions,
    createSession,
    switchSession,
    deleteSession,
    refreshSessions,
  } = useAgent();
  return {
    currentSessionId,
    sessions,
    createSession,
    switchSession,
    deleteSession,
    refreshSessions,
  };
}

export default AgentContext;
