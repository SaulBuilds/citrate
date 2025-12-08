/**
 * Chat-First Dashboard Component
 *
 * Sprint 7: AI-First Dashboard Transformation (WP-7.1)
 *
 * This is the main dashboard that puts the AI chat interface
 * at the center with quick access to all tools and node status.
 */

import React, { useState, useEffect, useRef } from 'react';
import {
  Send,
  Bot,
  User,
  Brain,
  Copy,
  ThumbsUp,
  ThumbsDown,
  Mic,
  MicOff,
  Settings,
  RefreshCw,
  Wallet,
  Network,
  FileCode,
  Database,
  Terminal,
  Eye,
  Wifi,
  WifiOff,
  Box,
  X
} from 'lucide-react';
import { useAvailableModels } from '../../hooks/useAvailableModels';
import { invoke } from '@tauri-apps/api/core';
import { nodeService } from '../../services/tauri';
import { NodeStatus } from '../../types';

interface ChatMessage {
  id: string;
  role: 'user' | 'assistant' | 'system';
  content: string;
  timestamp: Date;
  model?: string;
  tokens?: number;
  thinking?: string;
  toolCall?: ToolCall;
}

interface ToolCall {
  tool: string;
  status: 'pending' | 'executing' | 'completed' | 'failed';
  result?: string;
}

interface QuickAction {
  id: string;
  label: string;
  icon: React.ReactNode;
  action: () => void;
  description: string;
}

