/**
 * App Context
 *
 * Provides global application state with automatic persistence:
 * - Current active tab
 * - Recent addresses
 * - App settings
 * - Window state
 *
 * All state changes are automatically persisted to localStorage.
 */

import React, { createContext, useContext, useState, useEffect, ReactNode } from 'react';
import {
  StorageKeys,
  getStorageItem,
  setStorageItem,
  addRecentAddress as addRecentAddressToStorage,
  getRecentAddresses,
  clearRecentAddresses as clearRecentAddressesInStorage,
  initializeStorage,
  AppSettings,
  getDefaultSettings,
} from '../utils/storage';

// View type for navigation
export type View = 'dashboard' | 'wallet' | 'dag' | 'models' | 'lora' | 'marketplace' | 'chat' | 'ipfs' | 'contracts' | 'terminal' | 'gpu' | 'settings';

// App state interface
export interface AppState {
  currentTab: View;
  recentAddresses: string[];
  settings: AppSettings;
  windowSize: { width: number; height: number } | null;
  initialized: boolean;
}

// App context interface
export interface AppContextValue extends AppState {
  // Tab navigation
  setCurrentTab: (tab: View) => void;

  // Recent addresses
  addRecentAddress: (address: string) => void;
  clearRecentAddresses: () => void;

  // Settings
  updateSettings: (settings: Partial<AppSettings>) => void;
  resetSettings: () => void;

  // Window state
  setWindowSize: (size: { width: number; height: number }) => void;
}

// Create context with undefined default (will throw error if used without provider)
const AppContext = createContext<AppContextValue | undefined>(undefined);

// Provider props
interface AppProviderProps {
  children: ReactNode;
}

/**
 * App Context Provider
 *
 * Wraps the app and provides global state with automatic persistence
 */
export const AppProvider: React.FC<AppProviderProps> = ({ children }) => {
  // Initialize state from storage
  const [initialized, setInitialized] = useState(false);
  const [currentTab, setCurrentTabState] = useState<View>('dashboard');
  const [recentAddresses, setRecentAddresses] = useState<string[]>([]);
  const [settings, setSettings] = useState<AppSettings>(getDefaultSettings());
  const [windowSize, setWindowSizeState] = useState<{ width: number; height: number } | null>(null);

  // Initialize storage and load persisted state on mount
  useEffect(() => {
    console.log('[AppContext] Initializing storage...');

    // Initialize storage (runs migrations, sets defaults)
    initializeStorage();

    // Load persisted state
    const persistedTab = getStorageItem(StorageKeys.CURRENT_TAB);
    const persistedRecent = getRecentAddresses();
    const persistedSettings = getStorageItem(StorageKeys.SETTINGS);
    const persistedWindowSize = getStorageItem(StorageKeys.WINDOW_SIZE);

    // Restore state from storage
    if (persistedTab) {
      setCurrentTabState(persistedTab as View);
      console.log(`[AppContext] Restored tab: ${persistedTab}`);
    }

    if (persistedRecent) {
      setRecentAddresses(persistedRecent);
      console.log(`[AppContext] Restored ${persistedRecent.length} recent addresses`);
    }

    if (persistedSettings) {
      setSettings(persistedSettings);
      console.log('[AppContext] Restored settings');
    }

    if (persistedWindowSize) {
      setWindowSizeState(persistedWindowSize);
      console.log('[AppContext] Restored window size');
    }

    setInitialized(true);
    console.log('[AppContext] Initialization complete');
  }, []);

  // Set current tab with automatic persistence
  const setCurrentTab = (tab: View) => {
    setCurrentTabState(tab);
    setStorageItem(StorageKeys.CURRENT_TAB, tab);
    console.log(`[AppContext] Tab changed to: ${tab}`);
  };

  // Add recent address with deduplication
  const addRecentAddress = (address: string) => {
    if (!address || !address.trim()) return;

    const success = addRecentAddressToStorage(address);
    if (success) {
      // Reload from storage to ensure consistency
      const updated = getRecentAddresses();
      setRecentAddresses(updated);
      console.log(`[AppContext] Added recent address: ${address.substring(0, 10)}...`);
    }
  };

  // Clear all recent addresses
  const clearRecentAddresses = () => {
    const success = clearRecentAddressesInStorage();
    if (success) {
      setRecentAddresses([]);
      console.log('[AppContext] Cleared recent addresses');
    }
  };

  // Update settings (merge with existing)
  const updateSettings = (newSettings: Partial<AppSettings>) => {
    const updated = { ...settings, ...newSettings };
    setSettings(updated);
    setStorageItem(StorageKeys.SETTINGS, updated);
    console.log('[AppContext] Settings updated:', Object.keys(newSettings));
  };

  // Reset settings to defaults
  const resetSettings = () => {
    const defaults = getDefaultSettings();
    setSettings(defaults);
    setStorageItem(StorageKeys.SETTINGS, defaults);
    console.log('[AppContext] Settings reset to defaults');
  };

  // Set window size
  const setWindowSize = (size: { width: number; height: number }) => {
    setWindowSizeState(size);
    setStorageItem(StorageKeys.WINDOW_SIZE, size);
  };

  // Context value
  const value: AppContextValue = {
    // State
    currentTab,
    recentAddresses,
    settings,
    windowSize,
    initialized,

    // Actions
    setCurrentTab,
    addRecentAddress,
    clearRecentAddresses,
    updateSettings,
    resetSettings,
    setWindowSize,
  };

  return <AppContext.Provider value={value}>{children}</AppContext.Provider>;
};

/**
 * useApp Hook
 *
 * Access app context from any component.
 * Throws error if used outside AppProvider.
 */
export const useApp = (): AppContextValue => {
  const context = useContext(AppContext);

  if (context === undefined) {
    throw new Error(
      'useApp must be used within an AppProvider. ' +
      'Wrap your app with <AppProvider> in App.tsx.'
    );
  }

  return context;
};

/**
 * useAppTab Hook
 *
 * Convenience hook for tab navigation only
 */
export const useAppTab = () => {
  const { currentTab, setCurrentTab } = useApp();
  return { currentTab, setCurrentTab };
};

/**
 * useRecentAddresses Hook
 *
 * Convenience hook for recent addresses only
 */
export const useRecentAddresses = () => {
  const { recentAddresses, addRecentAddress, clearRecentAddresses } = useApp();
  return { recentAddresses, addRecentAddress, clearRecentAddresses };
};

/**
 * useAppSettings Hook
 *
 * Convenience hook for settings only
 */
export const useAppSettings = () => {
  const { settings, updateSettings, resetSettings } = useApp();
  return { settings, updateSettings, resetSettings };
};

export default AppContext;
