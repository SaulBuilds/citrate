import React, { useState, useEffect, useCallback } from 'react';
import { invoke } from '@tauri-apps/api/core';
import {
  Database,
  Upload,
  FolderOpen,
  File,
  Copy,
  Share,
  Globe,
  Server,
  Zap,
  Activity,
  HardDrive,
  Users,
  Clock,
  RefreshCw,
  Pin,
  Unlink,
  Play,
  Square,
  AlertCircle,
  CheckCircle
} from 'lucide-react';

// Types matching Rust backend
interface IpfsStatus {
  running: boolean;
  peer_id: string | null;
  addresses: string[];
  repo_size: number | null;
  num_objects: number | null;
  version: string | null;
}

interface IpfsConfig {
  binary_path: string | null;
  repo_path: string;
  api_port: number;
  gateway_port: number;
  swarm_port: number;
  external_gateways: string[];
  enable_pubsub: boolean;
  bootstrap_peers: string[];
}

interface IpfsAddResult {
  cid: string;
  size: number;
  name: string;
  gateway_url: string;
}

interface IPFSFile {
  cid: string;
  name: string;
  size: number;
  type: 'file' | 'directory';
  uploaded: Date;
  pinned: boolean;
  gateway?: string;
}

export const IPFS: React.FC = () => {
  const [status, setStatus] = useState<IpfsStatus | null>(null);
  const [config, setConfig] = useState<IpfsConfig | null>(null);
  const [pins, setPins] = useState<string[]>([]);
  const [peers, setPeers] = useState<string[]>([]);
  const [files, setFiles] = useState<IPFSFile[]>([]);
  const [selectedFile, setSelectedFile] = useState<IPFSFile | null>(null);
  const [isUploading, setIsUploading] = useState(false);
  const [uploadProgress, setUploadProgress] = useState(0);
  const [searchQuery, setSearchQuery] = useState('');
  const [showNodeSetup, setShowNodeSetup] = useState(false);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [activeTab, setActiveTab] = useState<'files' | 'pins' | 'peers'>('files');

  // Load IPFS status
  const loadStatus = useCallback(async () => {
    try {
      const ipfsStatus = await invoke<IpfsStatus>('ipfs_status');
      setStatus(ipfsStatus);
      setError(null);
    } catch (err: any) {
      console.error('Failed to load IPFS status:', err);
      setStatus({
        running: false,
        peer_id: null,
        addresses: [],
        repo_size: null,
        num_objects: null,
        version: null,
      });
    }
  }, []);

  // Load IPFS config
  const loadConfig = useCallback(async () => {
    try {
      const ipfsConfig = await invoke<IpfsConfig>('ipfs_get_config');
      setConfig(ipfsConfig);
    } catch (err: any) {
      console.error('Failed to load IPFS config:', err);
    }
  }, []);

  // Load pinned content
  const loadPins = useCallback(async () => {
    try {
      const pinnedCids = await invoke<string[]>('ipfs_list_pins');
      setPins(pinnedCids);

      // Convert pins to file entries
      const fileEntries: IPFSFile[] = pinnedCids.map(cid => ({
        cid,
        name: cid.slice(0, 12) + '...',
        size: 0, // Unknown without fetching
        type: 'file' as const,
        uploaded: new Date(),
        pinned: true,
        gateway: config?.external_gateways?.[0] || 'https://ipfs.io/ipfs/',
      }));
      setFiles(fileEntries);
    } catch (err: any) {
      console.error('Failed to load pins:', err);
      setPins([]);
    }
  }, [config]);

  // Load connected peers
  const loadPeers = useCallback(async () => {
    try {
      const connectedPeers = await invoke<string[]>('ipfs_get_peers');
      setPeers(connectedPeers);
    } catch (err: any) {
      console.error('Failed to load peers:', err);
      setPeers([]);
    }
  }, []);

  // Start IPFS daemon
  const startDaemon = async () => {
    setLoading(true);
    setError(null);
    try {
      await invoke('ipfs_start');
      await loadStatus();
      await loadPins();
      await loadPeers();
    } catch (err: any) {
      setError(err.toString());
    } finally {
      setLoading(false);
    }
  };

  // Stop IPFS daemon
  const stopDaemon = async () => {
    setLoading(true);
    try {
      await invoke('ipfs_stop');
      await loadStatus();
    } catch (err: any) {
      setError(err.toString());
    } finally {
      setLoading(false);
    }
  };

  // Upload file to IPFS
  const handleFileUpload = async (event: React.ChangeEvent<HTMLInputElement>) => {
    const file = event.target.files?.[0];
    if (!file) return;

    setIsUploading(true);
    setUploadProgress(0);
    setError(null);

    try {
      // Read file as bytes
      const buffer = await file.arrayBuffer();
      const bytes = Array.from(new Uint8Array(buffer));

      // Simulate progress while uploading
      const progressInterval = setInterval(() => {
        setUploadProgress(prev => Math.min(prev + 10, 90));
      }, 100);

      // Upload via Tauri
      const result = await invoke<IpfsAddResult>('ipfs_add', {
        data: bytes,
        name: file.name,
      });

      clearInterval(progressInterval);
      setUploadProgress(100);

      // Add to files list
      const newFile: IPFSFile = {
        cid: result.cid,
        name: file.name,
        size: result.size,
        type: 'file',
        uploaded: new Date(),
        pinned: true,
        gateway: result.gateway_url.replace(result.cid, ''),
      };

      setFiles(prev => [newFile, ...prev]);

      setTimeout(() => {
        setIsUploading(false);
        setUploadProgress(0);
      }, 500);
    } catch (err: any) {
      setError(`Upload failed: ${err}`);
      setIsUploading(false);
      setUploadProgress(0);
    }
  };

  // Pin content
  const pinContent = async (cid: string) => {
    try {
      await invoke('ipfs_pin', { cid });
      await loadPins();

      // Update file status
      setFiles(prev =>
        prev.map(f => (f.cid === cid ? { ...f, pinned: true } : f))
      );
    } catch (err: any) {
      setError(`Pin failed: ${err}`);
    }
  };

  // Unpin content
  const unpinContent = async (cid: string) => {
    try {
      await invoke('ipfs_unpin', { cid });
      await loadPins();

      // Update file status
      setFiles(prev =>
        prev.map(f => (f.cid === cid ? { ...f, pinned: false } : f))
      );
    } catch (err: any) {
      setError(`Unpin failed: ${err}`);
    }
  };

  // Toggle pin
  const togglePin = async (file: IPFSFile) => {
    if (file.pinned) {
      await unpinContent(file.cid);
    } else {
      await pinContent(file.cid);
    }
  };

  // Copy to clipboard
  const copyToClipboard = (text: string) => {
    navigator.clipboard.writeText(text);
  };

  // Format size
  const formatSize = (bytes: number | null) => {
    if (bytes === null || bytes === 0) return '0 B';
    const sizes = ['B', 'KB', 'MB', 'GB', 'TB'];
    const i = Math.floor(Math.log(bytes) / Math.log(1024));
    return Math.round((bytes / Math.pow(1024, i)) * 100) / 100 + ' ' + sizes[i];
  };

  // Initial load
  useEffect(() => {
    loadStatus();
    loadConfig();
  }, [loadStatus, loadConfig]);

  // Load data when daemon is running
  useEffect(() => {
    if (status?.running) {
      loadPins();
      loadPeers();
    }
  }, [status?.running, loadPins, loadPeers]);

  // Periodic status updates
  useEffect(() => {
    const interval = setInterval(loadStatus, 5000);
    return () => clearInterval(interval);
  }, [loadStatus]);

  // Filter files by search
  const filteredFiles = files.filter(
    file =>
      file.name.toLowerCase().includes(searchQuery.toLowerCase()) ||
      file.cid.toLowerCase().includes(searchQuery.toLowerCase())
  );

  return (
    <div className="ipfs-storage">
      <div className="ipfs-header">
        <div className="header-title">
          <Database size={24} />
          <h2>IPFS Storage</h2>
        </div>
        <div className="header-actions">
          <button
            className="btn btn-secondary"
            onClick={() => setShowNodeSetup(!showNodeSetup)}
          >
            <Server size={16} />
            Node Setup
          </button>
          <label className="btn btn-primary upload-btn">
            <Upload size={16} />
            Upload File
            <input
              type="file"
              onChange={handleFileUpload}
              style={{ display: 'none' }}
              disabled={isUploading || !status?.running}
            />
          </label>
        </div>
      </div>

      {/* Error display */}
      {error && (
        <div className="error-banner">
          <AlertCircle size={16} />
          <span>{error}</span>
          <button onClick={() => setError(null)}>×</button>
        </div>
      )}

      {/* Node Status */}
      <div className="node-status">
        <div className="status-card">
          <div className="status-header">
            <div className="status-indicator">
              <div className={`status-dot ${status?.running ? 'online' : 'offline'}`} />
              <span>IPFS Node {status?.running ? 'Online' : 'Offline'}</span>
            </div>
            <div className="status-actions">
              {status?.running ? (
                <button
                  className="btn-icon danger"
                  onClick={stopDaemon}
                  disabled={loading}
                  title="Stop IPFS daemon"
                >
                  <Square size={16} />
                </button>
              ) : (
                <button
                  className="btn-icon success"
                  onClick={startDaemon}
                  disabled={loading}
                  title="Start IPFS daemon"
                >
                  <Play size={16} />
                </button>
              )}
              <button
                className="btn-icon"
                onClick={loadStatus}
                title="Refresh status"
              >
                <RefreshCw size={16} className={loading ? 'spin' : ''} />
              </button>
            </div>
          </div>

          {status?.running && (
            <div className="status-stats">
              <div className="stat">
                <Users size={16} />
                <span>{peers.length} peers</span>
              </div>
              <div className="stat">
                <Pin size={16} />
                <span>{pins.length} pinned</span>
              </div>
              <div className="stat">
                <HardDrive size={16} />
                <span>{formatSize(status.repo_size)}</span>
              </div>
              {status.version && (
                <div className="stat">
                  <Activity size={16} />
                  <span>v{status.version}</span>
                </div>
              )}
            </div>
          )}

          {status?.peer_id && (
            <div className="peer-id">
              <span className="label">Peer ID:</span>
              <code>{status.peer_id.slice(0, 20)}...</code>
              <button
                className="btn-icon small"
                onClick={() => copyToClipboard(status.peer_id!)}
                title="Copy Peer ID"
              >
                <Copy size={12} />
              </button>
            </div>
          )}
        </div>

        {status?.running && status.repo_size && (
          <div className="storage-usage">
            <div className="usage-bar">
              <div
                className="usage-fill"
                style={{
                  width: `${Math.min((status.repo_size / (10 * 1024 * 1024 * 1024)) * 100, 100)}%`,
                }}
              />
            </div>
            <span className="usage-text">{formatSize(status.repo_size)} used</span>
          </div>
        )}
      </div>

      {showNodeSetup && (
        <div className="node-setup">
          <h3>IPFS Node Configuration</h3>
          {config && (
            <div className="config-info">
              <div className="config-item">
                <span className="label">API Port:</span>
                <span>{config.api_port}</span>
              </div>
              <div className="config-item">
                <span className="label">Gateway Port:</span>
                <span>{config.gateway_port}</span>
              </div>
              <div className="config-item">
                <span className="label">Swarm Port:</span>
                <span>{config.swarm_port}</span>
              </div>
              <div className="config-item">
                <span className="label">Repo Path:</span>
                <code>{config.repo_path}</code>
              </div>
            </div>
          )}
          <div className="setup-options">
            <button className="setup-btn" onClick={startDaemon} disabled={status?.running || loading}>
              <Server size={20} />
              <div>
                <strong>Start Local Node</strong>
                <p>Run IPFS daemon locally</p>
              </div>
              {status?.running && <CheckCircle size={16} className="check" />}
            </button>
            <button className="setup-btn">
              <Globe size={20} />
              <div>
                <strong>External Gateways</strong>
                <p>{config?.external_gateways?.length || 0} configured</p>
              </div>
            </button>
            <button className="setup-btn">
              <Zap size={20} />
              <div>
                <strong>Bootstrap Peers</strong>
                <p>{config?.bootstrap_peers?.length || 0} configured</p>
              </div>
            </button>
          </div>
        </div>
      )}

      {/* Upload Progress */}
      {isUploading && (
        <div className="upload-progress">
          <div className="progress-header">
            <Upload size={16} />
            <span>Uploading to IPFS...</span>
            <span>{Math.round(uploadProgress)}%</span>
          </div>
          <div className="progress-bar">
            <div className="progress-fill" style={{ width: `${uploadProgress}%` }} />
          </div>
        </div>
      )}

      {/* Content Tabs */}
      <div className="content-controls">
        <div className="search-bar">
          <input
            type="text"
            placeholder="Search files and CIDs..."
            value={searchQuery}
            onChange={e => setSearchQuery(e.target.value)}
          />
        </div>
        <div className="view-tabs">
          <button
            className={`tab ${activeTab === 'files' ? 'active' : ''}`}
            onClick={() => setActiveTab('files')}
          >
            Files ({files.length})
          </button>
          <button
            className={`tab ${activeTab === 'pins' ? 'active' : ''}`}
            onClick={() => setActiveTab('pins')}
          >
            Pins ({pins.length})
          </button>
          <button
            className={`tab ${activeTab === 'peers' ? 'active' : ''}`}
            onClick={() => setActiveTab('peers')}
          >
            Peers ({peers.length})
          </button>
        </div>
      </div>

      {/* Content Area */}
      <div className="files-container">
        {!status?.running ? (
          <div className="empty-state">
            <Server size={48} />
            <p>IPFS daemon is not running</p>
            <p className="text-muted">Start the daemon to view and manage content</p>
            <button className="btn btn-primary" onClick={startDaemon} disabled={loading}>
              <Play size={16} />
              Start IPFS Daemon
            </button>
          </div>
        ) : activeTab === 'files' ? (
          <>
            {filteredFiles.map(file => (
              <div key={file.cid} className="file-item" onClick={() => setSelectedFile(file)}>
                <div className="file-icon">
                  {file.type === 'directory' ? <FolderOpen size={24} /> : <File size={24} />}
                </div>

                <div className="file-info">
                  <h4>{file.name}</h4>
                  <div className="file-meta">
                    <span className="cid">{file.cid.slice(0, 20)}...</span>
                    <span className="size">{formatSize(file.size)}</span>
                    <span className="uploaded">
                      <Clock size={12} />
                      {file.uploaded.toLocaleDateString()}
                    </span>
                  </div>
                </div>

                <div className="file-status">
                  {file.pinned ? (
                    <div className="pinned">
                      <Pin size={16} />
                      <span>Pinned</span>
                    </div>
                  ) : (
                    <div className="unpinned">
                      <Unlink size={16} />
                      <span>Unpinned</span>
                    </div>
                  )}
                </div>

                <div className="file-actions">
                  <button
                    className="action-btn"
                    onClick={e => {
                      e.stopPropagation();
                      togglePin(file);
                    }}
                    title={file.pinned ? 'Unpin' : 'Pin'}
                  >
                    {file.pinned ? <Unlink size={16} /> : <Pin size={16} />}
                  </button>
                  <button
                    className="action-btn"
                    onClick={e => {
                      e.stopPropagation();
                      copyToClipboard(file.cid);
                    }}
                    title="Copy CID"
                  >
                    <Copy size={16} />
                  </button>
                  <button
                    className="action-btn"
                    onClick={e => {
                      e.stopPropagation();
                      if (file.gateway) {
                        window.open(file.gateway + file.cid, '_blank');
                      }
                    }}
                    title="Open in gateway"
                  >
                    <Share size={16} />
                  </button>
                </div>
              </div>
            ))}

            {filteredFiles.length === 0 && (
              <div className="empty-state">
                <Database size={48} />
                <p>No files found</p>
                <p className="text-muted">Upload your first file to get started</p>
              </div>
            )}
          </>
        ) : activeTab === 'pins' ? (
          <>
            {pins.map(cid => (
              <div key={cid} className="file-item">
                <div className="file-icon">
                  <Pin size={24} />
                </div>
                <div className="file-info">
                  <h4>{cid}</h4>
                  <div className="file-meta">
                    <span className="cid">Pinned content</span>
                  </div>
                </div>
                <div className="file-actions">
                  <button
                    className="action-btn"
                    onClick={() => unpinContent(cid)}
                    title="Unpin"
                  >
                    <Unlink size={16} />
                  </button>
                  <button
                    className="action-btn"
                    onClick={() => copyToClipboard(cid)}
                    title="Copy CID"
                  >
                    <Copy size={16} />
                  </button>
                </div>
              </div>
            ))}

            {pins.length === 0 && (
              <div className="empty-state">
                <Pin size={48} />
                <p>No pinned content</p>
                <p className="text-muted">Pin content to keep it available locally</p>
              </div>
            )}
          </>
        ) : (
          <>
            {peers.map((peer, idx) => (
              <div key={idx} className="file-item">
                <div className="file-icon">
                  <Users size={24} />
                </div>
                <div className="file-info">
                  <h4>{peer.slice(0, 32)}...</h4>
                  <div className="file-meta">
                    <span className="cid">Connected peer</span>
                  </div>
                </div>
                <div className="file-actions">
                  <button
                    className="action-btn"
                    onClick={() => copyToClipboard(peer)}
                    title="Copy Peer ID"
                  >
                    <Copy size={16} />
                  </button>
                </div>
              </div>
            ))}

            {peers.length === 0 && (
              <div className="empty-state">
                <Users size={48} />
                <p>No connected peers</p>
                <p className="text-muted">Peers will appear once connected to the network</p>
              </div>
            )}
          </>
        )}
      </div>

      {/* File Details Modal */}
      {selectedFile && (
        <FileDetailsModal
          file={selectedFile}
          onClose={() => setSelectedFile(null)}
          onPin={togglePin}
        />
      )}

      <style jsx>{`
        .ipfs-storage {
          padding: 2rem;
          background: #f9fafb;
          min-height: 100vh;
        }

        .ipfs-header {
          display: flex;
          justify-content: space-between;
          align-items: center;
          margin-bottom: 2rem;
        }

        .header-title {
          display: flex;
          align-items: center;
          gap: 0.75rem;
        }

        .header-title h2 {
          margin: 0;
          font-size: 1.75rem;
          font-weight: 600;
          color: #111827;
        }

        .header-actions {
          display: flex;
          gap: 1rem;
        }

        .error-banner {
          display: flex;
          align-items: center;
          gap: 0.5rem;
          padding: 1rem;
          background: #fef2f2;
          border: 1px solid #fecaca;
          border-radius: 0.5rem;
          color: #dc2626;
          margin-bottom: 1rem;
        }

        .error-banner button {
          margin-left: auto;
          background: none;
          border: none;
          cursor: pointer;
          font-size: 1.25rem;
          color: #dc2626;
        }

        .node-status {
          background: white;
          border-radius: 1rem;
          padding: 1.5rem;
          box-shadow: 0 1px 3px rgba(0, 0, 0, 0.1);
          margin-bottom: 2rem;
        }

        .status-header {
          display: flex;
          justify-content: space-between;
          align-items: center;
          margin-bottom: 1rem;
        }

        .status-indicator {
          display: flex;
          align-items: center;
          gap: 0.5rem;
          font-weight: 600;
        }

        .status-actions {
          display: flex;
          gap: 0.5rem;
        }

        .status-dot {
          width: 8px;
          height: 8px;
          border-radius: 50%;
        }

        .status-dot.online {
          background: #10b981;
        }

        .status-dot.offline {
          background: #ef4444;
        }

        .status-stats {
          display: flex;
          gap: 2rem;
          flex-wrap: wrap;
          margin-bottom: 1rem;
        }

        .stat {
          display: flex;
          align-items: center;
          gap: 0.5rem;
          color: #6b7280;
          font-size: 0.875rem;
        }

        .peer-id {
          display: flex;
          align-items: center;
          gap: 0.5rem;
          font-size: 0.875rem;
          color: #6b7280;
        }

        .peer-id code {
          background: #f3f4f6;
          padding: 0.25rem 0.5rem;
          border-radius: 0.25rem;
          font-family: monospace;
        }

        .storage-usage {
          display: flex;
          align-items: center;
          gap: 1rem;
          margin-top: 1rem;
        }

        .usage-bar {
          flex: 1;
          height: 6px;
          background: #f3f4f6;
          border-radius: 3px;
          overflow: hidden;
        }

        .usage-fill {
          height: 100%;
          background: linear-gradient(90deg, #10b981, #059669);
          transition: width 0.3s ease;
        }

        .usage-text {
          font-size: 0.875rem;
          color: #6b7280;
          white-space: nowrap;
        }

        .node-setup {
          background: white;
          border-radius: 1rem;
          padding: 1.5rem;
          box-shadow: 0 1px 3px rgba(0, 0, 0, 0.1);
          margin-bottom: 2rem;
        }

        .node-setup h3 {
          margin: 0 0 1rem 0;
          font-size: 1.125rem;
          font-weight: 600;
        }

        .config-info {
          display: grid;
          grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
          gap: 1rem;
          margin-bottom: 1.5rem;
          padding: 1rem;
          background: #f9fafb;
          border-radius: 0.5rem;
        }

        .config-item {
          display: flex;
          flex-direction: column;
          gap: 0.25rem;
        }

        .config-item .label {
          font-size: 0.75rem;
          color: #6b7280;
          text-transform: uppercase;
        }

        .config-item code {
          font-family: monospace;
          font-size: 0.875rem;
          color: #374151;
        }

        .setup-options {
          display: grid;
          grid-template-columns: repeat(auto-fit, minmax(250px, 1fr));
          gap: 1rem;
        }

        .setup-btn {
          display: flex;
          align-items: center;
          gap: 1rem;
          padding: 1rem;
          border: 1px solid #e5e7eb;
          border-radius: 0.75rem;
          background: white;
          cursor: pointer;
          transition: all 0.2s;
          text-align: left;
          position: relative;
        }

        .setup-btn:hover:not(:disabled) {
          border-color: #667eea;
          box-shadow: 0 2px 4px rgba(102, 126, 234, 0.1);
        }

        .setup-btn:disabled {
          opacity: 0.6;
          cursor: not-allowed;
        }

        .setup-btn .check {
          position: absolute;
          right: 1rem;
          color: #10b981;
        }

        .setup-btn strong {
          display: block;
          margin-bottom: 0.25rem;
          color: #111827;
        }

        .setup-btn p {
          margin: 0;
          color: #6b7280;
          font-size: 0.875rem;
        }

        .upload-progress {
          background: white;
          border-radius: 1rem;
          padding: 1.5rem;
          box-shadow: 0 1px 3px rgba(0, 0, 0, 0.1);
          margin-bottom: 2rem;
        }

        .progress-header {
          display: flex;
          align-items: center;
          gap: 0.5rem;
          margin-bottom: 1rem;
          font-weight: 600;
        }

        .progress-bar {
          height: 8px;
          background: #f3f4f6;
          border-radius: 4px;
          overflow: hidden;
        }

        .progress-fill {
          height: 100%;
          background: linear-gradient(90deg, #667eea, #764ba2);
          transition: width 0.3s ease;
        }

        .content-controls {
          display: flex;
          justify-content: space-between;
          align-items: center;
          margin-bottom: 1.5rem;
          gap: 1rem;
        }

        .search-bar {
          flex: 1;
          max-width: 400px;
        }

        .search-bar input {
          width: 100%;
          padding: 0.75rem 1rem;
          border: 1px solid #e5e7eb;
          border-radius: 0.5rem;
          font-size: 0.9375rem;
        }

        .view-tabs {
          display: flex;
          border: 1px solid #e5e7eb;
          border-radius: 0.5rem;
          overflow: hidden;
        }

        .tab {
          padding: 0.5rem 1rem;
          border: none;
          background: white;
          cursor: pointer;
          transition: all 0.2s;
          font-size: 0.875rem;
        }

        .tab.active {
          background: #667eea;
          color: white;
        }

        .files-container {
          background: white;
          border-radius: 1rem;
          box-shadow: 0 1px 3px rgba(0, 0, 0, 0.1);
          overflow: hidden;
          min-height: 300px;
        }

        .file-item {
          display: flex;
          align-items: center;
          gap: 1rem;
          padding: 1rem 1.5rem;
          border-bottom: 1px solid #f3f4f6;
          cursor: pointer;
          transition: background 0.2s;
        }

        .file-item:hover {
          background: #f9fafb;
        }

        .file-item:last-child {
          border-bottom: none;
        }

        .file-icon {
          color: #6b7280;
        }

        .file-info {
          flex: 1;
        }

        .file-info h4 {
          margin: 0 0 0.25rem 0;
          font-size: 1rem;
          font-weight: 600;
          color: #111827;
          word-break: break-all;
        }

        .file-meta {
          display: flex;
          align-items: center;
          gap: 1rem;
          font-size: 0.875rem;
          color: #6b7280;
        }

        .cid {
          font-family: monospace;
          background: #f3f4f6;
          padding: 0.125rem 0.5rem;
          border-radius: 0.25rem;
        }

        .uploaded {
          display: flex;
          align-items: center;
          gap: 0.25rem;
        }

        .file-status {
          display: flex;
          flex-direction: column;
          align-items: center;
          gap: 0.5rem;
        }

        .pinned,
        .unpinned {
          display: flex;
          align-items: center;
          gap: 0.25rem;
          font-size: 0.75rem;
          font-weight: 500;
        }

        .pinned {
          color: #059669;
        }

        .unpinned {
          color: #6b7280;
        }

        .file-actions {
          display: flex;
          gap: 0.5rem;
          opacity: 0;
          transition: opacity 0.2s;
        }

        .file-item:hover .file-actions {
          opacity: 1;
        }

        .action-btn {
          background: none;
          border: none;
          padding: 0.5rem;
          border-radius: 0.375rem;
          cursor: pointer;
          color: #6b7280;
          transition: all 0.2s;
        }

        .action-btn:hover {
          background: #f3f4f6;
          color: #374151;
        }

        .btn {
          display: flex;
          align-items: center;
          gap: 0.5rem;
          padding: 0.75rem 1.5rem;
          border: none;
          border-radius: 0.5rem;
          font-size: 0.9375rem;
          font-weight: 500;
          cursor: pointer;
          transition: all 0.2s;
          text-decoration: none;
        }

        .btn:disabled {
          opacity: 0.6;
          cursor: not-allowed;
        }

        .btn-primary {
          background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
          color: white;
        }

        .btn-primary:hover:not(:disabled) {
          transform: translateY(-1px);
          box-shadow: 0 4px 8px rgba(102, 126, 234, 0.3);
        }

        .btn-secondary {
          background: #f3f4f6;
          color: #374151;
          border: 1px solid #e5e7eb;
        }

        .btn-secondary:hover:not(:disabled) {
          background: #e5e7eb;
        }

        .upload-btn {
          position: relative;
          overflow: hidden;
        }

        .btn-icon {
          background: none;
          border: none;
          padding: 0.5rem;
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

        .btn-icon.success:hover:not(:disabled) {
          background: #d1fae5;
          color: #059669;
        }

        .btn-icon.danger:hover:not(:disabled) {
          background: #fee2e2;
          color: #dc2626;
        }

        .btn-icon.small {
          padding: 0.25rem;
        }

        .spin {
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

        .empty-state {
          padding: 3rem;
          text-align: center;
          color: #6b7280;
        }

        .empty-state svg {
          margin-bottom: 1rem;
          color: #9ca3af;
        }

        .empty-state p {
          margin: 0.5rem 0;
        }

        .empty-state .btn {
          margin-top: 1rem;
        }

        .text-muted {
          color: #9ca3af;
        }
      `}</style>
    </div>
  );
};

