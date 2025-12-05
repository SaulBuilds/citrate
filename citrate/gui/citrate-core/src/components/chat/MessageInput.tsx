/**
 * MessageInput Component
 *
 * Input area for composing and sending messages with:
 * - Auto-resizing textarea
 * - Keyboard shortcuts (Cmd/Ctrl+Enter to send)
 * - Attachment buttons (placeholder for future)
 * - Send button with loading state
 */

import React, { useState, useRef, useEffect, useCallback } from 'react';
import {
  Send,
  Paperclip,
  Image,
  Code,
  FileText,
  X,
  Loader,
} from 'lucide-react';

interface MessageInputProps {
  onSend: (content: string) => Promise<void>;
  disabled?: boolean;
  isStreaming?: boolean;
  placeholder?: string;
}

export const MessageInput: React.FC<MessageInputProps> = ({
  onSend,
  disabled = false,
  isStreaming = false,
  placeholder = 'Type a message...',
}) => {
  const [value, setValue] = useState('');
  const [isSending, setIsSending] = useState(false);
  const [showAttachments, setShowAttachments] = useState(false);
  const textareaRef = useRef<HTMLTextAreaElement>(null);

  // Auto-resize textarea
  useEffect(() => {
    const textarea = textareaRef.current;
    if (textarea) {
      textarea.style.height = 'auto';
      textarea.style.height = `${Math.min(textarea.scrollHeight, 150)}px`;
    }
  }, [value]);

  // Focus on mount
  useEffect(() => {
    textareaRef.current?.focus();
  }, []);

  const handleSend = useCallback(async () => {
    const trimmed = value.trim();
    if (!trimmed || disabled || isSending) return;

    setIsSending(true);
    try {
      await onSend(trimmed);
      setValue('');
      // Reset textarea height
      if (textareaRef.current) {
        textareaRef.current.style.height = 'auto';
      }
    } finally {
      setIsSending(false);
      // Refocus after sending
      textareaRef.current?.focus();
    }
  }, [value, disabled, isSending, onSend]);

  const handleKeyDown = useCallback(
    (e: React.KeyboardEvent<HTMLTextAreaElement>) => {
      // Send on Enter (without shift) or Cmd/Ctrl+Enter
      if (e.key === 'Enter' && !e.shiftKey) {
        e.preventDefault();
        handleSend();
      }
    },
    [handleSend]
  );

  const handleChange = useCallback((e: React.ChangeEvent<HTMLTextAreaElement>) => {
    setValue(e.target.value);
  }, []);

  const canSend = value.trim().length > 0 && !disabled && !isSending;

  return (
    <div className="message-input-container">
      {/* Attachment Options */}
      {showAttachments && (
        <div className="attachment-panel">
          <button className="attachment-option" title="Attach image">
            <Image size={18} />
            <span>Image</span>
          </button>
          <button className="attachment-option" title="Attach file">
            <FileText size={18} />
            <span>File</span>
          </button>
          <button className="attachment-option" title="Add code snippet">
            <Code size={18} />
            <span>Code</span>
          </button>
        </div>
      )}

      <div className={`input-wrapper ${disabled ? 'disabled' : ''}`}>
        {/* Attachment Toggle */}
        <button
          className={`btn-attachment ${showAttachments ? 'active' : ''}`}
          onClick={() => setShowAttachments(!showAttachments)}
          title="Attachments"
          disabled={disabled}
        >
          {showAttachments ? <X size={20} /> : <Paperclip size={20} />}
        </button>

        {/* Text Input */}
        <textarea
          ref={textareaRef}
          value={value}
          onChange={handleChange}
          onKeyDown={handleKeyDown}
          placeholder={isStreaming ? 'Waiting for response...' : placeholder}
          disabled={disabled}
          rows={1}
          className="message-textarea"
        />

        {/* Send Button */}
        <button
          className={`btn-send ${canSend ? 'active' : ''}`}
          onClick={handleSend}
          disabled={!canSend}
          title={canSend ? 'Send message (Enter)' : 'Type a message to send'}
        >
          {isSending ? (
            <Loader size={20} className="spinning" />
          ) : (
            <Send size={20} />
          )}
        </button>
      </div>

      {/* Hint Text */}
      <div className="input-hint">
        <span>Press <kbd>Enter</kbd> to send, <kbd>Shift+Enter</kbd> for new line</span>
      </div>

      <style jsx>{`
        .message-input-container {
          padding: 0.75rem 1rem 1rem;
          background: white;
          border-top: 1px solid #e5e7eb;
        }

        .attachment-panel {
          display: flex;
          gap: 0.5rem;
          margin-bottom: 0.75rem;
          padding: 0.5rem;
          background: #f9fafb;
          border-radius: 0.5rem;
        }

        .attachment-option {
          display: flex;
          align-items: center;
          gap: 0.5rem;
          padding: 0.5rem 0.75rem;
          background: white;
          border: 1px solid #e5e7eb;
          border-radius: 0.375rem;
          cursor: pointer;
          color: #6b7280;
          font-size: 0.875rem;
          transition: all 0.2s;
        }

        .attachment-option:hover {
          border-color: #ffa500;
          color: #ffa500;
        }

        .input-wrapper {
          display: flex;
          align-items: flex-end;
          gap: 0.5rem;
          padding: 0.5rem;
          background: #f9fafb;
          border: 1px solid #e5e7eb;
          border-radius: 1rem;
          transition: all 0.2s;
        }

        .input-wrapper:focus-within {
          border-color: #ffa500;
          box-shadow: 0 0 0 3px rgba(255, 165, 0, 0.1);
        }

        .input-wrapper.disabled {
          opacity: 0.6;
          cursor: not-allowed;
        }

        .message-textarea {
          flex: 1;
          border: none;
          background: transparent;
          resize: none;
          outline: none;
          font-size: 0.9375rem;
          line-height: 1.5;
          padding: 0.375rem 0;
          min-height: 24px;
          max-height: 150px;
          font-family: inherit;
        }

        .message-textarea::placeholder {
          color: #9ca3af;
        }

        .message-textarea:disabled {
          cursor: not-allowed;
        }

        .btn-attachment {
          display: flex;
          align-items: center;
          justify-content: center;
          width: 36px;
          height: 36px;
          background: none;
          border: none;
          border-radius: 0.5rem;
          cursor: pointer;
          color: #9ca3af;
          transition: all 0.2s;
        }

        .btn-attachment:hover:not(:disabled) {
          background: #e5e7eb;
          color: #374151;
        }

        .btn-attachment.active {
          background: #ffa50015;
          color: #ffa500;
        }

        .btn-attachment:disabled {
          opacity: 0.5;
          cursor: not-allowed;
        }

        .btn-send {
          display: flex;
          align-items: center;
          justify-content: center;
          width: 36px;
          height: 36px;
          background: #e5e7eb;
          border: none;
          border-radius: 0.5rem;
          cursor: not-allowed;
          color: #9ca3af;
          transition: all 0.2s;
        }

        .btn-send.active {
          background: #ffa500;
          color: white;
          cursor: pointer;
        }

        .btn-send.active:hover {
          background: #e69500;
        }

        .input-hint {
          margin-top: 0.5rem;
          text-align: center;
          font-size: 0.75rem;
          color: #9ca3af;
        }

        .input-hint kbd {
          display: inline-block;
          padding: 0.125rem 0.375rem;
          background: #f3f4f6;
          border: 1px solid #e5e7eb;
          border-radius: 0.25rem;
          font-family: inherit;
          font-size: 0.6875rem;
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
          .input-hint {
            display: none;
          }
        }
      `}</style>
    </div>
  );
};

export default MessageInput;
