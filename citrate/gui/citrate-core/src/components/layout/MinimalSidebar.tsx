/**
 * Minimal Sidebar Component
 *
 * Sprint 10: Full Feature Exposure (WP-10.1)
 *
 * A comprehensive sidebar with grouped navigation exposing all features:
 * - Blockchain: Wallet, DAG Explorer, Contracts
 * - AI & Models: Models, LoRA Training, Marketplace, IPFS Storage
 * - Developer: Terminal, GPU Compute
 * - Settings
 */

import React, { useState, useEffect } from 'react';
import {
  Menu,
  Wifi,
  WifiOff,
  Box,
  Users,
  Settings,
  Github,
  Wallet,
  Network,
  Brain,
  ShoppingBag,
  Database,
  FileCode,
  ChevronLeft,
  Activity,
  Terminal,
  Cpu,
  Sparkles,
  Layers,
  Code
} from 'lucide-react';
import { nodeService } from '../../services/tauri';
import { NodeStatus } from '../../types';
import citrateLogo from '../../assets/citrate_lockup.png';

type TabType = 'dashboard' | 'wallet' | 'dag' | 'models' | 'lora' | 'marketplace' | 'ipfs' | 'contracts' | 'terminal' | 'gpu' | 'settings';

interface MinimalSidebarProps {
  currentTab: TabType;
  onTabChange: (tab: TabType) => void;
}

interface NavItem {
  id: TabType;
  label: string;
  icon: React.ReactNode;
  shortcut?: string;
}

interface NavGroup {
  label: string;
  icon: React.ReactNode;
  items: NavItem[];
}

