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
  FileText
} from 'lucide-react';

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

interface ChatModel {
  id: string;
  name: string;
  provider: 'citrate' | 'openai' | 'anthropic' | 'local';
  type: 'text' | 'vision' | 'code';
  costPerToken: number;
  maxTokens: number;
  available: boolean;
}

export const ChatBot: React.FC = () => {
  const [messages, setMessages] = useState<ChatMessage[]>([]);
  const [input, setInput] = useState('');
  const [isLoading, setIsLoading] = useState(false);
  const [selectedModel, setSelectedModel] = useState<string>('citrate-gpt-4');
  const [isListening, setIsListening] = useState(false);
  const [showSettings, setShowSettings] = useState(false);
  const messagesEndRef = useRef<HTMLDivElement>(null);
  const inputRef = useRef<HTMLTextAreaElement>(null);

  const availableModels: ChatModel[] = [
    {
      id: 'citrate-gpt-4',
      name: 'Citrate GPT-4',
      provider: 'citrate',
      type: 'text',
      costPerToken: 0.00001,
      maxTokens: 8192,
      available: true
    },
    {
      id: 'citrate-claude-3',
      name: 'Citrate Claude-3',
      provider: 'citrate',
      type: 'text',
      costPerToken: 0.000008,
      maxTokens: 100000,
      available: true
    },
    {
      id: 'citrate-vision',
      name: 'Citrate Vision',
      provider: 'citrate',
      type: 'vision',
      costPerToken: 0.00002,
      maxTokens: 4096,
      available: true
    },
    {
      id: 'local-llama',
      name: 'Local Llama 3.1',
      provider: 'local',
      type: 'text',
      costPerToken: 0,
      maxTokens: 8192,
      available: false
    }
  ];

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
      // TODO: Integrate with actual MCP service
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
    // Mock MCP API call - replace with actual integration
    await new Promise(resolve => setTimeout(resolve, 1000 + Math.random() * 2000));

    const responses = [
      {
        content: `I can help you with that! Based on your request about "${message}", here are some suggestions:\n\n• Consider using the Citrate Model Registry to find pre-trained models\n• You can deploy custom models using our IPFS storage system\n• For smart contracts, I recommend starting with our Solidity templates\n\nWould you like me to provide more specific guidance on any of these areas?`,
        tokens: 127,
        cost: 0.00127,
        thinking: 'User is asking about AI/blockchain integration. Providing helpful overview.'
      },
      {
        content: `Here's a code example for deploying an AI model on Citrate:\n\n\`\`\`solidity\npragma solidity ^0.8.19;\n\nimport "./IAIModel.sol";\n\ncontract ModelDeployment {\n    mapping(bytes32 => IAIModel) public models;\n    \n    function deployModel(\n        string memory name,\n        string memory ipfsCid,\n        uint256 price\n    ) external returns (bytes32 modelId) {\n        modelId = keccak256(abi.encode(name, msg.sender, block.timestamp));\n        models[modelId] = IAIModel(address(new ModelContract(name, ipfsCid, price)));\n        emit ModelDeployed(modelId, msg.sender, name);\n    }\n}\n\`\`\`\n\nThis contract allows you to deploy AI models with IPFS storage integration. Would you like me to explain any part of this code?`,
        tokens: 189,
        cost: 0.00189,
        thinking: 'User wants code example. Providing Solidity smart contract for AI model deployment.'
      }
    ];

    return responses[Math.floor(Math.random() * responses.length)];
  };

  const handleKeyPress = (e: React.KeyboardEvent) => {
    if (e.key === 'Enter' && !e.shiftKey) {
      e.preventDefault();
      handleSend();
    }
  };

  const toggleVoiceInput = () => {
    if (isListening) {
      setIsListening(false);
      // TODO: Stop speech recognition
    } else {
      setIsListening(true);
      // TODO: Start speech recognition
    }
  };

  const copyMessage = (content: string) => {
    navigator.clipboard.writeText(content);
  };

  const formatCost = (cost: number) => {
    return `$${cost.toFixed(6)}`;
  };

  const getModelInfo = (modelId: string) => {
    return availableModels.find(m => m.id === modelId) || availableModels[0];
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
          >
            {availableModels.map(model => (
              <option key={model.id} value={model.id} disabled={!model.available}>
                {model.name} {!model.available && '(Unavailable)'}
              </option>
            ))}
          </select>
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
            <label>Temperature</label>
            <input type="range" min="0" max="1" step="0.1" defaultValue="0.7" />
          </div>
          <div className="setting-group">
            <label>Max Tokens</label>
            <input type="number" defaultValue="2048" min="1" max="8192" />
          </div>
          <div className="setting-group">
            <label>System Prompt</label>
            <textarea
              rows={3}
              defaultValue="You are a helpful AI assistant specialized in blockchain and AI development on the Citrate platform."
            />
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
                  <span className="message-model">{getModelInfo(message.model).name}</span>
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
      `}</style>
    </div>
  );
};