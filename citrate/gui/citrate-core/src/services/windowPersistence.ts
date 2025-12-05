/**
 * Window Persistence Service
 *
 * Sprint 5: Multi-Window & Terminal Integration
 *
 * Saves and restores window state across app restarts.
 */

import { WindowState, WindowType } from '../types/window';

const STORAGE_KEY = 'citrate_window_state';
const SESSION_STORAGE_KEY = 'citrate_window_session';

/**
 * Persisted window state
 */
interface PersistedWindowState {
  id: string;
  type: WindowType;
  title: string;
  position?: { x: number; y: number };
  size?: { width: number; height: number };
  data?: unknown;
}

/**
 * Window session data
 */
interface WindowSession {
  windows: PersistedWindowState[];
  lastActiveWindowId: string | null;
  savedAt: number;
}

/**
 * Save window states to localStorage
 */
export function saveWindowStates(windows: WindowState[]): void {
  try {
    const persistedWindows: PersistedWindowState[] = windows
      .filter((w) => w.isOpen && w.type !== 'main') // Don't persist main window
      .map((w) => ({
        id: w.id,
        type: w.type,
        title: w.title,
        position: w.position,
        size: w.size,
        data: w.data,
      }));

    const activeWindow = windows.find((w) => w.isFocused);

    const session: WindowSession = {
      windows: persistedWindows,
      lastActiveWindowId: activeWindow?.id || null,
      savedAt: Date.now(),
    };

    localStorage.setItem(STORAGE_KEY, JSON.stringify(session));
  } catch (err) {
    console.error('Failed to save window states:', err);
  }
}

/**
 * Load window states from localStorage
 */
export function loadWindowStates(): WindowSession | null {
  try {
    const data = localStorage.getItem(STORAGE_KEY);
    if (!data) return null;

    const session = JSON.parse(data) as WindowSession;

    // Check if session is too old (24 hours)
    const maxAge = 24 * 60 * 60 * 1000;
    if (Date.now() - session.savedAt > maxAge) {
      clearWindowStates();
      return null;
    }

    return session;
  } catch (err) {
    console.error('Failed to load window states:', err);
    return null;
  }
}

/**
 * Clear saved window states
 */
export function clearWindowStates(): void {
  try {
    localStorage.removeItem(STORAGE_KEY);
  } catch (err) {
    console.error('Failed to clear window states:', err);
  }
}

/**
 * Save session-specific data (current session only)
 */
export function saveSessionData(key: string, data: unknown): void {
  try {
    const sessionData = sessionStorage.getItem(SESSION_STORAGE_KEY);
    const session = sessionData ? JSON.parse(sessionData) : {};
    session[key] = data;
    sessionStorage.setItem(SESSION_STORAGE_KEY, JSON.stringify(session));
  } catch (err) {
    console.error('Failed to save session data:', err);
  }
}

/**
 * Load session-specific data
 */
export function loadSessionData<T>(key: string): T | null {
  try {
    const sessionData = sessionStorage.getItem(SESSION_STORAGE_KEY);
    if (!sessionData) return null;
    const session = JSON.parse(sessionData);
    return session[key] ?? null;
  } catch (err) {
    console.error('Failed to load session data:', err);
    return null;
  }
}

/**
 * Clear session data
 */
export function clearSessionData(): void {
  try {
    sessionStorage.removeItem(SESSION_STORAGE_KEY);
  } catch (err) {
    console.error('Failed to clear session data:', err);
  }
}

/**
 * Auto-save hook
 * Returns a function to trigger save
 */
export function useAutoSave(_windows: WindowState[], _interval: number = 5000): void {
  // Note: This is designed to be used with useEffect
  // The actual implementation should be in a React hook file
  // This is a utility function for reference
}

/**
 * Window state persistence manager
 */
export class WindowPersistenceManager {
  private autoSaveTimer: ReturnType<typeof setInterval> | null = null;
  private windows: WindowState[] = [];

  /**
   * Start auto-saving window states
   */
  startAutoSave(interval: number = 5000): void {
    if (this.autoSaveTimer) {
      clearInterval(this.autoSaveTimer);
    }

    this.autoSaveTimer = setInterval(() => {
      if (this.windows.length > 0) {
        saveWindowStates(this.windows);
      }
    }, interval);
  }

  /**
   * Stop auto-saving
   */
  stopAutoSave(): void {
    if (this.autoSaveTimer) {
      clearInterval(this.autoSaveTimer);
      this.autoSaveTimer = null;
    }
  }

  /**
   * Update tracked windows
   */
  updateWindows(windows: WindowState[]): void {
    this.windows = windows;
  }

  /**
   * Save current state immediately
   */
  saveNow(): void {
    saveWindowStates(this.windows);
  }

  /**
   * Get saved session
   */
  getSavedSession(): WindowSession | null {
    return loadWindowStates();
  }

  /**
   * Clear saved session
   */
  clearSaved(): void {
    clearWindowStates();
  }
}

// Singleton instance
let persistenceManager: WindowPersistenceManager | null = null;

/**
 * Get the persistence manager singleton
 */
export function getWindowPersistenceManager(): WindowPersistenceManager {
  if (!persistenceManager) {
    persistenceManager = new WindowPersistenceManager();
  }
  return persistenceManager;
}

export default {
  saveWindowStates,
  loadWindowStates,
  clearWindowStates,
  saveSessionData,
  loadSessionData,
  clearSessionData,
  getWindowPersistenceManager,
};
