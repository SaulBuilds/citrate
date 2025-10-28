# Sprint 2: User Stories - Detailed Breakdown

---

## Story 1: State Persistence

**Story ID:** S2-01
**Priority:** P0 (Critical - User Experience)
**Story Points:** 5
**Assignee:** TBD

### User Story
```
As a user
I want my preferences and app state to persist across sessions
So that I don't have to reconfigure the app every time I open it
```

### Current State (Problem)
**Issues:**
- Theme preference resets to light mode on restart
- Last viewed tab not remembered
- Window size/position not saved (Tauri app)
- Recent addresses lost on app close
- Settings reset to defaults
- No import/export functionality for settings

**User Impact:** Users must reconfigure preferences every session, leading to frustration and wasted time.

### Desired State (Solution)
```typescript
// Automatic persistence
- Theme saved: localStorage.setItem('theme', 'dark')
- Tab restored: localStorage.getItem('lastTab') → 'wallet'
- Settings synced: localStorage.setItem('settings', JSON.stringify(settings))
- Recent addresses: localStorage.setItem('recentAddresses', JSON.stringify(addresses))
```

---

### Acceptance Criteria

#### AC1: Theme Persistence
- [ ] Theme preference saved to localStorage on change
- [ ] Theme restored on app start
- [ ] Default to 'light' if no preference saved
- [ ] No flicker during theme load
- [ ] Works with system theme detection

#### AC2: Tab State Persistence
- [ ] Last viewed tab saved on navigation
- [ ] Tab restored on app start
- [ ] Valid tab names only (dashboard/wallet/dag/models/settings)
- [ ] Fallback to 'dashboard' if invalid tab saved
- [ ] Deep linking works with persisted tabs

#### AC3: Window State Persistence (Tauri)
- [ ] Window size saved on resize
- [ ] Window position saved on move
- [ ] Window restored to last position/size
- [ ] Handles multi-monitor setups
- [ ] Respects minimum window size constraints

#### AC4: Recent Addresses Storage
- [ ] Recent recipient addresses saved (max 10)
- [ ] Addresses displayed in dropdown/autocomplete
- [ ] Invalid addresses automatically removed
- [ ] Clear history option in Settings
- [ ] Addresses persisted across sessions

#### AC5: Settings Auto-Save
- [ ] All settings saved immediately on change
- [ ] No manual "Save" button required
- [ ] Settings validated before saving
- [ ] Error handling for quota exceeded
- [ ] Settings versioning for migrations

#### AC6: Import/Export Settings
- [ ] Export settings to JSON file
- [ ] Import settings from JSON file
- [ ] Settings validation on import
- [ ] Confirmation dialog before import
- [ ] Include version number in export

---

### Technical Tasks

#### Task 1.1: Create Storage Utility Module
**Estimated Time:** 2 hours

**Create file:** `gui/citrate-core/src/utils/storage.ts`

```typescript
/**
 * Centralized storage utility for Citrate GUI
 * Provides type-safe localStorage access with error handling
 */

export interface StorageSchema {
  // Theme
  theme: 'light' | 'dark' | 'system';

  // App State
  lastTab: 'dashboard' | 'wallet' | 'dag' | 'models' | 'settings';

  // Window State (Tauri)
  windowState: {
    width: number;
    height: number;
    x: number;
    y: number;
  };

  // Recent Addresses
  recentAddresses: string[];

  // Settings
  settings: {
    rpcUrl: string;
    bootnodes: string[];
    autoConnect: boolean;
    notifications: boolean;
    language: string;
  };

  // Version for migrations
  storageVersion: number;
}

const STORAGE_VERSION = 1;
const MAX_RECENT_ADDRESSES = 10;

/**
 * Get item from localStorage with type safety
 */
export function getStorageItem<K extends keyof StorageSchema>(
  key: K
): StorageSchema[K] | null {
  try {
    const item = localStorage.getItem(key);
    if (item === null) return null;

    return JSON.parse(item) as StorageSchema[K];
  } catch (error) {
    console.error(`Error reading ${key} from storage:`, error);
    return null;
  }
}

/**
 * Set item in localStorage with type safety
 */
export function setStorageItem<K extends keyof StorageSchema>(
  key: K,
  value: StorageSchema[K]
): boolean {
  try {
    localStorage.setItem(key, JSON.stringify(value));
    return true;
  } catch (error) {
    console.error(`Error writing ${key} to storage:`, error);

    // Handle quota exceeded
    if (error instanceof DOMException && error.name === 'QuotaExceededError') {
      console.warn('localStorage quota exceeded, clearing old data');
      clearOldData();

      // Retry
      try {
        localStorage.setItem(key, JSON.stringify(value));
        return true;
      } catch (retryError) {
        console.error('Retry failed:', retryError);
        return false;
      }
    }

    return false;
  }
}

/**
 * Remove item from localStorage
 */
export function removeStorageItem(key: keyof StorageSchema): void {
  try {
    localStorage.removeItem(key);
  } catch (error) {
    console.error(`Error removing ${key} from storage:`, error);
  }
}

/**
 * Clear all storage (use with caution)
 */
export function clearStorage(): void {
  try {
    localStorage.clear();
  } catch (error) {
    console.error('Error clearing storage:', error);
  }
}

/**
 * Add address to recent addresses list
 */
export function addRecentAddress(address: string): void {
  const recent = getStorageItem('recentAddresses') || [];

  // Remove duplicates
  const filtered = recent.filter(addr => addr.toLowerCase() !== address.toLowerCase());

  // Add to beginning
  const updated = [address, ...filtered].slice(0, MAX_RECENT_ADDRESSES);

  setStorageItem('recentAddresses', updated);
}

/**
 * Clear recent addresses
 */
export function clearRecentAddresses(): void {
  setStorageItem('recentAddresses', []);
}

/**
 * Export all settings to JSON
 */
export function exportSettings(): string {
  const settings = getStorageItem('settings');
  const theme = getStorageItem('theme');

  const exportData = {
    version: STORAGE_VERSION,
    exportDate: new Date().toISOString(),
    theme,
    settings
  };

  return JSON.stringify(exportData, null, 2);
}

/**
 * Import settings from JSON
 */
export function importSettings(jsonString: string): boolean {
  try {
    const data = JSON.parse(jsonString);

    // Validate version
    if (data.version !== STORAGE_VERSION) {
      console.warn('Settings version mismatch, may need migration');
    }

    // Validate and import theme
    if (data.theme && ['light', 'dark', 'system'].includes(data.theme)) {
      setStorageItem('theme', data.theme);
    }

    // Validate and import settings
    if (data.settings) {
      // Add validation logic here
      setStorageItem('settings', data.settings);
    }

    return true;
  } catch (error) {
    console.error('Error importing settings:', error);
    return false;
  }
}

/**
 * Migrate storage from old version to new version
 */
export function migrateStorage(oldVersion: number): void {
  console.log(`Migrating storage from v${oldVersion} to v${STORAGE_VERSION}`);

  // Add migration logic as needed
  // For now, just update version
  setStorageItem('storageVersion', STORAGE_VERSION);
}

/**
 * Clear old data to free up space
 */
function clearOldData(): void {
  // Remove non-essential data
  removeStorageItem('recentAddresses');

  console.log('Cleared old data to free up space');
}

/**
 * Check storage version and migrate if needed
 */
export function initStorage(): void {
  const version = getStorageItem('storageVersion');

  if (version === null) {
    // First time, set version
    setStorageItem('storageVersion', STORAGE_VERSION);
  } else if (version < STORAGE_VERSION) {
    // Migrate
    migrateStorage(version);
  }
}
```

