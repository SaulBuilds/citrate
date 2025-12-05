/**
 * LoadingStates Components
 *
 * Provides various loading state components including:
 * - Skeleton loaders for messages
 * - Spinners for operations
 * - Progress indicators for long operations
 */

import React from 'react';
import { Loader } from 'lucide-react';

// =============================================================================
// Spinner Component
// =============================================================================

interface SpinnerProps {
  size?: 'small' | 'medium' | 'large';
  color?: string;
  className?: string;
}

export const Spinner: React.FC<SpinnerProps> = ({
  size = 'medium',
  color = 'currentColor',
  className = '',
}) => {
  const sizeMap = {
    small: 16,
    medium: 24,
    large: 32,
  };

  return (
    <div className={`spinner ${className}`}>
      <Loader size={sizeMap[size]} color={color} />

      <style jsx>{`
        .spinner {
          display: inline-flex;
          animation: spin 1s linear infinite;
        }

        @keyframes spin {
          from {
            transform: rotate(0deg);
          }
          to {
            transform: rotate(360deg);
          }
        }
      `}</style>
    </div>
  );
};

// =============================================================================
// Loading Overlay
// =============================================================================

interface LoadingOverlayProps {
  visible: boolean;
  message?: string;
  transparent?: boolean;
}

export const LoadingOverlay: React.FC<LoadingOverlayProps> = ({
  visible,
  message = 'Loading...',
  transparent = false,
}) => {
  if (!visible) return null;

  return (
    <div className={`loading-overlay ${transparent ? 'transparent' : ''}`}>
      <div className="loading-content">
        <Spinner size="large" />
        {message && <p>{message}</p>}
      </div>

      <style jsx>{`
        .loading-overlay {
          position: absolute;
          inset: 0;
          display: flex;
          align-items: center;
          justify-content: center;
          background: rgba(255, 255, 255, 0.9);
          z-index: 50;
        }

        .loading-overlay.transparent {
          background: rgba(255, 255, 255, 0.7);
        }

        .loading-content {
          display: flex;
          flex-direction: column;
          align-items: center;
          gap: 1rem;
        }

        .loading-content p {
          margin: 0;
          font-size: 0.875rem;
          color: #6b7280;
        }
      `}</style>
    </div>
  );
};

// =============================================================================
// Message Skeleton
// =============================================================================

interface MessageSkeletonProps {
  role?: 'user' | 'assistant';
  lines?: number;
}

export const MessageSkeleton: React.FC<MessageSkeletonProps> = ({
  role = 'assistant',
  lines = 3,
}) => {
  return (
    <div className={`message-skeleton ${role}`}>
      <div className="avatar-skeleton" />
      <div className="content-skeleton">
        <div className="header-skeleton">
          <div className="name-skeleton" />
          <div className="time-skeleton" />
        </div>
        {Array.from({ length: lines }).map((_, i) => (
          <div
            key={i}
            className="line-skeleton"
            style={{ width: `${Math.random() * 40 + 60}%` }}
          />
        ))}
      </div>

      <style jsx>{`
        .message-skeleton {
          display: flex;
          gap: 0.75rem;
          padding: 0.5rem 0;
        }

        .message-skeleton.user {
          flex-direction: row-reverse;
        }

        .avatar-skeleton {
          width: 36px;
          height: 36px;
          border-radius: 50%;
          background: linear-gradient(
            90deg,
            #f0f0f0 25%,
            #e0e0e0 50%,
            #f0f0f0 75%
          );
          background-size: 200% 100%;
          animation: shimmer 1.5s infinite;
        }

        .content-skeleton {
          flex: 1;
          max-width: 70%;
          padding: 0.875rem 1rem;
          background: #f3f4f6;
          border-radius: 1rem;
        }

        .header-skeleton {
          display: flex;
          gap: 0.5rem;
          margin-bottom: 0.5rem;
        }

        .name-skeleton {
          width: 60px;
          height: 12px;
          border-radius: 4px;
          background: linear-gradient(
            90deg,
            #e5e7eb 25%,
            #d1d5db 50%,
            #e5e7eb 75%
          );
          background-size: 200% 100%;
          animation: shimmer 1.5s infinite;
        }

        .time-skeleton {
          width: 40px;
          height: 12px;
          border-radius: 4px;
          background: linear-gradient(
            90deg,
            #e5e7eb 25%,
            #d1d5db 50%,
            #e5e7eb 75%
          );
          background-size: 200% 100%;
          animation: shimmer 1.5s infinite;
        }

        .line-skeleton {
          height: 14px;
          border-radius: 4px;
          margin-bottom: 0.5rem;
          background: linear-gradient(
            90deg,
            #e5e7eb 25%,
            #d1d5db 50%,
            #e5e7eb 75%
          );
          background-size: 200% 100%;
          animation: shimmer 1.5s infinite;
        }

        .line-skeleton:last-child {
          margin-bottom: 0;
        }

        @keyframes shimmer {
          0% {
            background-position: 200% 0;
          }
          100% {
            background-position: -200% 0;
          }
        }
      `}</style>
    </div>
  );
};

