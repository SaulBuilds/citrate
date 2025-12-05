/**
 * StatusSidebar Component
 *
 * Displays real-time status of node connection, wallet state, and agent status.
 * Collapsible sections with auto-refresh capability.
 */

import React, { useState, useEffect, useCallback } from 'react';
import { invoke } from '@tauri-apps/api/core';
import {
  Server,
  Wallet,
  Bot,
  ChevronDown,
  ChevronUp,
  RefreshCw,
  Wifi,
  Box,
  Users,
  Clock,
  Coins,
  Activity,
  Cpu,
  HardDrive,
} from 'lucide-react';

interface NodeStatus {
  running: boolean;
  block_height?: number;
  peer_count?: number;
  sync_status?: 'synced' | 'syncing' | 'not_synced';
  network?: string;
}

interface WalletInfo {
  address: string;
  balance: string;
  pending_tx_count: number;
}

interface AgentStatusInfo {
  initialized: boolean;
  enabled: boolean;
  llm_backend?: string;
  active_sessions?: number;
  streaming_enabled?: boolean;
}

interface StatusSidebarProps {
  visible?: boolean;
  onClose?: () => void;
}

export const StatusSidebar: React.FC<StatusSidebarProps> = ({
  visible = true,
  onClose: _onClose,
}) => {
  const [nodeStatus, setNodeStatus] = useState<NodeStatus | null>(null);
  const [walletInfo, setWalletInfo] = useState<WalletInfo | null>(null);
  const [agentStatus, setAgentStatus] = useState<AgentStatusInfo | null>(null);
  const [loading, setLoading] = useState(true);
  const [expandedSections, setExpandedSections] = useState({
    node: true,
    wallet: true,
    agent: true,
  });

  const fetchStatus = useCallback(async () => {
    try {
      // Fetch node status
      try {
        const node = await invoke<NodeStatus>('get_node_status');
        setNodeStatus(node);
      } catch (e) {
        setNodeStatus({ running: false });
      }

      // Fetch wallet info
      try {
        const accounts = await invoke<any[]>('get_accounts');
        if (accounts && accounts.length > 0) {
          const account = accounts[0];
          setWalletInfo({
            address: account.address || account.pubkey,
            balance: account.balance?.toString() || '0',
            pending_tx_count: 0,
          });
        }
      } catch (e) {
        setWalletInfo(null);
      }

      // Fetch agent status
      try {
        const agent = await invoke<AgentStatusInfo>('agent_get_status');
        setAgentStatus(agent);
      } catch (e) {
        setAgentStatus({ initialized: false, enabled: false });
      }
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    fetchStatus();
    const interval = setInterval(fetchStatus, 5000); // Refresh every 5 seconds
    return () => clearInterval(interval);
  }, [fetchStatus]);

  const toggleSection = (section: keyof typeof expandedSections) => {
    setExpandedSections((prev) => ({
      ...prev,
      [section]: !prev[section],
    }));
  };

  const formatBalance = (balance: string) => {
    const num = Number(BigInt(balance)) / 1e18;
    return num.toLocaleString(undefined, {
      minimumFractionDigits: 2,
      maximumFractionDigits: 4,
    });
  };

  const truncateAddress = (address: string) => {
    if (!address) return '';
    return `${address.slice(0, 8)}...${address.slice(-6)}`;
  };

  if (!visible) return null;

  return (
    <aside className="status-sidebar">
      <div className="sidebar-header">
        <h2>Status</h2>
        <button className="btn-refresh" onClick={fetchStatus} title="Refresh">
          <RefreshCw size={16} className={loading ? 'spinning' : ''} />
        </button>
      </div>

      <div className="sidebar-content">
        {/* Node Status Section */}
        <section className="status-section">
          <button
            className="section-header"
            onClick={() => toggleSection('node')}
          >
            <div className="section-title">
              <Server size={18} />
              <span>Node</span>
              <span
                className={`status-dot ${nodeStatus?.running ? 'online' : 'offline'}`}
              />
            </div>
            {expandedSections.node ? (
              <ChevronUp size={16} />
            ) : (
              <ChevronDown size={16} />
            )}
          </button>

          {expandedSections.node && (
            <div className="section-content">
              <StatusRow
                icon={<Wifi size={14} />}
                label="Connection"
                value={nodeStatus?.running ? 'Connected' : 'Disconnected'}
                status={nodeStatus?.running ? 'success' : 'error'}
              />
              <StatusRow
                icon={<Box size={14} />}
                label="Block Height"
                value={
                  nodeStatus?.block_height?.toLocaleString() || 'N/A'
                }
              />
              <StatusRow
                icon={<Users size={14} />}
                label="Peers"
                value={nodeStatus?.peer_count?.toString() || '0'}
              />
              <StatusRow
                icon={<Activity size={14} />}
                label="Sync Status"
                value={
                  nodeStatus?.sync_status === 'synced'
                    ? 'Synced'
                    : nodeStatus?.sync_status === 'syncing'
                    ? 'Syncing...'
                    : 'Not Synced'
                }
                status={
                  nodeStatus?.sync_status === 'synced'
                    ? 'success'
                    : nodeStatus?.sync_status === 'syncing'
                    ? 'warning'
                    : 'error'
                }
              />
              <StatusRow
                icon={<HardDrive size={14} />}
                label="Network"
                value={nodeStatus?.network || 'Unknown'}
              />
            </div>
          )}
        </section>

        {/* Wallet Status Section */}
        <section className="status-section">
          <button
            className="section-header"
            onClick={() => toggleSection('wallet')}
          >
            <div className="section-title">
              <Wallet size={18} />
              <span>Wallet</span>
              <span
                className={`status-dot ${walletInfo ? 'online' : 'offline'}`}
              />
            </div>
            {expandedSections.wallet ? (
              <ChevronUp size={16} />
            ) : (
              <ChevronDown size={16} />
            )}
          </button>

          {expandedSections.wallet && (
            <div className="section-content">
              {walletInfo ? (
                <>
                  <StatusRow
                    icon={<Wallet size={14} />}
                    label="Address"
                    value={truncateAddress(walletInfo.address)}
                    copyable={walletInfo.address}
                  />
                  <StatusRow
                    icon={<Coins size={14} />}
                    label="Balance"
                    value={`${formatBalance(walletInfo.balance)} CIT`}
                    highlight
                  />
                  <StatusRow
                    icon={<Clock size={14} />}
                    label="Pending Txs"
                    value={walletInfo.pending_tx_count.toString()}
                  />
                </>
              ) : (
                <div className="no-data">No wallet connected</div>
              )}
            </div>
          )}
        </section>

        {/* Agent Status Section */}
        <section className="status-section">
          <button
            className="section-header"
            onClick={() => toggleSection('agent')}
          >
            <div className="section-title">
              <Bot size={18} />
              <span>Agent</span>
              <span
                className={`status-dot ${
                  agentStatus?.initialized && agentStatus?.enabled
                    ? 'online'
                    : 'offline'
                }`}
              />
            </div>
            {expandedSections.agent ? (
              <ChevronUp size={16} />
            ) : (
              <ChevronDown size={16} />
            )}
          </button>

          {expandedSections.agent && (
            <div className="section-content">
              <StatusRow
                icon={<Activity size={14} />}
                label="Status"
                value={
                  agentStatus?.initialized
                    ? agentStatus?.enabled
                      ? 'Active'
                      : 'Disabled'
                    : 'Not Initialized'
                }
                status={
                  agentStatus?.initialized && agentStatus?.enabled
                    ? 'success'
                    : agentStatus?.initialized
                    ? 'warning'
                    : 'error'
                }
              />
              <StatusRow
                icon={<Cpu size={14} />}
                label="LLM Backend"
                value={agentStatus?.llm_backend || 'N/A'}
              />
              <StatusRow
                icon={<Users size={14} />}
                label="Active Sessions"
                value={agentStatus?.active_sessions?.toString() || '0'}
              />
              <StatusRow
                icon={<Activity size={14} />}
                label="Streaming"
                value={agentStatus?.streaming_enabled ? 'Enabled' : 'Disabled'}
                status={agentStatus?.streaming_enabled ? 'success' : 'neutral'}
              />
            </div>
          )}
        </section>
      </div>

      <style jsx>{`
        .status-sidebar {
          width: 280px;
          background: white;
          border-left: 1px solid #e5e7eb;
          display: flex;
          flex-direction: column;
          height: 100%;
        }

        .sidebar-header {
          display: flex;
          justify-content: space-between;
          align-items: center;
          padding: 1rem;
          border-bottom: 1px solid #e5e7eb;
        }

        .sidebar-header h2 {
          margin: 0;
          font-size: 1rem;
          font-weight: 600;
          color: #111827;
        }

        .btn-refresh {
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

        .btn-refresh:hover {
          background: #f3f4f6;
          color: #374151;
        }

        .sidebar-content {
          flex: 1;
          overflow-y: auto;
          padding: 0.5rem;
        }

        .status-section {
          margin-bottom: 0.5rem;
          border: 1px solid #e5e7eb;
          border-radius: 0.5rem;
          overflow: hidden;
        }

        .section-header {
          display: flex;
          justify-content: space-between;
          align-items: center;
          width: 100%;
          padding: 0.75rem;
          background: #f9fafb;
          border: none;
          cursor: pointer;
          transition: background 0.2s;
        }

        .section-header:hover {
          background: #f3f4f6;
        }

        .section-title {
          display: flex;
          align-items: center;
          gap: 0.5rem;
          font-weight: 500;
          color: #374151;
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

        .section-content {
          padding: 0.5rem 0.75rem;
          background: white;
        }

        .no-data {
          padding: 0.5rem 0;
          text-align: center;
          color: #9ca3af;
          font-size: 0.875rem;
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

        @media (max-width: 1024px) {
          .status-sidebar {
            position: absolute;
            right: 0;
            top: 0;
            bottom: 0;
            z-index: 30;
            box-shadow: -2px 0 8px rgba(0, 0, 0, 0.1);
          }
        }
      `}</style>
    </aside>
  );
};

// =============================================================================
// StatusRow Component
// =============================================================================

interface StatusRowProps {
  icon: React.ReactNode;
  label: string;
  value: string;
  status?: 'success' | 'warning' | 'error' | 'neutral';
  highlight?: boolean;
  copyable?: string;
}

const StatusRow: React.FC<StatusRowProps> = ({
  icon,
  label,
  value,
  status,
  highlight,
  copyable,
}) => {
  const [copied, setCopied] = useState(false);

  const handleCopy = async () => {
    if (copyable) {
      await navigator.clipboard.writeText(copyable);
      setCopied(true);
      setTimeout(() => setCopied(false), 2000);
    }
  };

  return (
    <div className={`status-row ${copyable ? 'copyable' : ''}`} onClick={handleCopy}>
      <div className="row-label">
        {icon}
        <span>{label}</span>
      </div>
      <div className={`row-value ${status || ''} ${highlight ? 'highlight' : ''}`}>
        <span>{copied ? 'Copied!' : value}</span>
      </div>

      <style jsx>{`
        .status-row {
          display: flex;
          justify-content: space-between;
          align-items: center;
          padding: 0.375rem 0;
          font-size: 0.8125rem;
        }

        .status-row.copyable {
          cursor: pointer;
          border-radius: 0.25rem;
          margin: 0 -0.25rem;
          padding: 0.375rem 0.25rem;
        }

        .status-row.copyable:hover {
          background: #f3f4f6;
        }

        .row-label {
          display: flex;
          align-items: center;
          gap: 0.375rem;
          color: #6b7280;
        }

        .row-value {
          font-weight: 500;
          color: #111827;
        }

        .row-value.success {
          color: #059669;
        }

        .row-value.warning {
          color: #d97706;
        }

        .row-value.error {
          color: #dc2626;
        }

        .row-value.highlight {
          color: #ffa500;
          font-weight: 600;
        }
      `}</style>
    </div>
  );
};

export default StatusSidebar;