**Files Created:**
- `gui/citrate-core/src/utils/storage.ts`

**Acceptance:**
- [ ] All functions properly typed
- [ ] Error handling for quota exceeded
- [ ] Recent addresses capped at 10
- [ ] Import/export validation
- [ ] Storage versioning for migrations

---

#### Task 1.2: Create AppContext for Global State
**Estimated Time:** 2 hours

**Create file:** `gui/citrate-core/src/contexts/AppContext.tsx`

```typescript
import React, { createContext, useContext, useState, useEffect, ReactNode } from 'react';
import { getStorageItem, setStorageItem, initStorage } from '../utils/storage';

interface AppState {
  // Current tab
  currentTab: 'dashboard' | 'wallet' | 'dag' | 'models' | 'settings';
  setCurrentTab: (tab: AppState['currentTab']) => void;

  // Recent addresses
  recentAddresses: string[];
  addRecentAddress: (address: string) => void;
  clearRecentAddresses: () => void;

  // Loading states
  isLoading: boolean;
  setIsLoading: (loading: boolean) => void;

  // Settings
  settings: {
    rpcUrl: string;
    bootnodes: string[];
    autoConnect: boolean;
    notifications: boolean;
    language: string;
  };
  updateSettings: (settings: Partial<AppState['settings']>) => void;
}

const defaultSettings = {
  rpcUrl: 'http://localhost:8545',
  bootnodes: [],
  autoConnect: true,
  notifications: true,
  language: 'en'
};

const AppContext = createContext<AppState | undefined>(undefined);

export const useApp = () => {
  const context = useContext(AppContext);
  if (!context) {
    throw new Error('useApp must be used within AppProvider');
  }
  return context;
};

interface AppProviderProps {
  children: ReactNode;
}

export const AppProvider: React.FC<AppProviderProps> = ({ children }) => {
  // Initialize storage
  useEffect(() => {
    initStorage();
  }, []);

  // Load persisted tab
  const [currentTab, setCurrentTabState] = useState<AppState['currentTab']>(() => {
    return getStorageItem('lastTab') || 'dashboard';
  });

  // Load recent addresses
  const [recentAddresses, setRecentAddresses] = useState<string[]>(() => {
    return getStorageItem('recentAddresses') || [];
  });

  // Load settings
  const [settings, setSettings] = useState<AppState['settings']>(() => {
    return getStorageItem('settings') || defaultSettings;
  });

  // Loading state
  const [isLoading, setIsLoading] = useState(false);

  // Persist tab on change
  const setCurrentTab = (tab: AppState['currentTab']) => {
    setCurrentTabState(tab);
    setStorageItem('lastTab', tab);
  };

  // Add recent address
  const addRecentAddress = (address: string) => {
    const updated = [address, ...recentAddresses.filter(a => a !== address)].slice(0, 10);
    setRecentAddresses(updated);
    setStorageItem('recentAddresses', updated);
  };

  // Clear recent addresses
  const clearRecentAddresses = () => {
    setRecentAddresses([]);
    setStorageItem('recentAddresses', []);
  };

  // Update settings
  const updateSettings = (newSettings: Partial<AppState['settings']>) => {
    const updated = { ...settings, ...newSettings };
    setSettings(updated);
    setStorageItem('settings', updated);
  };

  const value: AppState = {
    currentTab,
    setCurrentTab,
    recentAddresses,
    addRecentAddress,
    clearRecentAddresses,
    isLoading,
    setIsLoading,
    settings,
    updateSettings
  };

  return <AppContext.Provider value={value}>{children}</AppContext.Provider>;
};
```

**Files Created:**
- `gui/citrate-core/src/contexts/AppContext.tsx`

---

#### Task 1.3: Update App.tsx to Use AppContext
**Estimated Time:** 1.5 hours

**File:** `gui/citrate-core/src/App.tsx`

**Changes:**

1. Import AppProvider:
```typescript
import { AppProvider, useApp } from './contexts/AppContext';
```

2. Wrap app with AppProvider:
```typescript
function App() {
  return (
    <ErrorBoundary>
      <AppProvider>
        <AppContent />
      </AppProvider>
    </ErrorBoundary>
  );
}

function AppContent() {
  const { currentTab, setCurrentTab } = useApp();

  // Use context instead of local state
  // ...
}
```

3. Replace currentView state with context
4. Tab navigation automatically persisted

**Files Changed:**
- `gui/citrate-core/src/App.tsx`

---

#### Task 1.4: Add Recent Addresses to Wallet
**Estimated Time:** 0.5 hours

**File:** `gui/citrate-core/src/components/Wallet.tsx`

**Changes:**

1. Import context:
```typescript
import { useApp } from '../contexts/AppContext';
```

2. Add autocomplete to recipient field:
```typescript
const { recentAddresses, addRecentAddress } = useApp();

<div className="relative">
  <input
    type="text"
    value={recipient}
    onChange={(e) => setRecipient(e.target.value)}
    list="recent-addresses"
    placeholder="0x... or select recent"
  />
  <datalist id="recent-addresses">
    {recentAddresses.map(addr => (
      <option key={addr} value={addr} />
    ))}
  </datalist>
</div>
```

3. Save address on successful transaction:
```typescript
const handleSendTransaction = async () => {
  // ... existing logic

  if (success) {
    addRecentAddress(recipient);
  }
};
```

**Files Changed:**
- `gui/citrate-core/src/components/Wallet.tsx`

---

### Testing Plan

#### Unit Tests

**File:** `gui/citrate-core/src/utils/storage.test.ts`

