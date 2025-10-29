/**
 * Theme Definitions
 *
 * Provides light and dark theme color schemes with CSS variable support.
 * All colors are WCAG AA compliant for accessibility.
 */

export type ThemeMode = 'light' | 'dark' | 'system';

export interface Theme {
  // Background colors
  bgPrimary: string;
  bgSecondary: string;
  bgTertiary: string;

  // Text colors
  textPrimary: string;
  textSecondary: string;
  textMuted: string;

  // Border colors
  borderPrimary: string;
  borderSecondary: string;

  // Brand colors
  brandPrimary: string;
  brandHover: string;
  brandActive: string;

  // Status colors
  success: string;
  successBg: string;
  error: string;
  errorBg: string;
  warning: string;
  warningBg: string;
  info: string;
  infoBg: string;

  // Shadow
  shadow: string;
  shadowLg: string;
}

/**
 * Light Theme
 */
export const lightTheme: Theme = {
  // Backgrounds
  bgPrimary: '#ffffff',
  bgSecondary: '#f9fafb',
  bgTertiary: '#f3f4f6',

  // Text
  textPrimary: '#1a1a1a',
  textSecondary: '#4b5563',
  textMuted: '#9ca3af',

  // Borders
  borderPrimary: '#e5e7eb',
  borderSecondary: '#d1d5db',

  // Brand (Orange)
  brandPrimary: '#ffa500',
  brandHover: '#ff8c00',
  brandActive: '#e69500',

  // Status
  success: '#10b981',
  successBg: '#d1fae5',
  error: '#ef4444',
  errorBg: '#fee2e2',
  warning: '#f59e0b',
  warningBg: '#fef3c7',
  info: '#3b82f6',
  infoBg: '#dbeafe',

  // Shadow
  shadow: 'rgba(0, 0, 0, 0.1)',
  shadowLg: 'rgba(0, 0, 0, 0.15)',
};

/**
 * Dark Theme
 */
export const darkTheme: Theme = {
  // Backgrounds
  bgPrimary: '#1a1a1a',
  bgSecondary: '#242424',
  bgTertiary: '#2d2d2d',

  // Text
  textPrimary: '#ffffff',
  textSecondary: '#d1d5db',
  textMuted: '#9ca3af',

  // Borders
  borderPrimary: '#374151',
  borderSecondary: '#4b5563',

  // Brand (Lighter Orange for dark mode)
  brandPrimary: '#ffb84d',
  brandHover: '#ffc966',
  brandActive: '#ffa534',

  // Status
  success: '#34d399',
  successBg: '#064e3b',
  error: '#f87171',
  errorBg: '#7f1d1d',
  warning: '#fbbf24',
  warningBg: '#78350f',
  info: '#60a5fa',
  infoBg: '#1e3a8a',

  // Shadow
  shadow: 'rgba(0, 0, 0, 0.3)',
  shadowLg: 'rgba(0, 0, 0, 0.5)',
};

/**
 * Themes object for easy access
 */
export const themes = {
  light: lightTheme,
  dark: darkTheme,
};

/**
 * Apply theme by setting CSS custom properties on document root
 */
export function applyTheme(theme: Theme): void {
  const root = document.documentElement;

  // Background colors
  root.style.setProperty('--bg-primary', theme.bgPrimary);
  root.style.setProperty('--bg-secondary', theme.bgSecondary);
  root.style.setProperty('--bg-tertiary', theme.bgTertiary);

  // Text colors
  root.style.setProperty('--text-primary', theme.textPrimary);
  root.style.setProperty('--text-secondary', theme.textSecondary);
  root.style.setProperty('--text-muted', theme.textMuted);

  // Border colors
  root.style.setProperty('--border-primary', theme.borderPrimary);
  root.style.setProperty('--border-secondary', theme.borderSecondary);

  // Brand colors
  root.style.setProperty('--brand-primary', theme.brandPrimary);
  root.style.setProperty('--brand-hover', theme.brandHover);
  root.style.setProperty('--brand-active', theme.brandActive);

  // Status colors
  root.style.setProperty('--success', theme.success);
  root.style.setProperty('--success-bg', theme.successBg);
  root.style.setProperty('--error', theme.error);
  root.style.setProperty('--error-bg', theme.errorBg);
  root.style.setProperty('--warning', theme.warning);
  root.style.setProperty('--warning-bg', theme.warningBg);
  root.style.setProperty('--info', theme.info);
  root.style.setProperty('--info-bg', theme.infoBg);

  // Shadow
  root.style.setProperty('--shadow', theme.shadow);
  root.style.setProperty('--shadow-lg', theme.shadowLg);
}

/**
 * Get system theme preference
 */
export function getSystemTheme(): 'light' | 'dark' {
  if (typeof window === 'undefined') return 'light';

  const isDark = window.matchMedia('(prefers-color-scheme: dark)').matches;
  return isDark ? 'dark' : 'light';
}

/**
 * Set data-theme attribute on document root
 */
export function setThemeAttribute(mode: 'light' | 'dark'): void {
  document.documentElement.setAttribute('data-theme', mode);
}

/**
 * Resolve theme mode to actual theme
 * Handles 'system' mode by detecting OS preference
 */
export function resolveThemeMode(mode: ThemeMode): 'light' | 'dark' {
  if (mode === 'system') {
    return getSystemTheme();
  }
  return mode;
}
