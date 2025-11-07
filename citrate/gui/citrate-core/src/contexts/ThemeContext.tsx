/**
 * Theme Context
 *
 * Provides theme management with automatic persistence and system theme detection.
 * Handles light, dark, and system theme modes.
 */

import React, { createContext, useContext, useState, useEffect, ReactNode } from 'react';
import {
  Theme,
  ThemeMode,
  themes,
  applyTheme,
  setThemeAttribute,
  resolveThemeMode,
} from '../styles/themes';
import { getStorageItem, setStorageItem, StorageKeys } from '../utils/storage';

interface ThemeContextValue {
  themeMode: ThemeMode;
  currentTheme: 'light' | 'dark';
  theme: Theme;
  setThemeMode: (mode: ThemeMode) => void;
  toggleTheme: () => void;
}

const ThemeContext = createContext<ThemeContextValue | undefined>(undefined);

interface ThemeProviderProps {
  children: ReactNode;
}

/**
 * Theme Provider
 *
 * Manages theme state, persistence, and application
 */
export const ThemeProvider: React.FC<ThemeProviderProps> = ({ children }) => {
  // Load theme mode from storage, default to 'dark'
  const [themeMode, setThemeModeState] = useState<ThemeMode>(() => {
    const stored = getStorageItem(StorageKeys.THEME);
    return stored || 'dark';
  });

  // Resolve current theme (light or dark)
  const [currentTheme, setCurrentTheme] = useState<'light' | 'dark'>(() => {
    return resolveThemeMode(themeMode);
  });

  // Get the actual theme object
  const theme = themes[currentTheme];

  // Set theme mode with persistence
  const setThemeMode = (mode: ThemeMode) => {
    setThemeModeState(mode);
    setStorageItem(StorageKeys.THEME, mode);
    console.log(`[ThemeContext] Theme mode changed to: ${mode}`);
  };

  // Toggle between light and dark (ignores system)
  const toggleTheme = () => {
    const newMode = currentTheme === 'light' ? 'dark' : 'light';
    setThemeMode(newMode);
  };

  // Apply theme when it changes
  useEffect(() => {
    const resolved = resolveThemeMode(themeMode);
    setCurrentTheme(resolved);
    applyTheme(themes[resolved]);
    setThemeAttribute(resolved);
    console.log(`[ThemeContext] Theme applied: ${resolved}`);
  }, [themeMode]);

  // Listen for system theme changes (only when mode is 'system')
  useEffect(() => {
    if (themeMode !== 'system') return;

    const mediaQuery = window.matchMedia('(prefers-color-scheme: dark)');

    const handleChange = (e: MediaQueryListEvent) => {
      const systemTheme = e.matches ? 'dark' : 'light';
      setCurrentTheme(systemTheme);
      applyTheme(themes[systemTheme]);
      setThemeAttribute(systemTheme);
      console.log(`[ThemeContext] System theme changed to: ${systemTheme}`);
    };

    // Modern browsers
    if (mediaQuery.addEventListener) {
      mediaQuery.addEventListener('change', handleChange);
      return () => mediaQuery.removeEventListener('change', handleChange);
    } else {
      // Fallback for older browsers
      mediaQuery.addListener(handleChange);
      return () => mediaQuery.removeListener(handleChange);
    }
  }, [themeMode]);

  const value: ThemeContextValue = {
    themeMode,
    currentTheme,
    theme,
    setThemeMode,
    toggleTheme,
  };

  return <ThemeContext.Provider value={value}>{children}</ThemeContext.Provider>;
};

/**
 * useTheme Hook
 *
 * Access theme context from any component.
 * Throws error if used outside ThemeProvider.
 */
export const useTheme = (): ThemeContextValue => {
  const context = useContext(ThemeContext);

  if (context === undefined) {
    throw new Error(
      'useTheme must be used within a ThemeProvider. ' +
      'Wrap your app with <ThemeProvider> in App.tsx.'
    );
  }

  return context;
};

export default ThemeContext;
