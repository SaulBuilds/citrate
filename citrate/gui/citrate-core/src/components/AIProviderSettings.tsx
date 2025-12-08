import React, { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import {
  Bot,
  Key,
  Eye,
  EyeOff,
  CheckCircle,
  XCircle,
  Loader2,
  HardDrive,
  Cloud,
  Sparkles,
  RefreshCw,
  Trash2,
  Download
} from 'lucide-react';

/** AI Provider types matching Rust enum */
export type AIProviderType = 'open_ai' | 'anthropic' | 'gemini' | 'x_ai' | 'local';

/** Provider settings from backend */
export interface ProviderSettingsData {
  enabled: boolean;
  model_id: string;
  base_url?: string;
  verified: boolean;
}

/** Full providers config from backend */
export interface AIProvidersConfigData {
  openai: ProviderSettingsData;
  anthropic: ProviderSettingsData;
  gemini: ProviderSettingsData;
  xai: ProviderSettingsData;
  preferred_order: AIProviderType[];
  local_fallback: boolean;
  local_model_path?: string;
  local_model_cid?: string;
}

/** Provider info for display */
interface ProviderInfo {
  id: AIProviderType;
  name: string;
  description: string;
  icon: React.ReactNode;
  color: string;
  models: { id: string; name: string }[];
  apiKeyPlaceholder: string;
  docsUrl: string;
}

const PROVIDERS: ProviderInfo[] = [
  {
    id: 'open_ai',
    name: 'OpenAI',
    description: 'GPT-4, GPT-3.5 Turbo and more',
    icon: <Sparkles className="w-5 h-5" />,
    color: '#10a37f',
    models: [
      { id: 'gpt-4o-mini', name: 'GPT-4o Mini (Recommended)' },
      { id: 'gpt-4o', name: 'GPT-4o' },
      { id: 'gpt-4-turbo', name: 'GPT-4 Turbo' },
      { id: 'gpt-3.5-turbo', name: 'GPT-3.5 Turbo' },
    ],
    apiKeyPlaceholder: 'sk-...',
    docsUrl: 'https://platform.openai.com/api-keys',
  },
  {
    id: 'anthropic',
    name: 'Anthropic',
    description: 'Claude 3 Opus, Sonnet, and Haiku',
    icon: <Bot className="w-5 h-5" />,
    color: '#d4a574',
    models: [
      { id: 'claude-3-haiku-20240307', name: 'Claude 3 Haiku (Fast)' },
      { id: 'claude-3-sonnet-20240229', name: 'Claude 3 Sonnet' },
      { id: 'claude-3-opus-20240229', name: 'Claude 3 Opus (Most capable)' },
    ],
    apiKeyPlaceholder: 'sk-ant-...',
    docsUrl: 'https://console.anthropic.com/settings/keys',
  },
  {
    id: 'gemini',
    name: 'Google Gemini',
    description: 'Gemini Pro and Gemini Flash',
    icon: <Sparkles className="w-5 h-5" />,
    color: '#4285f4',
    models: [
      { id: 'gemini-1.5-flash', name: 'Gemini 1.5 Flash (Fast)' },
      { id: 'gemini-1.5-pro', name: 'Gemini 1.5 Pro' },
      { id: 'gemini-pro', name: 'Gemini Pro' },
    ],
    apiKeyPlaceholder: 'AIza...',
    docsUrl: 'https://aistudio.google.com/app/apikey',
  },
  {
    id: 'x_ai',
    name: 'xAI',
    description: 'Grok models from xAI',
    icon: <Bot className="w-5 h-5" />,
    color: '#1da1f2',
    models: [
      { id: 'grok-beta', name: 'Grok Beta' },
    ],
    apiKeyPlaceholder: 'xai-...',
    docsUrl: 'https://x.ai/',
  },
];

interface ProviderCardProps {
  provider: ProviderInfo;
  settings: ProviderSettingsData;
  apiKey: string;
  onApiKeyChange: (key: string) => void;
  onModelChange: (modelId: string) => void;
  onEnabledChange: (enabled: boolean) => void;
  onTestConnection: () => Promise<boolean>;
  isLoading: boolean;
}

const ProviderCard: React.FC<ProviderCardProps> = ({
  provider,
  settings,
  apiKey,
  onApiKeyChange,
  onModelChange,
  onEnabledChange,
  onTestConnection,
  isLoading,
}) => {
  const [showKey, setShowKey] = useState(false);
  const [testing, setTesting] = useState(false);
  const [testResult, setTestResult] = useState<'success' | 'error' | null>(null);

  const handleTest = async () => {
    setTesting(true);
    setTestResult(null);
    try {
      const success = await onTestConnection();
      setTestResult(success ? 'success' : 'error');
    } catch {
      setTestResult('error');
    } finally {
      setTesting(false);
    }
  };

  return (
    <div className={`provider-card ${settings.enabled ? 'enabled' : ''}`}>
      <div className="provider-header">
        <div className="provider-icon" style={{ backgroundColor: `${provider.color}20` }}>
          <div style={{ color: provider.color }}>{provider.icon}</div>
        </div>
        <div className="provider-info">
          <h4>{provider.name}</h4>
          <p>{provider.description}</p>
        </div>
        <label className="toggle">
          <input
            type="checkbox"
            checked={settings.enabled}
            onChange={(e) => onEnabledChange(e.target.checked)}
            disabled={isLoading}
          />
          <span className="slider"></span>
        </label>
      </div>

      {settings.enabled && (
        <div className="provider-config">
          <div className="field">
            <label>API Key</label>
            <div className="api-key-input">
              <Key className="w-4 h-4 text-gray-400" />
              <input
                type={showKey ? 'text' : 'password'}
                value={apiKey}
                onChange={(e) => onApiKeyChange(e.target.value)}
                placeholder={provider.apiKeyPlaceholder}
                disabled={isLoading}
              />
              <button
                type="button"
                onClick={() => setShowKey(!showKey)}
                className="icon-btn"
              >
                {showKey ? <EyeOff className="w-4 h-4" /> : <Eye className="w-4 h-4" />}
              </button>
            </div>
            <a
              href={provider.docsUrl}
              target="_blank"
              rel="noopener noreferrer"
              className="docs-link"
            >
              Get API key
            </a>
          </div>

          <div className="field">
            <label>Model</label>
            <select
              value={settings.model_id}
              onChange={(e) => onModelChange(e.target.value)}
              disabled={isLoading}
            >
              {provider.models.map((model) => (
                <option key={model.id} value={model.id}>
                  {model.name}
                </option>
              ))}
            </select>
          </div>

          <div className="provider-actions">
            <button
              onClick={handleTest}
              disabled={testing || !apiKey || isLoading}
              className="test-btn"
            >
              {testing ? (
                <Loader2 className="w-4 h-4 animate-spin" />
              ) : testResult === 'success' ? (
                <CheckCircle className="w-4 h-4 text-green-500" />
              ) : testResult === 'error' ? (
                <XCircle className="w-4 h-4 text-red-500" />
              ) : (
                <RefreshCw className="w-4 h-4" />
              )}
              <span>
                {testing
                  ? 'Testing...'
                  : testResult === 'success'
                  ? 'Connected'
                  : testResult === 'error'
                  ? 'Failed'
                  : 'Test Connection'}
              </span>
            </button>
          </div>
        </div>
      )}

      <style jsx>{`
        .provider-card {
          background: var(--bg-secondary, #f9fafb);
          border: 1px solid var(--border-primary, #e5e7eb);
          border-radius: 12px;
          padding: 16px;
          margin-bottom: 12px;
          transition: all 200ms ease;
        }
        .provider-card.enabled {
          border-color: var(--brand-primary, #f97316);
          background: var(--bg-primary, white);
        }
        .provider-header {
          display: flex;
          align-items: center;
          gap: 12px;
        }
        .provider-icon {
          width: 40px;
          height: 40px;
          border-radius: 10px;
          display: flex;
          align-items: center;
          justify-content: center;
        }
        .provider-info {
          flex: 1;
        }
        .provider-info h4 {
          margin: 0;
          font-size: 14px;
          font-weight: 600;
          color: var(--text-primary, #111827);
        }
        .provider-info p {
          margin: 2px 0 0;
          font-size: 12px;
          color: var(--text-secondary, #6b7280);
        }
        .toggle {
          position: relative;
          display: inline-block;
          width: 44px;
          height: 24px;
        }
        .toggle input {
          opacity: 0;
          width: 0;
          height: 0;
        }
        .slider {
          position: absolute;
          cursor: pointer;
          top: 0;
          left: 0;
          right: 0;
          bottom: 0;
          background-color: #ccc;
          transition: 0.3s;
          border-radius: 24px;
        }
        .slider:before {
          position: absolute;
          content: '';
          height: 18px;
          width: 18px;
          left: 3px;
          bottom: 3px;
          background-color: white;
          transition: 0.3s;
          border-radius: 50%;
        }
        input:checked + .slider {
          background-color: var(--brand-primary, #f97316);
        }
        input:checked + .slider:before {
          transform: translateX(20px);
        }
        .provider-config {
          margin-top: 16px;
          padding-top: 16px;
          border-top: 1px solid var(--border-primary, #e5e7eb);
        }
        .field {
          margin-bottom: 12px;
        }
        .field label {
          display: block;
          font-size: 12px;
          font-weight: 500;
          color: var(--text-secondary, #6b7280);
          margin-bottom: 6px;
        }
        .api-key-input {
          display: flex;
          align-items: center;
          gap: 8px;
          padding: 8px 12px;
          border: 1px solid var(--border-primary, #e5e7eb);
          border-radius: 8px;
          background: var(--bg-primary, white);
        }
        .api-key-input input {
          flex: 1;
          border: none;
          background: none;
          font-size: 13px;
          color: var(--text-primary, #111827);
          outline: none;
          font-family: monospace;
        }
        .api-key-input input::placeholder {
          color: var(--text-tertiary, #9ca3af);
        }
        .icon-btn {
          background: none;
          border: none;
          padding: 4px;
          cursor: pointer;
          color: var(--text-secondary, #6b7280);
          border-radius: 4px;
        }
        .icon-btn:hover {
          background: var(--bg-secondary, #f3f4f6);
        }
        .docs-link {
          font-size: 11px;
          color: var(--brand-primary, #f97316);
          margin-top: 4px;
          display: inline-block;
        }
        select {
          width: 100%;
          padding: 8px 12px;
          border: 1px solid var(--border-primary, #e5e7eb);
          border-radius: 8px;
          font-size: 13px;
          background: var(--bg-primary, white);
          color: var(--text-primary, #111827);
          cursor: pointer;
        }
        .provider-actions {
          display: flex;
          gap: 8px;
          margin-top: 12px;
        }
        .test-btn {
          display: flex;
          align-items: center;
          gap: 6px;
          padding: 8px 16px;
          border: 1px solid var(--border-primary, #e5e7eb);
          border-radius: 8px;
          background: var(--bg-primary, white);
          font-size: 13px;
          cursor: pointer;
          color: var(--text-primary, #111827);
        }
        .test-btn:hover:not(:disabled) {
          background: var(--bg-secondary, #f3f4f6);
        }
        .test-btn:disabled {
          opacity: 0.5;
          cursor: not-allowed;
        }
        @keyframes spin {
          from {
            transform: rotate(0deg);
          }
          to {
            transform: rotate(360deg);
          }
        }
        .animate-spin {
          animation: spin 1s linear infinite;
        }
      `}</style>
    </div>
  );
};

interface LocalModelCardProps {
  modelPath?: string;
  modelCid?: string;
  localFallback: boolean;
  onLocalFallbackChange: (enabled: boolean) => void;
  onPinToIpfs: () => Promise<void>;
  onDeleteLocal: () => Promise<void>;
  onDownloadFromIpfs: () => Promise<void>;
  isLoading: boolean;
}

const LocalModelCard: React.FC<LocalModelCardProps> = ({
  modelPath,
  modelCid,
  localFallback,
  onLocalFallbackChange,
  onPinToIpfs,
  onDeleteLocal,
  onDownloadFromIpfs,
  isLoading,
}) => {
  const [pinning, setPinning] = useState(false);
  const [deleting, setDeleting] = useState(false);
  const [downloading, setDownloading] = useState(false);

  const handlePin = async () => {
    setPinning(true);
    try {
      await onPinToIpfs();
    } finally {
      setPinning(false);
    }
  };

  const handleDelete = async () => {
    setDeleting(true);
    try {
      await onDeleteLocal();
    } finally {
      setDeleting(false);
    }
  };

  const handleDownload = async () => {
    setDownloading(true);
    try {
      await onDownloadFromIpfs();
    } finally {
      setDownloading(false);
    }
  };

  const hasLocalModel = !!modelPath;
  const hasIpfsCid = !!modelCid;

  return (
    <div className="local-model-card">
      <div className="card-header">
        <div className="card-icon">
          <HardDrive className="w-5 h-5" />
        </div>
        <div className="card-info">
          <h4>Local Model</h4>
          <p>Offline AI - No API key needed</p>
        </div>
        <label className="toggle">
          <input
            type="checkbox"
            checked={localFallback}
            onChange={(e) => onLocalFallbackChange(e.target.checked)}
            disabled={isLoading}
          />
          <span className="slider"></span>
        </label>
      </div>

      <div className="card-content">
        {hasLocalModel ? (
          <div className="model-status success">
            <CheckCircle className="w-4 h-4" />
            <span>Model loaded: {modelPath?.split('/').pop()}</span>
          </div>
        ) : hasIpfsCid ? (
          <div className="model-status warning">
            <Cloud className="w-4 h-4" />
            <span>Available on IPFS (not downloaded)</span>
          </div>
        ) : (
          <div className="model-status error">
            <XCircle className="w-4 h-4" />
            <span>No local model configured</span>
          </div>
        )}

        {hasIpfsCid && (
          <div className="ipfs-cid">
            <span className="label">IPFS CID:</span>
            <code>{modelCid?.slice(0, 20)}...</code>
          </div>
        )}

        <div className="model-actions">
          {hasLocalModel && !hasIpfsCid && (
            <button
              onClick={handlePin}
              disabled={pinning || isLoading}
              className="action-btn"
            >
              {pinning ? (
                <Loader2 className="w-4 h-4 animate-spin" />
              ) : (
                <Cloud className="w-4 h-4" />
              )}
              <span>{pinning ? 'Pinning...' : 'Pin to IPFS'}</span>
            </button>
          )}

          {hasLocalModel && hasIpfsCid && (
            <button
              onClick={handleDelete}
              disabled={deleting || isLoading}
              className="action-btn danger"
            >
              {deleting ? (
                <Loader2 className="w-4 h-4 animate-spin" />
              ) : (
                <Trash2 className="w-4 h-4" />
              )}
              <span>{deleting ? 'Deleting...' : 'Delete Local'}</span>
            </button>
          )}

          {!hasLocalModel && hasIpfsCid && (
            <button
              onClick={handleDownload}
              disabled={downloading || isLoading}
              className="action-btn primary"
            >
              {downloading ? (
                <Loader2 className="w-4 h-4 animate-spin" />
              ) : (
                <Download className="w-4 h-4" />
              )}
              <span>{downloading ? 'Downloading...' : 'Download from IPFS'}</span>
            </button>
          )}
        </div>
      </div>

      <style jsx>{`
        .local-model-card {
          background: linear-gradient(135deg, #f0fdf4 0%, #dcfce7 100%);
          border: 1px solid #86efac;
          border-radius: 12px;
          padding: 16px;
          margin-bottom: 12px;
        }
        .card-header {
          display: flex;
          align-items: center;
          gap: 12px;
        }
        .card-icon {
          width: 40px;
          height: 40px;
          border-radius: 10px;
          background: #22c55e20;
          display: flex;
          align-items: center;
          justify-content: center;
          color: #22c55e;
        }
        .card-info {
          flex: 1;
        }
        .card-info h4 {
          margin: 0;
          font-size: 14px;
          font-weight: 600;
          color: #166534;
        }
        .card-info p {
          margin: 2px 0 0;
          font-size: 12px;
          color: #15803d;
        }
        .toggle {
          position: relative;
          display: inline-block;
          width: 44px;
          height: 24px;
        }
        .toggle input {
          opacity: 0;
          width: 0;
          height: 0;
        }
        .slider {
          position: absolute;
          cursor: pointer;
          top: 0;
          left: 0;
          right: 0;
          bottom: 0;
          background-color: #86efac;
          transition: 0.3s;
          border-radius: 24px;
        }
        .slider:before {
          position: absolute;
          content: '';
          height: 18px;
          width: 18px;
          left: 3px;
          bottom: 3px;
          background-color: white;
          transition: 0.3s;
          border-radius: 50%;
        }
        input:checked + .slider {
          background-color: #22c55e;
        }
        input:checked + .slider:before {
          transform: translateX(20px);
        }
        .card-content {
          margin-top: 12px;
          padding-top: 12px;
          border-top: 1px solid #86efac;
        }
        .model-status {
          display: flex;
          align-items: center;
          gap: 8px;
          padding: 8px 12px;
          border-radius: 8px;
          font-size: 13px;
        }
        .model-status.success {
          background: #dcfce7;
          color: #166534;
        }
        .model-status.warning {
          background: #fef3c7;
          color: #92400e;
        }
        .model-status.error {
          background: #fee2e2;
          color: #991b1b;
        }
        .ipfs-cid {
          margin-top: 8px;
          font-size: 12px;
        }
        .ipfs-cid .label {
          color: #15803d;
          margin-right: 4px;
        }
        .ipfs-cid code {
          background: white;
          padding: 2px 6px;
          border-radius: 4px;
          font-family: monospace;
          color: #166534;
        }
        .model-actions {
          display: flex;
          gap: 8px;
          margin-top: 12px;
        }
        .action-btn {
          display: flex;
          align-items: center;
          gap: 6px;
          padding: 8px 16px;
          border: 1px solid #86efac;
          border-radius: 8px;
          background: white;
          font-size: 13px;
          cursor: pointer;
          color: #166534;
        }
        .action-btn:hover:not(:disabled) {
          background: #f0fdf4;
        }
        .action-btn:disabled {
          opacity: 0.5;
          cursor: not-allowed;
        }
        .action-btn.danger {
          border-color: #fca5a5;
          color: #991b1b;
        }
        .action-btn.danger:hover:not(:disabled) {
          background: #fee2e2;
        }
        .action-btn.primary {
          background: #22c55e;
          border-color: #22c55e;
          color: white;
        }
        .action-btn.primary:hover:not(:disabled) {
          background: #16a34a;
        }
      `}</style>
    </div>
  );
};

export const AIProviderSettings: React.FC = () => {
  const [config, setConfig] = useState<AIProvidersConfigData | null>(null);
  const [apiKeys, setApiKeys] = useState<Record<AIProviderType, string>>({
    open_ai: '',
    anthropic: '',
    gemini: '',
    x_ai: '',
    local: '',
  });
  const [loading, setLoading] = useState(true);
  const [saving, setSaving] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [success, setSuccess] = useState<string | null>(null);

  useEffect(() => {
    loadConfig();
  }, []);

  const loadConfig = async () => {
    setLoading(true);
    try {
      const cfg = await invoke<AIProvidersConfigData>('get_ai_providers_config');
      setConfig(cfg);
      // Load API keys separately (they're not serialized in config)
      const keys = await invoke<Record<AIProviderType, string>>('get_ai_provider_keys');
      setApiKeys(keys);
    } catch (e) {
      console.error('Failed to load AI providers config:', e);
      // Use defaults if backend doesn't have the command yet
      setConfig({
        openai: { enabled: false, model_id: 'gpt-4o-mini', verified: false },
        anthropic: { enabled: false, model_id: 'claude-3-haiku-20240307', verified: false },
        gemini: { enabled: false, model_id: 'gemini-1.5-flash', verified: false },
        xai: { enabled: false, model_id: 'grok-beta', verified: false },
        preferred_order: ['local', 'open_ai', 'anthropic'],
        local_fallback: true,
        local_model_path: undefined,
        local_model_cid: undefined,
      });
    } finally {
      setLoading(false);
    }
  };

  const saveConfig = async () => {
    if (!config) return;
    setSaving(true);
    setError(null);
    setSuccess(null);
    try {
      // Use save_ai_providers_config which accepts full config and apiKeys
      await invoke('save_ai_providers_config', {
        config: {
          openai: config.openai,
          anthropic: config.anthropic,
          gemini: config.gemini,
          xai: config.xai,
          preferred_order: config.preferred_order,
          local_fallback: config.local_fallback,
          local_model_path: config.local_model_path,
          local_model_cid: config.local_model_cid,
        },
        api_keys: apiKeys
      });
      setSuccess('AI provider settings saved successfully');
    } catch (e) {
      console.error('Failed to save AI providers config:', e);
      setError(`Failed to save: ${e}`);
    } finally {
      setSaving(false);
    }
  };

  const updateProviderSetting = (
    providerId: AIProviderType,
    field: keyof ProviderSettingsData,
    value: any
  ) => {
    if (!config) return;
    const providerMap: Record<AIProviderType, keyof AIProvidersConfigData> = {
      open_ai: 'openai',
      anthropic: 'anthropic',
      gemini: 'gemini',
      x_ai: 'xai',
      local: 'openai', // dummy, local handled separately
    };
    const key = providerMap[providerId];
    if (key && key !== 'openai' || providerId === 'open_ai') {
      setConfig({
        ...config,
        [key]: {
          ...(config[key] as ProviderSettingsData),
          [field]: value,
        },
      });
    }
  };

  const testConnection = async (providerId: AIProviderType): Promise<boolean> => {
    try {
      const result = await invoke<boolean>('test_ai_provider_connection', {
        provider: providerId,
        apiKey: apiKeys[providerId],
      });
      return result;
    } catch {
      return false;
    }
  };

  const handlePinToIpfs = async () => {
    try {
      const cid = await invoke<string>('pin_local_model_to_ipfs');
      if (config) {
        setConfig({ ...config, local_model_cid: cid });
      }
      setSuccess('Model pinned to IPFS successfully');
    } catch (e) {
      setError(`Failed to pin model: ${e}`);
    }
  };

  const handleDeleteLocal = async () => {
    try {
      await invoke('delete_local_model');
      if (config) {
        setConfig({ ...config, local_model_path: undefined });
      }
      setSuccess('Local model deleted (still available on IPFS)');
    } catch (e) {
      setError(`Failed to delete model: ${e}`);
    }
  };

  const handleDownloadFromIpfs = async () => {
    try {
      const path = await invoke<string>('download_model_from_ipfs', {
        cid: config?.local_model_cid,
      });
      if (config) {
        setConfig({ ...config, local_model_path: path });
      }
      setSuccess('Model downloaded successfully');
    } catch (e) {
      setError(`Failed to download model: ${e}`);
    }
  };

  if (loading) {
    return (
      <div className="ai-provider-settings loading">
        <Loader2 className="w-6 h-6 animate-spin" />
        <span>Loading AI provider settings...</span>
      </div>
    );
  }

  if (!config) {
    return (
      <div className="ai-provider-settings error">
        <XCircle className="w-6 h-6" />
        <span>Failed to load AI provider settings</span>
      </div>
    );
  }

  const providerSettingsMap: Record<AIProviderType, ProviderSettingsData> = {
    open_ai: config.openai,
    anthropic: config.anthropic,
    gemini: config.gemini,
    x_ai: config.xai,
    local: { enabled: true, model_id: '', verified: true },
  };

  return (
    <div className="ai-provider-settings">
      <div className="section-header">
        <h3>AI Providers</h3>
        <p>Configure AI models for the assistant. Local model works offline.</p>
      </div>

      {error && (
        <div className="alert error">
          <XCircle className="w-4 h-4" />
          {error}
        </div>
      )}

      {success && (
        <div className="alert success">
          <CheckCircle className="w-4 h-4" />
          {success}
        </div>
      )}

      <LocalModelCard
        modelPath={config.local_model_path}
        modelCid={config.local_model_cid}
        localFallback={config.local_fallback}
        onLocalFallbackChange={(enabled) =>
          setConfig({ ...config, local_fallback: enabled })
        }
        onPinToIpfs={handlePinToIpfs}
        onDeleteLocal={handleDeleteLocal}
        onDownloadFromIpfs={handleDownloadFromIpfs}
        isLoading={saving}
      />

      <div className="cloud-providers-header">
        <Cloud className="w-4 h-4" />
        <span>Cloud Providers</span>
      </div>

      {PROVIDERS.map((provider) => (
        <ProviderCard
          key={provider.id}
          provider={provider}
          settings={providerSettingsMap[provider.id]}
          apiKey={apiKeys[provider.id]}
          onApiKeyChange={(key) =>
            setApiKeys({ ...apiKeys, [provider.id]: key })
          }
          onModelChange={(modelId) =>
            updateProviderSetting(provider.id, 'model_id', modelId)
          }
          onEnabledChange={(enabled) =>
            updateProviderSetting(provider.id, 'enabled', enabled)
          }
          onTestConnection={() => testConnection(provider.id)}
          isLoading={saving}
        />
      ))}

      <button
        onClick={saveConfig}
        disabled={saving}
        className="save-btn"
      >
        {saving ? (
          <>
            <Loader2 className="w-4 h-4 animate-spin" />
            Saving...
          </>
        ) : (
          <>
            <CheckCircle className="w-4 h-4" />
            Save Settings
          </>
        )}
      </button>

      <style jsx>{`
        .ai-provider-settings {
          padding: 0;
        }
        .ai-provider-settings.loading,
        .ai-provider-settings.error {
          display: flex;
          align-items: center;
          justify-content: center;
          gap: 8px;
          padding: 24px;
          color: var(--text-secondary, #6b7280);
        }
        .section-header {
          margin-bottom: 16px;
        }
        .section-header h3 {
          margin: 0;
          font-size: 16px;
          font-weight: 600;
          color: var(--text-primary, #111827);
        }
        .section-header p {
          margin: 4px 0 0;
          font-size: 13px;
          color: var(--text-secondary, #6b7280);
        }
        .alert {
          display: flex;
          align-items: center;
          gap: 8px;
          padding: 12px 16px;
          border-radius: 8px;
          margin-bottom: 12px;
          font-size: 13px;
        }
        .alert.error {
          background: #fee2e2;
          color: #991b1b;
        }
        .alert.success {
          background: #dcfce7;
          color: #166534;
        }
        .cloud-providers-header {
          display: flex;
          align-items: center;
          gap: 8px;
          font-size: 12px;
          font-weight: 500;
          color: var(--text-secondary, #6b7280);
          text-transform: uppercase;
          letter-spacing: 0.5px;
          margin: 20px 0 12px;
        }
        .save-btn {
          display: flex;
          align-items: center;
          justify-content: center;
          gap: 8px;
          width: 100%;
          padding: 12px 24px;
          border: none;
          border-radius: 8px;
          background: var(--brand-primary, #f97316);
          color: white;
          font-size: 14px;
          font-weight: 500;
          cursor: pointer;
          margin-top: 16px;
          transition: background 200ms ease;
        }
        .save-btn:hover:not(:disabled) {
          background: var(--brand-hover, #ea580c);
        }
        .save-btn:disabled {
          opacity: 0.7;
          cursor: not-allowed;
        }
      `}</style>
    </div>
  );
};

export default AIProviderSettings;
