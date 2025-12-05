/**
 * Error Context for GUI-wide error handling
 *
 * Provides centralized error management with:
 * - Error type classification (transient, blocking, warning)
 * - User-friendly message mapping
 * - Toast notifications for transient errors
 * - Modal dialogs for blocking errors
 * - Error logging to backend
 * - Error reporting support
 */

import React, { createContext, useContext, useState, useCallback, ReactNode } from 'react';

// ============================================================================
// Error Types
// ============================================================================

/** Error severity levels */
export type ErrorSeverity = 'info' | 'warning' | 'error' | 'critical';

/** Error category for classification */
export type ErrorCategory =
  | 'network'
  | 'blockchain'
  | 'wallet'
  | 'contract'
  | 'ipfs'
  | 'ai'
  | 'validation'
  | 'auth'
  | 'unknown';

/** Application error with structured data */
export interface AppError {
  /** Unique error ID for tracking */
  id: string;
  /** Error code for support reference */
  code: string;
  /** User-friendly message */
  message: string;
  /** Technical details (for debugging) */
  details?: string;
  /** Error severity */
  severity: ErrorSeverity;
  /** Error category */
  category: ErrorCategory;
  /** Timestamp when error occurred */
  timestamp: number;
  /** Whether error has been dismissed */
  dismissed: boolean;
  /** Suggested actions for user */
  actions?: ErrorAction[];
  /** Stack trace (development only) */
  stack?: string;
  /** Original error object */
  originalError?: Error;
}

/** Action button for error resolution */
export interface ErrorAction {
  label: string;
  action: () => void;
  primary?: boolean;
}

// ============================================================================
// Error Code Mapping
// ============================================================================

/** Map of error codes to user-friendly messages */
const ERROR_MESSAGES: Record<string, { message: string; suggestion: string }> = {
  // Network errors
  NETWORK_OFFLINE: {
    message: 'No internet connection',
    suggestion: 'Check your network connection and try again.',
  },
  NETWORK_TIMEOUT: {
    message: 'Request timed out',
    suggestion: 'The server took too long to respond. Please try again.',
  },
  RPC_UNAVAILABLE: {
    message: 'Blockchain node unavailable',
    suggestion: 'The blockchain node is not responding. Check if the node is running.',
  },
  RPC_ERROR: {
    message: 'Blockchain request failed',
    suggestion: 'There was an error communicating with the blockchain.',
  },

  // Wallet errors
  WALLET_LOCKED: {
    message: 'Wallet is locked',
    suggestion: 'Please unlock your wallet to continue.',
  },
  WALLET_NOT_FOUND: {
    message: 'No wallet found',
    suggestion: 'Create or import a wallet to get started.',
  },
  INSUFFICIENT_FUNDS: {
    message: 'Insufficient balance',
    suggestion: 'You need more funds to complete this transaction.',
  },
  INVALID_ADDRESS: {
    message: 'Invalid address',
    suggestion: 'Please check the address format and try again.',
  },

  // Transaction errors
  TX_REJECTED: {
    message: 'Transaction rejected',
    suggestion: 'The transaction was rejected by the network.',
  },
  TX_FAILED: {
    message: 'Transaction failed',
    suggestion: 'The transaction could not be completed.',
  },
  NONCE_TOO_LOW: {
    message: 'Transaction nonce error',
    suggestion: 'There may be a pending transaction. Please wait and try again.',
  },
  GAS_TOO_LOW: {
    message: 'Gas limit too low',
    suggestion: 'Increase the gas limit for this transaction.',
  },

  // Contract errors
  CONTRACT_REVERTED: {
    message: 'Smart contract error',
    suggestion: 'The contract rejected this operation.',
  },
  CONTRACT_NOT_FOUND: {
    message: 'Contract not found',
    suggestion: 'The smart contract could not be found at this address.',
  },

  // IPFS errors
  IPFS_UNAVAILABLE: {
    message: 'IPFS unavailable',
    suggestion: 'Could not connect to IPFS. Check your IPFS configuration.',
  },
  IPFS_UPLOAD_FAILED: {
    message: 'Upload failed',
    suggestion: 'Could not upload file to IPFS. Please try again.',
  },
  IPFS_PIN_FAILED: {
    message: 'Pinning failed',
    suggestion: 'Could not pin content to IPFS.',
  },

  // AI errors
  MODEL_NOT_FOUND: {
    message: 'Model not found',
    suggestion: 'The requested AI model could not be found.',
  },
  MODEL_LOAD_FAILED: {
    message: 'Failed to load model',
    suggestion: 'The AI model could not be loaded. It may be corrupted or unavailable.',
  },
  INFERENCE_FAILED: {
    message: 'AI inference failed',
    suggestion: 'The AI model encountered an error processing your request.',
  },
  INFERENCE_TIMEOUT: {
    message: 'AI request timed out',
    suggestion: 'The AI model took too long to respond. Try a smaller input.',
  },

  // Validation errors
  INVALID_INPUT: {
    message: 'Invalid input',
    suggestion: 'Please check your input and try again.',
  },
  MISSING_REQUIRED: {
    message: 'Required field missing',
    suggestion: 'Please fill in all required fields.',
  },

  // Generic errors
  UNKNOWN_ERROR: {
    message: 'An unexpected error occurred',
    suggestion: 'Please try again. If the problem persists, contact support.',
  },
};

