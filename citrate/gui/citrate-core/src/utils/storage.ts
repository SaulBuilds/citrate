/**
 * Storage Utility Module
 *
 * Provides type-safe localStorage wrapper with:
 * - Type-safe get/set operations
 * - Error handling for quota exceeded
 * - Storage versioning and migration
 * - Export/import functionality
 * - Recent addresses management
 */

// Storage version for migrations
const STORAGE_VERSION = 1;
const VERSION_KEY = 'citrate_storage_version';

// Storage keys
export const StorageKeys = {
  VERSION: VERSION_KEY,
  THEME: 'citrate_theme',
  CURRENT_TAB: 'citrate_current_tab',
  RECENT_ADDRESSES: 'citrate_recent_addresses',
  WINDOW_SIZE: 'citrate_window_size',
  SETTINGS: 'citrate_settings',
  LAST_WALLET_ADDRESS: 'citrate_last_wallet_address',
} as const;

// Storage schema interface
export interface StorageSchema {
  [StorageKeys.VERSION]: number;
  [StorageKeys.THEME]: 'light' | 'dark' | 'system';
  [StorageKeys.CURRENT_TAB]: string;
  [StorageKeys.RECENT_ADDRESSES]: string[];
  [StorageKeys.WINDOW_SIZE]: { width: number; height: number };
  [StorageKeys.SETTINGS]: AppSettings;
  [StorageKeys.LAST_WALLET_ADDRESS]: string;
}

export interface AppSettings {
  autoStart: boolean;
  notifications: boolean;
  soundEnabled: boolean;
  language: string;
  currency: string;
  gasPrice: number;
  customBootnodes: string[];
}

// Default settings
const DEFAULT_SETTINGS: AppSettings = {
  autoStart: false,
  notifications: true,
  soundEnabled: true,
  language: 'en',
  currency: 'USD',
  gasPrice: 20,
  customBootnodes: [],
};

// Error types
export class StorageError extends Error {
  constructor(message: string) {
    super(message);
    this.name = 'StorageError';
  }
}

export class QuotaExceededError extends StorageError {
  constructor() {
    super('LocalStorage quota exceeded. Please clear some data.');
    this.name = 'QuotaExceededError';
  }
}

/**
 * Check if localStorage is available
 */
function isStorageAvailable(): boolean {
  try {
    const testKey = '__storage_test__';
    localStorage.setItem(testKey, 'test');
    localStorage.removeItem(testKey);
    return true;
  } catch {
    return false;
  }
}

/**
 * Get item from storage with type safety
 */
export function getStorageItem<K extends keyof StorageSchema>(
  key: K
): StorageSchema[K] | null {
  if (!isStorageAvailable()) {
    console.warn('LocalStorage not available');
    return null;
  }

  try {
    const item = localStorage.getItem(key);
    if (item === null) return null;

    // Parse JSON for non-string types
    return JSON.parse(item) as StorageSchema[K];
  } catch (error) {
    console.error(`Error reading from storage (${key}):`, error);
    return null;
  }
}

/**
 * Set item in storage with type safety
 */
export function setStorageItem<K extends keyof StorageSchema>(
  key: K,
  value: StorageSchema[K]
): boolean {
  if (!isStorageAvailable()) {
    console.warn('LocalStorage not available');
    return false;
  }

  try {
    const serialized = JSON.stringify(value);
    localStorage.setItem(key, serialized);
    return true;
  } catch (error) {
    if (error instanceof DOMException && error.name === 'QuotaExceededError') {
      console.error('Storage quota exceeded, attempting to clear old data...');
      clearOldData();

      // Retry once after clearing
      try {
        const serialized = JSON.stringify(value);
        localStorage.setItem(key, serialized);
        return true;
      } catch {
        throw new QuotaExceededError();
      }
    }

    console.error(`Error writing to storage (${key}):`, error);
    return false;
  }
}

/**
 * Remove item from storage
 */
export function removeStorageItem(key: keyof StorageSchema): boolean {
  if (!isStorageAvailable()) return false;

  try {
    localStorage.removeItem(key);
    return true;
  } catch (error) {
    console.error(`Error removing from storage (${key}):`, error);
    return false;
  }
}

/**
 * Clear all storage (use with caution)
 */
export function clearStorage(): boolean {
  if (!isStorageAvailable()) return false;

  try {
    // Get all citrate keys
    const citrateKeys = Object.keys(localStorage).filter(key =>
      key.startsWith('citrate_')
    );

    // Remove only citrate keys
    citrateKeys.forEach(key => localStorage.removeItem(key));

    return true;
  } catch (error) {
    console.error('Error clearing storage:', error);
    return false;
  }
}

/**
 * Clear old/unused data to free space
 */
export function clearOldData(): void {
  if (!isStorageAvailable()) return;

  try {
    // Remove any old migration keys or temporary data
    const keysToCheck = Object.keys(localStorage);

    keysToCheck.forEach(key => {
      // Remove old version keys
      if (key.includes('_old_') || key.includes('_temp_')) {
        localStorage.removeItem(key);
      }

      // Limit recent addresses to 10
      if (key === StorageKeys.RECENT_ADDRESSES) {
        const addresses = getStorageItem(StorageKeys.RECENT_ADDRESSES);
        if (addresses && addresses.length > 10) {
          setStorageItem(StorageKeys.RECENT_ADDRESSES, addresses.slice(0, 10));
        }
      }
    });
  } catch (error) {
    console.error('Error clearing old data:', error);
  }
}

/**
 * Add recent address with deduplication and max limit
 */
