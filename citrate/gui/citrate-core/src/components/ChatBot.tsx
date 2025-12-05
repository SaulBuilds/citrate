import React, { useState, useEffect, useRef } from 'react';
import {
  MessageSquare,
  Send,
  Bot,
  User,
  Settings,
  Brain,
  Copy,
  ThumbsUp,
  ThumbsDown,
  Mic,
  MicOff,
  Image,
  Code,
  FileText,
  RefreshCw
} from 'lucide-react';
import { useAvailableModels } from '../hooks/useAvailableModels';
import { invoke } from '@tauri-apps/api/core';

interface ChatMessage {
  id: string;
  role: 'user' | 'assistant' | 'system';
  content: string;
  timestamp: Date;
  model?: string;
  tokens?: number;
  cost?: number;
  attachments?: ChatAttachment[];
  thinking?: string;
}

interface ChatAttachment {
  type: 'image' | 'file' | 'code';
  name: string;
  content: string;
  size?: number;
}

// ChatModel interface - used for model management features
interface ChatModel {
  id: string;
  name: string;
  provider: 'citrate' | 'openai' | 'anthropic' | 'local';
  type: 'text' | 'vision' | 'code';
  costPerToken: number;
  maxTokens: number;
  available: boolean;
}

// Export for type compatibility
export type { ChatModel };

