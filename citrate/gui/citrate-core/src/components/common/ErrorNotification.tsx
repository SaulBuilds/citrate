/**
 * Error Notification Component
 *
 * Displays error notifications to users with:
 * - Toast notifications for transient errors
 * - Modal dialogs for blocking errors
 * - Error code for support reference
 * - Actionable buttons for resolution
 * - Report issue functionality
 */

import React, { useEffect, useState, useCallback } from 'react';
import { useError, AppError, ErrorSeverity, ErrorCategory } from '../../contexts/ErrorContext';

// ============================================================================
// Styles
// ============================================================================

const styles = {
  container: {
    position: 'fixed' as const,
    top: '16px',
    right: '16px',
    zIndex: 9999,
    display: 'flex',
    flexDirection: 'column' as const,
    gap: '8px',
    maxWidth: '400px',
    width: '100%',
    pointerEvents: 'none' as const,
  },
  toast: {
    backgroundColor: '#1e1e2e',
    borderRadius: '8px',
    boxShadow: '0 4px 12px rgba(0, 0, 0, 0.3)',
    padding: '12px 16px',
    display: 'flex',
    alignItems: 'flex-start',
    gap: '12px',
    animation: 'slideIn 0.3s ease-out',
    pointerEvents: 'auto' as const,
    border: '1px solid',
  },
  icon: {
    fontSize: '20px',
    flexShrink: 0,
  },
  content: {
    flex: 1,
    minWidth: 0,
  },
  title: {
    fontWeight: 600,
    fontSize: '14px',
    marginBottom: '4px',
    color: '#fff',
  },
  message: {
    fontSize: '13px',
    color: '#a0a0a0',
    marginBottom: '8px',
    lineHeight: 1.4,
  },
  code: {
    fontSize: '11px',
    color: '#666',
    fontFamily: 'monospace',
  },
  actions: {
    display: 'flex',
    gap: '8px',
    marginTop: '8px',
  },
  button: {
    padding: '6px 12px',
    borderRadius: '4px',
    fontSize: '12px',
    fontWeight: 500,
    cursor: 'pointer',
    border: 'none',
    transition: 'background-color 0.2s',
  },
  primaryButton: {
    backgroundColor: '#6366f1',
    color: '#fff',
  },
  secondaryButton: {
    backgroundColor: 'transparent',
    border: '1px solid #333',
    color: '#a0a0a0',
  },
  closeButton: {
    background: 'none',
    border: 'none',
    color: '#666',
    cursor: 'pointer',
    padding: '4px',
    fontSize: '16px',
    lineHeight: 1,
  },
  modalOverlay: {
    position: 'fixed' as const,
    top: 0,
    left: 0,
    right: 0,
    bottom: 0,
    backgroundColor: 'rgba(0, 0, 0, 0.75)',
    display: 'flex',
    alignItems: 'center',
    justifyContent: 'center',
    zIndex: 10000,
    pointerEvents: 'auto' as const,
  },
  modal: {
    backgroundColor: '#1e1e2e',
    borderRadius: '12px',
    padding: '24px',
    maxWidth: '500px',
    width: '90%',
    boxShadow: '0 8px 32px rgba(0, 0, 0, 0.5)',
    border: '1px solid',
  },
  modalTitle: {
    fontSize: '18px',
    fontWeight: 600,
    color: '#fff',
    marginBottom: '12px',
    display: 'flex',
    alignItems: 'center',
    gap: '8px',
  },
  modalDetails: {
    backgroundColor: '#141420',
    borderRadius: '6px',
    padding: '12px',
    marginTop: '12px',
    fontFamily: 'monospace',
    fontSize: '12px',
    color: '#888',
    maxHeight: '200px',
    overflow: 'auto',
  },
  reportLink: {
    color: '#6366f1',
    textDecoration: 'none',
    fontSize: '12px',
    cursor: 'pointer',
    marginTop: '8px',
    display: 'inline-block',
  },
};

// ============================================================================
// Severity Styles
// ============================================================================

const severityStyles: Record<ErrorSeverity, { borderColor: string; iconColor: string; icon: string }> = {
  info: { borderColor: '#3b82f6', iconColor: '#3b82f6', icon: 'info' },
  warning: { borderColor: '#f59e0b', iconColor: '#f59e0b', icon: 'warning' },
  error: { borderColor: '#ef4444', iconColor: '#ef4444', icon: 'error' },
  critical: { borderColor: '#dc2626', iconColor: '#dc2626', icon: 'error_outline' },
};

// ============================================================================
// Toast Notification
// ============================================================================

interface ToastProps {
  error: AppError;
  onDismiss: (id: string) => void;
  onReport: (id: string) => void;
}

