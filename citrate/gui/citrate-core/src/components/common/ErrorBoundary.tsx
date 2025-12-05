/**
 * ErrorBoundary Components
 *
 * Provides error boundaries for graceful error handling throughout the chat UI.
 * Includes retry functionality and error reporting.
 */

import React, { Component, ReactNode, ErrorInfo } from 'react';
import { AlertTriangle, RefreshCw, Bug, Copy, Check, ChevronDown, ChevronUp } from 'lucide-react';

// =============================================================================
// Generic Error Boundary
// =============================================================================

interface ErrorBoundaryProps {
  children: ReactNode;
  fallback?: ReactNode;
  onError?: (error: Error, errorInfo: ErrorInfo) => void;
  onReset?: () => void;
}

interface ErrorBoundaryState {
  hasError: boolean;
  error: Error | null;
  errorInfo: ErrorInfo | null;
}

export class GenericErrorBoundary extends Component<ErrorBoundaryProps, ErrorBoundaryState> {
  constructor(props: ErrorBoundaryProps) {
    super(props);
    this.state = { hasError: false, error: null, errorInfo: null };
  }

  static getDerivedStateFromError(error: Error): Partial<ErrorBoundaryState> {
    return { hasError: true, error };
  }

  componentDidCatch(error: Error, errorInfo: ErrorInfo) {
    this.setState({ errorInfo });
    this.props.onError?.(error, errorInfo);
    console.error('[ErrorBoundary] Caught error:', error, errorInfo);
  }

  handleReset = () => {
    this.setState({ hasError: false, error: null, errorInfo: null });
    this.props.onReset?.();
  };

  render() {
    if (this.state.hasError) {
      if (this.props.fallback) {
        return this.props.fallback;
      }

      return (
        <ErrorFallback
          error={this.state.error}
          errorInfo={this.state.errorInfo}
          onReset={this.handleReset}
        />
      );
    }

    return this.props.children;
  }
}

// =============================================================================
// Chat Error Boundary
// =============================================================================

interface ChatErrorBoundaryProps {
  children: ReactNode;
  onError?: (error: Error, errorInfo: ErrorInfo) => void;
}

export class ChatErrorBoundary extends Component<
  ChatErrorBoundaryProps,
  ErrorBoundaryState
> {
  constructor(props: ChatErrorBoundaryProps) {
    super(props);
    this.state = { hasError: false, error: null, errorInfo: null };
  }

  static getDerivedStateFromError(error: Error): Partial<ErrorBoundaryState> {
    return { hasError: true, error };
  }

  componentDidCatch(error: Error, errorInfo: ErrorInfo) {
    this.setState({ errorInfo });
    this.props.onError?.(error, errorInfo);
    console.error('[ChatErrorBoundary] Caught error:', error, errorInfo);
  }

  handleReset = () => {
    this.setState({ hasError: false, error: null, errorInfo: null });
  };

  render() {
    if (this.state.hasError) {
      return (
        <div className="chat-error-fallback">
          <div className="error-content">
            <AlertTriangle size={48} />
            <h3>Something went wrong</h3>
            <p>The chat interface encountered an error.</p>
            <button className="btn-retry" onClick={this.handleReset}>
              <RefreshCw size={16} />
              <span>Try Again</span>
            </button>
          </div>

          <style jsx>{`
            .chat-error-fallback {
              flex: 1;
              display: flex;
              align-items: center;
              justify-content: center;
              padding: 2rem;
              background: #fef2f2;
            }

            .error-content {
              text-align: center;
              color: #991b1b;
            }

            .error-content h3 {
              margin: 1rem 0 0.5rem;
              font-size: 1.25rem;
            }

            .error-content p {
              margin: 0 0 1rem;
              color: #dc2626;
            }

            .btn-retry {
              display: inline-flex;
              align-items: center;
              gap: 0.5rem;
              padding: 0.75rem 1.5rem;
              background: #dc2626;
              color: white;
              border: none;
              border-radius: 0.5rem;
              font-size: 0.875rem;
              font-weight: 500;
              cursor: pointer;
              transition: background 0.2s;
            }

            .btn-retry:hover {
              background: #b91c1c;
            }
          `}</style>
        </div>
      );
    }

    return this.props.children;
  }
}

// =============================================================================
// Message Error Boundary
// =============================================================================

interface MessageErrorBoundaryProps {
  children: ReactNode;
  messageId: string;
}

export class MessageErrorBoundary extends Component<
  MessageErrorBoundaryProps,
  ErrorBoundaryState
> {
  constructor(props: MessageErrorBoundaryProps) {
    super(props);
    this.state = { hasError: false, error: null, errorInfo: null };
  }

  static getDerivedStateFromError(error: Error): Partial<ErrorBoundaryState> {
    return { hasError: true, error };
  }

  componentDidCatch(error: Error, errorInfo: ErrorInfo) {
    this.setState({ errorInfo });
    console.error(`[MessageErrorBoundary] Error in message ${this.props.messageId}:`, error);
  }

  render() {
    if (this.state.hasError) {
      return (
        <div className="message-error">
          <AlertTriangle size={16} />
          <span>Failed to render message</span>

          <style jsx>{`
            .message-error {
              display: flex;
              align-items: center;
              gap: 0.5rem;
              padding: 0.75rem 1rem;
              background: #fef2f2;
              border-radius: 0.5rem;
              color: #dc2626;
              font-size: 0.875rem;
            }
          `}</style>
        </div>
      );
    }

    return this.props.children;
  }
}