export const ChatDashboard: React.FC = () => {
  // Chat state
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

  // Node status state
  const [nodeStatus, setNodeStatus] = useState<NodeStatus | null>(null);

  // Models
  const { models: availableModels, loading: modelsLoading, refresh: refreshModels } = useAvailableModels();

  // Quick actions
  const quickActions: QuickAction[] = [
    {
      id: 'wallet',
      label: 'Check Balance',
      icon: <Wallet size={16} />,
      action: () => handleQuickAction('Show my wallet balance'),
      description: 'View your wallet balance'
    },
    {
      id: 'dag',
      label: 'DAG Status',
      icon: <Network size={16} />,
      action: () => handleQuickAction('Show me the current DAG status and tips'),
      description: 'View DAG visualization'
    },
    {
      id: 'contract',
      label: 'Deploy Contract',
      icon: <FileCode size={16} />,
      action: () => handleQuickAction('Help me deploy a new smart contract'),
      description: 'Deploy a smart contract'
    },
    {
      id: 'ipfs',
      label: 'IPFS Storage',
      icon: <Database size={16} />,
      action: () => handleQuickAction('Show my IPFS storage status'),
      description: 'Manage IPFS files'
    },
    {
      id: 'terminal',
      label: 'Open Terminal',
      icon: <Terminal size={16} />,
      action: () => window.dispatchEvent(new CustomEvent('open-tool-panel', { detail: { tool: 'terminal' } })),
      description: 'Open development terminal'
    },
    {
      id: 'preview',
      label: 'Preview Contract',
      icon: <Eye size={16} />,
      action: () => handleQuickAction('Show me the last deployed contract'),
      description: 'Preview contract code'
    }
  ];

  // Initialize
  useEffect(() => {
    // Welcome message
    const welcomeMessage: ChatMessage = {
      id: 'welcome',
      role: 'system',
      content: `Welcome to Citrate! I'm your AI assistant with full access to all platform tools.

You can ask me to:
- Check wallet balances and send transactions
- Visualize the DAG and explore blocks
- Generate and deploy smart contracts
- Manage IPFS storage and models
- Open terminal for development

What would you like to do?`,
      timestamp: new Date()
    };
    setMessages([welcomeMessage]);

    // Fetch node status
    fetchNodeStatus();
    const statusInterval = setInterval(fetchNodeStatus, 5000);

    return () => clearInterval(statusInterval);
  }, []);

  // Set default model
  useEffect(() => {
    if (availableModels.length > 0 && !selectedModel) {
      const defaultModel = availableModels.find(m => m.id === 'mistral-7b-instruct-v0.3')
        || availableModels.find(m => m.type === 'text' && m.available)
        || availableModels[0];
      setSelectedModel(defaultModel.id);
    }
  }, [availableModels, selectedModel]);

  // Auto-scroll
  useEffect(() => {
    messagesEndRef.current?.scrollIntoView({ behavior: 'smooth' });
  }, [messages]);

  const fetchNodeStatus = async () => {
    try {
      const status = await nodeService.getStatus();
      setNodeStatus(status);
    } catch (err) {
      console.error('Failed to fetch node status:', err);
    }
  };

  const handleQuickAction = (prompt: string) => {
    setInput(prompt);
    setTimeout(() => handleSend(prompt), 100);
  };

  const handleSend = async (overrideInput?: string) => {
    const messageText = overrideInput || input.trim();
    if (!messageText || isLoading) return;

    const userMessage: ChatMessage = {
      id: Date.now().toString(),
      role: 'user',
      content: messageText,
      timestamp: new Date()
    };

    setMessages(prev => [...prev, userMessage]);
    setInput('');
    setIsLoading(true);

    try {
      // Parse for tool commands
      const toolCall = parseToolCommand(messageText);

      if (toolCall) {
        // Execute tool and get response
        const response = await executeToolCommand(toolCall, messageText);
        const assistantMessage: ChatMessage = {
          id: (Date.now() + 1).toString(),
          role: 'assistant',
          content: response.content,
          timestamp: new Date(),
          model: selectedModel,
          tokens: response.tokens,
          toolCall: {
            tool: toolCall.tool,
            status: 'completed',
            result: response.toolResult
          }
        };
        setMessages(prev => [...prev, assistantMessage]);
      } else {
        // Regular chat
        const response = await sendToMCP(messageText);
        const assistantMessage: ChatMessage = {
          id: (Date.now() + 1).toString(),
          role: 'assistant',
          content: response.content,
          timestamp: new Date(),
          model: selectedModel,
          tokens: response.tokens,
          thinking: response.thinking
        };
        setMessages(prev => [...prev, assistantMessage]);
      }
    } catch (error) {
      const errorMessage: ChatMessage = {
        id: (Date.now() + 1).toString(),
        role: 'assistant',
        content: `I encountered an error: ${error}. Please try again.`,
        timestamp: new Date(),
        model: selectedModel
      };
      setMessages(prev => [...prev, errorMessage]);
    } finally {
      setIsLoading(false);
    }
  };

  const parseToolCommand = (message: string): { tool: string; params: Record<string, string> } | null => {
    const lowerMessage = message.toLowerCase();

    // Wallet commands
    if (lowerMessage.includes('balance') || lowerMessage.includes('wallet')) {
      return { tool: 'QUERY_BALANCE', params: {} };
    }

    // DAG commands
    if (lowerMessage.includes('dag') || lowerMessage.includes('tips') || lowerMessage.includes('block')) {
      return { tool: 'DAG_STATUS', params: {} };
    }

    // IPFS commands
    if (lowerMessage.includes('ipfs') || lowerMessage.includes('storage')) {
      return { tool: 'IPFS_STATUS', params: {} };
    }

    // Contract commands
    if (lowerMessage.includes('deploy') && lowerMessage.includes('contract')) {
      return { tool: 'DEPLOY_CONTRACT', params: {} };
    }

    // Node commands
    if (lowerMessage.includes('node status') || lowerMessage.includes('network status')) {
      return { tool: 'NODE_STATUS', params: {} };
    }

    return null;
  };

  const executeToolCommand = async (
    toolCall: { tool: string; params: Record<string, string> },
    _originalMessage: string
  ): Promise<{ content: string; tokens?: number; toolResult?: string }> => {
    switch (toolCall.tool) {
      case 'QUERY_BALANCE': {
        try {
          const balance = await invoke<string>('get_wallet_balance');
          const content = `**Wallet Balance**

Your current balance: **${balance} CIT**

Would you like to:
- Send a transaction
- View transaction history
- Generate a new address`;
          return { content, toolResult: balance };
        } catch {
          return { content: 'Unable to fetch wallet balance. Please ensure your wallet is connected.' };
        }
      }

      case 'DAG_STATUS': {
        if (nodeStatus) {
          const content = `**DAG Status**

- **Block Height**: ${nodeStatus.blockHeight.toLocaleString()}
- **DAG Tips**: ${nodeStatus.dagTips || 0}
- **Last Block**: ${nodeStatus.lastBlockHash?.slice(0, 16)}...
- **Network**: ${nodeStatus.networkId}
- **Peers**: ${nodeStatus.peerCount}

Would you like me to open the DAG visualization?`;
          return { content, toolResult: JSON.stringify(nodeStatus) };
        }
        return { content: 'Node is currently offline. Please start the node first.' };
      }

      case 'IPFS_STATUS': {
        try {
          const ipfsStatus = await invoke<{ connected: boolean; peerId?: string }>('ipfs_check_status');
          const content = ipfsStatus.connected
            ? `**IPFS Status**: Connected

- **Peer ID**: ${ipfsStatus.peerId?.slice(0, 16)}...
- **Status**: Online

What would you like to do with IPFS?
- Upload a file
- Download a model
- View pinned content`
            : `**IPFS Status**: Disconnected

The IPFS daemon is not running. Would you like me to start it?`;
          return { content, toolResult: JSON.stringify(ipfsStatus) };
        } catch {
          return {
            content: `**IPFS Status**: Not configured

IPFS is not set up yet. Would you like me to help you configure it?`
          };
        }
      }

      case 'DEPLOY_CONTRACT': {
        const content = `**Contract Deployment**

I can help you deploy a smart contract. What type of contract would you like to create?

1. **ERC-20 Token** - Fungible token with minting capability
2. **ERC-721 NFT** - Non-fungible token collection
3. **ERC-1155 Multi-Token** - Both fungible and non-fungible
4. **Custom Contract** - Start from a template

Just tell me which one, or describe your requirements!`;
        return { content };
      }

      case 'NODE_STATUS': {
        if (nodeStatus) {
          const content = `**Node Status**

- **Status**: ${nodeStatus.running ? '**Online**' : '**Offline**'}
- **Uptime**: ${formatUptime(nodeStatus.uptime)}
- **Block Height**: ${nodeStatus.blockHeight.toLocaleString()}
- **Peers**: ${nodeStatus.peerCount}
- **Version**: v${nodeStatus.version || '0.0.0'}
- **Network ID**: ${nodeStatus.networkId}
- **Syncing**: ${nodeStatus.syncing ? 'Yes' : 'No'}`;
          return { content, toolResult: JSON.stringify(nodeStatus) };
        }
        return { content: 'Unable to get node status. The node may be offline.' };
      }

      default:
        return { content: 'Unknown tool command.' };
    }
  };

  // Session ID for agent communication (lazily created)
  const sessionIdRef = useRef<string | null>(null);

  const sendToMCP = async (message: string) => {
    try {
      const model = availableModels.find(m => m.id === selectedModel);
      if (!model) {
        throw new Error('No model selected');
      }

      // Create a session if we don't have one yet
      if (!sessionIdRef.current) {
        try {
          const session = await invoke<{ id: string }>('agent_create_session');
          sessionIdRef.current = session.id;
        } catch (sessionError) {
          // Agent may not be initialized, use fallback response
          console.warn('Agent not initialized:', sessionError);
          return {
            content: `I'm still initializing. Please try again in a moment, or check that a local AI model is loaded in Settings.\n\nYou can also use the quick actions above to interact with the blockchain directly.`,
            tokens: 0,
            thinking: 'Agent initializing...'
          };
        }
      }

      // Send message to the agent
      const response = await invoke<{
        content: string;
        intent?: string;
        intent_confidence?: number;
        tool_invoked: boolean;
        tool_name?: string;
      }>('agent_send_message', {
        sessionId: sessionIdRef.current,
        message: message
      });

      return {
        content: response.content,
        tokens: 0, // Agent doesn't report token counts
        thinking: response.intent ? `Intent: ${response.intent} (${((response.intent_confidence || 0) * 100).toFixed(0)}%)` : `Using ${model.name}`
      };
    } catch (error) {
      // If session expired or agent reset, clear session and retry once
      if (sessionIdRef.current && String(error).includes('Session not found')) {
        sessionIdRef.current = null;
        return sendToMCP(message);
      }
      throw new Error(`Failed to generate response: ${error instanceof Error ? error.message : String(error)}`);
    }
  };

  const formatUptime = (seconds: number) => {
    const hours = Math.floor(seconds / 3600);
    const minutes = Math.floor((seconds % 3600) / 60);
    return `${hours}h ${minutes}m`;
  };

  const handleKeyPress = (e: React.KeyboardEvent) => {
    if (e.key === 'Enter' && !e.shiftKey) {
      e.preventDefault();
      handleSend();
    }
  };

  const copyMessage = (content: string) => {
    navigator.clipboard.writeText(content);
  };

  const getModelInfo = (modelId: string) => {
    return availableModels.find(m => m.id === modelId);
  };

  return (
    <div className="chat-dashboard">
      {/* Status Bar */}
      <div className="status-bar">
        <div className="status-left">
          <div className={`status-indicator ${nodeStatus?.running ? 'online' : 'offline'}`}>
            {nodeStatus?.running ? <Wifi size={14} /> : <WifiOff size={14} />}
            <span>{nodeStatus?.running ? 'Online' : 'Offline'}</span>
          </div>
          {nodeStatus?.running && (
            <>
              <div className="status-item">
                <Box size={14} />
                <span>{nodeStatus.blockHeight.toLocaleString()}</span>
              </div>
              <div className="status-item">
                <Network size={14} />
                <span>{nodeStatus.dagTips || 0} tips</span>
              </div>
            </>
          )}
        </div>
        <div className="status-right">
          <select
            value={selectedModel}
            onChange={(e) => setSelectedModel(e.target.value)}
            className="model-selector"
            disabled={modelsLoading}
          >
            {modelsLoading ? (
              <option>Loading...</option>
            ) : availableModels.length === 0 ? (
              <option>No models</option>
            ) : (
              availableModels.map(model => (
                <option key={model.id} value={model.id} disabled={!model.available}>
                  {model.name}
                </option>
              ))
            )}
          </select>
          <button
            className="icon-btn"
            onClick={refreshModels}
            disabled={modelsLoading}
            title="Refresh models"
          >
            <RefreshCw size={14} className={modelsLoading ? 'spinning' : ''} />
          </button>
          <button
            className="icon-btn"
            onClick={() => setShowSettings(!showSettings)}
            title="Settings"
          >
            <Settings size={14} />
          </button>
        </div>
      </div>

      {/* Settings Panel */}
      {showSettings && (
        <div className="settings-panel">
          <div className="settings-header">
            <h3>Chat Settings</h3>
            <button className="close-btn" onClick={() => setShowSettings(false)}>
              <X size={16} />
            </button>
          </div>
          <div className="settings-content">
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
              <label>Max Tokens: {maxTokens}</label>
              <input
                type="range"
                min="128"
                max="4096"
                step="128"
                value={maxTokens}
                onChange={(e) => setMaxTokens(parseInt(e.target.value))}
              />
            </div>
          </div>
        </div>
      )}

      {/* Main Chat Area */}
      <div className="chat-main">
        <div className="chat-messages">
          {messages.map(message => (
            <div key={message.id} className={`message ${message.role}`}>
              <div className="message-avatar">
                {message.role === 'user' ? (
                  <User size={18} />
                ) : message.role === 'assistant' ? (
                  <Bot size={18} />
                ) : (
                  <Brain size={18} />
                )}
              </div>

              <div className="message-content">
                <div className="message-header">
                  <span className="message-role">
                    {message.role === 'user' ? 'You' :
                     message.role === 'assistant' ? 'Citrate AI' : 'System'}
                  </span>
                  <span className="message-time">
                    {message.timestamp.toLocaleTimeString()}
                  </span>
                  {message.model && (
                    <span className="message-model">
                      {getModelInfo(message.model)?.name || message.model}
                    </span>
                  )}
                </div>

                {message.toolCall && (
                  <div className={`tool-badge ${message.toolCall.status}`}>
                    <span>Tool: {message.toolCall.tool}</span>
                  </div>
                )}

                <div className="message-text">
                  {message.content.split('\n').map((line, i) => (
                    <React.Fragment key={i}>
                      {line.startsWith('**') && line.endsWith('**') ? (
                        <strong>{line.replace(/\*\*/g, '')}</strong>
                      ) : line.startsWith('- ') ? (
                        <div className="list-item">{line.substring(2)}</div>
                      ) : (
                        line
                      )}
                      {i < message.content.split('\n').length - 1 && <br />}
                    </React.Fragment>
                  ))}
                </div>

                {message.tokens && (
                  <div className="message-stats">
                    <span>{message.tokens} tokens</span>
                  </div>
                )}

                <div className="message-actions">
                  <button
                    className="action-btn"
                    onClick={() => copyMessage(message.content)}
                    title="Copy"
                  >
                    <Copy size={12} />
                  </button>
                  {message.role === 'assistant' && (
                    <>
                      <button className="action-btn" title="Good">
                        <ThumbsUp size={12} />
                      </button>
                      <button className="action-btn" title="Bad">
                        <ThumbsDown size={12} />
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
                <Bot size={18} />
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
      </div>

      {/* Quick Actions */}
      <div className="quick-actions">
        {quickActions.map(action => (
          <button
            key={action.id}
            className="quick-action-btn"
            onClick={action.action}
            title={action.description}
          >
            {action.icon}
            <span>{action.label}</span>
          </button>
        ))}
      </div>

      {/* Input Area */}
      <div className="chat-input">
        <div className="input-container">
          <textarea
            ref={inputRef}
            value={input}
            onChange={(e) => setInput(e.target.value)}
            onKeyPress={handleKeyPress}
            placeholder="Ask me anything or use a quick action above..."
            rows={1}
          />
          <div className="input-actions">
            <button
              className={`voice-btn ${isListening ? 'listening' : ''}`}
              onClick={() => setIsListening(!isListening)}
              title={isListening ? 'Stop' : 'Voice'}
            >
              {isListening ? <MicOff size={16} /> : <Mic size={16} />}
            </button>
            <button
              className="send-btn"
              onClick={() => handleSend()}
              disabled={!input.trim() || isLoading}
            >
              <Send size={16} />
            </button>
          </div>
        </div>
      </div>

      <style jsx>{`
        .chat-dashboard {
          display: flex;
          flex-direction: column;
          height: 100vh;
          background: #f9fafb;
        }

        .status-bar {
          display: flex;
          justify-content: space-between;
          align-items: center;
          padding: 0.75rem 1.5rem;
          background: white;
          border-bottom: 1px solid #e5e7eb;
        }

        .status-left {
          display: flex;
          align-items: center;
          gap: 1.5rem;
        }

        .status-indicator {
          display: flex;
          align-items: center;
          gap: 0.5rem;
          padding: 0.375rem 0.75rem;
          border-radius: 1rem;
          font-size: 0.8125rem;
          font-weight: 500;
        }

        .status-indicator.online {
          background: #d1fae5;
          color: #059669;
        }

        .status-indicator.offline {
          background: #fee2e2;
          color: #dc2626;
        }

        .status-item {
          display: flex;
          align-items: center;
          gap: 0.375rem;
          font-size: 0.8125rem;
          color: #6b7280;
        }

        .status-right {
          display: flex;
          align-items: center;
          gap: 0.75rem;
        }

        .model-selector {
          padding: 0.375rem 0.75rem;
          border: 1px solid #e5e7eb;
          border-radius: 0.375rem;
          background: white;
          font-size: 0.8125rem;
          min-width: 140px;
        }

        .icon-btn {
          display: flex;
          align-items: center;
          justify-content: center;
          width: 32px;
          height: 32px;
          background: #f3f4f6;
          border: none;
          border-radius: 0.375rem;
          cursor: pointer;
          color: #6b7280;
          transition: all 0.15s;
        }

        .icon-btn:hover:not(:disabled) {
          background: #e5e7eb;
          color: #374151;
        }

        .icon-btn:disabled {
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

        .settings-panel {
          background: white;
          border-bottom: 1px solid #e5e7eb;
          padding: 1rem 1.5rem;
        }

        .settings-header {
          display: flex;
          justify-content: space-between;
          align-items: center;
          margin-bottom: 1rem;
        }

        .settings-header h3 {
          margin: 0;
          font-size: 0.9375rem;
          font-weight: 600;
        }

        .close-btn {
          background: none;
          border: none;
          padding: 0.25rem;
          cursor: pointer;
          color: #6b7280;
        }

        .settings-content {
          display: flex;
          gap: 2rem;
        }

        .setting-group {
          flex: 1;
        }

        .setting-group label {
          display: block;
          font-size: 0.8125rem;
          font-weight: 500;
          margin-bottom: 0.5rem;
          color: #374151;
        }

        .setting-group input[type="range"] {
          width: 100%;
        }

        .chat-main {
          flex: 1;
          overflow: hidden;
          display: flex;
          flex-direction: column;
        }

        .chat-messages {
          flex: 1;
          overflow-y: auto;
          padding: 1.5rem;
          display: flex;
          flex-direction: column;
          gap: 1rem;
        }

        .message {
          display: flex;
          gap: 0.75rem;
          max-width: 85%;
        }

        .message.user {
          align-self: flex-end;
          flex-direction: row-reverse;
        }

        .message-avatar {
          width: 28px;
          height: 28px;
          border-radius: 50%;
          display: flex;
          align-items: center;
          justify-content: center;
          flex-shrink: 0;
        }

        .message.user .message-avatar {
          background: linear-gradient(135deg, #ffa500 0%, #ff8c00 100%);
          color: white;
        }

        .message.assistant .message-avatar {
          background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
          color: white;
        }

        .message.system .message-avatar {
          background: linear-gradient(135deg, #10b981 0%, #059669 100%);
          color: white;
        }

        .message-content {
          flex: 1;
          background: white;
          border-radius: 0.75rem;
          padding: 0.875rem 1rem;
          box-shadow: 0 1px 2px rgba(0, 0, 0, 0.05);
        }

        .message.user .message-content {
          background: linear-gradient(135deg, #ffa500 0%, #ff8c00 100%);
          color: white;
        }

        .message-header {
          display: flex;
          align-items: center;
          gap: 0.5rem;
          margin-bottom: 0.375rem;
          font-size: 0.6875rem;
          opacity: 0.7;
        }

        .message-role {
          font-weight: 600;
        }

        .message-model {
          background: rgba(0, 0, 0, 0.1);
          padding: 0.125rem 0.375rem;
          border-radius: 0.25rem;
          font-size: 0.625rem;
        }

        .tool-badge {
          display: inline-flex;
          align-items: center;
          gap: 0.25rem;
          padding: 0.25rem 0.5rem;
          border-radius: 0.25rem;
          font-size: 0.6875rem;
          margin-bottom: 0.5rem;
        }

        .tool-badge.completed {
          background: #d1fae5;
          color: #059669;
        }

        .tool-badge.failed {
          background: #fee2e2;
          color: #dc2626;
        }

        .message-text {
          font-size: 0.9375rem;
          line-height: 1.6;
          white-space: pre-wrap;
        }

        .list-item {
          padding-left: 1rem;
          position: relative;
        }

        .list-item::before {
          content: "•";
          position: absolute;
          left: 0;
        }

        .message-stats {
          margin-top: 0.5rem;
          font-size: 0.6875rem;
          opacity: 0.6;
        }

        .message-actions {
          display: flex;
          gap: 0.25rem;
          margin-top: 0.5rem;
          opacity: 0;
          transition: opacity 0.15s;
        }

        .message-content:hover .message-actions {
          opacity: 1;
        }

        .action-btn {
          background: none;
          border: none;
          padding: 0.25rem;
          cursor: pointer;
          color: currentColor;
          opacity: 0.5;
          border-radius: 0.25rem;
          transition: all 0.15s;
        }

        .action-btn:hover {
          opacity: 1;
          background: rgba(0, 0, 0, 0.05);
        }

        .typing-indicator {
          display: flex;
          gap: 0.25rem;
          padding: 0.5rem 0;
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

        .quick-actions {
          display: flex;
          gap: 0.5rem;
          padding: 0.75rem 1.5rem;
          background: white;
          border-top: 1px solid #e5e7eb;
          overflow-x: auto;
        }

        .quick-action-btn {
          display: flex;
          align-items: center;
          gap: 0.375rem;
          padding: 0.5rem 0.875rem;
          background: #f3f4f6;
          border: none;
          border-radius: 1rem;
          font-size: 0.8125rem;
          font-weight: 500;
          cursor: pointer;
          color: #374151;
          white-space: nowrap;
          transition: all 0.15s;
        }

        .quick-action-btn:hover {
          background: #ffa500;
          color: white;
        }

        .chat-input {
          padding: 1rem 1.5rem;
          background: white;
          border-top: 1px solid #e5e7eb;
        }

        .input-container {
          display: flex;
          align-items: flex-end;
          gap: 0.75rem;
          background: #f9fafb;
          border: 1px solid #e5e7eb;
          border-radius: 1rem;
          padding: 0.75rem 1rem;
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
          min-height: 24px;
          font-family: inherit;
        }

        .input-actions {
          display: flex;
          gap: 0.5rem;
        }

        .voice-btn, .send-btn {
          display: flex;
          align-items: center;
          justify-content: center;
          width: 36px;
          height: 36px;
          background: #f3f4f6;
          border: none;
          border-radius: 0.5rem;
          cursor: pointer;
          color: #6b7280;
          transition: all 0.15s;
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
          background: linear-gradient(135deg, #ffa500 0%, #ff8c00 100%);
          color: white;
        }
      `}</style>
    </div>
  );
};
