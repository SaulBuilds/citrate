/**
 * MessageThread Component
 *
 * Displays the conversation history with support for:
 * - Different message types (user, assistant, system, tool)
 * - Real-time streaming with cursor animation
 * - Markdown rendering
 * - Message actions (copy, etc.)
 */

import React, { useRef, useEffect } from 'react';
import {
  User,
  Bot,
  Cpu,
  Wrench,
  Copy,
  Check,
  RefreshCw,
} from 'lucide-react';
import { Message, MessageRole, BalanceResult } from '../../types/agent';
import { TransactionCard } from './TransactionCard';
import { ChainResultCard } from './ChainResultCard';

interface MessageThreadProps {
  messages: Message[];
  isStreaming: boolean;
  onRetry?: (messageId: string) => void;
}

export const MessageThread: React.FC<MessageThreadProps> = ({
  messages,
  isStreaming,
  onRetry,
}) => {
  const threadRef = useRef<HTMLDivElement>(null);

  // Auto-scroll on new messages
  useEffect(() => {
    if (threadRef.current) {
      threadRef.current.scrollTop = threadRef.current.scrollHeight;
    }
  }, [messages]);

  if (messages.length === 0) {
    return (
      <div className="message-thread empty">
        <div className="empty-state">
          <Bot size={48} />
          <h3>Start a conversation</h3>
          <p>Ask me anything about blockchain, smart contracts, or AI models.</p>
        </div>

        <style jsx>{`
          .message-thread.empty {
            flex: 1;
            display: flex;
            align-items: center;
            justify-content: center;
            padding: 2rem;
          }

          .empty-state {
            text-align: center;
            color: #9ca3af;
          }

          .empty-state h3 {
            margin: 1rem 0 0.5rem;
            color: #374151;
            font-size: 1.125rem;
          }

          .empty-state p {
            margin: 0;
            font-size: 0.875rem;
          }
        `}</style>
      </div>
    );
  }

  return (
    <div className="message-thread" ref={threadRef}>
      {messages.map((message) => (
        <MessageBubble
          key={message.id}
          message={message}
          isStreaming={isStreaming && message.isStreaming}
          onRetry={onRetry}
        />
      ))}

      <style jsx>{`
        .message-thread {
          flex: 1;
          overflow-y: auto;
          padding: 1rem;
          display: flex;
          flex-direction: column;
          gap: 1rem;
        }
      `}</style>
    </div>
  );
};

// =============================================================================
// MessageBubble Component
// =============================================================================

interface MessageBubbleProps {
  message: Message;
  isStreaming?: boolean;
  onRetry?: (messageId: string) => void;
}