export const ChatBot: React.FC = () => {
  const [messages, setMessages] = useState<ChatMessage[]>([]);
  const [input, setInput] = useState('');
  const [isLoading, setIsLoading] = useState(false);
  const [selectedModel, setSelectedModel] = useState<string>('');
  const [isListening, setIsListening] = useState(false);
  const [showSettings, setShowSettings] = useState(false);
  const [temperature, setTemperature] = useState(0.7);
  const [maxTokens, setMaxTokens] = useState(512);
  const messagesEndRef = useRef<HTMLDivElement>(null);
  const inputRef = useRef<HTMLTextAreaElement>(null);

  // Use the availableModels hook
  const { models: availableModels, loading: modelsLoading, refresh: refreshModels } = useAvailableModels();

  // Set default model when models are loaded
  useEffect(() => {
    if (availableModels.length > 0 && !selectedModel) {
      // Prefer Mistral 7B for text generation if available
      const defaultModel = availableModels.find(m => m.id === 'mistral-7b-instruct-v0.3')
        || availableModels.find(m => m.type === 'text' && m.available)
        || availableModels[0];
      setSelectedModel(defaultModel.id);
    }
  }, [availableModels, selectedModel]);

  useEffect(() => {
    // Initialize with welcome message
    const welcomeMessage: ChatMessage = {
      id: 'welcome',
      role: 'system',
      content: 'Welcome to Citrate AI Chat! I can help you with:\n\n• AI model development and deployment\n• Smart contract code generation\n• Blockchain data analysis\n• IPFS storage management\n• General programming assistance\n\nWhat would you like to work on today?',
      timestamp: new Date()
    };
    setMessages([welcomeMessage]);
  }, []);

  useEffect(() => {
    scrollToBottom();
  }, [messages]);

  const scrollToBottom = () => {
    messagesEndRef.current?.scrollIntoView({ behavior: 'smooth' });
  };

  const handleSend = async () => {
    if (!input.trim() || isLoading) return;

    const userMessage: ChatMessage = {
      id: Date.now().toString(),
      role: 'user',
      content: input.trim(),
      timestamp: new Date()
    };

    setMessages(prev => [...prev, userMessage]);
    setInput('');
    setIsLoading(true);

    try {
      // Send message to MCP service via agent orchestrator with fallback to direct inference
      const response = await sendToMCP(input.trim());

      const assistantMessage: ChatMessage = {
        id: (Date.now() + 1).toString(),
        role: 'assistant',
        content: response.content,
        timestamp: new Date(),
        model: selectedModel,
        tokens: response.tokens,
        cost: response.cost,
        thinking: response.thinking
      };

      setMessages(prev => [...prev, assistantMessage]);
    } catch (error) {
      const errorMessage: ChatMessage = {
        id: (Date.now() + 1).toString(),
        role: 'assistant',
        content: `I apologize, but I encountered an error: ${error}. Please try again or check your connection to the Citrate network.`,
        timestamp: new Date(),
        model: selectedModel
      };

      setMessages(prev => [...prev, errorMessage]);
    } finally {
      setIsLoading(false);
    }
  };

  const sendToMCP = async (message: string) => {
    try {
      // Get selected model info
      const model = availableModels.find(m => m.id === selectedModel);
      if (!model) {
        throw new Error('No model selected');
      }

      console.log('[ChatBot] Sending message to model:', model.name);

      // First, ensure we have a session
      let sessionId: string | null = sessionStorage.getItem('agent_session_id');

      if (!sessionId) {
        // Create a new agent session
        try {
          const session = await invoke<{ id: string }>('agent_create_session');
          sessionId = session.id;
          sessionStorage.setItem('agent_session_id', sessionId!);
          console.log('[ChatBot] Created new session:', sessionId);
        } catch (sessionError) {
          console.warn('[ChatBot] Could not create agent session, falling back to direct inference:', sessionError);
          // Fall back to direct model inference
          return await directModelInference(message, model);
        }
      }

      // Try to send via agent
      try {
        const response = await invoke<any>('agent_send_message', {
          sessionId,
          message
        });

        console.log('[ChatBot] Agent response:', response);

        return {
          content: response.content,
          tokens: 0, // Agent doesn't report tokens yet
          cost: 0,
          thinking: response.intent ? `Intent: ${response.intent}` : `Using ${model.name}`
        };
      } catch (agentError) {
        console.warn('[ChatBot] Agent error, falling back to direct inference:', agentError);
        // Clear session and try direct inference
        sessionStorage.removeItem('agent_session_id');
        return await directModelInference(message, model);
      }
    } catch (error) {
      console.error('[ChatBot] Error:', error);
      throw new Error(`Failed to generate response: ${error instanceof Error ? error.message : String(error)}`);
    }
  };

  // Direct model inference fallback using run_inference command
  const directModelInference = async (message: string, model: any) => {
    try {
      console.log('[ChatBot] Trying direct inference with model:', model.id);

      // Try the run_inference command - parameters is a HashMap in Rust
      const response = await invoke<any>('run_inference', {
        request: {
          model_id: model.id,
          input: message,
          parameters: {
            max_tokens: maxTokens,
            temperature: temperature
          }
        }
      });

      return {
        content: response.result || response.output || 'Response generated',
        tokens: response.latency_ms ? Math.floor(response.latency_ms / 50) : 0, // Estimate tokens from latency
        cost: response.cost || 0,
        thinking: `Direct inference with ${model.name} (${response.latency_ms}ms)`
      };
    } catch (inferenceError) {
      // Last fallback - provide helpful error message
      console.error('[ChatBot] Direct inference failed:', inferenceError);
      throw new Error(
        `Model inference not available. To enable local inference:\n` +
        `1. Ensure llama.cpp is installed at ~/llama.cpp\n` +
        `2. Model file must exist at ~/Models or ~/.citrate/models\n\n` +
        `Error: ${inferenceError instanceof Error ? inferenceError.message : String(inferenceError)}`
      );
    }
  }

  const handleKeyPress = (e: React.KeyboardEvent) => {
    if (e.key === 'Enter' && !e.shiftKey) {
      e.preventDefault();
      handleSend();
    }
  };

  const toggleVoiceInput = () => {
    // Check if Web Speech API is available
    const SpeechRecognition = (window as any).SpeechRecognition || (window as any).webkitSpeechRecognition;

    if (!SpeechRecognition) {
      console.warn('[ChatBot] Web Speech API not supported in this browser');
      return;
    }

    if (isListening) {
      // Stop speech recognition
      setIsListening(false);
      const recognition = (window as any).__speechRecognition;
      if (recognition) {
        recognition.stop();
        (window as any).__speechRecognition = null;
      }
    } else {
      // Start speech recognition
      setIsListening(true);
      const recognition = new SpeechRecognition();
      recognition.continuous = false;
      recognition.interimResults = true;
      recognition.lang = 'en-US';

      recognition.onresult = (event: any) => {
        const transcript = Array.from(event.results)
          .map((result: any) => result[0].transcript)
          .join('');

        setInput(transcript);
      };

      recognition.onerror = (event: any) => {
        console.error('[ChatBot] Speech recognition error:', event.error);
        setIsListening(false);
      };

      recognition.onend = () => {
        setIsListening(false);
        (window as any).__speechRecognition = null;
      };

      (window as any).__speechRecognition = recognition;
      recognition.start();
    }
  };

  const copyMessage = (content: string) => {
    navigator.clipboard.writeText(content);
  };

  const formatCost = (cost: number) => {
    return `$${cost.toFixed(6)}`;
  };

  const getModelInfo = (modelId: string) => {
    return availableModels.find(m => m.id === modelId);
  };

  return (
    <div className="chatbot">
      <div className="chat-header">
        <div className="header-title">
          <MessageSquare size={24} />
          <h2>AI Chat Assistant</h2>
        </div>
        <div className="header-controls">
          <select
            value={selectedModel}
            onChange={(e) => setSelectedModel(e.target.value)}
            className="model-selector"
            disabled={modelsLoading}
          >
            {modelsLoading ? (
              <option>Loading models...</option>
            ) : availableModels.length === 0 ? (
              <option>No models available</option>
            ) : (
              availableModels.map(model => (
                <option key={model.id} value={model.id} disabled={!model.available}>
                  {model.name}
                  {!model.available && ' (Unavailable)'}
                  {model.available && model.provider === 'genesis' && ' ⚡'}
                  {model.available && model.provider === 'ipfs' && ' 📦'}
                  {model.inferencePrice !== '0' && ` ($${(parseInt(model.inferencePrice) / 1e18).toFixed(6)})`}
                </option>
              ))
            )}
          </select>
          <button
            className="btn btn-icon"
            onClick={refreshModels}
            disabled={modelsLoading}
            title="Refresh models"
          >
            <RefreshCw size={16} className={modelsLoading ? 'spinning' : ''} />
          </button>
          <button
            className="btn btn-secondary"
            onClick={() => setShowSettings(!showSettings)}
          >
            <Settings size={16} />
          </button>
        </div>
      </div>

      {showSettings && (
        <div className="settings-panel">
          <h3>Chat Settings</h3>
          <div className="setting-group">
            <label>Temperature: {temperature.toFixed(1)}</label>
            <input
              type="range"
              min="0"
              max="1"
              step="0.1"
              value={temperature}
              onChange={(e) => setTemperature(parseFloat(e.target.value))}
            />
          </div>
          <div className="setting-group">
            <label>Max Tokens</label>
            <input
              type="number"
              value={maxTokens}
              onChange={(e) => setMaxTokens(parseInt(e.target.value))}
              min="1"
              max="4096"
            />
          </div>
          <div className="setting-group">
            <label>Selected Model</label>
            <div className="model-info">
              {selectedModel && getModelInfo(selectedModel) ? (
                <>
                  <div><strong>Name:</strong> {getModelInfo(selectedModel)?.name}</div>
                  <div><strong>Provider:</strong> {getModelInfo(selectedModel)?.provider}</div>
                  <div><strong>Type:</strong> {getModelInfo(selectedModel)?.type}</div>
                  <div><strong>Framework:</strong> {getModelInfo(selectedModel)?.framework}</div>
                  {getModelInfo(selectedModel)?.metadata?.maxTokens && (
                    <div><strong>Max Tokens:</strong> {getModelInfo(selectedModel)?.metadata?.maxTokens}</div>
                  )}
                </>
              ) : (
                <div>No model selected</div>
              )}
            </div>
          </div>
        </div>
      )}

      <div className="chat-messages">
        {messages.map(message => (
          <div key={message.id} className={`message ${message.role}`}>
            <div className="message-avatar">
              {message.role === 'user' ? (
                <User size={20} />
              ) : message.role === 'assistant' ? (
                <Bot size={20} />
              ) : (
                <Brain size={20} />
              )}
            </div>

            <div className="message-content">
              <div className="message-header">
                <span className="message-role">
                  {message.role === 'user' ? 'You' :
                   message.role === 'assistant' ? 'Assistant' : 'System'}
                </span>
                <span className="message-time">
                  {message.timestamp.toLocaleTimeString()}
                </span>
                {message.model && (
                  <span className="message-model">{getModelInfo(message.model)?.name || message.model}</span>
                )}
              </div>

              {message.thinking && (
                <div className="thinking-process">
                  <Brain size={14} />
                  <span>{message.thinking}</span>
                </div>
              )}

              <div className="message-text">
                {message.content}
              </div>

              {message.tokens && (
                <div className="message-stats">
                  <span className="tokens">{message.tokens} tokens</span>
                  {message.cost && (
                    <span className="cost">{formatCost(message.cost)}</span>
                  )}
                </div>
              )}

              <div className="message-actions">
                <button
                  className="action-btn"
                  onClick={() => copyMessage(message.content)}
                  title="Copy message"
                >
                  <Copy size={14} />
                </button>
                {message.role === 'assistant' && (
                  <>
                    <button className="action-btn" title="Good response">
                      <ThumbsUp size={14} />
                    </button>
                    <button className="action-btn" title="Poor response">
                      <ThumbsDown size={14} />
                    </button>
                  </>
                )}
              </div>
            </div>
          </div>
        ))}

        {isLoading && (
          <div className="message assistant">
            <div className="message-avatar">
              <Bot size={20} />
            </div>
            <div className="message-content">
              <div className="typing-indicator">
                <span></span>
                <span></span>
                <span></span>
              </div>
            </div>
          </div>
        )}

        <div ref={messagesEndRef} />
      </div>

      <div className="chat-input">
        <div className="input-attachments">
          <button className="attachment-btn" title="Upload image">
            <Image size={16} />
          </button>
          <button className="attachment-btn" title="Upload file">
            <FileText size={16} />
          </button>
          <button className="attachment-btn" title="Code snippet">
            <Code size={16} />
          </button>
        </div>

        <div className="input-container">
          <textarea
            ref={inputRef}
            value={input}
            onChange={(e) => setInput(e.target.value)}
            onKeyPress={handleKeyPress}
            placeholder="Ask me anything about AI, blockchain, or development..."
            rows={1}
            style={{ resize: 'none' }}
          />

          <div className="input-actions">
            <button
              className={`voice-btn ${isListening ? 'listening' : ''}`}
              onClick={toggleVoiceInput}
              title={isListening ? 'Stop listening' : 'Voice input'}
            >
              {isListening ? <MicOff size={16} /> : <Mic size={16} />}
            </button>

            <button
              className="send-btn"
              onClick={handleSend}
              disabled={!input.trim() || isLoading}
              title="Send message"
            >
              <Send size={16} />
            </button>
          </div>
        </div>
      </div>

      <style jsx>{`
        .chatbot {
          display: flex;
          flex-direction: column;
          height: 100vh;
          background: #f9fafb;
        }

        .chat-header {
          display: flex;
          justify-content: space-between;
          align-items: center;
          padding: 1rem 1.5rem;
          background: white;
          border-bottom: 1px solid #e5e7eb;
        }

        .header-title {
          display: flex;
          align-items: center;
          gap: 0.75rem;
        }

        .header-title h2 {
          margin: 0;
          font-size: 1.25rem;
          font-weight: 600;
          color: #111827;
        }

        .header-controls {
          display: flex;
          align-items: center;
          gap: 1rem;
        }

        .model-selector {
          padding: 0.5rem 1rem;
          border: 1px solid #e5e7eb;
          border-radius: 0.5rem;
          background: white;
          font-size: 0.875rem;
          min-width: 150px;
        }

        .settings-panel {
          background: white;
          padding: 1.5rem;
          border-bottom: 1px solid #e5e7eb;
        }

        .settings-panel h3 {
          margin: 0 0 1rem 0;
          font-size: 1rem;
          font-weight: 600;
        }

        .setting-group {
          margin-bottom: 1rem;
        }

        .setting-group label {
          display: block;
          margin-bottom: 0.5rem;
          font-size: 0.875rem;
          font-weight: 500;
          color: #374151;
        }

        .setting-group input,
        .setting-group textarea {
          width: 100%;
          padding: 0.5rem;
          border: 1px solid #e5e7eb;
          border-radius: 0.375rem;
          font-size: 0.875rem;
        }

        .chat-messages {
          flex: 1;
          overflow-y: auto;
          padding: 1rem;
          display: flex;
          flex-direction: column;
          gap: 1rem;
        }

        .message {
          display: flex;
          gap: 0.75rem;
          max-width: 80%;
        }

        .message.user {
          align-self: flex-end;
          flex-direction: row-reverse;
        }

        .message-avatar {
          width: 32px;
          height: 32px;
          border-radius: 50%;
          display: flex;
          align-items: center;
          justify-content: center;
          flex-shrink: 0;
        }

        .message.user .message-avatar {
          background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
          color: white;
        }

        .message.assistant .message-avatar {
          background: linear-gradient(135deg, #10b981 0%, #059669 100%);
          color: white;
        }

        .message.system .message-avatar {
          background: linear-gradient(135deg, #f59e0b 0%, #d97706 100%);
          color: white;
        }

        .message-content {
          flex: 1;
          background: white;
          border-radius: 1rem;
          padding: 1rem;
          box-shadow: 0 1px 3px rgba(0, 0, 0, 0.1);
        }

        .message.user .message-content {
          background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
          color: white;
        }

        .message-header {
          display: flex;
          align-items: center;
          gap: 0.5rem;
          margin-bottom: 0.5rem;
          font-size: 0.75rem;
          opacity: 0.8;
        }

        .message-role {
          font-weight: 600;
        }

        .message-model {
          background: rgba(0, 0, 0, 0.1);
          padding: 0.125rem 0.5rem;
          border-radius: 0.25rem;
        }

        .thinking-process {
          display: flex;
          align-items: center;
          gap: 0.5rem;
          margin-bottom: 0.75rem;
          padding: 0.5rem;
          background: rgba(0, 0, 0, 0.05);
          border-radius: 0.5rem;
          font-size: 0.875rem;
          font-style: italic;
          color: #6b7280;
        }

        .message-text {
          line-height: 1.6;
          white-space: pre-wrap;
          word-wrap: break-word;
        }

        .message-text code {
          background: rgba(0, 0, 0, 0.1);
          padding: 0.125rem 0.25rem;
          border-radius: 0.25rem;
          font-family: monospace;
          font-size: 0.875rem;
        }

        .message-stats {
          display: flex;
          gap: 1rem;
          margin-top: 0.75rem;
          font-size: 0.75rem;
          opacity: 0.7;
        }

        .message-actions {
          display: flex;
          gap: 0.5rem;
          margin-top: 0.75rem;
          opacity: 0;
          transition: opacity 0.2s;
        }

        .message-content:hover .message-actions {
          opacity: 1;
        }

        .action-btn {
          background: none;
          border: none;
          padding: 0.25rem;
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

        .typing-indicator {
          display: flex;
          gap: 0.25rem;
          align-items: center;
        }

        .typing-indicator span {
          width: 6px;
          height: 6px;
          border-radius: 50%;
          background: #6b7280;
          animation: typing 1.4s infinite ease-in-out;
        }

        .typing-indicator span:nth-child(1) { animation-delay: -0.32s; }
        .typing-indicator span:nth-child(2) { animation-delay: -0.16s; }

        @keyframes typing {
          0%, 80%, 100% { transform: scale(0.8); opacity: 0.5; }
          40% { transform: scale(1); opacity: 1; }
        }

        .chat-input {
          background: white;
          border-top: 1px solid #e5e7eb;
          padding: 1rem;
        }

        .input-attachments {
          display: flex;
          gap: 0.5rem;
          margin-bottom: 0.75rem;
        }

        .attachment-btn {
          background: #f3f4f6;
          border: none;
          border-radius: 0.5rem;
          padding: 0.5rem;
          cursor: pointer;
          color: #6b7280;
          transition: all 0.2s;
        }

        .attachment-btn:hover {
          background: #e5e7eb;
          color: #374151;
        }

        .input-container {
          display: flex;
          align-items: flex-end;
          gap: 0.75rem;
          background: #f9fafb;
          border: 1px solid #e5e7eb;
          border-radius: 1rem;
          padding: 0.75rem;
        }

        .input-container textarea {
          flex: 1;
          border: none;
          background: none;
          resize: none;
          outline: none;
          font-size: 0.9375rem;
          line-height: 1.5;
          max-height: 120px;
          min-height: 20px;
        }

        .input-actions {
          display: flex;
          gap: 0.5rem;
          align-items: center;
        }

        .voice-btn, .send-btn {
          background: none;
          border: none;
          border-radius: 0.5rem;
          padding: 0.5rem;
          cursor: pointer;
          color: #6b7280;
          transition: all 0.2s;
        }

        .voice-btn:hover, .send-btn:hover:not(:disabled) {
          background: #e5e7eb;
          color: #374151;
        }

        .voice-btn.listening {
          background: #ef4444;
          color: white;
        }

        .send-btn:disabled {
          opacity: 0.5;
          cursor: not-allowed;
        }

        .send-btn:not(:disabled) {
          background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
          color: white;
        }

        .btn {
          display: flex;
          align-items: center;
          gap: 0.5rem;
          padding: 0.5rem 1rem;
          border: none;
          border-radius: 0.5rem;
          font-size: 0.875rem;
          font-weight: 500;
          cursor: pointer;
          transition: all 0.2s;
        }

        .btn-secondary {
          background: #f3f4f6;
          color: #374151;
        }

        .btn-secondary:hover {
          background: #e5e7eb;
        }

        .btn-icon {
          display: flex;
          align-items: center;
          justify-content: center;
          padding: 0.5rem;
          border: none;
          border-radius: 0.5rem;
          font-size: 0.875rem;
          font-weight: 500;
          cursor: pointer;
          transition: all 0.2s;
          background: #f3f4f6;
          color: #374151;
        }

        .btn-icon:hover:not(:disabled) {
          background: #e5e7eb;
        }

        .btn-icon:disabled {
          opacity: 0.5;
          cursor: not-allowed;
        }

        .spinning {
          animation: spin 1s linear infinite;
        }

        @keyframes spin {
          from { transform: rotate(0deg); }
          to { transform: rotate(360deg); }
        }

        .model-info {
          background: #f9fafb;
          padding: 0.75rem;
          border-radius: 0.5rem;
          font-size: 0.875rem;
        }

        .model-info > div {
          padding: 0.25rem 0;
        }

        .model-info strong {
          font-weight: 600;
          color: #374151;
          margin-right: 0.5rem;
        }
      `}</style>
    </div>
  );
};