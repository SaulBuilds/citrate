import React, { useState, useEffect } from 'react';
import {
  Database,
  Upload,
  Download,
  FolderOpen,
  File,
  Link,
  Copy,
  Trash2,
  Share,
  Globe,
  Server,
  Zap,
  Activity,
  HardDrive,
  Users,
  Clock,
  CheckCircle,
  AlertCircle,
  RefreshCw,
  Pin,
  Unlink
} from 'lucide-react';

interface IPFSFile {
  cid: string;
  name: string;
  size: number;
  type: 'file' | 'directory';
  uploaded: Date;
  pinned: boolean;
  replicas: number;
  gateway?: string;
}

interface IPFSNode {
  id: string;
  status: 'online' | 'offline' | 'syncing';
  version: string;
  peers: number;
  storage: {
    used: number;
    available: number;
    total: number;
  };
  bandwidth: {
    in: number;
    out: number;
  };
}

interface IPFSPin {
  cid: string;
  name: string;
  size: number;
  pinTime: Date;
  type: 'recursive' | 'direct';
}

export const IPFS: React.FC = () => {
  const [files, setFiles] = useState<IPFSFile[]>([]);
  const [nodeInfo, setNodeInfo] = useState<IPFSNode | null>(null);
  const [pins, setPins] = useState<IPFSPin[]>([]);
  const [selectedFile, setSelectedFile] = useState<IPFSFile | null>(null);
  const [isUploading, setIsUploading] = useState(false);
  const [uploadProgress, setUploadProgress] = useState(0);
  const [searchQuery, setSearchQuery] = useState('');
  const [showNodeSetup, setShowNodeSetup] = useState(false);
  const [loading, setLoading] = useState(false);

  useEffect(() => {
    loadNodeInfo();
    loadFiles();
    loadPins();

    // Set up periodic updates
    const interval = setInterval(() => {
      loadNodeInfo();
    }, 5000);

    return () => clearInterval(interval);
  }, []);

  const loadNodeInfo = async () => {
    try {
      // TODO: Integrate with actual IPFS node
      const mockNodeInfo: IPFSNode = {
        id: 'QmYourNodeId12345...',
        status: 'online',
        version: '0.28.0',
        peers: 247,
        storage: {
          used: 2.4 * 1024 * 1024 * 1024, // 2.4 GB
          available: 7.6 * 1024 * 1024 * 1024, // 7.6 GB
          total: 10 * 1024 * 1024 * 1024 // 10 GB
        },
        bandwidth: {
          in: 1024 * 150, // 150 KB/s
          out: 1024 * 89  // 89 KB/s
        }
      };

      setNodeInfo(mockNodeInfo);
    } catch (error) {
      console.error('Failed to load IPFS node info:', error);
      setNodeInfo(null);
    }
  };

  const loadFiles = async () => {
    setLoading(true);
    try {
      // TODO: Integrate with actual IPFS API
      const mockFiles: IPFSFile[] = [
        {
          cid: 'QmXoypizjW3WknFiJnKLwHCnL72vedxjQkDDP1mXWo6uco',
          name: 'my-ai-model.pt',
          size: 1024 * 1024 * 500, // 500 MB
          type: 'file',
          uploaded: new Date('2024-01-15'),
          pinned: true,
          replicas: 12,
          gateway: 'https://ipfs.io/ipfs/'
        },
        {
          cid: 'QmYzApizjW3WknFiJnKLwHCnL72vedxjQkDDP1mXWo6usr',
          name: 'dataset-images',
          size: 1024 * 1024 * 1024 * 2.1, // 2.1 GB
          type: 'directory',
          uploaded: new Date('2024-01-10'),
          pinned: true,
          replicas: 8,
          gateway: 'https://gateway.pinata.cloud/ipfs/'
        },
        {
          cid: 'QmZrBpizjW3WknFiJnKLwHCnL72vedxjQkDDP1mXWo6upr',
          name: 'smart-contract.sol',
          size: 1024 * 15, // 15 KB
          type: 'file',
          uploaded: new Date('2024-01-20'),
          pinned: false,
          replicas: 3
        }
      ];

      setFiles(mockFiles);
    } catch (error) {
      console.error('Failed to load IPFS files:', error);
    } finally {
      setLoading(false);
    }
  };

  const loadPins = async () => {
    try {
      // TODO: Load actual pinned content
      const mockPins: IPFSPin[] = [
        {
          cid: 'QmXoypizjW3WknFiJnKLwHCnL72vedxjQkDDP1mXWo6uco',
          name: 'my-ai-model.pt',
          size: 1024 * 1024 * 500,
          pinTime: new Date('2024-01-15'),
          type: 'recursive'
        },
        {
          cid: 'QmYzApizjW3WknFiJnKLwHCnL72vedxjQkDDP1mXWo6usr',
          name: 'dataset-images',
          size: 1024 * 1024 * 1024 * 2.1,
          pinTime: new Date('2024-01-10'),
          type: 'recursive'
        }
      ];

      setPins(mockPins);
    } catch (error) {
      console.error('Failed to load IPFS pins:', error);
    }
  };

  const handleFileUpload = async (event: React.ChangeEvent<HTMLInputElement>) => {
    const file = event.target.files?.[0];
    if (!file) return;

    setIsUploading(true);
    setUploadProgress(0);

    try {
      // TODO: Implement actual IPFS upload
      const progressInterval = setInterval(() => {
        setUploadProgress(prev => {
          if (prev >= 90) {
            clearInterval(progressInterval);
            return 90;
          }
          return prev + Math.random() * 10;
        });
      }, 200);

      // Simulate upload
      await new Promise(resolve => setTimeout(resolve, 3000));

      clearInterval(progressInterval);
      setUploadProgress(100);

      // Add mock file to list
      const newFile: IPFSFile = {
        cid: 'Qm' + Math.random().toString(36).substr(2, 44),
        name: file.name,
        size: file.size,
        type: 'file',
        uploaded: new Date(),
        pinned: true,
        replicas: 1
      };

      setFiles(prev => [newFile, ...prev]);

      setTimeout(() => {
        setIsUploading(false);
        setUploadProgress(0);
      }, 1000);
    } catch (error) {
      console.error('Failed to upload file:', error);
      setIsUploading(false);
      setUploadProgress(0);
    }
  };

  const togglePin = async (file: IPFSFile) => {
    try {
      // TODO: Implement actual pin/unpin
      const updatedFiles = files.map(f =>
        f.cid === file.cid ? { ...f, pinned: !f.pinned } : f
      );
      setFiles(updatedFiles);

      if (!file.pinned) {
        // Add to pins
        const newPin: IPFSPin = {
          cid: file.cid,
          name: file.name,
          size: file.size,
          pinTime: new Date(),
          type: 'recursive'
        };
        setPins(prev => [newPin, ...prev]);
      } else {
        // Remove from pins
        setPins(prev => prev.filter(p => p.cid !== file.cid));
      }
    } catch (error) {
      console.error('Failed to toggle pin:', error);
    }
  };

  const copyToClipboard = (text: string) => {
    navigator.clipboard.writeText(text);
  };

  const formatSize = (bytes: number) => {
    const sizes = ['B', 'KB', 'MB', 'GB', 'TB'];
    if (bytes === 0) return '0 B';
    const i = Math.floor(Math.log(bytes) / Math.log(1024));
    return Math.round(bytes / Math.pow(1024, i) * 100) / 100 + ' ' + sizes[i];
  };

  const formatBandwidth = (bytesPerSecond: number) => {
    return formatSize(bytesPerSecond) + '/s';
  };

  const filteredFiles = files.filter(file =>
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
              disabled={isUploading}
            />
          </label>
        </div>
      </div>

      {/* Node Status */}
      <div className="node-status">
        <div className="status-card">
          <div className="status-header">
            <div className="status-indicator">
              <div className={`status-dot ${nodeInfo?.status || 'offline'}`} />
              <span>IPFS Node {nodeInfo?.status || 'Offline'}</span>
            </div>
            <button
              className="btn-icon"
              onClick={loadNodeInfo}
              title="Refresh status"
            >
              <RefreshCw size={16} />
            </button>
          </div>

          {nodeInfo && (
            <div className="status-stats">
              <div className="stat">
                <Users size={16} />
                <span>{nodeInfo.peers} peers</span>
              </div>
              <div className="stat">
                <Activity size={16} />
                <span>↓ {formatBandwidth(nodeInfo.bandwidth.in)}</span>
              </div>
              <div className="stat">
                <Activity size={16} />
                <span>↑ {formatBandwidth(nodeInfo.bandwidth.out)}</span>
              </div>
              <div className="stat">
                <HardDrive size={16} />
                <span>{formatSize(nodeInfo.storage.used)} / {formatSize(nodeInfo.storage.total)}</span>
              </div>
            </div>
          )}
        </div>

        {nodeInfo && (
          <div className="storage-usage">
            <div className="usage-bar">
              <div
                className="usage-fill"
                style={{
                  width: `${(nodeInfo.storage.used / nodeInfo.storage.total) * 100}%`
                }}
              />
            </div>
            <span className="usage-text">
              {Math.round((nodeInfo.storage.used / nodeInfo.storage.total) * 100)}% used
            </span>
          </div>
        )}
      </div>

      {showNodeSetup && (
        <div className="node-setup">
          <h3>IPFS Node Configuration</h3>
          <div className="setup-options">
            <button className="setup-btn">
              <Server size={20} />
              <div>
                <strong>Start Local Node</strong>
                <p>Run IPFS node alongside Citrate</p>
              </div>
            </button>
            <button className="setup-btn">
              <Globe size={20} />
              <div>
                <strong>Connect to Remote</strong>
                <p>Connect to existing IPFS node</p>
              </div>
            </button>
            <button className="setup-btn">
              <Zap size={20} />
              <div>
                <strong>Use Pinning Service</strong>
                <p>Configure Pinata, Infura, or custom</p>
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
            <div
              className="progress-fill"
              style={{ width: `${uploadProgress}%` }}
            />
          </div>
        </div>
      )}

      {/* Search and Filters */}
      <div className="content-controls">
        <div className="search-bar">
          <input
            type="text"
            placeholder="Search files and CIDs..."
            value={searchQuery}
            onChange={(e) => setSearchQuery(e.target.value)}
          />
        </div>
        <div className="view-tabs">
          <button className="tab active">Files</button>
          <button className="tab">Pins</button>
          <button className="tab">Peers</button>
        </div>
      </div>

      {/* Files List */}
      <div className="files-container">
        {loading ? (
          <div className="loading">Loading IPFS content...</div>
        ) : (
          <>
            {filteredFiles.map(file => (
              <div
                key={file.cid}
                className="file-item"
                onClick={() => setSelectedFile(file)}
              >
                <div className="file-icon">
                  {file.type === 'directory' ? (
                    <FolderOpen size={24} />
                  ) : (
                    <File size={24} />
                  )}
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
                  <div className="replicas">
                    <Users size={14} />
                    <span>{file.replicas}</span>
                  </div>
                </div>

                <div className="file-actions">
                  <button
                    className="action-btn"
                    onClick={(e) => {
                      e.stopPropagation();
                      togglePin(file);
                    }}
                    title={file.pinned ? 'Unpin' : 'Pin'}
                  >
                    {file.pinned ? <Unlink size={16} /> : <Pin size={16} />}
                  </button>
                  <button
                    className="action-btn"
                    onClick={(e) => {
                      e.stopPropagation();
                      copyToClipboard(file.cid);
                    }}
                    title="Copy CID"
                  >
                    <Copy size={16} />
                  </button>
                  <button
                    className="action-btn"
                    onClick={(e) => {
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

            {filteredFiles.length === 0 && !loading && (
              <div className="empty-state">
                <Database size={48} />
                <p>No files found</p>
                <p className="text-muted">Upload your first file to get started</p>
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

        .node-status {
          background: white;
          border-radius: 1rem;
          padding: 1.5rem;
          box-shadow: 0 1px 3px rgba(0, 0, 0, 0.1);
          margin-bottom: 2rem;
        }

        .status-card {
          margin-bottom: 1rem;
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

        .status-dot.syncing {
          background: #f59e0b;
          animation: pulse 2s infinite;
        }

        @keyframes pulse {
          0%, 100% { opacity: 1; }
          50% { opacity: 0.5; }
        }

        .status-stats {
          display: flex;
          gap: 2rem;
          flex-wrap: wrap;
        }

        .stat {
          display: flex;
          align-items: center;
          gap: 0.5rem;
          color: #6b7280;
          font-size: 0.875rem;
        }

        .storage-usage {
          display: flex;
          align-items: center;
          gap: 1rem;
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
        }

        .setup-btn:hover {
          border-color: #667eea;
          box-shadow: 0 2px 4px rgba(102, 126, 234, 0.1);
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

        .pinned, .unpinned {
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

        .replicas {
          display: flex;
          align-items: center;
          gap: 0.25rem;
          font-size: 0.75rem;
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

        .btn-primary {
          background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
          color: white;
        }

        .btn-primary:hover {
          transform: translateY(-1px);
          box-shadow: 0 4px 8px rgba(102, 126, 234, 0.3);
        }

        .btn-secondary {
          background: #f3f4f6;
          color: #374151;
          border: 1px solid #e5e7eb;
        }

        .btn-secondary:hover {
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

        .btn-icon:hover {
          background: #f3f4f6;
          color: #374151;
        }

        .loading, .empty-state {
          padding: 3rem;
          text-align: center;
          color: #6b7280;
        }

        .empty-state svg {
          margin-bottom: 1rem;
          color: #9ca3af;
        }

        .text-muted {
          color: #9ca3af;
          margin-top: 0.5rem;
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
    return Math.round(bytes / Math.pow(1024, i) * 100) / 100 + ' ' + sizes[i];
  };

  return (
    <div className="modal-overlay" onClick={onClose}>
      <div className="modal" onClick={e => e.stopPropagation()}>
        <div className="modal-header">
          <h3>{file.name}</h3>
          <button className="close-btn" onClick={onClose}>×</button>
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
                <span className="label">Replicas:</span>
                <span className="value">{file.replicas}</span>
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
          <button
            className="btn btn-primary"
            onClick={() => onPin(file)}
          >
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