function Toast({ error, onDismiss, onReport }: ToastProps) {
  const [visible, setVisible] = useState(true);
  const severity = severityStyles[error.severity];

  // Auto-dismiss after 8 seconds for non-critical errors
  useEffect(() => {
    if (error.severity !== 'critical') {
      const timer = setTimeout(() => {
        setVisible(false);
        setTimeout(() => onDismiss(error.id), 300);
      }, 8000);
      return () => clearTimeout(timer);
    }
  }, [error.id, error.severity, onDismiss]);

  if (!visible) return null;

  return (
    <div
      style={{
        ...styles.toast,
        borderColor: severity.borderColor,
        opacity: visible ? 1 : 0,
        transform: visible ? 'translateX(0)' : 'translateX(100%)',
        transition: 'opacity 0.3s, transform 0.3s',
      }}
    >
      <span style={{ ...styles.icon, color: severity.iconColor }}>
        {severity.icon === 'info' && 'i'}
        {severity.icon === 'warning' && '!'}
        {severity.icon === 'error' && 'X'}
        {severity.icon === 'error_outline' && '!!'}
      </span>
      <div style={styles.content}>
        <div style={styles.title}>{error.message}</div>
        {error.details && <div style={styles.message}>{error.details}</div>}
        <div style={styles.code}>Code: {error.code}</div>
        {error.actions && error.actions.length > 0 && (
          <div style={styles.actions}>
            {error.actions.map((action, i) => (
              <button
                key={i}
                style={{
                  ...styles.button,
                  ...(action.primary ? styles.primaryButton : styles.secondaryButton),
                }}
                onClick={action.action}
              >
                {action.label}
              </button>
            ))}
          </div>
        )}
        <span
          style={styles.reportLink}
          onClick={() => onReport(error.id)}
        >
          Report this issue
        </span>
      </div>
      <button
        style={styles.closeButton}
        onClick={() => {
          setVisible(false);
          setTimeout(() => onDismiss(error.id), 300);
        }}
        aria-label="Dismiss"
      >
        x
      </button>
    </div>
  );
}

// ============================================================================
// Error Modal
// ============================================================================

interface ErrorModalProps {
  error: AppError;
  onDismiss: (id: string) => void;
  onReport: (id: string) => void;
}

function ErrorModal({ error, onDismiss, onReport }: ErrorModalProps) {
  const [showDetails, setShowDetails] = useState(false);
  const [reportSent, setReportSent] = useState(false);
  const severity = severityStyles[error.severity];

  const handleReport = useCallback(() => {
    onReport(error.id);
    setReportSent(true);
  }, [error.id, onReport]);

  return (
    <div style={styles.modalOverlay} onClick={() => error.severity !== 'critical' && onDismiss(error.id)}>
      <div style={{ ...styles.modal, borderColor: severity.borderColor }} onClick={(e) => e.stopPropagation()}>
        <div style={{ ...styles.modalTitle, color: severity.iconColor }}>
          <span>{severity.icon === 'error_outline' ? 'Critical Error' : 'Error'}</span>
        </div>
        <div style={{ ...styles.title, fontSize: '16px', marginBottom: '8px' }}>{error.message}</div>
        {error.details && <div style={styles.message}>{error.details}</div>}
        <div style={styles.code}>Error Code: {error.code}</div>

        {(error.stack || error.originalError) && (
          <>
            <button
              style={{ ...styles.button, ...styles.secondaryButton, marginTop: '12px' }}
              onClick={() => setShowDetails(!showDetails)}
            >
              {showDetails ? 'Hide Details' : 'Show Details'}
            </button>
            {showDetails && (
              <div style={styles.modalDetails}>
                <pre style={{ margin: 0, whiteSpace: 'pre-wrap', wordBreak: 'break-all' }}>
                  {error.stack || JSON.stringify(error.originalError, null, 2)}
                </pre>
              </div>
            )}
          </>
        )}

        <div style={{ ...styles.actions, marginTop: '16px', justifyContent: 'flex-end' }}>
          {!reportSent ? (
            <button
              style={{ ...styles.button, ...styles.secondaryButton }}
              onClick={handleReport}
            >
              Report Issue
            </button>
          ) : (
            <span style={{ color: '#22c55e', fontSize: '12px' }}>Report sent. Thank you!</span>
          )}
          {error.actions?.map((action, i) => (
            <button
              key={i}
              style={{
                ...styles.button,
                ...(action.primary ? styles.primaryButton : styles.secondaryButton),
              }}
              onClick={action.action}
            >
              {action.label}
            </button>
          ))}
          <button
            style={{ ...styles.button, ...styles.primaryButton }}
            onClick={() => onDismiss(error.id)}
            disabled={error.severity === 'critical'}
          >
            {error.severity === 'critical' ? 'Reload App' : 'Dismiss'}
          </button>
        </div>
      </div>
    </div>
  );
}

// ============================================================================
// Main Component
// ============================================================================

interface ErrorNotificationProps {
  /** Maximum number of toasts to show */
  maxToasts?: number;
  /** Show modal for critical errors */
  showCriticalModal?: boolean;
}

export function ErrorNotification({
  maxToasts = 5,
  showCriticalModal = true,
}: ErrorNotificationProps) {
  const { activeErrors, dismissError, reportError, hasCriticalError } = useError();

  // Separate critical errors for modal display
  const criticalError = showCriticalModal
    ? activeErrors.find((e) => e.severity === 'critical')
    : undefined;

  // Filter non-critical errors for toast display
  const toastErrors = activeErrors
    .filter((e) => e.severity !== 'critical')
    .slice(-maxToasts);

  const handleDismiss = useCallback((id: string) => {
    dismissError(id);
  }, [dismissError]);

  const handleReport = useCallback(async (id: string) => {
    await reportError(id);
  }, [reportError]);

  // Handle critical error modal
  if (criticalError) {
    return (
      <ErrorModal
        error={criticalError}
        onDismiss={handleDismiss}
        onReport={handleReport}
      />
    );
  }

  // Render toast notifications
  return (
    <div style={styles.container}>
      <style>
        {`
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
        `}
      </style>
      {toastErrors.map((error) => (
        <Toast
          key={error.id}
          error={error}
          onDismiss={handleDismiss}
          onReport={handleReport}
        />
      ))}
    </div>
  );
}

export default ErrorNotification;