// ============================================================================
// Error Context
// ============================================================================

interface ErrorContextValue {
  /** Current list of errors */
  errors: AppError[];
  /** Add a new error */
  addError: (error: Partial<AppError> | Error | string) => string;
  /** Remove an error by ID */
  removeError: (id: string) => void;
  /** Dismiss an error (mark as read but keep in list) */
  dismissError: (id: string) => void;
  /** Clear all errors */
  clearErrors: () => void;
  /** Get active (non-dismissed) errors */
  activeErrors: AppError[];
  /** Check if there are any critical errors */
  hasCriticalError: boolean;
  /** Report error to backend for logging */
  reportError: (id: string, additionalInfo?: string) => Promise<void>;
}

const ErrorContext = createContext<ErrorContextValue | undefined>(undefined);

// ============================================================================
// Error Provider
// ============================================================================

interface ErrorProviderProps {
  children: ReactNode;
}

/** Generate unique error ID */
function generateErrorId(): string {
  return `err_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`;
}

/** Classify error category from error message/code */
function classifyError(error: Partial<AppError> | Error | string): ErrorCategory {
  const message = typeof error === 'string' ? error : (error as Error).message || '';
  const lowerMessage = message.toLowerCase();

  if (lowerMessage.includes('network') || lowerMessage.includes('fetch') || lowerMessage.includes('timeout')) {
    return 'network';
  }
  if (lowerMessage.includes('wallet') || lowerMessage.includes('balance') || lowerMessage.includes('key')) {
    return 'wallet';
  }
  if (lowerMessage.includes('contract') || lowerMessage.includes('revert')) {
    return 'contract';
  }
  if (lowerMessage.includes('ipfs') || lowerMessage.includes('pin') || lowerMessage.includes('cid')) {
    return 'ipfs';
  }
  if (lowerMessage.includes('model') || lowerMessage.includes('inference') || lowerMessage.includes('ai')) {
    return 'ai';
  }
  if (lowerMessage.includes('rpc') || lowerMessage.includes('block') || lowerMessage.includes('transaction')) {
    return 'blockchain';
  }
  if (lowerMessage.includes('invalid') || lowerMessage.includes('required')) {
    return 'validation';
  }

  return 'unknown';
}

/** Map error to user-friendly format */
function mapErrorToAppError(error: Partial<AppError> | Error | string): AppError {
  const id = generateErrorId();
  const timestamp = Date.now();

  // Handle string errors
  if (typeof error === 'string') {
    const errorInfo = ERROR_MESSAGES[error] || ERROR_MESSAGES.UNKNOWN_ERROR;
    return {
      id,
      code: error,
      message: errorInfo.message,
      details: errorInfo.suggestion,
      severity: 'error',
      category: classifyError(error),
      timestamp,
      dismissed: false,
    };
  }

  // Handle Error objects
  if (error instanceof Error) {
    const category = classifyError(error);
    return {
      id,
      code: 'UNKNOWN_ERROR',
      message: error.message || 'An unexpected error occurred',
      details: ERROR_MESSAGES.UNKNOWN_ERROR.suggestion,
      severity: 'error',
      category,
      timestamp,
      dismissed: false,
      stack: error.stack,
      originalError: error,
    };
  }

  // Handle partial AppError objects
  const appError = error as Partial<AppError>;
  const code = appError.code || 'UNKNOWN_ERROR';
  const errorInfo = ERROR_MESSAGES[code];

  return {
    id: appError.id || id,
    code,
    message: appError.message || errorInfo?.message || 'An error occurred',
    details: appError.details || errorInfo?.suggestion,
    severity: appError.severity || 'error',
    category: appError.category || classifyError(appError),
    timestamp: appError.timestamp || timestamp,
    dismissed: appError.dismissed || false,
    actions: appError.actions,
    stack: appError.stack,
    originalError: appError.originalError,
  };
}