const MessageBubble: React.FC<MessageBubbleProps> = ({
  message,
  isStreaming = false,
  onRetry,
}) => {
  const [copied, setCopied] = React.useState(false);

  const handleCopy = async () => {
    await navigator.clipboard.writeText(message.content);
    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
  };

  const getIcon = (role: MessageRole) => {
    switch (role) {
      case 'user':
        return <User size={18} />;
      case 'assistant':
        return <Bot size={18} />;
      case 'system':
        return <Cpu size={18} />;
      case 'tool':
        return <Wrench size={18} />;
      default:
        return <Bot size={18} />;
    }
  };

  const getRoleName = (role: MessageRole) => {
    switch (role) {
      case 'user':
        return 'You';
      case 'assistant':
        return 'Assistant';
      case 'system':
        return 'System';
      case 'tool':
        return 'Tool Result';
      default:
        return 'Unknown';
    }
  };

  const formatTime = (timestamp: number) => {
    return new Date(timestamp).toLocaleTimeString([], {
      hour: '2-digit',
      minute: '2-digit',
    });
  };

  // Check if this message contains a tool result that should be displayed as a card
  const hasToolResult = message.toolResult !== undefined;
  const isTransactionApproval = message.toolResult?.toolName === 'send_transaction';
  const isChainResult = ['chain_query', 'get_balance', 'get_block'].includes(
    message.toolResult?.toolName || ''
  );

  return (
    <div className={`message-bubble ${message.role}`}>
      <div className="message-avatar">{getIcon(message.role)}</div>

      <div className="message-content">
        <div className="message-header">
          <span className="message-role">{getRoleName(message.role)}</span>
          <span className="message-time">{formatTime(message.timestamp)}</span>
          {message.intent && (
            <span className="message-intent">
              {message.intent.intent} ({Math.round(message.intent.confidence * 100)}%)
            </span>
          )}
        </div>

        {/* Regular message content */}
        {!hasToolResult && (
          <div className="message-text">
            <MessageContent content={message.content} isStreaming={isStreaming} />
          </div>
        )}

        {/* Tool result cards */}
        {hasToolResult && isTransactionApproval ? (
          <TransactionCard
            transaction={{
              id: message.id,
              type: 'send',
              from: '',
              to: '',
              value: '',
            }}
            onApprove={async () => {}}
            onReject={async () => {}}
          />
        ) : null}

        {hasToolResult && isChainResult && message.toolResult?.data ? (
          <ChainResultCard
            result={{
              type: 'balance',
              data: message.toolResult.data as BalanceResult,
              timestamp: message.timestamp,
            }}
          />
        ) : null}

        {/* Message actions */}
        {!isStreaming && message.role !== 'system' && (
          <div className="message-actions">
            <button
              className="action-btn"
              onClick={handleCopy}
              title={copied ? 'Copied!' : 'Copy message'}
            >
              {copied ? <Check size={14} /> : <Copy size={14} />}
            </button>

            {message.role === 'assistant' && onRetry && (
              <button
                className="action-btn"
                onClick={() => onRetry(message.id)}
                title="Regenerate response"
              >
                <RefreshCw size={14} />
              </button>
            )}
          </div>
        )}
      </div>

      <style jsx>{`
        .message-bubble {
          display: flex;
          gap: 0.75rem;
          max-width: 85%;
          animation: fadeIn 0.2s ease-out;
        }

        @keyframes fadeIn {
          from {
            opacity: 0;
            transform: translateY(10px);
          }
          to {
            opacity: 1;
            transform: translateY(0);
          }
        }

        .message-bubble.user {
          align-self: flex-end;
          flex-direction: row-reverse;
        }

        .message-bubble.system {
          align-self: center;
          max-width: 90%;
        }

        .message-avatar {
          width: 36px;
          height: 36px;
          border-radius: 50%;
          display: flex;
          align-items: center;
          justify-content: center;
          flex-shrink: 0;
        }

        .message-bubble.user .message-avatar {
          background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
          color: white;
        }

        .message-bubble.assistant .message-avatar {
          background: linear-gradient(135deg, #ffa500 0%, #ff8c00 100%);
          color: white;
        }

        .message-bubble.system .message-avatar {
          background: #e5e7eb;
          color: #6b7280;
        }

        .message-bubble.tool .message-avatar {
          background: #dbeafe;
          color: #2563eb;
        }

        .message-content {
          flex: 1;
          background: white;
          border-radius: 1rem;
          padding: 0.875rem 1rem;
          box-shadow: 0 1px 3px rgba(0, 0, 0, 0.08);
          min-width: 0;
        }

        .message-bubble.user .message-content {
          background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
          color: white;
        }

        .message-bubble.system .message-content {
          background: #f3f4f6;
          text-align: center;
        }

        .message-header {
          display: flex;
          align-items: center;
          gap: 0.5rem;
          margin-bottom: 0.375rem;
          font-size: 0.75rem;
        }

        .message-role {
          font-weight: 600;
          opacity: 0.9;
        }

        .message-time {
          opacity: 0.6;
        }

        .message-intent {
          margin-left: auto;
          padding: 0.125rem 0.375rem;
          background: rgba(0, 0, 0, 0.1);
          border-radius: 0.25rem;
          font-size: 0.625rem;
        }

        .message-text {
          line-height: 1.6;
          word-wrap: break-word;
          overflow-wrap: break-word;
        }

        .message-actions {
          display: flex;
          gap: 0.25rem;
          margin-top: 0.5rem;
          opacity: 0;
          transition: opacity 0.2s;
        }

        .message-content:hover .message-actions {
          opacity: 1;
        }

        .action-btn {
          display: flex;
          align-items: center;
          justify-content: center;
          width: 28px;
          height: 28px;
          background: none;
          border: none;
          border-radius: 0.25rem;
          cursor: pointer;
          color: currentColor;
          opacity: 0.6;
          transition: all 0.2s;
        }

        .action-btn:hover {
          opacity: 1;
          background: rgba(0, 0, 0, 0.1);
        }
      `}</style>
    </div>
  );
};

