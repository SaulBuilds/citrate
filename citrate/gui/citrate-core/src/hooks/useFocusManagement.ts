/**
 * useFocusManagement Hook
 *
 * Utilities for managing focus programmatically for accessibility.
 * Useful for modals, dialogs, and focus traps.
 */

import { useEffect, useRef, useCallback } from 'react';

export interface UseFocusManagementOptions {
  /** Whether to trap focus within the container (default: false) */
  trapFocus?: boolean;
  /** Whether to restore focus to previous element when unmounting (default: true) */
  restoreFocus?: boolean;
  /** Auto-focus first focusable element on mount (default: true) */
  autoFocus?: boolean;
}

/**
 * Hook for managing focus within a container
 *
 * @param options - Focus management options
 * @returns Ref to attach to the container element
 *
 * @example
 * ```tsx
 * function Modal({ isOpen, onClose }) {
 *   const containerRef = useFocusManagement({ trapFocus: true });
 *
 *   if (!isOpen) return null;
 *
 *   return (
 *     <div ref={containerRef} role="dialog">
 *       <button onClick={onClose}>Close</button>
 *       <input type="text" placeholder="Search..." />
 *     </div>
 *   );
 * }
 * ```
 */
export function useFocusManagement<T extends HTMLElement>(
  options: UseFocusManagementOptions = {}
) {
  const { trapFocus = false, restoreFocus = true, autoFocus = true } = options;
  const containerRef = useRef<T>(null);
  const previousFocusRef = useRef<HTMLElement | null>(null);

  // Get all focusable elements within container
  const getFocusableElements = useCallback((): HTMLElement[] => {
    if (!containerRef.current) return [];

    const focusableSelectors = [
      'a[href]',
      'button:not([disabled])',
      'textarea:not([disabled])',
      'input:not([disabled])',
      'select:not([disabled])',
      '[tabindex]:not([tabindex="-1"])',
    ].join(', ');

    const elements = containerRef.current.querySelectorAll<HTMLElement>(focusableSelectors);
    return Array.from(elements).filter((el) => {
      // Filter out hidden elements
      const style = window.getComputedStyle(el);
      return style.display !== 'none' && style.visibility !== 'hidden';
    });
  }, []);

  // Handle focus trap
  useEffect(() => {
    if (!trapFocus || !containerRef.current) return;

    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.key !== 'Tab') return;

      const focusableElements = getFocusableElements();
      if (focusableElements.length === 0) return;

      const firstElement = focusableElements[0];
      const lastElement = focusableElements[focusableElements.length - 1];

      // Shift + Tab
      if (e.shiftKey) {
        if (document.activeElement === firstElement) {
          e.preventDefault();
          lastElement.focus();
        }
      }
      // Tab
      else {
        if (document.activeElement === lastElement) {
          e.preventDefault();
          firstElement.focus();
        }
      }
    };

    document.addEventListener('keydown', handleKeyDown);
    return () => document.removeEventListener('keydown', handleKeyDown);
  }, [trapFocus, getFocusableElements]);

  // Handle auto focus and focus restoration
  useEffect(() => {
    // Save previous focus
    if (restoreFocus) {
      previousFocusRef.current = document.activeElement as HTMLElement;
    }

    // Auto focus first element
    if (autoFocus && containerRef.current) {
      const focusableElements = getFocusableElements();
      if (focusableElements.length > 0) {
        focusableElements[0].focus();
      }
    }

    // Restore focus on unmount
    return () => {
      if (restoreFocus && previousFocusRef.current) {
        previousFocusRef.current.focus();
      }
    };
  }, [autoFocus, restoreFocus, getFocusableElements]);

  return containerRef;
}

/**
 * Focus the first focusable element in a container
 *
 * @param container - The container element
 */
export function focusFirst(container: HTMLElement | null): void {
  if (!container) return;

  const focusableSelectors = [
    'a[href]',
    'button:not([disabled])',
    'textarea:not([disabled])',
    'input:not([disabled])',
    'select:not([disabled])',
    '[tabindex]:not([tabindex="-1"])',
  ].join(', ');

  const firstFocusable = container.querySelector<HTMLElement>(focusableSelectors);
  if (firstFocusable) {
    firstFocusable.focus();
  }
}

/**
 * Focus an element by ID
 *
 * @param id - Element ID to focus
 */
export function focusById(id: string): void {
  const element = document.getElementById(id);
  if (element) {
    element.focus();
  }
}

/**
 * Check if an element is focusable
 *
 * @param element - Element to check
 * @returns Whether the element is focusable
 */
export function isFocusable(element: HTMLElement): boolean {
  const focusableSelectors = [
    'a[href]',
    'button:not([disabled])',
    'textarea:not([disabled])',
    'input:not([disabled])',
    'select:not([disabled])',
    '[tabindex]:not([tabindex="-1"])',
  ].join(', ');

  const isFocusableElement = element.matches(focusableSelectors);
  if (!isFocusableElement) return false;

  // Check visibility
  const style = window.getComputedStyle(element);
  return style.display !== 'none' && style.visibility !== 'hidden';
}
