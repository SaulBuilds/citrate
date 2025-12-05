/**
 * Toast Context
 *
 * Provides toast notification functionality throughout the app.
 * Supports success, error, warning, and info toast types.
 */

import React, {
  createContext,
  useContext,
  useState,
  useCallback,
  ReactNode,
} from 'react';
import { CheckCircle, XCircle, AlertTriangle, Info, X } from 'lucide-react';

// =============================================================================
// Types
// =============================================================================

export type ToastType = 'success' | 'error' | 'warning' | 'info';

export interface Toast {
  id: string;
  type: ToastType;
  title: string;
  message?: string;
  duration?: number;
  action?: {
    label: string;
    onClick: () => void;
  };
}

interface ToastContextValue {
  toasts: Toast[];
  addToast: (toast: Omit<Toast, 'id'>) => string;
  removeToast: (id: string) => void;
  clearToasts: () => void;
  success: (title: string, message?: string) => string;
  error: (title: string, message?: string) => string;
  warning: (title: string, message?: string) => string;
  info: (title: string, message?: string) => string;
}

// =============================================================================
// Context
// =============================================================================

const ToastContext = createContext<ToastContextValue | undefined>(undefined);

// =============================================================================
// Provider
// =============================================================================

interface ToastProviderProps {
  children: ReactNode;
  position?: 'top-right' | 'top-left' | 'bottom-right' | 'bottom-left' | 'top-center' | 'bottom-center';
  maxToasts?: number;
}

export const ToastProvider: React.FC<ToastProviderProps> = ({
  children,
  position = 'bottom-right',
  maxToasts = 5,
}) => {
  const [toasts, setToasts] = useState<Toast[]>([]);

  const addToast = useCallback(
    (toast: Omit<Toast, 'id'>): string => {
      const id = `toast-${Date.now()}-${Math.random().toString(36).slice(2)}`;
      const newToast: Toast = { ...toast, id };

      setToasts((prev) => {
        const updated = [...prev, newToast];
        // Keep only the latest maxToasts
        return updated.slice(-maxToasts);
      });

      // Auto-remove after duration
      const duration = toast.duration ?? 5000;
      if (duration > 0) {
        setTimeout(() => {
          removeToast(id);
        }, duration);
      }

      return id;
    },
    [maxToasts]
  );

  const removeToast = useCallback((id: string) => {
    setToasts((prev) => prev.filter((t) => t.id !== id));
  }, []);

  const clearToasts = useCallback(() => {
    setToasts([]);
  }, []);

  // Convenience methods
  const success = useCallback(
    (title: string, message?: string) => addToast({ type: 'success', title, message }),
    [addToast]
  );

  const error = useCallback(
    (title: string, message?: string) => addToast({ type: 'error', title, message }),
    [addToast]
  );

  const warning = useCallback(
    (title: string, message?: string) => addToast({ type: 'warning', title, message }),
    [addToast]
  );

  const info = useCallback(
    (title: string, message?: string) => addToast({ type: 'info', title, message }),
    [addToast]
  );

  const value: ToastContextValue = {
    toasts,
    addToast,
    removeToast,
    clearToasts,
    success,
    error,
    warning,
    info,
  };

  return (
    <ToastContext.Provider value={value}>
      {children}
      <ToastContainer toasts={toasts} position={position} onRemove={removeToast} />
    </ToastContext.Provider>
  );
};

// =============================================================================
// Toast Container
// =============================================================================

interface ToastContainerProps {
  toasts: Toast[];
  position: ToastProviderProps['position'];
  onRemove: (id: string) => void;
}

const ToastContainer: React.FC<ToastContainerProps> = ({
  toasts,
  position = 'bottom-right',
  onRemove,
}) => {
  const getPositionStyles = () => {
    switch (position) {
      case 'top-right':
        return { top: '1rem', right: '1rem' };
      case 'top-left':
        return { top: '1rem', left: '1rem' };
      case 'bottom-right':
        return { bottom: '1rem', right: '1rem' };
      case 'bottom-left':
        return { bottom: '1rem', left: '1rem' };
      case 'top-center':
        return { top: '1rem', left: '50%', transform: 'translateX(-50%)' };
      case 'bottom-center':
        return { bottom: '1rem', left: '50%', transform: 'translateX(-50%)' };
      default:
        return { bottom: '1rem', right: '1rem' };
    }
  };

  if (toasts.length === 0) return null;

  return (
    <div className="toast-container" style={getPositionStyles()}>
      {toasts.map((toast) => (
        <ToastItem key={toast.id} toast={toast} onRemove={onRemove} />
      ))}

      <style jsx>{`
        .toast-container {
          position: fixed;
          z-index: 9999;
          display: flex;
          flex-direction: column;
          gap: 0.5rem;
          max-width: 400px;
          width: 100%;
          pointer-events: none;
        }
      `}</style>
    </div>
  );
};

