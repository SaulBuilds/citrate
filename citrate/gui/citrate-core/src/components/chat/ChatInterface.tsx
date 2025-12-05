/**
 * ChatInterface Component
 *
 * Main chat container that houses the message thread, input area,
 * and manages the overall chat layout for the AI agent interface.
 */

import React, { useState, useRef, useEffect, useCallback } from 'react';
import {
  MessageSquare,
  Plus,
  Trash2,
  ChevronLeft,
  ChevronRight,
  RefreshCw,
} from 'lucide-react';
import { useAgent, useSessions, useMessages } from '../../contexts/AgentContext';
import { MessageThread } from './MessageThread';
import { MessageInput } from './MessageInput';
import { ChatHeader } from './ChatHeader';

interface ChatInterfaceProps {
  /** Whether to show the sidebar toggle */
  showSidebarToggle?: boolean;
  /** Callback when sidebar visibility changes */
  onSidebarToggle?: (visible: boolean) => void;
  /** Initial sidebar visibility */
  sidebarVisible?: boolean;
}

export const ChatInterface: React.FC<ChatInterfaceProps> = ({
  showSidebarToggle = true,
  onSidebarToggle,
  sidebarVisible = false,
}) => {
  const { isInitialized, isConnected, refreshStatus } = useAgent();
  const {
    currentSessionId,
    sessions,
    createSession,
    switchSession,
    deleteSession,
  } = useSessions();
  const { messages, isStreaming, sendMessage, clearHistory } = useMessages();

  const [showSessionList, setShowSessionList] = useState(false);
  const [isCreatingSession, setIsCreatingSession] = useState(false);
  const messagesEndRef = useRef<HTMLDivElement>(null);

  // Auto-scroll to bottom when new messages arrive
  useEffect(() => {
    if (messagesEndRef.current) {
      messagesEndRef.current.scrollIntoView({ behavior: 'smooth' });
    }
  }, [messages]);

  // Create initial session if none exists
  useEffect(() => {
    if (isInitialized && !currentSessionId && sessions.length === 0) {
      handleCreateSession();
    }
  }, [isInitialized, currentSessionId, sessions.length]);

  const handleCreateSession = useCallback(async () => {
    setIsCreatingSession(true);
    try {
      await createSession();
    } finally {
      setIsCreatingSession(false);
    }
  }, [createSession]);

  const handleSendMessage = useCallback(
    async (content: string) => {
      if (!content.trim()) return;
      await sendMessage(content);
    },
    [sendMessage]
  );

  const handleClearHistory = useCallback(async () => {
    await clearHistory();
  }, [clearHistory]);

  const handleDeleteSession = useCallback(
    async (sessionId: string) => {
      const confirmed = window.confirm(
        'Are you sure you want to delete this conversation?'
      );
      if (confirmed) {
        await deleteSession(sessionId);
        if (sessions.length <= 1) {
          await createSession();
        }
      }
    },
    [deleteSession, sessions.length, createSession]
  );

  const formatSessionTime = (timestamp: number) => {
    const date = new Date(timestamp);
    const now = new Date();
    const diffDays = Math.floor((now.getTime() - date.getTime()) / (1000 * 60 * 60 * 24));

    if (diffDays === 0) {
      return date.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' });
    } else if (diffDays === 1) {
      return 'Yesterday';
    } else if (diffDays < 7) {
      return date.toLocaleDateString([], { weekday: 'short' });
    } else {
      return date.toLocaleDateString([], { month: 'short', day: 'numeric' });
    }
  };

  return (
    <div className="chat-interface">
      {/* Session List Sidebar */}
      {showSessionList && (
        <div className="session-sidebar">
          <div className="session-sidebar-header">
            <h3>Conversations</h3>
            <button
              className="btn-icon"
              onClick={handleCreateSession}
              disabled={isCreatingSession}
              title="New conversation"
            >
              <Plus size={18} />
            </button>
          </div>

          <div className="session-list">
            {sessions.map((session) => (
              <div
                key={session.id}
                className={`session-item ${
                  session.id === currentSessionId ? 'active' : ''
                }`}
                onClick={() => switchSession(session.id)}
              >
                <div className="session-info">
                  <span className="session-title">
                    Conversation {session.id.slice(0, 8)}
                  </span>
                  <span className="session-time">
                    {formatSessionTime(session.lastActivity)}
                  </span>
                </div>
                <div className="session-meta">
                  <span className="message-count">{session.messageCount} messages</span>
                  <button
                    className="btn-icon-small"
                    onClick={(e) => {
                      e.stopPropagation();
                      handleDeleteSession(session.id);
                    }}
                    title="Delete conversation"
                  >
                    <Trash2 size={14} />
                  </button>
                </div>
              </div>
            ))}

            {sessions.length === 0 && (
              <div className="empty-sessions">
                <MessageSquare size={32} />
                <p>No conversations yet</p>
                <button className="btn-primary" onClick={handleCreateSession}>
                  Start a conversation
                </button>
              </div>
            )}
          </div>
        </div>
      )}

      {/* Main Chat Area */}
      <div className="chat-main">
        <ChatHeader
          sessionId={currentSessionId}
          isConnected={isConnected}
          isStreaming={isStreaming}
          onNewSession={handleCreateSession}
          onClearHistory={handleClearHistory}
          onToggleSessionList={() => setShowSessionList(!showSessionList)}
          showSessionListButton={true}
          sessionListOpen={showSessionList}
        />

        {!isInitialized ? (
          <div className="chat-loading">
            <RefreshCw className="spinning" size={32} />
            <p>Initializing agent...</p>
          </div>
        ) : !isConnected ? (
          <div className="chat-error">
            <MessageSquare size={48} />
            <h3>Agent Not Available</h3>
            <p>The AI agent is not connected. Please check your configuration.</p>
            <button className="btn-primary" onClick={refreshStatus}>
              Retry Connection
            </button>
          </div>
        ) : (
          <>
            <MessageThread messages={messages} isStreaming={isStreaming} />
            <div ref={messagesEndRef} />
            <MessageInput
              onSend={handleSendMessage}
              disabled={!currentSessionId || isStreaming}
              isStreaming={isStreaming}
            />
          </>
        )}
      </div>

      {/* Sidebar Toggle */}
      {showSidebarToggle && (
        <button
          className="sidebar-toggle"
          onClick={() => onSidebarToggle?.(!sidebarVisible)}
          title={sidebarVisible ? 'Hide status' : 'Show status'}
        >
          {sidebarVisible ? <ChevronRight size={20} /> : <ChevronLeft size={20} />}
        </button>
      )}

      <style jsx>{`
        .chat-interface {
          display: flex;
          height: 100%;
          background: #f9fafb;
          position: relative;
        }

        /* Session Sidebar */
        .session-sidebar {
          width: 280px;
          background: white;
          border-right: 1px solid #e5e7eb;
          display: flex;
          flex-direction: column;
        }

        .session-sidebar-header {
          display: flex;
          justify-content: space-between;
          align-items: center;
          padding: 1rem;
          border-bottom: 1px solid #e5e7eb;
        }

        .session-sidebar-header h3 {
          margin: 0;
          font-size: 0.875rem;
          font-weight: 600;
          color: #374151;
        }

        .session-list {
          flex: 1;
          overflow-y: auto;
          padding: 0.5rem;
        }

        .session-item {
          display: flex;
          flex-direction: column;
          padding: 0.75rem;
          margin-bottom: 0.25rem;
          border-radius: 0.5rem;
          cursor: pointer;
          transition: all 0.2s;
        }

        .session-item:hover {
          background: #f3f4f6;
        }

        .session-item.active {
          background: #ffa50015;
          border: 1px solid #ffa50030;
        }

        .session-info {
          display: flex;
          justify-content: space-between;
          align-items: center;
          margin-bottom: 0.25rem;
        }

        .session-title {
          font-size: 0.875rem;
          font-weight: 500;
          color: #111827;
        }

        .session-time {
          font-size: 0.75rem;
          color: #6b7280;
        }

        .session-meta {
          display: flex;
          justify-content: space-between;
          align-items: center;
        }

        .message-count {
          font-size: 0.75rem;
          color: #9ca3af;
        }

        .empty-sessions {
          display: flex;
          flex-direction: column;
          align-items: center;
          justify-content: center;
          padding: 2rem;
          color: #9ca3af;
          text-align: center;
        }

        .empty-sessions p {
          margin: 0.75rem 0 1rem;
          font-size: 0.875rem;
        }

        /* Main Chat Area */
        .chat-main {
          flex: 1;
          display: flex;
          flex-direction: column;
          min-width: 0;
        }

        .chat-loading,
        .chat-error {
          flex: 1;
          display: flex;
          flex-direction: column;
          align-items: center;
          justify-content: center;
          color: #6b7280;
          text-align: center;
          padding: 2rem;
        }

        .chat-loading p,
        .chat-error p {
          margin-top: 1rem;
          font-size: 0.875rem;
        }

        .chat-error h3 {
          margin: 1rem 0 0.5rem;
          color: #374151;
        }

        /* Sidebar Toggle */
        .sidebar-toggle {
          position: absolute;
          right: 0;
          top: 50%;
          transform: translateY(-50%);
          background: white;
          border: 1px solid #e5e7eb;
          border-right: none;
          border-radius: 0.5rem 0 0 0.5rem;
          padding: 0.5rem 0.25rem;
          cursor: pointer;
          color: #6b7280;
          transition: all 0.2s;
          z-index: 10;
        }

        .sidebar-toggle:hover {
          background: #f3f4f6;
          color: #374151;
        }

        /* Buttons */
        .btn-icon {
          display: flex;
          align-items: center;
          justify-content: center;
          width: 32px;
          height: 32px;
          background: none;
          border: none;
          border-radius: 0.375rem;
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

        .btn-icon-small {
          display: flex;
          align-items: center;
          justify-content: center;
          width: 24px;
          height: 24px;
          background: none;
          border: none;
          border-radius: 0.25rem;
          cursor: pointer;
          color: #9ca3af;
          opacity: 0;
          transition: all 0.2s;
        }

        .session-item:hover .btn-icon-small {
          opacity: 1;
        }

        .btn-icon-small:hover {
          background: #fee2e2;
          color: #dc2626;
        }

        .btn-primary {
          display: inline-flex;
          align-items: center;
          gap: 0.5rem;
          padding: 0.5rem 1rem;
          background: #ffa500;
          color: white;
          border: none;
          border-radius: 0.5rem;
          font-size: 0.875rem;
          font-weight: 500;
          cursor: pointer;
          transition: all 0.2s;
        }

        .btn-primary:hover {
          background: #e69500;
        }

        /* Animations */
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

        /* Responsive */
        @media (max-width: 768px) {
          .session-sidebar {
            position: absolute;
            left: 0;
            top: 0;
            bottom: 0;
            z-index: 20;
            box-shadow: 2px 0 8px rgba(0, 0, 0, 0.1);
          }
        }
      `}</style>
    </div>
  );
};

export default ChatInterface;