// =============================================================================
// Error Fallback Component
// =============================================================================

interface ErrorFallbackProps {
  error: Error | null;
  errorInfo: ErrorInfo | null;
  onReset?: () => void;
  onReport?: () => void;
}

export const ErrorFallback: React.FC<ErrorFallbackProps> = ({
  error,
  errorInfo,
  onReset,
  onReport,
}) => {
  const [showDetails, setShowDetails] = React.useState(false);
  const [copied, setCopied] = React.useState(false);

  const errorDetails = `Error: ${error?.message}\n\nStack: ${error?.stack}\n\nComponent Stack: ${errorInfo?.componentStack}`;

  const copyError = async () => {
    await navigator.clipboard.writeText(errorDetails);
    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
  };

  return (
    <div className="error-fallback">
      <div className="error-icon">
        <AlertTriangle size={48} />
      </div>

      <h2>Something went wrong</h2>
      <p className="error-message">{error?.message || 'An unexpected error occurred'}</p>

      <div className="error-actions">
        {onReset && (
          <button className="btn-primary" onClick={onReset}>
            <RefreshCw size={16} />
            <span>Try Again</span>
          </button>
        )}
        {onReport && (
          <button className="btn-secondary" onClick={onReport}>
            <Bug size={16} />
            <span>Report Issue</span>
          </button>
        )}
      </div>

      <button
        className="btn-details"
        onClick={() => setShowDetails(!showDetails)}
      >
        {showDetails ? <ChevronUp size={16} /> : <ChevronDown size={16} />}
        <span>{showDetails ? 'Hide Details' : 'Show Details'}</span>
      </button>

      {showDetails && (
        <div className="error-details">
          <div className="details-header">
            <span>Error Details</span>
            <button className="btn-copy" onClick={copyError}>
              {copied ? <Check size={14} /> : <Copy size={14} />}
              <span>{copied ? 'Copied!' : 'Copy'}</span>
            </button>
          </div>
          <pre>{errorDetails}</pre>
        </div>
      )}

      <style jsx>{`
        .error-fallback {
          display: flex;
          flex-direction: column;
          align-items: center;
          justify-content: center;
          padding: 3rem 2rem;
          text-align: center;
          background: #f9fafb;
          min-height: 300px;
        }

        .error-icon {
          color: #dc2626;
          margin-bottom: 1rem;
        }

        h2 {
          margin: 0 0 0.5rem;
          font-size: 1.5rem;
          color: #111827;
        }

        .error-message {
          margin: 0 0 1.5rem;
          color: #6b7280;
          max-width: 400px;
        }

        .error-actions {
          display: flex;
          gap: 0.75rem;
          margin-bottom: 1.5rem;
        }

        .btn-primary,
        .btn-secondary {
          display: inline-flex;
          align-items: center;
          gap: 0.5rem;
          padding: 0.75rem 1.25rem;
          border: none;
          border-radius: 0.5rem;
          font-size: 0.875rem;
          font-weight: 500;
          cursor: pointer;
          transition: all 0.2s;
        }

        .btn-primary {
          background: #ffa500;
          color: white;
        }

        .btn-primary:hover {
          background: #e69500;
        }

        .btn-secondary {
          background: white;
          border: 1px solid #e5e7eb;
          color: #374151;
        }

        .btn-secondary:hover {
          background: #f3f4f6;
        }

        .btn-details {
          display: inline-flex;
          align-items: center;
          gap: 0.375rem;
          padding: 0.5rem 0.75rem;
          background: none;
          border: none;
          color: #6b7280;
          font-size: 0.875rem;
          cursor: pointer;
          transition: color 0.2s;
        }

        .btn-details:hover {
          color: #374151;
        }

        .error-details {
          width: 100%;
          max-width: 600px;
          margin-top: 1rem;
          text-align: left;
        }

        .details-header {
          display: flex;
          justify-content: space-between;
          align-items: center;
          margin-bottom: 0.5rem;
          font-size: 0.75rem;
          font-weight: 600;
          color: #6b7280;
          text-transform: uppercase;
        }

        .btn-copy {
          display: inline-flex;
          align-items: center;
          gap: 0.25rem;
          padding: 0.25rem 0.5rem;
          background: none;
          border: none;
          color: #6b7280;
          font-size: 0.75rem;
          cursor: pointer;
          border-radius: 0.25rem;
        }

        .btn-copy:hover {
          background: #e5e7eb;
          color: #374151;
        }

        pre {
          background: #1e1e1e;
          color: #d4d4d4;
          padding: 1rem;
          border-radius: 0.5rem;
          font-size: 0.75rem;
          overflow-x: auto;
          white-space: pre-wrap;
          word-wrap: break-word;
          max-height: 200px;
          overflow-y: auto;
        }
      `}</style>
    </div>
  );
};

export default GenericErrorBoundary;