```typescript
import { describe, it, expect, beforeEach } from 'vitest';
import {
  getStorageItem,
  setStorageItem,
  addRecentAddress,
  exportSettings,
  importSettings
} from './storage';

describe('Storage Utility', () => {
  beforeEach(() => {
    localStorage.clear();
  });

  describe('getStorageItem / setStorageItem', () => {
    it('should store and retrieve theme', () => {
      setStorageItem('theme', 'dark');
      expect(getStorageItem('theme')).toBe('dark');
    });

    it('should return null for non-existent keys', () => {
      expect(getStorageItem('theme')).toBeNull();
    });

    it('should handle objects', () => {
      const settings = { rpcUrl: 'http://localhost:8545', bootnodes: [] };
      setStorageItem('settings', settings);
      expect(getStorageItem('settings')).toEqual(settings);
    });
  });

  describe('addRecentAddress', () => {
    it('should add address to list', () => {
      addRecentAddress('0x1234');
      const addresses = getStorageItem('recentAddresses');
      expect(addresses).toContain('0x1234');
    });

    it('should limit to 10 addresses', () => {
      for (let i = 0; i < 15; i++) {
        addRecentAddress(`0x${i}`);
      }
      const addresses = getStorageItem('recentAddresses');
      expect(addresses?.length).toBe(10);
    });

    it('should not duplicate addresses', () => {
      addRecentAddress('0x1234');
      addRecentAddress('0x1234');
      const addresses = getStorageItem('recentAddresses');
      expect(addresses?.filter(a => a === '0x1234').length).toBe(1);
    });
  });

  describe('exportSettings / importSettings', () => {
    it('should export settings as JSON', () => {
      setStorageItem('theme', 'dark');
      setStorageItem('settings', { rpcUrl: 'test', bootnodes: [] });

      const json = exportSettings();
      expect(json).toContain('"theme":"dark"');
      expect(json).toContain('"rpcUrl":"test"');
    });

    it('should import valid settings', () => {
      const json = JSON.stringify({
        version: 1,
        theme: 'dark',
        settings: { rpcUrl: 'imported', bootnodes: [] }
      });

      const success = importSettings(json);
      expect(success).toBe(true);
      expect(getStorageItem('theme')).toBe('dark');
    });

    it('should reject invalid JSON', () => {
      const success = importSettings('invalid json');
      expect(success).toBe(false);
    });
  });
});
```

#### Integration Tests

```typescript
describe('AppContext Integration', () => {
  it('should persist tab across remount', () => {
    const { unmount, rerender } = render(
      <AppProvider>
        <TestComponent />
      </AppProvider>
    );

    // Change tab
    fireEvent.click(screen.getByText('Wallet'));
    expect(getStorageItem('lastTab')).toBe('wallet');

    // Unmount and remount
    unmount();
    rerender(
      <AppProvider>
        <TestComponent />
      </AppProvider>
    );

    // Should restore wallet tab
    expect(screen.getByText('Wallet')).toHaveClass('active');
  });
});
```

#### Manual Testing Checklist
- [ ] Theme persists after app restart
- [ ] Last tab restored on launch
- [ ] Window size/position saved (Tauri)
- [ ] Recent addresses appear in dropdown
- [ ] Settings auto-save on change
- [ ] Export settings creates valid JSON
- [ ] Import settings updates UI
- [ ] Invalid import shows error
- [ ] Storage quota exceeded handled gracefully
- [ ] Works in both Tauri and web builds

---

### Definition of Done
- [ ] All acceptance criteria met
- [ ] Storage utility fully tested
- [ ] AppContext integrated into App.tsx
- [ ] Recent addresses working in Wallet
- [ ] Import/export settings functional
- [ ] Unit tests passing (>90% coverage)
- [ ] Integration tests passing
- [ ] Manual testing completed
- [ ] No console errors
- [ ] Code reviewed

---

## Story 2: Dark Mode Theme

**Story ID:** S2-02
**Priority:** P1 (High - User Experience)
**Story Points:** 3
**Assignee:** TBD

### User Story
```
As a user
I want to switch between dark and light themes
So that I can use the app comfortably in different lighting conditions
```

### Current State (Problem)
**File:** `gui/citrate-core/src/App.css`
**Issue:** Only light theme implemented with hardcoded colors

