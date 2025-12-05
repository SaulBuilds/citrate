/**
 * ChatHeader Component
 *
 * Header for the chat interface showing session info,
 * connection status, and action buttons.
 */

import React from 'react';
import {
  MessageSquare,
  Plus,
  Trash2,
  Settings,
  Menu,
  X,
  Wifi,
  WifiOff,
  Loader,
} from 'lucide-react';

interface ChatHeaderProps {
  sessionId: string | null;
  isConnected: boolean;
  isStreaming: boolean;
  onNewSession: () => void;
  onClearHistory: () => void;
  onToggleSessionList?: () => void;
  onOpenSettings?: () => void;
  showSessionListButton?: boolean;
  sessionListOpen?: boolean;
}

export const ChatHeader: React.FC<ChatHeaderProps> = ({
  sessionId,
  isConnected,
  isStreaming,
  onNewSession,
  onClearHistory,
  onToggleSessionList,
  onOpenSettings,
  showSessionListButton = false,
  sessionListOpen = false,
}) => {
  return (
    <header className="chat-header">
      <div className="header-left">
        {showSessionListButton && (
          <button
            className="btn-icon"
            onClick={onToggleSessionList}
            title={sessionListOpen ? 'Hide conversations' : 'Show conversations'}
          >
            {sessionListOpen ? <X size={20} /> : <Menu size={20} />}
          </button>
        )}

        <div className="header-title">
          <MessageSquare size={22} className="title-icon" />
          <div className="title-text">
            <h1>AI Assistant</h1>
            {sessionId && (
              <span className="session-id">Session: {sessionId.slice(0, 8)}</span>
            )}
          </div>
        </div>
      </div>

      <div className="header-center">
        <div className={`connection-status ${isConnected ? 'connected' : 'disconnected'}`}>
          {isConnected ? (
            <>
              <Wifi size={14} />
              <span>Connected</span>
            </>
          ) : (
            <>
              <WifiOff size={14} />
              <span>Disconnected</span>
            </>
          )}
        </div>

        {isStreaming && (
          <div className="streaming-indicator">
            <Loader size={14} className="spinning" />
            <span>Thinking...</span>
          </div>
        )}
      </div>

      <div className="header-right">
        <button
          className="btn-icon"
          onClick={onNewSession}
          title="New conversation"
          disabled={isStreaming}
        >
          <Plus size={20} />
        </button>

        <button
          className="btn-icon"
          onClick={onClearHistory}
          title="Clear history"
          disabled={isStreaming || !sessionId}
        >
          <Trash2 size={20} />
        </button>

        {onOpenSettings && (
          <button className="btn-icon" onClick={onOpenSettings} title="Settings">
            <Settings size={20} />
          </button>
        )}
      </div>

      <style jsx>{`
        .chat-header {
          display: flex;
          align-items: center;
          justify-content: space-between;
          padding: 0.75rem 1rem;
          background: white;
          border-bottom: 1px solid #e5e7eb;
          min-height: 60px;
        }

        .header-left {
          display: flex;
          align-items: center;
          gap: 0.75rem;
        }

        .header-title {
          display: flex;
          align-items: center;
          gap: 0.75rem;
        }

        .title-icon {
          color: #ffa500;
        }

        .title-text {
          display: flex;
          flex-direction: column;
        }

        .title-text h1 {
          margin: 0;
          font-size: 1rem;
          font-weight: 600;
          color: #111827;
          line-height: 1.2;
        }

        .session-id {
          font-size: 0.75rem;
          color: #9ca3af;
          font-family: monospace;
        }

        .header-center {
          display: flex;
          align-items: center;
          gap: 1rem;
        }

        .connection-status {
          display: flex;
          align-items: center;
          gap: 0.375rem;
          padding: 0.25rem 0.5rem;
          border-radius: 9999px;
          font-size: 0.75rem;
          font-weight: 500;
        }

        .connection-status.connected {
          background: #d1fae5;
          color: #059669;
        }

        .connection-status.disconnected {
          background: #fee2e2;
          color: #dc2626;
        }

        .streaming-indicator {
          display: flex;
          align-items: center;
          gap: 0.375rem;
          padding: 0.25rem 0.5rem;
          background: #fef3c7;
          color: #d97706;
          border-radius: 9999px;
          font-size: 0.75rem;
          font-weight: 500;
        }

        .header-right {
          display: flex;
          align-items: center;
          gap: 0.25rem;
        }

        .btn-icon {
          display: flex;
          align-items: center;
          justify-content: center;
          width: 36px;
          height: 36px;
          background: none;
          border: none;
          border-radius: 0.5rem;
          cursor: pointer;
          color: #6b7280;
          transition: all 0.2s;
        }

        .btn-icon:hover:not(:disabled) {
          background: #f3f4f6;
          color: #374151;
        }

        .btn-icon:disabled {
          opacity: 0.5;
          cursor: not-allowed;
        }

        .spinning {
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

        @media (max-width: 640px) {
          .header-center {
            display: none;
          }

          .session-id {
            display: none;
          }
        }
      `}</style>
    </header>
  );
};

export default ChatHeader;
