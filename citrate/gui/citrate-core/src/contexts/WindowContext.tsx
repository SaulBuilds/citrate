/**
 * Window Context - Multi-Window State Management
 *
 * Sprint 5: Multi-Window & Terminal Integration
 *
 * Manages the state of all windows in the application and provides
 * methods for opening, closing, and communicating between windows.
 */

import React, { createContext, useContext, useReducer, useCallback, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { listen, emit } from '@tauri-apps/api/event';
import {
  WindowState,
  WindowType,
  WindowData,
  WindowManagerState,
  WindowContextType,
  CreateWindowOptions,
  DEFAULT_WINDOW_SIZES,
  DEFAULT_WINDOW_TITLES,
  WindowEvent,
} from '../types/window';

// Generate unique window ID
function generateWindowId(type: WindowType): string {
  return `${type}_${Date.now()}_${Math.random().toString(36).substr(2, 6)}`;
}

// Initial state
const initialState: WindowManagerState = {
  windows: [],
  activeWindowId: null,
  mainWindowId: 'main',
};

// Action types
type WindowAction =
  | { type: 'ADD_WINDOW'; window: WindowState }
  | { type: 'REMOVE_WINDOW'; id: string }
  | { type: 'FOCUS_WINDOW'; id: string }
  | { type: 'BLUR_WINDOW'; id: string }
  | { type: 'UPDATE_WINDOW'; id: string; updates: Partial<WindowState> }
  | { type: 'UPDATE_WINDOW_DATA'; id: string; data: Partial<WindowData> }
  | { type: 'SET_MAIN_WINDOW'; id: string };

// Reducer
function windowReducer(state: WindowManagerState, action: WindowAction): WindowManagerState {
  switch (action.type) {
    case 'ADD_WINDOW':
      return {
        ...state,
        windows: [...state.windows, action.window],
        activeWindowId: action.window.id,
      };

    case 'REMOVE_WINDOW':
      const remainingWindows = state.windows.filter((w) => w.id !== action.id);
      return {
        ...state,
        windows: remainingWindows,
        activeWindowId:
          state.activeWindowId === action.id
            ? remainingWindows[remainingWindows.length - 1]?.id || null
            : state.activeWindowId,
      };

    case 'FOCUS_WINDOW':
      return {
        ...state,
        windows: state.windows.map((w) => ({
          ...w,
          isFocused: w.id === action.id,
        })),
        activeWindowId: action.id,
      };

    case 'BLUR_WINDOW':
      return {
        ...state,
        windows: state.windows.map((w) =>
          w.id === action.id ? { ...w, isFocused: false } : w
        ),
      };

    case 'UPDATE_WINDOW':
      return {
        ...state,
        windows: state.windows.map((w) =>
          w.id === action.id ? { ...w, ...action.updates } : w
        ),
      };

    case 'UPDATE_WINDOW_DATA':
      return {
        ...state,
        windows: state.windows.map((w) =>
          w.id === action.id
            ? { ...w, data: { ...w.data, ...action.data } as WindowData }
            : w
        ),
      };

    case 'SET_MAIN_WINDOW':
      return {
        ...state,
        mainWindowId: action.id,
      };

    default:
      return state;
  }
}

// Context
const WindowContext = createContext<WindowContextType | null>(null);

// Provider props
interface WindowProviderProps {
  children: React.ReactNode;
}

/**
 * Window Provider Component
 *
 * Wraps the application and provides window management capabilities.
 */
export function WindowProvider({ children }: WindowProviderProps) {
  const [state, dispatch] = useReducer(windowReducer, initialState);

  // Listen for window events from Tauri
  useEffect(() => {
    const unlisten = Promise.all([
      listen<WindowEvent>('window-opened', (event) => {
        console.log('Window opened:', event.payload);
      }),
      listen<WindowEvent>('window-closed', (event) => {
        dispatch({ type: 'REMOVE_WINDOW', id: event.payload.windowId });
      }),
      listen<WindowEvent>('window-focused', (event) => {
        dispatch({ type: 'FOCUS_WINDOW', id: event.payload.windowId });
      }),
      listen<WindowEvent>('window-blurred', (event) => {
        dispatch({ type: 'BLUR_WINDOW', id: event.payload.windowId });
      }),
    ]);

    return () => {
      unlisten.then((listeners) => listeners.forEach((u) => u()));
    };
  }, []);

  // Open a new window
  const openWindow = useCallback(
    async (options: CreateWindowOptions): Promise<string> => {
      const id = generateWindowId(options.type);
      const title = options.title || DEFAULT_WINDOW_TITLES[options.type];
      const size = options.size || DEFAULT_WINDOW_SIZES[options.type];

      // Create window state
      const windowState: WindowState = {
        id,
        type: options.type,
        title,
        isOpen: true,
        isFocused: true,
        position: options.position,
        size,
        data: options.data as WindowData,
        createdAt: Date.now(),
      };

      try {
        // Call Tauri to create the window
        await invoke('create_window', {
          windowId: id,
          windowType: options.type,
          title,
          width: size.width,
          height: size.height,
          x: options.position?.x,
          y: options.position?.y,
          data: options.data ? JSON.stringify(options.data) : null,
        });

        // Add to state
        dispatch({ type: 'ADD_WINDOW', window: windowState });

        // Emit event for other windows
        await emit('window-created', { windowId: id, type: options.type });

        return id;
      } catch (error) {
        console.error('Failed to create window:', error);
        throw error;
      }
    },
    []
  );

  // Close a window
  const closeWindow = useCallback(async (id: string): Promise<void> => {
    try {
      await invoke('close_window', { windowId: id });
      dispatch({ type: 'REMOVE_WINDOW', id });
      await emit('window-closed', { windowId: id });
    } catch (error) {
      console.error('Failed to close window:', error);
      throw error;
    }
  }, []);

  // Focus a window
  const focusWindow = useCallback(async (id: string): Promise<void> => {
    try {
      await invoke('focus_window', { windowId: id });
      dispatch({ type: 'FOCUS_WINDOW', id });
    } catch (error) {
      console.error('Failed to focus window:', error);
      throw error;
    }
  }, []);

  // Get window by ID
  const getWindow = useCallback(
    (id: string): WindowState | undefined => {
      return state.windows.find((w) => w.id === id);
    },
    [state.windows]
  );

  // Get windows by type
  const getWindowsByType = useCallback(
    (type: WindowType): WindowState[] => {
      return state.windows.filter((w) => w.type === type && w.isOpen);
    },
    [state.windows]
  );

  // Update window data
  const updateWindowData = useCallback((id: string, data: Partial<WindowData>): void => {
    dispatch({ type: 'UPDATE_WINDOW_DATA', id, data });
  }, []);

  // Check if window type is open
  const hasOpenWindow = useCallback(
    (type: WindowType): boolean => {
      return state.windows.some((w) => w.type === type && w.isOpen);
    },
    [state.windows]
  );

  const contextValue: WindowContextType = {
    state,
    openWindow,
    closeWindow,
    focusWindow,
    getWindow,
    getWindowsByType,
    updateWindowData,
    hasOpenWindow,
  };

  return <WindowContext.Provider value={contextValue}>{children}</WindowContext.Provider>;
}

/**
 * Hook to use window context
 */
export function useWindows(): WindowContextType {
  const context = useContext(WindowContext);
  if (!context) {
    throw new Error('useWindows must be used within a WindowProvider');
  }
  return context;
}

/**
 * Hook to get current window info
 */
export function useCurrentWindow(): WindowState | null {
  const { state, getWindow } = useWindows();

  // In a real implementation, we'd get the current window ID from Tauri
  // For now, return the active window
  if (state.activeWindowId) {
    return getWindow(state.activeWindowId) || null;
  }
  return null;
}

/**
 * Hook to open a terminal window
 */
export function useOpenTerminal() {
  const { openWindow, getWindowsByType } = useWindows();

  return useCallback(
    async (cwd?: string): Promise<string> => {
      return openWindow({
        type: 'terminal',
        title: 'Terminal',
        data: {
          type: 'terminal',
          sessionId: '',
          cwd: cwd || process.cwd?.() || '~',
        },
      });
    },
    [openWindow, getWindowsByType]
  );
}

/**
 * Hook to open a preview window
 */
export function useOpenPreview() {
  const { openWindow } = useWindows();

  return useCallback(
    async (url: string, title?: string): Promise<string> => {
      return openWindow({
        type: 'preview',
        title: title || 'App Preview',
        data: {
          type: 'preview',
          url,
          canGoBack: false,
          canGoForward: false,
        },
      });
    },
    [openWindow]
  );
}

/**
 * Hook to open an editor window
 */
export function useOpenEditor() {
  const { openWindow } = useWindows();

  return useCallback(
    async (filePath?: string): Promise<string> => {
      return openWindow({
        type: 'editor',
        title: filePath ? filePath.split('/').pop() || 'Editor' : 'Code Editor',
        data: {
          type: 'editor',
          files: [],
          activeFile: filePath || null,
        },
      });
    },
    [openWindow]
  );
}

export default WindowContext;