export function ErrorProvider({ children }: ErrorProviderProps) {
  const [errors, setErrors] = useState<AppError[]>([]);

  const addError = useCallback((error: Partial<AppError> | Error | string): string => {
    const appError = mapErrorToAppError(error);

    // Log error to console in development
    if (process.env.NODE_ENV === 'development') {
      console.error('[ErrorContext]', appError);
    }

    setErrors((prev) => {
      // Prevent duplicate errors (same code within 5 seconds)
      const isDuplicate = prev.some(
        (e) => e.code === appError.code && appError.timestamp - e.timestamp < 5000
      );
      if (isDuplicate) {
        return prev;
      }

      // Keep only last 50 errors
      const newErrors = [...prev, appError];
      if (newErrors.length > 50) {
        return newErrors.slice(-50);
      }
      return newErrors;
    });

    return appError.id;
  }, []);

  const removeError = useCallback((id: string) => {
    setErrors((prev) => prev.filter((e) => e.id !== id));
  }, []);

  const dismissError = useCallback((id: string) => {
    setErrors((prev) =>
      prev.map((e) => (e.id === id ? { ...e, dismissed: true } : e))
    );
  }, []);

  const clearErrors = useCallback(() => {
    setErrors([]);
  }, []);

  const activeErrors = errors.filter((e) => !e.dismissed);

  const hasCriticalError = errors.some((e) => e.severity === 'critical' && !e.dismissed);

  const reportError = useCallback(async (id: string, additionalInfo?: string) => {
    const error = errors.find((e) => e.id === id);
    if (!error) return;

    try {
      // In production, this would send to a backend endpoint
      // For now, we'll just log it
      const reportData = {
        errorId: error.id,
        code: error.code,
        message: error.message,
        category: error.category,
        severity: error.severity,
        timestamp: error.timestamp,
        additionalInfo,
        userAgent: navigator.userAgent,
        url: window.location.href,
        stack: error.stack,
      };

      console.log('[Error Report]', reportData);

      // TODO: Send to backend when endpoint is available
      // await fetch('/api/error-report', {
      //   method: 'POST',
      //   headers: { 'Content-Type': 'application/json' },
      //   body: JSON.stringify(reportData),
      // });
    } catch (e) {
      console.error('Failed to report error:', e);
    }
  }, [errors]);

  const value: ErrorContextValue = {
    errors,
    addError,
    removeError,
    dismissError,
    clearErrors,
    activeErrors,
    hasCriticalError,
    reportError,
  };

  return <ErrorContext.Provider value={value}>{children}</ErrorContext.Provider>;
}

// ============================================================================
// Hook
// ============================================================================

export function useError(): ErrorContextValue {
  const context = useContext(ErrorContext);
  if (!context) {
    throw new Error('useError must be used within an ErrorProvider');
  }
  return context;
}

// ============================================================================
// Utility Functions
// ============================================================================

/** Create a network error */
export function createNetworkError(message: string, details?: string): Partial<AppError> {
  return {
    code: 'NETWORK_ERROR',
    message,
    details,
    severity: 'error',
    category: 'network',
  };
}

/** Create a wallet error */
export function createWalletError(code: string, message?: string): Partial<AppError> {
  const errorInfo = ERROR_MESSAGES[code];
  return {
    code,
    message: message || errorInfo?.message || 'Wallet error',
    details: errorInfo?.suggestion,
    severity: 'error',
    category: 'wallet',
  };
}

/** Create a transaction error */
export function createTransactionError(code: string, txHash?: string): Partial<AppError> {
  const errorInfo = ERROR_MESSAGES[code];
  return {
    code,
    message: errorInfo?.message || 'Transaction failed',
    details: txHash ? `Transaction: ${txHash}\n${errorInfo?.suggestion}` : errorInfo?.suggestion,
    severity: 'error',
    category: 'blockchain',
  };
}

/** Create an IPFS error */
export function createIPFSError(code: string, cid?: string): Partial<AppError> {
  const errorInfo = ERROR_MESSAGES[code];
  return {
    code,
    message: errorInfo?.message || 'IPFS error',
    details: cid ? `CID: ${cid}\n${errorInfo?.suggestion}` : errorInfo?.suggestion,
    severity: 'error',
    category: 'ipfs',
  };
}

/** Create an AI error */
export function createAIError(code: string, modelId?: string): Partial<AppError> {
  const errorInfo = ERROR_MESSAGES[code];
  return {
    code,
    message: errorInfo?.message || 'AI error',
    details: modelId ? `Model: ${modelId}\n${errorInfo?.suggestion}` : errorInfo?.suggestion,
    severity: 'error',
    category: 'ai',
  };
}

export default ErrorContext;
