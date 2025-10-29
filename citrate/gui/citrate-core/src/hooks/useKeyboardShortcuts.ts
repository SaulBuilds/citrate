/**
 * useKeyboardShortcuts Hook
 *
 * Provides global keyboard shortcut handling with customizable actions.
 * Supports modifier keys (Ctrl, Alt, Shift, Meta) and prevents conflicts
 * with input fields.
 */

import { useEffect, useCallback, useRef } from 'react';

export interface KeyboardShortcut {
  key: string;
  ctrl?: boolean;
  alt?: boolean;
  shift?: boolean;
  meta?: boolean;
  description: string;
  action: () => void;
  preventDefault?: boolean;
}

export interface UseKeyboardShortcutsOptions {
  /** Whether shortcuts are enabled (default: true) */
  enabled?: boolean;
  /** Whether to allow shortcuts when input elements are focused (default: false) */
  allowInInputs?: boolean;
}

/**
 * Hook for registering keyboard shortcuts
 *
 * @param shortcuts - Array of keyboard shortcuts to register
 * @param options - Configuration options
 *
 * @example
 * ```tsx
 * useKeyboardShortcuts([
 *   { key: 'k', ctrl: true, description: 'Search', action: () => openSearch() },
 *   { key: ',', ctrl: true, description: 'Settings', action: () => openSettings() },
 *   { key: '?', shift: true, description: 'Help', action: () => showHelp() }
 * ]);
 * ```
 */
export function useKeyboardShortcuts(
  shortcuts: KeyboardShortcut[],
  options: UseKeyboardShortcutsOptions = {}
): void {
  const { enabled = true, allowInInputs = false } = options;
  const shortcutsRef = useRef(shortcuts);

  // Keep shortcuts ref up to date
  useEffect(() => {
    shortcutsRef.current = shortcuts;
  }, [shortcuts]);

  const handleKeyDown = useCallback(
    (event: KeyboardEvent) => {
      if (!enabled) return;

      // Check if we're in an input field and should ignore
      if (!allowInInputs) {
        const target = event.target as HTMLElement;
        const tagName = target.tagName.toLowerCase();
        if (
          tagName === 'input' ||
          tagName === 'textarea' ||
          tagName === 'select' ||
          target.isContentEditable
        ) {
          return;
        }
      }

      // Find matching shortcut
      const matchedShortcut = shortcutsRef.current.find((shortcut) => {
        const keyMatch = event.key.toLowerCase() === shortcut.key.toLowerCase();
        const ctrlMatch = shortcut.ctrl ? event.ctrlKey || event.metaKey : !event.ctrlKey && !event.metaKey;
        const altMatch = shortcut.alt ? event.altKey : !event.altKey;
        const shiftMatch = shortcut.shift ? event.shiftKey : !event.shiftKey;
        const metaMatch = shortcut.meta ? event.metaKey : true;

        return keyMatch && ctrlMatch && altMatch && shiftMatch && metaMatch;
      });

      if (matchedShortcut) {
        if (matchedShortcut.preventDefault !== false) {
          event.preventDefault();
        }
        matchedShortcut.action();
      }
    },
    [enabled, allowInInputs]
  );

  useEffect(() => {
    if (!enabled) return;

    window.addEventListener('keydown', handleKeyDown);
    return () => window.removeEventListener('keydown', handleKeyDown);
  }, [enabled, handleKeyDown]);
}

/**
 * Format shortcut for display
 *
 * @param shortcut - Keyboard shortcut to format
 * @returns Human-readable shortcut string (e.g., "Ctrl+K", "Shift+?")
 */
export function formatShortcut(shortcut: KeyboardShortcut): string {
  const parts: string[] = [];

  const isMac = typeof navigator !== 'undefined' && navigator.platform.toUpperCase().indexOf('MAC') >= 0;

  if (shortcut.ctrl) parts.push(isMac ? '⌘' : 'Ctrl');
  if (shortcut.alt) parts.push(isMac ? '⌥' : 'Alt');
  if (shortcut.shift) parts.push(isMac ? '⇧' : 'Shift');
  if (shortcut.meta && !shortcut.ctrl) parts.push(isMac ? '⌘' : 'Meta');

  parts.push(shortcut.key.toUpperCase());

  return parts.join(isMac ? '' : '+');
}

/**
 * Global keyboard shortcuts registry
 * These are the default shortcuts available throughout the app
 */
export const GLOBAL_SHORTCUTS = {
  SEARCH: { key: 'k', ctrl: true, description: 'Search transactions' },
  SETTINGS: { key: ',', ctrl: true, description: 'Open settings' },
  HELP: { key: '/', shift: true, description: 'Show keyboard shortcuts' },
  CLOSE_MODAL: { key: 'Escape', description: 'Close modal/dialog' },
  REFRESH: { key: 'r', ctrl: true, description: 'Refresh data' },
  NEW_TRANSACTION: { key: 'n', ctrl: true, description: 'New transaction' },
  COPY_ADDRESS: { key: 'c', alt: true, description: 'Copy wallet address' },
} as const;