export const MinimalSidebar: React.FC<MinimalSidebarProps> = ({
  currentTab,
  onTabChange
}) => {
  const [nodeStatus, setNodeStatus] = useState<NodeStatus | null>(null);
  const [isExpanded, setIsExpanded] = useState(false);
  const [isHovered, setIsHovered] = useState(false);

  // Grouped navigation
  const navGroups: NavGroup[] = [
    {
      label: 'Blockchain',
      icon: <Layers size={14} />,
      items: [
        { id: 'wallet', label: 'Wallet', icon: <Wallet size={20} />, shortcut: 'Ctrl+2' },
        { id: 'dag', label: 'DAG Explorer', icon: <Network size={20} />, shortcut: 'Ctrl+3' },
        { id: 'contracts', label: 'Contracts', icon: <FileCode size={20} />, shortcut: 'Ctrl+4' },
      ]
    },
    {
      label: 'AI & Models',
      icon: <Brain size={14} />,
      items: [
        { id: 'models', label: 'Models', icon: <Brain size={20} />, shortcut: 'Ctrl+5' },
        { id: 'lora', label: 'LoRA Training', icon: <Sparkles size={20} />, shortcut: 'Ctrl+6' },
        { id: 'marketplace', label: 'Marketplace', icon: <ShoppingBag size={20} />, shortcut: 'Ctrl+7' },
        { id: 'ipfs', label: 'IPFS Storage', icon: <Database size={20} />, shortcut: 'Ctrl+8' },
      ]
    },
    {
      label: 'Developer',
      icon: <Code size={14} />,
      items: [
        { id: 'terminal', label: 'Terminal', icon: <Terminal size={20} />, shortcut: 'Ctrl+T' },
        { id: 'gpu', label: 'GPU Compute', icon: <Cpu size={20} />, shortcut: 'Ctrl+G' },
      ]
    },
  ];

  // Flat list for collapsed mode
  const navItems: NavItem[] = navGroups.flatMap(g => g.items);

  useEffect(() => {
    fetchNodeStatus();
    const interval = setInterval(fetchNodeStatus, 5000);
    return () => clearInterval(interval);
  }, []);

  const fetchNodeStatus = async () => {
    try {
      const status = await nodeService.getStatus();
      setNodeStatus(status);
    } catch (err) {
      console.error('Failed to fetch node status:', err);
    }
  };

  const formatUptime = (seconds: number) => {
    const hours = Math.floor(seconds / 3600);
    const minutes = Math.floor((seconds % 3600) / 60);
    if (hours > 0) return `${hours}h ${minutes}m`;
    return `${minutes}m`;
  };

  const shouldShowExpanded = isExpanded || isHovered;

  return (
    <div
      className={`minimal-sidebar ${shouldShowExpanded ? 'expanded' : 'collapsed'}`}
      onMouseEnter={() => setIsHovered(true)}
      onMouseLeave={() => setIsHovered(false)}
    >
      {/* Header */}
      <div className="sidebar-header">
        {shouldShowExpanded ? (
          <>
            <img src={citrateLogo} alt="Citrate" className="logo" />
            <button
              className="toggle-btn"
              onClick={() => setIsExpanded(!isExpanded)}
              title={isExpanded ? 'Collapse' : 'Lock open'}
            >
              {isExpanded ? <ChevronLeft size={16} /> : <Menu size={16} />}
            </button>
          </>
        ) : (
          <button
            className="menu-btn"
            onClick={() => setIsExpanded(true)}
            title="Expand sidebar"
          >
            <Menu size={20} />
          </button>
        )}
      </div>

      {/* Node Status */}
      <div className="node-status">
        <div className={`status-indicator ${nodeStatus?.running ? 'online' : 'offline'}`}>
          {nodeStatus?.running ? <Wifi size={16} /> : <WifiOff size={16} />}
          {shouldShowExpanded && (
            <span>{nodeStatus?.running ? 'Online' : 'Offline'}</span>
          )}
        </div>

        {shouldShowExpanded && nodeStatus?.running && (
          <div className="status-details">
            <div className="status-row">
              <Box size={14} />
              <span>{nodeStatus.blockHeight.toLocaleString()}</span>
            </div>
            <div className="status-row">
              <Users size={14} />
              <span>{nodeStatus.peerCount} peers</span>
            </div>
            <div className="status-row">
              <Activity size={14} />
              <span>{formatUptime(nodeStatus.uptime)}</span>
            </div>
          </div>
        )}
      </div>

      {/* Navigation */}
      <nav className="sidebar-nav">
        {/* Dashboard/Chat is always active since it's the main view */}
        <button
          className={`nav-item ${currentTab === 'dashboard' ? 'active' : ''}`}
          onClick={() => onTabChange('dashboard')}
          title="Chat Dashboard (Ctrl+1)"
        >
          <div className="nav-icon">
            <Activity size={20} />
          </div>
          {shouldShowExpanded && (
            <span className="nav-label">Dashboard</span>
          )}
        </button>

        {shouldShowExpanded ? (
          /* Grouped navigation when expanded */
          <>
            {navGroups.map((group, groupIndex) => (
              <div key={group.label} className="nav-group">
                <div className="nav-group-label">
                  {group.icon}
                  <span>{group.label}</span>
                </div>
                {group.items.map(item => (
                  <button
                    key={item.id}
                    className={`nav-item ${currentTab === item.id ? 'active' : ''}`}
                    onClick={() => onTabChange(item.id)}
                    title={`${item.label}${item.shortcut ? ` (${item.shortcut})` : ''}`}
                  >
                    <div className="nav-icon">{item.icon}</div>
                    <span className="nav-label">{item.label}</span>
                  </button>
                ))}
              </div>
            ))}
          </>
        ) : (
          /* Flat navigation when collapsed */
          <>
            <div className="nav-divider" />
            {navItems.map(item => (
              <button
                key={item.id}
                className={`nav-item ${currentTab === item.id ? 'active' : ''}`}
                onClick={() => onTabChange(item.id)}
                title={`${item.label}${item.shortcut ? ` (${item.shortcut})` : ''}`}
              >
                <div className="nav-icon">{item.icon}</div>
              </button>
            ))}
          </>
        )}

        {/* Settings always at the bottom */}
        <div className="nav-divider" style={{ marginTop: 'auto' }} />
        <button
          className={`nav-item ${currentTab === 'settings' ? 'active' : ''}`}
          onClick={() => onTabChange('settings')}
          title="Settings (Ctrl+,)"
        >
          <div className="nav-icon"><Settings size={20} /></div>
          {shouldShowExpanded && (
            <span className="nav-label">Settings</span>
          )}
        </button>
      </nav>

      {/* Footer */}
      <div className="sidebar-footer">
        <a
          href="https://github.com/saulbuilds/citrate"
          target="_blank"
          rel="noopener noreferrer"
          className="github-link"
          title="View on GitHub"
        >
          <Github size={18} />
          {shouldShowExpanded && <span>GitHub</span>}
        </a>
        {shouldShowExpanded && (
          <span className="version">v3.0.0</span>
        )}
      </div>

      <style jsx>{`
        .minimal-sidebar {
          display: flex;
          flex-direction: column;
          background: #ffffff;
          border-right: 1px solid #e5e7eb;
          transition: width 0.2s ease;
          overflow: hidden;
        }

        .minimal-sidebar.collapsed {
          width: 64px;
        }

        .minimal-sidebar.expanded {
          width: 220px;
        }

        .sidebar-header {
          display: flex;
          align-items: center;
          justify-content: space-between;
          padding: 1rem;
          min-height: 64px;
          border-bottom: 1px solid #e5e7eb;
        }

        .logo {
          max-width: 140px;
          height: auto;
        }

        .toggle-btn, .menu-btn {
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

        .toggle-btn:hover, .menu-btn:hover {
          background: #e5e7eb;
          color: #374151;
        }

        .node-status {
          padding: 1rem;
          border-bottom: 1px solid #e5e7eb;
        }

        .status-indicator {
          display: flex;
          align-items: center;
          gap: 0.5rem;
          padding: 0.5rem 0.75rem;
          border-radius: 0.5rem;
          font-size: 0.8125rem;
          font-weight: 500;
        }

        .collapsed .status-indicator {
          justify-content: center;
          padding: 0.5rem;
        }

        .status-indicator.online {
          background: #d1fae5;
          color: #059669;
        }

        .status-indicator.offline {
          background: #fee2e2;
          color: #dc2626;
        }

        .status-details {
          margin-top: 0.75rem;
          display: flex;
          flex-direction: column;
          gap: 0.375rem;
        }

        .status-row {
          display: flex;
          align-items: center;
          gap: 0.5rem;
          font-size: 0.75rem;
          color: #6b7280;
          padding-left: 0.25rem;
        }

        .sidebar-nav {
          flex: 1;
          padding: 0.75rem 0.5rem;
          overflow-y: auto;
        }

        .nav-divider {
          height: 1px;
          background: #e5e7eb;
          margin: 0.5rem 0.75rem;
        }

        .nav-group {
          margin-top: 0.75rem;
        }

        .nav-group-label {
          display: flex;
          align-items: center;
          gap: 0.5rem;
          padding: 0.5rem 0.75rem;
          font-size: 0.6875rem;
          font-weight: 600;
          text-transform: uppercase;
          letter-spacing: 0.05em;
          color: #9ca3af;
        }

        .nav-item {
          display: flex;
          align-items: center;
          gap: 0.75rem;
          width: 100%;
          padding: 0.625rem 0.75rem;
          margin-bottom: 0.25rem;
          background: none;
          border: none;
          border-radius: 0.5rem;
          color: #374151;
          font-size: 0.875rem;
          font-weight: 500;
          cursor: pointer;
          transition: all 0.15s;
        }

        .collapsed .nav-item {
          justify-content: center;
          padding: 0.625rem;
        }

        .nav-item:hover {
          background: #f9f9f9;
        }

        .nav-item.active {
          background: #ffa500;
          color: white;
        }

        .nav-icon {
          display: flex;
          align-items: center;
          justify-content: center;
          flex-shrink: 0;
        }

        .nav-label {
          white-space: nowrap;
          overflow: hidden;
          text-overflow: ellipsis;
        }

        .sidebar-footer {
          padding: 1rem;
          border-top: 1px solid #e5e7eb;
          display: flex;
          align-items: center;
          justify-content: space-between;
        }

        .collapsed .sidebar-footer {
          justify-content: center;
        }

        .github-link {
          display: flex;
          align-items: center;
          gap: 0.5rem;
          color: #374151;
          text-decoration: none;
          font-size: 0.8125rem;
          transition: color 0.15s;
        }

        .github-link:hover {
          color: #ffa500;
        }

        .version {
          font-size: 0.6875rem;
          color: #9ca3af;
        }
      `}</style>
    </div>
  );
};
