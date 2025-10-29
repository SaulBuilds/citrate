/**
 * KeyboardShortcutsHelp Component
 *
 * Modal that displays all available keyboard shortcuts.
 * Activated by pressing Shift+? or manually.
 */

import React from 'react';
import { X } from 'lucide-react';
import { KeyboardShortcut, formatShortcut } from '../hooks/useKeyboardShortcuts';
import { useFocusManagement } from '../hooks/useFocusManagement';

export interface KeyboardShortcutsHelpProps {
  shortcuts: KeyboardShortcut[];
  isOpen: boolean;
  onClose: () => void;
}

export const KeyboardShortcutsHelp: React.FC<KeyboardShortcutsHelpProps> = ({
  shortcuts,
  isOpen,
  onClose,
}) => {
  // Focus management - trap focus within modal when open
  const modalRef = useFocusManagement<HTMLDivElement>({
    trapFocus: isOpen,
    restoreFocus: true,
    autoFocus: isOpen,
  });

  if (!isOpen) return null;

  // Group shortcuts by category (based on description keywords)
  const categorized = shortcuts.reduce((acc, shortcut) => {
    let category = 'General';
    if (shortcut.description.toLowerCase().includes('transaction')) {
      category = 'Transactions';
    } else if (shortcut.description.toLowerCase().includes('search')) {
      category = 'Search';
    } else if (shortcut.description.toLowerCase().includes('setting')) {
      category = 'Settings';
    } else if (shortcut.description.toLowerCase().includes('navigation')) {
      category = 'Navigation';
    }

    if (!acc[category]) {
      acc[category] = [];
    }
    acc[category].push(shortcut);
    return acc;
  }, {} as Record<string, KeyboardShortcut[]>);

  return (
    <div className="keyboard-shortcuts-overlay" onClick={onClose}>
      <div
        ref={modalRef}
        className="keyboard-shortcuts-modal"
        onClick={(e) => e.stopPropagation()}
        role="dialog"
        aria-modal="true"
        aria-labelledby="shortcuts-title"
      >
        <div className="modal-header">
          <h2 id="shortcuts-title">Keyboard Shortcuts</h2>
          <button
            className="close-button"
            onClick={onClose}
            aria-label="Close keyboard shortcuts"
          >
            <X size={20} />
          </button>
        </div>

        <div className="modal-content">
          {Object.entries(categorized).map(([category, categoryShortcuts]) => (
            <div key={category} className="shortcut-category">
              <h3>{category}</h3>
              <div className="shortcuts-list">
                {categoryShortcuts.map((shortcut, index) => (
                  <div key={index} className="shortcut-item">
                    <span className="shortcut-description">{shortcut.description}</span>
                    <kbd className="shortcut-key">{formatShortcut(shortcut)}</kbd>
                  </div>
                ))}
              </div>
            </div>
          ))}
        </div>

        <div className="modal-footer">
          <p className="hint">Press <kbd>Esc</kbd> to close</p>
        </div>
      </div>

      <style jsx>{`
        .keyboard-shortcuts-overlay {
          position: fixed;
          top: 0;
          left: 0;
          right: 0;
          bottom: 0;
          background: rgba(0, 0, 0, 0.5);
          display: flex;
          align-items: center;
          justify-content: center;
          z-index: 9999;
          backdrop-filter: blur(4px);
        }

        .keyboard-shortcuts-modal {
          background: var(--bg-primary);
          border: 1px solid var(--border-primary);
          border-radius: 1rem;
          max-width: 600px;
          width: 90%;
          max-height: 80vh;
          overflow: hidden;
          display: flex;
          flex-direction: column;
          box-shadow: 0 20px 60px rgba(0, 0, 0, 0.3);
        }

        .modal-header {
          display: flex;
          justify-content: space-between;
          align-items: center;
          padding: 1.5rem;
          border-bottom: 1px solid var(--border-primary);
        }

        .modal-header h2 {
          margin: 0;
          font-size: 1.5rem;
          font-weight: 600;
          color: var(--text-primary);
        }

        .close-button {
          background: transparent;
          border: none;
          color: var(--text-secondary);
          cursor: pointer;
          padding: 0.5rem;
          border-radius: 0.5rem;
          display: flex;
          align-items: center;
          justify-content: center;
          transition: background-color 200ms ease, color 200ms ease;
        }

        .close-button:hover {
          background: var(--bg-tertiary);
          color: var(--text-primary);
        }

        .modal-content {
          padding: 1.5rem;
          overflow-y: auto;
          flex: 1;
        }

        .shortcut-category {
          margin-bottom: 2rem;
        }

        .shortcut-category:last-child {
          margin-bottom: 0;
        }

        .shortcut-category h3 {
          margin: 0 0 1rem 0;
          font-size: 0.875rem;
          font-weight: 600;
          color: var(--text-secondary);
          text-transform: uppercase;
          letter-spacing: 0.05em;
        }

        .shortcuts-list {
          display: flex;
          flex-direction: column;
          gap: 0.75rem;
        }

        .shortcut-item {
          display: flex;
          justify-content: space-between;
          align-items: center;
          padding: 0.75rem;
          background: var(--bg-secondary);
          border-radius: 0.5rem;
          transition: background-color 200ms ease;
        }

        .shortcut-item:hover {
          background: var(--bg-tertiary);
        }

        .shortcut-description {
          color: var(--text-primary);
          font-size: 0.9rem;
        }

        .shortcut-key,
        kbd {
          background: var(--bg-tertiary);
          border: 1px solid var(--border-primary);
          border-radius: 0.375rem;
          padding: 0.375rem 0.625rem;
          font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, 'Liberation Mono', 'Courier New', monospace;
          font-size: 0.75rem;
          font-weight: 600;
          color: var(--text-primary);
          box-shadow: 0 1px 2px rgba(0, 0, 0, 0.1);
          white-space: nowrap;
        }

        .modal-footer {
          padding: 1rem 1.5rem;
          border-top: 1px solid var(--border-primary);
          background: var(--bg-secondary);
        }

        .hint {
          margin: 0;
          font-size: 0.875rem;
          color: var(--text-secondary);
          text-align: center;
        }

        .hint kbd {
          margin: 0 0.25rem;
        }
      `}</style>
    </div>
  );
};

export default KeyboardShortcutsHelp;