// File Details Modal
const FileDetailsModal: React.FC<{
  file: IPFSFile;
  onClose: () => void;
  onPin: (file: IPFSFile) => void;
}> = ({ file, onClose, onPin }) => {
  const formatSize = (bytes: number) => {
    const sizes = ['B', 'KB', 'MB', 'GB', 'TB'];
    if (bytes === 0) return '0 B';
    const i = Math.floor(Math.log(bytes) / Math.log(1024));
    return (Math.round((bytes / Math.pow(1024, i)) * 100) / 100) + ' ' + sizes[i];
  };

  return (
    <div className="modal-overlay" onClick={onClose}>
      <div className="modal" onClick={e => e.stopPropagation()}>
        <div className="modal-header">
          <h3>{file.name}</h3>
          <button className="close-btn" onClick={onClose}>
            ×
          </button>
        </div>

        <div className="file-details">
          <div className="detail-section">
            <h4>IPFS Information</h4>
            <div className="detail-grid">
              <div className="detail-item">
                <span className="label">CID:</span>
                <span className="value mono">{file.cid}</span>
              </div>
              <div className="detail-item">
                <span className="label">Size:</span>
                <span className="value">{formatSize(file.size)}</span>
              </div>
              <div className="detail-item">
                <span className="label">Type:</span>
                <span className="value">{file.type}</span>
              </div>
              <div className="detail-item">
                <span className="label">Uploaded:</span>
                <span className="value">{file.uploaded.toLocaleString()}</span>
              </div>
              <div className="detail-item">
                <span className="label">Status:</span>
                <span className={`value ${file.pinned ? 'pinned' : 'unpinned'}`}>
                  {file.pinned ? 'Pinned' : 'Unpinned'}
                </span>
              </div>
            </div>
          </div>

          {file.gateway && (
            <div className="detail-section">
              <h4>Gateway Access</h4>
              <div className="gateway-links">
                <a
                  href={file.gateway + file.cid}
                  target="_blank"
                  rel="noopener noreferrer"
                  className="gateway-link"
                >
                  <Globe size={16} />
                  Open in Browser
                </a>
              </div>
            </div>
          )}
        </div>

        <div className="modal-actions">
          <button className="btn btn-secondary" onClick={onClose}>
            Close
          </button>
          <button className="btn btn-primary" onClick={() => onPin(file)}>
            {file.pinned ? <Unlink size={16} /> : <Pin size={16} />}
            {file.pinned ? 'Unpin' : 'Pin'}
          </button>
        </div>
      </div>

      <style jsx>{`
        .modal-overlay {
          position: fixed;
          top: 0;
          left: 0;
          right: 0;
          bottom: 0;
          background: rgba(0, 0, 0, 0.5);
          display: flex;
          align-items: center;
          justify-content: center;
          z-index: 1000;
        }

        .modal {
          background: white;
          border-radius: 1rem;
          width: 90%;
          max-width: 600px;
          max-height: 90vh;
          overflow: hidden;
          display: flex;
          flex-direction: column;
        }

        .modal-header {
          display: flex;
          justify-content: space-between;
          align-items: center;
          padding: 1.5rem;
          border-bottom: 1px solid #e5e7eb;
        }

        .modal-header h3 {
          margin: 0;
          font-size: 1.25rem;
          font-weight: 600;
        }

        .close-btn {
          background: none;
          border: none;
          font-size: 1.5rem;
          cursor: pointer;
          color: #6b7280;
          padding: 0.5rem;
        }

        .file-details {
          padding: 1.5rem;
          overflow-y: auto;
          flex: 1;
        }

        .detail-section {
          margin-bottom: 2rem;
        }

        .detail-section h4 {
          margin: 0 0 1rem 0;
          font-size: 1rem;
          font-weight: 600;
          color: #111827;
        }

        .detail-grid {
          display: flex;
          flex-direction: column;
          gap: 0.75rem;
        }

        .detail-item {
          display: flex;
          justify-content: space-between;
          align-items: center;
          padding: 0.75rem 0;
          border-bottom: 1px solid #f3f4f6;
        }

        .label {
          color: #6b7280;
          font-weight: 500;
        }

        .value {
          font-weight: 600;
          text-align: right;
          max-width: 60%;
          word-break: break-all;
        }

        .mono {
          font-family: monospace;
          font-size: 0.875rem;
        }

        .pinned {
          color: #059669;
        }

        .unpinned {
          color: #6b7280;
        }

        .gateway-links {
          display: flex;
          flex-direction: column;
          gap: 0.5rem;
        }

        .gateway-link {
          display: flex;
          align-items: center;
          gap: 0.5rem;
          padding: 0.75rem 1rem;
          border: 1px solid #e5e7eb;
          border-radius: 0.5rem;
          text-decoration: none;
          color: #374151;
          transition: all 0.2s;
        }

        .gateway-link:hover {
          background: #f9fafb;
          border-color: #667eea;
        }

        .modal-actions {
          display: flex;
          justify-content: flex-end;
          gap: 1rem;
          padding: 1.5rem;
          border-top: 1px solid #e5e7eb;
        }

        .btn {
          display: flex;
          align-items: center;
          gap: 0.5rem;
          padding: 0.75rem 1.5rem;
          border: none;
          border-radius: 0.5rem;
          font-size: 0.9375rem;
          font-weight: 500;
          cursor: pointer;
          transition: all 0.2s;
        }

        .btn-primary {
          background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
          color: white;
        }

        .btn-secondary {
          background: #f3f4f6;
          color: #374151;
        }
      `}</style>
    </div>
  );
};