// =============================================================================
// Toast Item
// =============================================================================

interface ToastItemProps {
  toast: Toast;
  onRemove: (id: string) => void;
}

const ToastItem: React.FC<ToastItemProps> = ({ toast, onRemove }) => {
  const getIcon = () => {
    switch (toast.type) {
      case 'success':
        return <CheckCircle size={20} />;
      case 'error':
        return <XCircle size={20} />;
      case 'warning':
        return <AlertTriangle size={20} />;
      case 'info':
        return <Info size={20} />;
      default:
        return <Info size={20} />;
    }
  };

  return (
    <div className={`toast-item ${toast.type}`}>
      <div className="toast-icon">{getIcon()}</div>
      <div className="toast-content">
        <span className="toast-title">{toast.title}</span>
        {toast.message && <p className="toast-message">{toast.message}</p>}
        {toast.action && (
          <button className="toast-action" onClick={toast.action.onClick}>
            {toast.action.label}
          </button>
        )}
      </div>
      <button className="toast-close" onClick={() => onRemove(toast.id)}>
        <X size={16} />
      </button>

      <style jsx>{`
        .toast-item {
          display: flex;
          align-items: flex-start;
          gap: 0.75rem;
          padding: 1rem;
          background: white;
          border-radius: 0.75rem;
          box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
          pointer-events: auto;
          animation: slideIn 0.3s ease-out;
        }

        @keyframes slideIn {
          from {
            opacity: 0;
            transform: translateX(100%);
          }
          to {
            opacity: 1;
            transform: translateX(0);
          }
        }

        .toast-item.success {
          border-left: 4px solid #10b981;
        }

        .toast-item.success .toast-icon {
          color: #10b981;
        }

        .toast-item.error {
          border-left: 4px solid #ef4444;
        }

        .toast-item.error .toast-icon {
          color: #ef4444;
        }

        .toast-item.warning {
          border-left: 4px solid #f59e0b;
        }

        .toast-item.warning .toast-icon {
          color: #f59e0b;
        }

        .toast-item.info {
          border-left: 4px solid #3b82f6;
        }

        .toast-item.info .toast-icon {
          color: #3b82f6;
        }

        .toast-icon {
          flex-shrink: 0;
          margin-top: 2px;
        }

        .toast-content {
          flex: 1;
          min-width: 0;
        }

        .toast-title {
          font-weight: 600;
          color: #111827;
          font-size: 0.875rem;
        }

        .toast-message {
          margin: 0.25rem 0 0;
          font-size: 0.8125rem;
          color: #6b7280;
        }

        .toast-action {
          margin-top: 0.5rem;
          padding: 0.25rem 0.5rem;
          background: none;
          border: 1px solid #e5e7eb;
          border-radius: 0.25rem;
          font-size: 0.75rem;
          font-weight: 500;
          color: #374151;
          cursor: pointer;
          transition: all 0.2s;
        }

        .toast-action:hover {
          background: #f3f4f6;
        }

        .toast-close {
          flex-shrink: 0;
          display: flex;
          align-items: center;
          justify-content: center;
          width: 24px;
          height: 24px;
          background: none;
          border: none;
          border-radius: 0.25rem;
          color: #9ca3af;
          cursor: pointer;
          transition: all 0.2s;
        }

        .toast-close:hover {
          background: #f3f4f6;
          color: #374151;
        }
      `}</style>
    </div>
  );
};

// =============================================================================
// Hook
// =============================================================================

export function useToast(): ToastContextValue {
  const context = useContext(ToastContext);
  if (context === undefined) {
    throw new Error('useToast must be used within a ToastProvider');
  }
  return context;
}

export default ToastContext;
