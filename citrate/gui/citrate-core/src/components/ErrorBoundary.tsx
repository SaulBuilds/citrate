/**
 * ErrorBoundary Component
 *
 * Catches JavaScript errors in child component tree and displays fallback UI.
 * Prevents the entire app from crashing when component errors occur.
 *
 * Usage:
 * <ErrorBoundary>
 *   <App />
 * </ErrorBoundary>
 */

import { Component, ErrorInfo, ReactNode } from 'react';
import { AlertTriangle, RefreshCw, Home } from 'lucide-react';

interface Props {
  children: ReactNode;
  fallback?: ReactNode;
  onError?: (error: Error, errorInfo: ErrorInfo) => void;
}

interface State {
  hasError: boolean;
  error: Error | null;
  errorInfo: ErrorInfo | null;
}

class ErrorBoundary extends Component<Props, State> {
  constructor(props: Props) {
    super(props);
    this.state = {
      hasError: false,
      error: null,
      errorInfo: null
    };
  }

  /**
   * Update state when error is caught
   * This lifecycle is called during the "render" phase
   */
  static getDerivedStateFromError(error: Error): State {
    return {
      hasError: true,
      error,
      errorInfo: null
    };
  }

  /**
   * Log error details and call optional error handler
   * This lifecycle is called during the "commit" phase
   */
  componentDidCatch(error: Error, errorInfo: ErrorInfo): void {
    console.error('ErrorBoundary caught an error:', error);
    console.error('Error info:', errorInfo);
    console.error('Component stack:', errorInfo.componentStack);

    // Update state with error info
    this.setState({
      errorInfo
    });

    // Call optional error handler prop
    if (this.props.onError) {
      this.props.onError(error, errorInfo);
    }
  }

  /**
   * Reset error state and attempt to recover
   */
  handleReset = (): void => {
    this.setState({
      hasError: false,
      error: null,
      errorInfo: null
    });
  };

  /**
   * Reload the entire application
   */
  handleReload = (): void => {
    window.location.reload();
  };

  render(): ReactNode {
    if (this.state.hasError) {
      // If custom fallback provided, use it
      if (this.props.fallback) {
        return this.props.fallback;
      }

      // Otherwise, render default error UI
      const { error, errorInfo } = this.state;

      return (
        <div className="error-boundary-container">
          <div className="error-boundary-content">
            {/* Error Icon */}
            <div className="error-icon">
              <AlertTriangle size={64} />
            </div>

            {/* Error Title */}
            <h1 className="error-title">Something went wrong</h1>

            {/* Error Message */}
            <p className="error-message">
              We're sorry for the inconvenience. The application encountered an unexpected error.
            </p>

            {/* Error Details (collapsible) */}
            {error && (
              <details className="error-details">
                <summary className="error-details-summary">
                  Technical Details (click to expand)
                </summary>
                <div className="error-details-content">
                  <div className="error-section">
                    <strong>Error:</strong>
                    <pre className="error-pre">{error.toString()}</pre>
                  </div>
                  {errorInfo && (
                    <div className="error-section">
                      <strong>Component Stack:</strong>
                      <pre className="error-pre">{errorInfo.componentStack}</pre>
                    </div>
                  )}
                </div>
              </details>
            )}

            {/* Action Buttons */}
            <div className="error-actions">
              <button
                onClick={this.handleReset}
                className="btn btn-primary error-btn"
              >
                <RefreshCw size={18} />
                Try Again
              </button>
              <button
                onClick={this.handleReload}
                className="btn btn-secondary error-btn"
              >
                <Home size={18} />
                Reload Application
              </button>
            </div>

            {/* Help Text */}
            <p className="error-help">
              If this problem persists, please report it on our GitHub repository.
            </p>
          </div>

          <style>{`
            .error-boundary-container {
              min-height: 100vh;
              display: flex;
              align-items: center;
              justify-content: center;
              background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
              padding: 2rem;
            }

            .error-boundary-content {
              background: white;
              border-radius: 12px;
              padding: 3rem;
              max-width: 600px;
              width: 100%;
              box-shadow: 0 20px 60px rgba(0, 0, 0, 0.3);
              text-align: center;
            }

            .error-icon {
              color: #ef4444;
              margin-bottom: 1.5rem;
              display: flex;
              justify-content: center;
            }

            .error-title {
              font-size: 2rem;
              font-weight: 700;
              color: #1f2937;
              margin: 0 0 1rem 0;
            }

            .error-message {
              font-size: 1.125rem;
              color: #6b7280;
              margin: 0 0 2rem 0;
              line-height: 1.6;
            }

            .error-details {
              background: #f9fafb;
              border: 1px solid #e5e7eb;
              border-radius: 8px;
              padding: 1rem;
              margin-bottom: 2rem;
              text-align: left;
            }

            .error-details-summary {
              cursor: pointer;
              font-weight: 600;
              color: #4b5563;
              user-select: none;
              list-style: none;
              display: flex;
              align-items: center;
              gap: 0.5rem;
            }

            .error-details-summary::-webkit-details-marker {
              display: none;
            }

            .error-details-summary::before {
              content: '▶';
              display: inline-block;
              transition: transform 0.2s;
            }

            .error-details[open] .error-details-summary::before {
              transform: rotate(90deg);
            }

            .error-details-content {
              margin-top: 1rem;
              max-height: 300px;
              overflow-y: auto;
            }

            .error-section {
              margin-bottom: 1rem;
            }

            .error-section:last-child {
              margin-bottom: 0;
            }

            .error-section strong {
              display: block;
              margin-bottom: 0.5rem;
              color: #374151;
              font-size: 0.875rem;
            }

            .error-pre {
              background: #1f2937;
              color: #f9fafb;
              padding: 1rem;
              border-radius: 6px;
              font-size: 0.75rem;
              overflow-x: auto;
              margin: 0;
              font-family: 'Courier New', Courier, monospace;
              white-space: pre-wrap;
              word-break: break-word;
            }

            .error-actions {
              display: flex;
              gap: 1rem;
              justify-content: center;
              margin-bottom: 1.5rem;
            }

            .error-btn {
              display: flex;
              align-items: center;
              gap: 0.5rem;
              padding: 0.75rem 1.5rem;
              font-size: 1rem;
              font-weight: 600;
              border-radius: 8px;
              cursor: pointer;
              transition: all 0.2s;
              border: none;
            }

            .btn-primary.error-btn {
              background: #3b82f6;
              color: white;
            }

            .btn-primary.error-btn:hover {
              background: #2563eb;
              transform: translateY(-1px);
              box-shadow: 0 4px 12px rgba(59, 130, 246, 0.4);
            }

            .btn-secondary.error-btn {
              background: #6b7280;
              color: white;
            }

            .btn-secondary.error-btn:hover {
              background: #4b5563;
              transform: translateY(-1px);
              box-shadow: 0 4px 12px rgba(107, 114, 128, 0.4);
            }

            .error-help {
              font-size: 0.875rem;
              color: #9ca3af;
              margin: 0;
            }

            @media (max-width: 640px) {
              .error-boundary-container {
                padding: 1rem;
              }

              .error-boundary-content {
                padding: 2rem 1.5rem;
              }

              .error-title {
                font-size: 1.5rem;
              }

              .error-actions {
                flex-direction: column;
              }

              .error-btn {
                width: 100%;
                justify-content: center;
              }
            }
          `}</style>
        </div>
      );
    }

    // No error, render children normally
    return this.props.children;
  }
}

export default ErrorBoundary;