// =============================================================================
// MessageContent Component (with streaming cursor)
// =============================================================================

interface MessageContentProps {
  content: string;
  isStreaming?: boolean;
}

const MessageContent: React.FC<MessageContentProps> = ({ content, isStreaming = false }) => {
  // Basic markdown-like rendering
  const renderContent = (text: string) => {
    // Handle code blocks
    const codeBlockRegex = /```(\w*)\n?([\s\S]*?)```/g;
    const parts: React.ReactNode[] = [];
    let lastIndex = 0;
    let match;

    while ((match = codeBlockRegex.exec(text)) !== null) {
      // Add text before code block
      if (match.index > lastIndex) {
        parts.push(
          <span key={`text-${lastIndex}`}>
            {renderInlineContent(text.slice(lastIndex, match.index))}
          </span>
        );
      }

      // Add code block
      const language = match[1] || 'text';
      const code = match[2].trim();
      parts.push(
        <pre key={`code-${match.index}`} className="code-block">
          <code className={`language-${language}`}>{code}</code>
        </pre>
      );

      lastIndex = match.index + match[0].length;
    }

    // Add remaining text
    if (lastIndex < text.length) {
      parts.push(
        <span key={`text-${lastIndex}`}>
          {renderInlineContent(text.slice(lastIndex))}
        </span>
      );
    }

    return parts.length > 0 ? parts : renderInlineContent(text);
  };

  const renderInlineContent = (text: string) => {
    // Handle inline code
    const inlineCodeRegex = /`([^`]+)`/g;
    const parts: React.ReactNode[] = [];
    let lastIndex = 0;
    let match;

    while ((match = inlineCodeRegex.exec(text)) !== null) {
      if (match.index > lastIndex) {
        parts.push(text.slice(lastIndex, match.index));
      }
      parts.push(
        <code key={match.index} className="inline-code">
          {match[1]}
        </code>
      );
      lastIndex = match.index + match[0].length;
    }

    if (lastIndex < text.length) {
      parts.push(text.slice(lastIndex));
    }

    return parts.length > 0 ? parts : text;
  };

  return (
    <div className="message-content-text">
      {renderContent(content)}
      {isStreaming && <span className="streaming-cursor" />}

      <style jsx>{`
        .message-content-text {
          white-space: pre-wrap;
        }

        .streaming-cursor {
          display: inline-block;
          width: 8px;
          height: 1em;
          background: currentColor;
          margin-left: 2px;
          animation: blink 1s infinite;
          vertical-align: text-bottom;
        }

        @keyframes blink {
          0%,
          50% {
            opacity: 1;
          }
          51%,
          100% {
            opacity: 0;
          }
        }

        :global(.code-block) {
          background: #1e1e1e;
          color: #d4d4d4;
          padding: 1rem;
          border-radius: 0.5rem;
          overflow-x: auto;
          margin: 0.75rem 0;
          font-family: 'Menlo', 'Monaco', 'Courier New', monospace;
          font-size: 0.875rem;
          line-height: 1.5;
        }

        :global(.inline-code) {
          background: rgba(0, 0, 0, 0.1);
          padding: 0.125rem 0.375rem;
          border-radius: 0.25rem;
          font-family: 'Menlo', 'Monaco', 'Courier New', monospace;
          font-size: 0.875em;
        }
      `}</style>
    </div>
  );
};

export default MessageThread;
