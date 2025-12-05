/**
 * Window Persistence Hook
 *
 * Sprint 5: Multi-Window & Terminal Integration
 *
 * React hook for automatic window state persistence.
 */

import { useEffect, useCallback, useRef } from 'react';
import { WindowState } from '../types/window';
import {
  saveWindowStates,
  loadWindowStates,
  clearWindowStates,
  getWindowPersistenceManager,
} from '../services/windowPersistence';

interface UseWindowPersistenceOptions {
  /** Auto-save interval in milliseconds */
  autoSaveInterval?: number;
  /** Whether to restore windows on mount */
  restoreOnMount?: boolean;
  /** Callback to restore a window */
  onRestore?: (state: {
    type: string;
    title: string;
    position?: { x: number; y: number };
    size?: { width: number; height: number };
    data?: unknown;
  }) => Promise<void>;
}

interface UseWindowPersistenceReturn {
  /** Save current window states */
  save: () => void;
  /** Load saved window states */
  load: () => ReturnType<typeof loadWindowStates>;
  /** Clear saved window states */
  clear: () => void;
  /** Check if there are saved states */
  hasSavedState: () => boolean;
}

/**
 * Hook for window persistence
 */
export function useWindowPersistence(
  windows: WindowState[],
  options: UseWindowPersistenceOptions = {}
): UseWindowPersistenceReturn {
  const {
    autoSaveInterval = 5000,
    restoreOnMount = false,
    onRestore,
  } = options;

  const windowsRef = useRef(windows);
  windowsRef.current = windows;

  const restoredRef = useRef(false);

  // Save function
  const save = useCallback(() => {
    saveWindowStates(windowsRef.current);
  }, []);

  // Load function
  const load = useCallback(() => {
    return loadWindowStates();
  }, []);

  // Clear function
  const clear = useCallback(() => {
    clearWindowStates();
  }, []);

  // Check if has saved state
  const hasSavedState = useCallback(() => {
    const session = loadWindowStates();
    return session !== null && session.windows.length > 0;
  }, []);

  // Auto-save effect
  useEffect(() => {
    const manager = getWindowPersistenceManager();
    manager.updateWindows(windows);
    manager.startAutoSave(autoSaveInterval);

    return () => {
      // Save on unmount
      manager.saveNow();
      manager.stopAutoSave();
    };
  }, [windows, autoSaveInterval]);

  // Restore on mount effect
  useEffect(() => {
    if (restoreOnMount && !restoredRef.current && onRestore) {
      restoredRef.current = true;

      const session = loadWindowStates();
      if (session && session.windows.length > 0) {
        // Restore windows sequentially
        (async () => {
          for (const windowState of session.windows) {
            try {
              await onRestore({
                type: windowState.type,
                title: windowState.title,
                position: windowState.position,
                size: windowState.size,
                data: windowState.data,
              });
            } catch (err) {
              console.error('Failed to restore window:', err);
            }
          }
        })();
      }
    }
  }, [restoreOnMount, onRestore]);

  // Save on beforeunload
  useEffect(() => {
    const handleBeforeUnload = () => {
      saveWindowStates(windowsRef.current);
    };

    window.addEventListener('beforeunload', handleBeforeUnload);

    return () => {
      window.removeEventListener('beforeunload', handleBeforeUnload);
    };
  }, []);

  return {
    save,
    load,
    clear,
    hasSavedState,
  };
}

/**
 * Hook to prompt user to restore previous session
 */
export function useRestorePrompt(): {
  shouldPrompt: boolean;
  restore: () => void;
  dismiss: () => void;
} {
  const session = loadWindowStates();
  const shouldPrompt = session !== null && session.windows.length > 0;

  const restore = useCallback(() => {
    // This would be handled by the caller with onRestore callback
  }, []);

  const dismiss = useCallback(() => {
    clearWindowStates();
  }, []);

  return {
    shouldPrompt,
    restore,
    dismiss,
  };
}

export default useWindowPersistence;