**Problems:**
- Hard-coded color values throughout CSS
- No theme system or CSS variables
- Eye strain in low-light environments
- No system theme detection
- Orange (#ffa500) brand color not optimized for dark backgrounds

### Desired State (Solution)
```css
/* CSS Variables for theming */
:root[data-theme="light"] {
  --bg-primary: #ffffff;
  --text-primary: #1a1a1a;
  --brand-primary: #ffa500;
}

:root[data-theme="dark"] {
  --bg-primary: #1a1a1a;
  --text-primary: #ffffff;
  --brand-primary: #ffb84d; /* Lighter orange for dark mode */
}
```

---

### Acceptance Criteria

#### AC1: Theme System Architecture
- [ ] CSS variables for all colors
- [ ] ThemeContext for React state
- [ ] Theme persistence in localStorage
- [ ] No hardcoded colors in components
- [ ] TypeScript types for theme values

#### AC2: Theme Toggle UI
- [ ] Toggle switch in Settings component
- [ ] Visual indicator of current theme
- [ ] Smooth transition between themes (CSS transitions)
- [ ] No flicker or flash on theme change
- [ ] Icon changes (Sun/Moon)

#### AC3: Dark Theme Design
- [ ] All components support dark theme
- [ ] High contrast ratios (WCAG AA)
- [ ] Orange brand color adjusted for dark backgrounds
- [ ] Borders and shadows adjusted
- [ ] Input fields readable in both themes

#### AC4: System Theme Detection
- [ ] Detect system preference on first launch
- [ ] `prefers-color-scheme` media query
- [ ] "System" option in theme selector
- [ ] Auto-switch with system changes (optional)

#### AC5: Theme Transitions
- [ ] Smooth color transitions (200ms ease)
- [ ] No jarring layout shifts
- [ ] Images/icons adapt to theme
- [ ] Loading skeletons match theme

---

### Technical Tasks

#### Task 2.1: Create Theme Definitions
**Estimated Time:** 1.5 hours

**Create file:** `gui/citrate-core/src/styles/themes.ts`

```typescript
/**
 * Theme definitions for Citrate GUI
 */

export interface Theme {
  name: string;
  colors: {
    // Backgrounds
    bgPrimary: string;
    bgSecondary: string;
    bgTertiary: string;

    // Text
    textPrimary: string;
    textSecondary: string;
    textTertiary: string;

    // Brand
    brandPrimary: string;
    brandSecondary: string;
    brandHover: string;

    // Status
    success: string;
    warning: string;
    error: string;
    info: string;

    // Borders
    borderPrimary: string;
    borderSecondary: string;

    // Shadows
    shadowLight: string;
    shadowMedium: string;
    shadowHeavy: string;
  };
}

export const lightTheme: Theme = {
  name: 'light',
  colors: {
    // Backgrounds
    bgPrimary: '#ffffff',
    bgSecondary: '#f9fafb',
    bgTertiary: '#f3f4f6',

    // Text
    textPrimary: '#1a1a1a',
    textSecondary: '#4b5563',
    textTertiary: '#9ca3af',

    // Brand
    brandPrimary: '#ffa500',
    brandSecondary: '#ff8c00',
    brandHover: '#ff9500',

    // Status
    success: '#10b981',
    warning: '#f59e0b',
    error: '#ef4444',
    info: '#3b82f6',

    // Borders
    borderPrimary: '#e5e7eb',
    borderSecondary: '#d1d5db',

    // Shadows
    shadowLight: 'rgba(0, 0, 0, 0.05)',
    shadowMedium: 'rgba(0, 0, 0, 0.1)',
    shadowHeavy: 'rgba(0, 0, 0, 0.2)'
  }
};

export const darkTheme: Theme = {
  name: 'dark',
  colors: {
    // Backgrounds
    bgPrimary: '#1a1a1a',
    bgSecondary: '#242424',
    bgTertiary: '#2e2e2e',

    // Text
    textPrimary: '#ffffff',
    textSecondary: '#d1d5db',
    textTertiary: '#9ca3af',

    // Brand
    brandPrimary: '#ffb84d',  // Lighter for dark backgrounds
    brandSecondary: '#ffa500',
    brandHover: '#ffc266',

    // Status
    success: '#34d399',
    warning: '#fbbf24',
    error: '#f87171',
    info: '#60a5fa',

    // Borders
    borderPrimary: '#374151',
    borderSecondary: '#4b5563',

    // Shadows
    shadowLight: 'rgba(0, 0, 0, 0.3)',
    shadowMedium: 'rgba(0, 0, 0, 0.5)',
    shadowHeavy: 'rgba(0, 0, 0, 0.7)'
  }
};

export const themes = {
  light: lightTheme,
  dark: darkTheme
};

export type ThemeMode = 'light' | 'dark' | 'system';

/**
 * Apply theme to document
 */
export function applyTheme(theme: Theme): void {
  const root = document.documentElement;

  Object.entries(theme.colors).forEach(([key, value]) => {
    // Convert camelCase to kebab-case for CSS variables
    const cssVar = `--${key.replace(/([A-Z])/g, '-$1').toLowerCase()}`;
    root.style.setProperty(cssVar, value);
  });

  root.setAttribute('data-theme', theme.name);
}

/**
 * Detect system theme preference
 */
export function getSystemTheme(): 'light' | 'dark' {
  if (window.matchMedia && window.matchMedia('(prefers-color-scheme: dark)').matches) {
    return 'dark';
  }
  return 'light';
}
```

**Files Created:**
- `gui/citrate-core/src/styles/themes.ts`

---

#### Task 2.2: Create ThemeContext
**Estimated Time:** 1.5 hours

**Create file:** `gui/citrate-core/src/contexts/ThemeContext.tsx`

```typescript
import React, { createContext, useContext, useState, useEffect, ReactNode } from 'react';
import { Theme, ThemeMode, themes, applyTheme, getSystemTheme } from '../styles/themes';
import { getStorageItem, setStorageItem } from '../utils/storage';

interface ThemeContextType {
  theme: Theme;
  themeMode: ThemeMode;
  setThemeMode: (mode: ThemeMode) => void;
  toggleTheme: () => void;
}

const ThemeContext = createContext<ThemeContextType | undefined>(undefined);

export const useTheme = () => {
  const context = useContext(ThemeContext);
  if (!context) {
    throw new Error('useTheme must be used within ThemeProvider');
  }
  return context;
};

interface ThemeProviderProps {
  children: ReactNode;
}

export const ThemeProvider: React.FC<ThemeProviderProps> = ({ children }) => {
  // Load theme preference
  const [themeMode, setThemeModeState] = useState<ThemeMode>(() => {
    return getStorageItem('theme') || 'system';
  });

  // Resolve actual theme
  const [theme, setTheme] = useState<Theme>(() => {
    const mode = getStorageItem('theme') || 'system';
    const resolved = mode === 'system' ? getSystemTheme() : mode;
    return themes[resolved];
  });

  // Apply theme on mount and when it changes
  useEffect(() => {
    applyTheme(theme);
  }, [theme]);

  // Update theme when mode changes
  useEffect(() => {
    const resolved = themeMode === 'system' ? getSystemTheme() : themeMode;
    setTheme(themes[resolved]);
  }, [themeMode]);

  // Listen for system theme changes
  useEffect(() => {
    if (themeMode !== 'system') return;

    const mediaQuery = window.matchMedia('(prefers-color-scheme: dark)');

    const handleChange = (e: MediaQueryListEvent) => {
      const systemTheme = e.matches ? 'dark' : 'light';
      setTheme(themes[systemTheme]);
    };

    mediaQuery.addEventListener('change', handleChange);

    return () => {
      mediaQuery.removeEventListener('change', handleChange);
    };
  }, [themeMode]);

  const setThemeMode = (mode: ThemeMode) => {
    setThemeModeState(mode);
    setStorageItem('theme', mode);
  };

  const toggleTheme = () => {
    const current = theme.name;
    const next = current === 'light' ? 'dark' : 'light';
    setThemeMode(next as ThemeMode);
  };

  const value: ThemeContextType = {
    theme,
    themeMode,
    setThemeMode,
    toggleTheme
  };

  return <ThemeContext.Provider value={value}>{children}</ThemeContext.Provider>;
};
```

**Files Created:**
- `gui/citrate-core/src/contexts/ThemeContext.tsx`

---

#### Task 2.3: Update CSS with Variables
**Estimated Time:** 2 hours

**File:** `gui/citrate-core/src/App.css`

**Changes:** Replace all hardcoded colors with CSS variables

```css
/* Theme CSS Variables (set by ThemeContext) */
:root {
  /* Variables will be injected by JavaScript */
  /* Default to light theme */
  --bg-primary: #ffffff;
  --bg-secondary: #f9fafb;
  --text-primary: #1a1a1a;
  --brand-primary: #ffa500;
  /* ... etc */
}

/* Smooth theme transitions */
* {
  transition: background-color 200ms ease, color 200ms ease, border-color 200ms ease;
}

/* Base styles using variables */
body {
  background-color: var(--bg-primary);
  color: var(--text-primary);
}

.button-primary {
  background-color: var(--brand-primary);
  color: var(--text-primary);
}

.button-primary:hover {
  background-color: var(--brand-hover);
}

/* Cards */
.card {
  background-color: var(--bg-secondary);
  border: 1px solid var(--border-primary);
  box-shadow: 0 2px 4px var(--shadow-light);
}

/* Inputs */
input, textarea, select {
  background-color: var(--bg-primary);
  color: var(--text-primary);
  border: 1px solid var(--border-primary);
}

input:focus {
  border-color: var(--brand-primary);
}

/* ... update all components to use variables ... */
```

**Files Changed:**
- `gui/citrate-core/src/App.css`
- Update all component stylesheets

---

#### Task 2.4: Add Theme Toggle to Settings
**Estimated Time:** 1 hour

**File:** `gui/citrate-core/src/components/Settings.tsx`

**Changes:**

```typescript
import { useTheme } from '../contexts/ThemeContext';
import { Sun, Moon, Monitor } from 'lucide-react';

export const Settings = () => {
  const { themeMode, setThemeMode } = useTheme();

  return (
    <div className="settings">
      <h2>Appearance</h2>

      <div className="setting-group">
        <label>Theme</label>
        <div className="theme-selector">
          <button
            onClick={() => setThemeMode('light')}
            className={themeMode === 'light' ? 'active' : ''}
          >
            <Sun size={20} />
            Light
          </button>

          <button
            onClick={() => setThemeMode('dark')}
            className={themeMode === 'dark' ? 'active' : ''}
          >
            <Moon size={20} />
            Dark
          </button>

          <button
            onClick={() => setThemeMode('system')}
            className={themeMode === 'system' ? 'active' : ''}
          >
            <Monitor size={20} />
            System
          </button>
        </div>
      </div>
    </div>
  );
};
```

**Files Changed:**
- `gui/citrate-core/src/components/Settings.tsx`

---

### Testing Plan

#### Visual Testing Checklist
- [ ] Dashboard looks good in both themes
- [ ] Wallet component readable in dark mode
- [ ] DAG visualization clear in both themes
- [ ] Models cards styled correctly
- [ ] Settings panel theme-aware
- [ ] Orange brand color visible in both themes
- [ ] All text has sufficient contrast (WCAG AA)
- [ ] Borders visible in both themes
- [ ] Shadows appropriate for each theme

#### Functional Testing
- [ ] Theme toggle switches immediately
- [ ] No flicker during theme change
- [ ] Theme persists across restart
- [ ] System theme detection works
- [ ] System theme changes trigger update (when set to "system")
- [ ] All icons change appropriately
- [ ] Loading skeletons match theme

---

### Definition of Done
- [ ] All acceptance criteria met
- [ ] CSS variables for all colors
- [ ] ThemeContext fully functional
- [ ] Theme toggle in Settings
- [ ] System theme detection working
- [ ] All components support both themes
- [ ] No hardcoded colors remaining
- [ ] Smooth transitions
- [ ] WCAG AA contrast compliance
- [ ] Manual testing completed
- [ ] Code reviewed

---

## Story 3: Performance Optimization

**Story ID:** S2-03
**Priority:** P1 (High - User Experience)
**Story Points:** 3
**Assignee:** TBD

### User Story
```
As a user
I want smooth, responsive performance even with large datasets
So that the app remains usable during heavy operations
```

### Current State (Problem)
**Performance Issues:**
- Wallet activity list re-renders entire list (1000+ items)
- DAG visualization lags with 500+ blocks
- Search inputs trigger immediate re-renders
- Dashboard stats recalculate on every render
- No code splitting or lazy loading

**Measured Performance (Baseline):**
- Dashboard render: ~150ms
- Wallet activity (100 items): ~80ms
- DAG visualization (500 blocks): ~300ms
- Bundle size: 2.5MB

### Desired State (Solution)
**Target Performance:**
- Dashboard render: <100ms (33% improvement)
- Wallet activity (1000 items): <100ms (virtual scrolling)
- DAG visualization (1000 blocks): <200ms (lazy loading)
- Bundle size: <2.2MB (code splitting)
- 60fps maintained during all interactions

---

### Acceptance Criteria

#### AC1: Virtual Scrolling
- [ ] VirtualList component created
- [ ] Wallet activity uses virtual scrolling
- [ ] Only visible items rendered
- [ ] Smooth scrolling maintained
- [ ] 1000+ items without lag

#### AC2: Lazy Loading
- [ ] DAG blocks loaded incrementally
- [ ] "Load More" or infinite scroll
- [ ] Skeleton loaders during load
- [ ] No full-dataset renders

#### AC3: Optimized Calculations
- [ ] React.memo on expensive components
- [ ] useMemo for heavy calculations
- [ ] useCallback for event handlers
- [ ] Prevent unnecessary re-renders

#### AC4: Debounced Inputs
- [ ] Search inputs debounced (300ms)
- [ ] Filter operations debounced
- [ ] Immediate visual feedback
- [ ] Cancel pending operations

#### AC5: Web Workers (Optional)
- [ ] DAG calculations in Web Worker
- [ ] Blue set computation offloaded
- [ ] Non-blocking UI thread
- [ ] Fallback if Workers unavailable

#### AC6: Performance Metrics
- [ ] 60fps during scrolling
- [ ] <100ms interaction response
- [ ] <200ms page transitions
- [ ] Lighthouse score >90

---

### Technical Tasks

#### Task 3.1: Create VirtualList Component
**Estimated Time:** 2 hours

**Create file:** `gui/citrate-core/src/components/VirtualList.tsx`

```typescript
import React, { useRef, useState, useEffect, ReactNode } from 'react';

interface VirtualListProps<T> {
  items: T[];
  height: number;  // Container height in px
  itemHeight: number;  // Each item height in px
  renderItem: (item: T, index: number) => ReactNode;
  overscan?: number;  // Extra items to render above/below viewport
}

export function VirtualList<T>({
  items,
  height,
  itemHeight,
  renderItem,
  overscan = 3
}: VirtualListProps<T>) {
  const [scrollTop, setScrollTop] = useState(0);
  const containerRef = useRef<HTMLDivElement>(null);

  const totalHeight = items.length * itemHeight;
  const visibleCount = Math.ceil(height / itemHeight);

  const startIndex = Math.max(0, Math.floor(scrollTop / itemHeight) - overscan);
  const endIndex = Math.min(
    items.length - 1,
    Math.floor((scrollTop + height) / itemHeight) + overscan
  );

  const visibleItems = items.slice(startIndex, endIndex + 1);

  const offsetY = startIndex * itemHeight;

  const handleScroll = (e: React.UIEvent<HTMLDivElement>) => {
    setScrollTop(e.currentTarget.scrollTop);
  };

  return (
    <div
      ref={containerRef}
      onScroll={handleScroll}
      style={{
        height: `${height}px`,
        overflow: 'auto',
        position: 'relative'
      }}
    >
      <div style={{ height: `${totalHeight}px`, position: 'relative' }}>
        <div style={{ transform: `translateY(${offsetY}px)` }}>
          {visibleItems.map((item, index) =>
            renderItem(item, startIndex + index)
          )}
        </div>
      </div>
    </div>
  );
}
```

**Alternative:** Use `react-window` or `react-virtuoso` library

```bash
npm install react-window
npm install --save-dev @types/react-window
```

```typescript
import { FixedSizeList } from 'react-window';

<FixedSizeList
  height={600}
  itemCount={items.length}
  itemSize={60}
  width="100%"
>
  {({ index, style }) => (
    <div style={style}>
      {renderItem(items[index], index)}
    </div>
  )}
</FixedSizeList>
```

**Files Created:**
- `gui/citrate-core/src/components/VirtualList.tsx`

---

#### Task 3.2: Add Virtual Scrolling to Wallet
**Estimated Time:** 1.5 hours

**File:** `gui/citrate-core/src/components/Wallet.tsx`

**Changes:**

```typescript
import { VirtualList } from './VirtualList';
// or
import { FixedSizeList } from 'react-window';

// Replace flat list rendering
<VirtualList
  items={transactions}
  height={600}
  itemHeight={60}
  renderItem={(tx, index) => (
    <div key={tx.hash} className="transaction-item">
      <div className="tx-hash">{tx.hash.slice(0, 10)}...</div>
      <div className="tx-amount">{tx.amount} LATT</div>
      <div className="tx-status">{tx.status}</div>
    </div>
  )}
/>
```

**Files Changed:**
- `gui/citrate-core/src/components/Wallet.tsx`

---

#### Task 3.3: Optimize Dashboard with React.memo
**Estimated Time:** 1 hour

**File:** `gui/citrate-core/src/components/Dashboard.tsx`

**Changes:**

```typescript
import React, { memo, useMemo, useCallback } from 'react';

// Memoize stat cards
const StatCard = memo(({ title, value, icon }: StatCardProps) => {
  return (
    <div className="stat-card">
      {icon}
      <h3>{title}</h3>
      <p>{value}</p>
    </div>
  );
});

export const Dashboard = () => {
  const [nodeStatus, setNodeStatus] = useState<NodeStatus | null>(null);
  const [blocks, setBlocks] = useState<Block[]>([]);

  // Memoize expensive calculations
  const stats = useMemo(() => {
    return {
      totalBlocks: blocks.length,
      avgBlockTime: calculateAvgBlockTime(blocks),
      networkHashRate: calculateHashRate(blocks),
      activeNodes: nodeStatus?.peerCount || 0
    };
  }, [blocks, nodeStatus]);

  // Memoize callbacks
  const refreshData = useCallback(async () => {
    const status = await fetchNodeStatus();
    setNodeStatus(status);
  }, []);

  return (
    <div className="dashboard">
      <div className="stats-grid">
        <StatCard title="Total Blocks" value={stats.totalBlocks} icon={<Blocks />} />
        <StatCard title="Avg Block Time" value={`${stats.avgBlockTime}s`} icon={<Clock />} />
        <StatCard title="Hash Rate" value={stats.networkHashRate} icon={<Cpu />} />
        <StatCard title="Active Nodes" value={stats.activeNodes} icon={<Network />} />
      </div>
    </div>
  );
};
```

**Files Changed:**
- `gui/citrate-core/src/components/Dashboard.tsx`

---

#### Task 3.4: Add Debounced Search
**Estimated Time:** 1 hour

**Create file:** `gui/citrate-core/src/hooks/useDebounce.ts`

```typescript
import { useState, useEffect } from 'react';

export function useDebounce<T>(value: T, delay: number = 300): T {
  const [debouncedValue, setDebouncedValue] = useState<T>(value);

  useEffect(() => {
    const timer = setTimeout(() => {
      setDebouncedValue(value);
    }, delay);

    return () => {
      clearTimeout(timer);
    };
  }, [value, delay]);

  return debouncedValue;
}
```

**Usage in components:**

```typescript
import { useDebounce } from '../hooks/useDebounce';

const [searchQuery, setSearchQuery] = useState('');
const debouncedQuery = useDebounce(searchQuery, 300);

useEffect(() => {
  // This only runs 300ms after user stops typing
  if (debouncedQuery) {
    performSearch(debouncedQuery);
  }
}, [debouncedQuery]);
```

**Files Created:**
- `gui/citrate-core/src/hooks/useDebounce.ts`

---

#### Task 3.5: Create Web Worker for DAG Calculations (Optional)
**Estimated Time:** 1.5 hours

**Create file:** `gui/citrate-core/src/workers/dagWorker.ts`

```typescript
// Web Worker for expensive DAG calculations
self.onmessage = (e: MessageEvent) => {
  const { type, payload } = e.data;

  switch (type) {
    case 'CALCULATE_BLUE_SET': {
      const blueSet = calculateBlueSet(payload.blocks, payload.k);
      self.postMessage({ type: 'BLUE_SET_RESULT', payload: blueSet });
      break;
    }

    case 'TOPOLOGICAL_SORT': {
      const sorted = topologicalSort(payload.blocks);
      self.postMessage({ type: 'SORT_RESULT', payload: sorted });
      break;
    }

    default:
      console.warn('Unknown worker message type:', type);
  }
};

function calculateBlueSet(blocks: Block[], k: number): string[] {
  // Heavy computation here
  // ...
  return blueBlockHashes;
}

function topologicalSort(blocks: Block[]): Block[] {
  // Heavy computation here
  // ...
  return sortedBlocks;
}
```

**Usage in component:**

```typescript
import { useEffect, useRef } from 'react';

const workerRef = useRef<Worker | null>(null);

useEffect(() => {
  // Create worker
  workerRef.current = new Worker(
    new URL('../workers/dagWorker.ts', import.meta.url),
    { type: 'module' }
  );

  // Listen for results
  workerRef.current.onmessage = (e) => {
    const { type, payload } = e.data;
    if (type === 'BLUE_SET_RESULT') {
      setBlueSet(payload);
    }
  };

  return () => {
    workerRef.current?.terminate();
  };
}, []);

const calculateBlueSet = (blocks: Block[]) => {
  workerRef.current?.postMessage({
    type: 'CALCULATE_BLUE_SET',
    payload: { blocks, k: 18 }
  });
};
```

**Files Created:**
- `gui/citrate-core/src/workers/dagWorker.ts`

---

### Testing Plan

#### Performance Tests

```typescript
import { render, screen } from '@testing-library/react';
import { performance } from 'perf_hooks';

describe('Performance Tests', () => {
  it('should render 1000 items in virtual list within 100ms', () => {
    const items = Array.from({ length: 1000 }, (_, i) => ({ id: i, value: `Item ${i}` }));

    const start = performance.now();
    render(<VirtualList items={items} height={600} itemHeight={60} renderItem={(item) => <div>{item.value}</div>} />);
    const end = performance.now();

    expect(end - start).toBeLessThan(100);
  });

  it('should debounce search input', async () => {
    const onSearch = vi.fn();
    render(<SearchInput onSearch={onSearch} />);

    const input = screen.getByRole('textbox');
    fireEvent.change(input, { target: { value: 'test' } });

    // Should not call immediately
    expect(onSearch).not.toHaveBeenCalled();

    // Should call after 300ms
    await waitFor(() => expect(onSearch).toHaveBeenCalledWith('test'), { timeout: 400 });
  });
});
```

#### Manual Performance Testing
- [ ] Open dev tools Performance tab
- [ ] Record while scrolling 1000-item list
- [ ] Verify 60fps maintained
- [ ] Check FPS meter during interactions
- [ ] Verify no long tasks (>50ms)
- [ ] Test on low-end device (if available)

---

### Definition of Done
- [ ] All acceptance criteria met
- [ ] VirtualList component created and working
- [ ] Wallet activity uses virtual scrolling
- [ ] Dashboard optimized with React.memo
- [ ] Debounced search implemented
- [ ] Web Worker created (optional)
- [ ] Performance benchmarks met
- [ ] 60fps maintained during scrolling
- [ ] No console warnings about performance
- [ ] Manual testing completed
- [ ] Code reviewed

---

## Story 4: Accessibility & Keyboard Shortcuts

**Story ID:** S2-04
**Priority:** P1 (High - Accessibility)
**Story Points:** 2
**Assignee:** TBD

### User Story
```
As a user with accessibility needs
I want to navigate the app using keyboard and screen readers
So that I can use the app regardless of my abilities
```

### Current State (Problem)
**Accessibility Issues:**
- Missing ARIA labels on buttons/links
- No keyboard shortcuts
- Poor focus indicators
- Tab order not optimized
- No skip navigation links
- Screen reader announcements missing
- Color contrast issues

### Desired State (Solution)
- WCAG 2.1 AA compliance
- Full keyboard navigation
- Keyboard shortcuts for common actions
- Clear focus indicators
- Screen reader friendly
- Skip navigation

---

### Acceptance Criteria

#### AC1: Keyboard Navigation
- [ ] All interactive elements tabbable
- [ ] Tab order logical and intuitive
- [ ] Escape key closes modals
- [ ] Arrow keys navigate lists
- [ ] Enter/Space activate buttons
- [ ] Skip navigation link at top

#### AC2: Keyboard Shortcuts
- [ ] Ctrl+K: Open search/command palette
- [ ] Ctrl+S: Quick send transaction
- [ ] Ctrl+,: Open settings
- [ ] Ctrl+1-5: Switch tabs
- [ ] ?: Show keyboard shortcuts help
- [ ] Shortcuts displayed in UI (tooltips)

#### AC3: ARIA Labels
- [ ] All buttons have aria-label
- [ ] All icons have alt text or aria-label
- [ ] Form inputs have labels
- [ ] Error messages have aria-live
- [ ] Loading states announced
- [ ] Navigation landmarks (nav, main, aside)

#### AC4: Focus Indicators
- [ ] Visible focus ring on all elements
- [ ] High contrast focus indicators
- [ ] Focus ring consistent across themes
- [ ] No :focus { outline: none } without replacement

#### AC5: Screen Reader Support
- [ ] Page titles updated on navigation
- [ ] Live regions for dynamic content
- [ ] Status announcements
- [ ] Error announcements
- [ ] Loading announcements

---

### Technical Tasks

#### Task 4.1: Create Keyboard Shortcuts Hook
**Estimated Time:** 2 hours

**Create file:** `gui/citrate-core/src/hooks/useKeyboardShortcuts.ts`

```typescript
import { useEffect, useCallback } from 'react';

export interface KeyboardShortcut {
  key: string;
  ctrl?: boolean;
  shift?: boolean;
  alt?: boolean;
  handler: () => void;
  description: string;
}

export function useKeyboardShortcuts(shortcuts: KeyboardShortcut[]) {
  const handleKeyDown = useCallback((event: KeyboardEvent) => {
    const matchingShortcut = shortcuts.find(shortcut => {
      const keyMatches = event.key.toLowerCase() === shortcut.key.toLowerCase();
      const ctrlMatches = !!shortcut.ctrl === (event.ctrlKey || event.metaKey);
      const shiftMatches = !!shortcut.shift === event.shiftKey;
      const altMatches = !!shortcut.alt === event.altKey;

      return keyMatches && ctrlMatches && shiftMatches && altMatches;
    });

    if (matchingShortcut) {
      event.preventDefault();
      matchingShortcut.handler();
    }
  }, [shortcuts]);

  useEffect(() => {
    window.addEventListener('keydown', handleKeyDown);

    return () => {
      window.removeEventListener('keydown', handleKeyDown);
    };
  }, [handleKeyDown]);
}

// Global shortcuts registry
export const globalShortcuts: KeyboardShortcut[] = [
  {
    key: 'k',
    ctrl: true,
    description: 'Open command palette',
    handler: () => {
      // Will be set by App component
    }
  },
  {
    key: 's',
    ctrl: true,
    description: 'Quick send transaction',
    handler: () => {
      // Will be set by App component
    }
  },
  {
    key: ',',
    ctrl: true,
    description: 'Open settings',
    handler: () => {
      // Will be set by App component
    }
  },
  {
    key: '?',
    description: 'Show keyboard shortcuts',
    handler: () => {
      // Will be set by App component
    }
  }
];
```

**Usage in App:**

```typescript
import { useKeyboardShortcuts, globalShortcuts } from './hooks/useKeyboardShortcuts';

function App() {
  const { setCurrentTab } = useApp();
  const [showShortcuts, setShowShortcuts] = useState(false);

  useKeyboardShortcuts([
    {
      key: 'k',
      ctrl: true,
      description: 'Open command palette',
      handler: () => {
        // Open command palette
      }
    },
    {
      key: 's',
      ctrl: true,
      description: 'Quick send',
      handler: () => {
        setCurrentTab('wallet');
        // Focus send form
      }
    },
    {
      key: ',',
      ctrl: true,
      description: 'Settings',
      handler: () => {
        setCurrentTab('settings');
      }
    },
    {
      key: '1',
      ctrl: true,
      description: 'Dashboard',
      handler: () => setCurrentTab('dashboard')
    },
    {
      key: '2',
      ctrl: true,
      description: 'Wallet',
      handler: () => setCurrentTab('wallet')
    },
    {
      key: '3',
      ctrl: true,
      description: 'DAG',
      handler: () => setCurrentTab('dag')
    },
    {
      key: '4',
      ctrl: true,
      description: 'Models',
      handler: () => setCurrentTab('models')
    },
    {
      key: '5',
      ctrl: true,
      description: 'Settings',
      handler: () => setCurrentTab('settings')
    },
    {
      key: '?',
      description: 'Show shortcuts',
      handler: () => setShowShortcuts(true)
    }
  ]);

  return (
    // ...
  );
}
```

**Files Created:**
- `gui/citrate-core/src/hooks/useKeyboardShortcuts.ts`

---

#### Task 4.2: Add ARIA Labels to Components
**Estimated Time:** 2 hours

**Files to Update:**
- `gui/citrate-core/src/App.tsx`
- All component files

**Changes:**

```typescript
// Add ARIA labels to buttons
<button
  onClick={handleClick}
  aria-label="Send transaction"
  aria-describedby="send-button-help"
>
  Send
</button>
<span id="send-button-help" className="sr-only">
  Ctrl+S to quick send
</span>

// Add roles and landmarks
<nav aria-label="Main navigation">
  <ul role="list">
    <li role="listitem">
      <button
        onClick={() => setTab('dashboard')}
        aria-label="Dashboard"
        aria-current={tab === 'dashboard' ? 'page' : undefined}
      >
        Dashboard
      </button>
    </li>
  </ul>
</nav>

<main aria-label="Main content">
  {/* Main content */}
</main>

<aside aria-label="Settings panel">
  {/* Settings */}
</aside>

// Add live regions for announcements
<div
  role="status"
  aria-live="polite"
  aria-atomic="true"
  className="sr-only"
>
  {statusMessage}
</div>

// Add skip navigation
<a href="#main-content" className="skip-link">
  Skip to main content
</a>

<div id="main-content" tabIndex={-1}>
  {/* Main content */}
</div>
```

**CSS for skip link:**

```css
.skip-link {
  position: absolute;
  top: -40px;
  left: 0;
  background: var(--brand-primary);
  color: white;
  padding: 8px;
  text-decoration: none;
  z-index: 100;
}

.skip-link:focus {
  top: 0;
}

.sr-only {
  position: absolute;
  width: 1px;
  height: 1px;
  padding: 0;
  margin: -1px;
  overflow: hidden;
  clip: rect(0, 0, 0, 0);
  white-space: nowrap;
  border-width: 0;
}
```

---

#### Task 4.3: Create Keyboard Shortcuts Help Modal
**Estimated Time:** 1 hour

**Create file:** `gui/citrate-core/src/components/KeyboardShortcutsHelp.tsx`

```typescript
import React from 'react';
import { X } from 'lucide-react';
import { globalShortcuts } from '../hooks/useKeyboardShortcuts';

interface KeyboardShortcutsHelpProps {
  isOpen: boolean;
  onClose: () => void;
}

export const KeyboardShortcutsHelp: React.FC<KeyboardShortcutsHelpProps> = ({
  isOpen,
  onClose
}) => {
  if (!isOpen) return null;

  const formatShortcut = (shortcut: KeyboardShortcut): string => {
    const parts: string[] = [];
    if (shortcut.ctrl) parts.push('Ctrl');
    if (shortcut.shift) parts.push('Shift');
    if (shortcut.alt) parts.push('Alt');
    parts.push(shortcut.key.toUpperCase());
    return parts.join('+');
  };

  return (
    <div className="modal-overlay" onClick={onClose}>
      <div className="modal-content" onClick={e => e.stopPropagation()}>
        <div className="modal-header">
          <h2>Keyboard Shortcuts</h2>
          <button
            onClick={onClose}
            aria-label="Close keyboard shortcuts help"
          >
            <X size={24} />
          </button>
        </div>

        <div className="shortcuts-list">
          {globalShortcuts.map((shortcut, index) => (
            <div key={index} className="shortcut-item">
              <kbd className="shortcut-key">{formatShortcut(shortcut)}</kbd>
              <span className="shortcut-description">{shortcut.description}</span>
            </div>
          ))}
        </div>

        <div className="modal-footer">
          <p className="text-sm text-secondary">
            Press <kbd>Esc</kbd> to close this dialog
          </p>
        </div>
      </div>
    </div>
  );
};
```

**Files Created:**
- `gui/citrate-core/src/components/KeyboardShortcutsHelp.tsx`

---

#### Task 4.4: Accessibility Audit and Fixes
**Estimated Time:** 2 hours

**Audit Checklist:**

##### Color Contrast
- [ ] All text has 4.5:1 contrast ratio (AA)
- [ ] Large text has 3:1 contrast ratio
- [ ] Interactive elements have 3:1 contrast
- [ ] Test with Chrome DevTools color picker
- [ ] Test with WAVE browser extension

##### Keyboard Navigation
- [ ] Tab through entire app
- [ ] Verify logical tab order
- [ ] Test Escape key in modals
- [ ] Test arrow keys in lists
- [ ] Test Enter/Space on buttons

##### Screen Reader Testing
- [ ] Test with NVDA (Windows) or VoiceOver (Mac)
- [ ] Verify page titles announced
- [ ] Verify button labels read correctly
- [ ] Verify form labels associated
- [ ] Verify error messages announced

##### Focus Management
- [ ] Focus trapped in modals
- [ ] Focus returns after modal close
- [ ] Focus visible on all elements
- [ ] No :focus { outline: none }

##### Semantic HTML
- [ ] Use semantic elements (nav, main, aside, article)
- [ ] Heading hierarchy correct (h1 → h2 → h3)
- [ ] Lists use ul/ol tags
- [ ] Forms use fieldset/legend
- [ ] Tables use caption/th/td

---

### Testing Plan

#### Automated Accessibility Tests

```typescript
import { render } from '@testing-library/react';
import { axe, toHaveNoViolations } from 'jest-axe';

expect.extend(toHaveNoViolations);

describe('Accessibility Tests', () => {
  it('should have no accessibility violations', async () => {
    const { container } = render(<App />);
    const results = await axe(container);
    expect(results).toHaveNoViolations();
  });
});
```

#### Manual Testing Checklist
- [ ] Tab through all elements
- [ ] Test all keyboard shortcuts
- [ ] Test with screen reader (NVDA/VoiceOver)
- [ ] Test color contrast with tools
- [ ] Test focus indicators visible
- [ ] Test skip navigation link
- [ ] Test keyboard navigation in forms
- [ ] Test Escape key closes modals
- [ ] Test ? key shows shortcuts help
- [ ] Verify WCAG 2.1 AA compliance

---

### Definition of Done
- [ ] All acceptance criteria met
- [ ] Keyboard shortcuts implemented
- [ ] ARIA labels on all elements
- [ ] Focus indicators visible
- [ ] Screen reader tested
- [ ] WCAG 2.1 AA compliant
- [ ] Automated accessibility tests passing
- [ ] Manual testing completed
- [ ] Keyboard shortcuts help modal working
- [ ] Code reviewed

---

**Total Stories:** 4
**Total Story Points:** 13
**Estimated Duration:** 5 days