export function addRecentAddress(address: string): boolean {
  if (!address || !address.trim()) return false;

  const normalized = address.trim().toLowerCase();
  let recent = getStorageItem(StorageKeys.RECENT_ADDRESSES) || [];

  // Remove if already exists (to move to front)
  recent = recent.filter(addr => addr.toLowerCase() !== normalized);

  // Add to front
  recent.unshift(address.trim());

  // Limit to 10 addresses
  if (recent.length > 10) {
    recent = recent.slice(0, 10);
  }

  return setStorageItem(StorageKeys.RECENT_ADDRESSES, recent);
}

/**
 * Get recent addresses
 */
export function getRecentAddresses(): string[] {
  return getStorageItem(StorageKeys.RECENT_ADDRESSES) || [];
}

/**
 * Clear recent addresses
 */
export function clearRecentAddresses(): boolean {
  return setStorageItem(StorageKeys.RECENT_ADDRESSES, []);
}

/**
 * Export all settings to JSON string
 */
export function exportSettings(): string {
  const data: Partial<StorageSchema> = {};

  // Export relevant keys
  const keysToExport: (keyof StorageSchema)[] = [
    StorageKeys.THEME,
    StorageKeys.SETTINGS,
    StorageKeys.RECENT_ADDRESSES,
  ];

  keysToExport.forEach(key => {
    const value = getStorageItem(key);
    if (value !== null) {
      (data as any)[key] = value;
    }
  });

  // Add export metadata
  const exportData = {
    version: STORAGE_VERSION,
    exported_at: new Date().toISOString(),
    app: 'citrate',
    data,
  };

  return JSON.stringify(exportData, null, 2);
}

/**
 * Import settings from JSON string
 */
export function importSettings(jsonString: string): { success: boolean; error?: string; imported: number } {
  try {
    const parsed = JSON.parse(jsonString);

    // Validate structure
    if (!parsed.app || parsed.app !== 'citrate') {
      return { success: false, error: 'Invalid import file: not a Citrate export', imported: 0 };
    }

    if (!parsed.data || typeof parsed.data !== 'object') {
      return { success: false, error: 'Invalid import file: missing data', imported: 0 };
    }

    // Check version compatibility
    if (parsed.version > STORAGE_VERSION) {
      return {
        success: false,
        error: 'Import file from newer version. Please update Citrate.',
        imported: 0
      };
    }

    // Import data
    let imported = 0;
    const data = parsed.data as Partial<StorageSchema>;

    Object.entries(data).forEach(([key, value]) => {
      if (key in StorageKeys) {
        const success = setStorageItem(key as keyof StorageSchema, value as any);
        if (success) imported++;
      }
    });

    return { success: true, imported };
  } catch (error) {
    return {
      success: false,
      error: error instanceof Error ? error.message : 'Invalid JSON',
      imported: 0
    };
  }
}

/**
 * Migrate storage to new version
 */
export function migrateStorage(): void {
  const currentVersion = getStorageItem(StorageKeys.VERSION) || 0;

  if (currentVersion === STORAGE_VERSION) {
    // Already up to date
    return;
  }

  console.log(`Migrating storage from v${currentVersion} to v${STORAGE_VERSION}`);

  // Perform migrations based on version
  if (currentVersion < 1) {
    // Migration from v0 to v1
    // Example: Rename old keys, restructure data, etc.

    // Set new version
    setStorageItem(StorageKeys.VERSION, STORAGE_VERSION);
  }

  // Future migrations go here
  // if (currentVersion < 2) { ... }

  console.log('Storage migration complete');
}

/**
 * Initialize storage (run on app start)
 */
export function initializeStorage(): void {
  if (!isStorageAvailable()) {
    console.warn('LocalStorage not available - app will run without persistence');
    return;
  }

  // Run migrations
  migrateStorage();

  // Initialize default settings if not present
  const settings = getStorageItem(StorageKeys.SETTINGS);
  if (!settings) {
    setStorageItem(StorageKeys.SETTINGS, DEFAULT_SETTINGS);
  }

  // Initialize empty recent addresses if not present
  const recent = getStorageItem(StorageKeys.RECENT_ADDRESSES);
  if (!recent) {
    setStorageItem(StorageKeys.RECENT_ADDRESSES, []);
  }

  // Clean up old data
  clearOldData();

  console.log('Storage initialized successfully');
}

/**
 * Get storage usage info
 */
export function getStorageInfo(): {
  available: boolean;
  used: number;
  total: number;
  percentage: number;
} {
  if (!isStorageAvailable()) {
    return { available: false, used: 0, total: 0, percentage: 0 };
  }

  try {
    // Estimate storage size
    let used = 0;
    Object.keys(localStorage).forEach(key => {
      if (key.startsWith('citrate_')) {
        const value = localStorage.getItem(key);
        used += key.length + (value?.length || 0);
      }
    });

    // Most browsers allow 5-10MB for localStorage
    // We'll use 5MB as a conservative estimate
    const total = 5 * 1024 * 1024; // 5MB in bytes
    const percentage = (used / total) * 100;

    return {
      available: true,
      used,
      total,
      percentage: Math.min(percentage, 100),
    };
  } catch {
    return { available: true, used: 0, total: 0, percentage: 0 };
  }
}

/**
 * Get default settings
 */
export function getDefaultSettings(): AppSettings {
  return { ...DEFAULT_SETTINGS };
}

export default {
  getStorageItem,
  setStorageItem,
  removeStorageItem,
  clearStorage,
  addRecentAddress,
  getRecentAddresses,
  clearRecentAddresses,
  exportSettings,
  importSettings,
  migrateStorage,
  initializeStorage,
  getStorageInfo,
  getDefaultSettings,
  clearOldData,
};