// =============================================================================
// Chat Loading State
// =============================================================================

interface ChatLoadingProps {
  count?: number;
}

export const ChatLoading: React.FC<ChatLoadingProps> = ({ count = 3 }) => {
  return (
    <div className="chat-loading">
      {Array.from({ length: count }).map((_, i) => (
        <MessageSkeleton
          key={i}
          role={i % 2 === 0 ? 'assistant' : 'user'}
          lines={Math.floor(Math.random() * 2) + 2}
        />
      ))}

      <style jsx>{`
        .chat-loading {
          padding: 1rem;
          display: flex;
          flex-direction: column;
          gap: 0.5rem;
        }
      `}</style>
    </div>
  );
};

// =============================================================================
// Progress Bar
// =============================================================================

interface ProgressBarProps {
  progress: number; // 0-100
  showLabel?: boolean;
  height?: number;
  color?: string;
}

export const ProgressBar: React.FC<ProgressBarProps> = ({
  progress,
  showLabel = true,
  height = 8,
  color = '#ffa500',
}) => {
  const clampedProgress = Math.min(100, Math.max(0, progress));

  return (
    <div className="progress-container">
      <div className="progress-bar" style={{ height }}>
        <div
          className="progress-fill"
          style={{ width: `${clampedProgress}%`, backgroundColor: color }}
        />
      </div>
      {showLabel && <span className="progress-label">{Math.round(clampedProgress)}%</span>}

      <style jsx>{`
        .progress-container {
          display: flex;
          align-items: center;
          gap: 0.75rem;
          width: 100%;
        }

        .progress-bar {
          flex: 1;
          background: #e5e7eb;
          border-radius: 9999px;
          overflow: hidden;
        }

        .progress-fill {
          height: 100%;
          border-radius: 9999px;
          transition: width 0.3s ease-out;
        }

        .progress-label {
          font-size: 0.75rem;
          font-weight: 500;
          color: #6b7280;
          min-width: 36px;
          text-align: right;
        }
      `}</style>
    </div>
  );
};

// =============================================================================
// Indeterminate Progress
// =============================================================================

interface IndeterminateProgressProps {
  height?: number;
  color?: string;
}

export const IndeterminateProgress: React.FC<IndeterminateProgressProps> = ({
  height = 4,
  color = '#ffa500',
}) => {
  return (
    <div className="indeterminate-progress" style={{ height }}>
      <div className="progress-bar" style={{ backgroundColor: color }} />

      <style jsx>{`
        .indeterminate-progress {
          width: 100%;
          background: #e5e7eb;
          overflow: hidden;
          border-radius: 9999px;
        }

        .progress-bar {
          width: 30%;
          height: 100%;
          border-radius: 9999px;
          animation: indeterminate 1.5s infinite ease-in-out;
        }

        @keyframes indeterminate {
          0% {
            transform: translateX(-100%);
          }
          100% {
            transform: translateX(400%);
          }
        }
      `}</style>
    </div>
  );
};

// =============================================================================
// Typing Indicator
// =============================================================================

export const TypingIndicator: React.FC = () => {
  return (
    <div className="typing-indicator">
      <span />
      <span />
      <span />

      <style jsx>{`
        .typing-indicator {
          display: flex;
          gap: 4px;
          align-items: center;
          padding: 8px 12px;
        }

        .typing-indicator span {
          width: 8px;
          height: 8px;
          border-radius: 50%;
          background: #9ca3af;
          animation: bounce 1.4s infinite ease-in-out;
        }

        .typing-indicator span:nth-child(1) {
          animation-delay: -0.32s;
        }

        .typing-indicator span:nth-child(2) {
          animation-delay: -0.16s;
        }

        @keyframes bounce {
          0%,
          80%,
          100% {
            transform: scale(0.8);
            opacity: 0.5;
          }
          40% {
            transform: scale(1);
            opacity: 1;
          }
        }
      `}</style>
    </div>
  );
};

// =============================================================================
// Pulse Animation
// =============================================================================

interface PulseProps {
  children: React.ReactNode;
  active?: boolean;
}

export const Pulse: React.FC<PulseProps> = ({ children, active = true }) => {
  return (
    <div className={`pulse-wrapper ${active ? 'active' : ''}`}>
      {children}

      <style jsx>{`
        .pulse-wrapper {
          display: inline-block;
        }

        .pulse-wrapper.active {
          animation: pulse 2s infinite;
        }

        @keyframes pulse {
          0%,
          100% {
            opacity: 1;
          }
          50% {
            opacity: 0.5;
          }
        }
      `}</style>
    </div>
  );
};

export default {
  Spinner,
  LoadingOverlay,
  MessageSkeleton,
  ChatLoading,
  ProgressBar,
  IndeterminateProgress,
  TypingIndicator,
  Pulse,
};
