import { useEffect, useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import './App.css';
import citrateLogo from './assets/citrate_lockup.png';
import { Dashboard } from './components/Dashboard';
import { Wallet } from './components/Wallet';
import { DAGVisualization } from './components/DAGVisualization';
import { Models } from './components/Models';
import { Marketplace } from './components/Marketplace';
import { ChatBot } from './components/ChatBot';
import { IPFS } from './components/IPFS';
import { Contracts } from './components/Contracts';
import { Settings as SettingsView } from './components/Settings';
import { FirstTimeSetup } from './components/FirstTimeSetup';
import ErrorBoundary from './components/ErrorBoundary';
import { AppProvider, useAppTab } from './contexts/AppContext';
import { ThemeProvider } from './contexts/ThemeContext';
import { useKeyboardShortcuts, KeyboardShortcut } from './hooks/useKeyboardShortcuts';
import KeyboardShortcutsHelp from './components/KeyboardShortcutsHelp';
import {
  LayoutDashboard,
  Wallet as WalletIcon,
  Network,
  Brain,
  ShoppingBag,
  MessageSquare,
  Database,
  FileCode,
  Settings,
  Github
} from 'lucide-react';

/**
 * Main App Component (Inner)
 *
 * Uses AppContext for state persistence
 */
function AppInner() {
  const { currentTab, setCurrentTab } = useAppTab();
  const isNativeApp = typeof window !== 'undefined' && window.__TAURI__ !== undefined;
  const [showShortcutsHelp, setShowShortcutsHelp] = useState(false);

  // Define keyboard shortcuts
  const shortcuts: KeyboardShortcut[] = [
    {
      key: '/',
      shift: true,
      description: 'Show keyboard shortcuts',
      action: () => setShowShortcutsHelp(true),
    },
    {
      key: 'Escape',
      description: 'Close modal/dialog',
      action: () => setShowShortcutsHelp(false),
    },
    {
      key: '1',
      ctrl: true,
      description: 'Navigate to Dashboard',
      action: () => setCurrentTab('dashboard'),
    },
    {
      key: '2',
      ctrl: true,
      description: 'Navigate to Wallet',
      action: () => setCurrentTab('wallet'),
    },
    {
      key: '3',
      ctrl: true,
      description: 'Navigate to DAG Explorer',
      action: () => setCurrentTab('dag'),
    },
    {
      key: '4',
      ctrl: true,
      description: 'Navigate to AI Models',
      action: () => setCurrentTab('models'),
    },
    {
      key: '5',
      ctrl: true,
      description: 'Navigate to Marketplace',
      action: () => setCurrentTab('marketplace'),
    },
    {
      key: '6',
      ctrl: true,
      description: 'Navigate to AI Chat',
      action: () => setCurrentTab('chat'),
    },
    {
      key: '7',
      ctrl: true,
      description: 'Navigate to IPFS Storage',
      action: () => setCurrentTab('ipfs'),
    },
    {
      key: '8',
      ctrl: true,
      description: 'Navigate to Contracts',
      action: () => setCurrentTab('contracts'),
    },
    {
      key: ',',
      ctrl: true,
      description: 'Open Settings',
      action: () => setCurrentTab('settings'),
    },
  ];

  // Register keyboard shortcuts
  useKeyboardShortcuts(shortcuts);

  useEffect(() => {
    const handler = (e: any) => {
      const hash = e?.detail?.hash;
      if (hash) {
        try { localStorage.setItem('dag_focus_hash', hash); } catch {}
      }
      setCurrentTab('dag');
    };
    window.addEventListener('open-dag-for-hash' as any, handler);

    // Auto-initialize on app start
    if (isNativeApp) {
      initializeApp();
    }

    return () => window.removeEventListener('open-dag-for-hash' as any, handler);
  }, [isNativeApp, setCurrentTab]);

  const initializeApp = async () => {
    try {
      console.log('Initializing Citrate app...');

      // Check for first-time setup and handle it
      const setupResult = await invoke('check_first_time_and_setup_if_needed');
      if (setupResult) {
        console.log('First-time setup completed:', setupResult);
      }

      // Switch to testnet mode and ensure connectivity
      await invoke('switch_to_testnet');

      // Start the node
      await invoke('start_node');

      // Ensure connectivity after a brief delay
      setTimeout(async () => {
        try {
          await invoke('ensure_connectivity');
          console.log('Connectivity check completed');
        } catch (error) {
          console.warn('Connectivity check failed:', error);
        }
      }, 3000);

    } catch (error) {
      console.error('App initialization failed:', error);
    }
  };

  const renderView = () => {
    switch (currentTab) {
      case 'dashboard':
        return <Dashboard />;
      case 'wallet':
        return <Wallet />;
      case 'dag':
        return <DAGVisualization />;
      case 'models':
        return <Models />;
      case 'marketplace':
        return <Marketplace />;
      case 'chat':
        return <ChatBot />;
      case 'ipfs':
        return <IPFS />;
      case 'contracts':
        return <Contracts />;
      case 'settings':
        return <SettingsView />;
      default:
        return <Dashboard />;
    }
  };

  return (
    <>
      <div className="app">
        <div className="sidebar">
        <div className="sidebar-header">
          <img src={citrateLogo} alt="Citrate" className="app-logo" />
          <p className="app-version">v3.0.0</p>
          <p className="app-mode">{isNativeApp ? '🖥️ Native' : '🌐 Web Mode'}</p>
        </div>

        <nav className="sidebar-nav" aria-label="Main navigation">
          <button
            className={`nav-item ${currentTab === 'dashboard' ? 'active' : ''}`}
            onClick={() => setCurrentTab('dashboard')}
            aria-label="Navigate to Dashboard"
            aria-current={currentTab === 'dashboard' ? 'page' : undefined}
          >
            <LayoutDashboard size={20} aria-hidden="true" />
            <span>Dashboard</span>
          </button>

          <button
            className={`nav-item ${currentTab === 'wallet' ? 'active' : ''}`}
            onClick={() => setCurrentTab('wallet')}
            aria-label="Navigate to Wallet"
            aria-current={currentTab === 'wallet' ? 'page' : undefined}
          >
            <WalletIcon size={20} aria-hidden="true" />
            <span>Wallet</span>
          </button>

          <button
            className={`nav-item ${currentTab === 'dag' ? 'active' : ''}`}
            onClick={() => setCurrentTab('dag')}
            aria-label="Navigate to DAG Explorer"
            aria-current={currentTab === 'dag' ? 'page' : undefined}
          >
            <Network size={20} aria-hidden="true" />
            <span>DAG Explorer</span>
          </button>

          <button
            className={`nav-item ${currentTab === 'models' ? 'active' : ''}`}
            onClick={() => setCurrentTab('models')}
            aria-label="Navigate to AI Models"
            aria-current={currentTab === 'models' ? 'page' : undefined}
          >
            <Brain size={20} aria-hidden="true" />
            <span>AI Models</span>
          </button>

          <button
            className={`nav-item ${currentTab === 'marketplace' ? 'active' : ''}`}
            onClick={() => setCurrentTab('marketplace')}
            aria-label="Navigate to Marketplace"
            aria-current={currentTab === 'marketplace' ? 'page' : undefined}
          >
            <ShoppingBag size={20} aria-hidden="true" />
            <span>Marketplace</span>
          </button>

          <button
            className={`nav-item ${currentTab === 'chat' ? 'active' : ''}`}
            onClick={() => setCurrentTab('chat')}
            aria-label="Navigate to AI Chat"
            aria-current={currentTab === 'chat' ? 'page' : undefined}
          >
            <MessageSquare size={20} aria-hidden="true" />
            <span>AI Chat</span>
          </button>

          <button
            className={`nav-item ${currentTab === 'ipfs' ? 'active' : ''}`}
            onClick={() => setCurrentTab('ipfs')}
            aria-label="Navigate to IPFS Storage"
            aria-current={currentTab === 'ipfs' ? 'page' : undefined}
          >
            <Database size={20} aria-hidden="true" />
            <span>IPFS Storage</span>
          </button>

          <button
            className={`nav-item ${currentTab === 'contracts' ? 'active' : ''}`}
            onClick={() => setCurrentTab('contracts')}
            aria-label="Navigate to Smart Contracts"
            aria-current={currentTab === 'contracts' ? 'page' : undefined}
          >
            <FileCode size={20} aria-hidden="true" />
            <span>Contracts</span>
          </button>

          <button
            className={`nav-item ${currentTab === 'settings' ? 'active' : ''}`}
            onClick={() => setCurrentTab('settings')}
            aria-label="Navigate to Settings"
            aria-current={currentTab === 'settings' ? 'page' : undefined}
          >
            <Settings size={20} aria-hidden="true" />
            <span>Settings</span>
          </button>
        </nav>

          <div className="sidebar-footer">
            <a
              href="https://github.com/saulbuilds/citrate"
              target="_blank"
              rel="noopener noreferrer"
              className="github-link"
            >
              <Github size={20} />
              <span>GitHub</span>
            </a>
          </div>
        </div>

        <main className="main-content" role="main" aria-label="Main content">
          {renderView()}
        </main>

        <style jsx>{`
          .app {
            display: flex;
            height: 100vh;
            background: #ffffff;
            font-family: 'Superclarendon', 'Clarendon', Georgia, serif;
          }

          .sidebar {
            width: 260px;
            background: #ffffff;
            border-right: 1px solid #e5e7eb;
            display: flex;
            flex-direction: column;
          }

          .sidebar-header {
            padding: 2rem 1.5rem;
            border-bottom: 1px solid #e5e7eb;
          }

          .app-logo {
            width: 100%;
            max-width: 200px;
            height: auto;
            margin-bottom: 0.5rem;
          }

          .app-version {
            margin: 0.5rem 0 0 0;
            color: #666666;
            font-size: 0.875rem;
            font-weight: 400;
          }

          .app-mode {
            margin: 0.5rem 0 0 0;
            padding: 0.25rem 0.5rem;
            background: #ffa50020;
            color: #ffa500;
            border-radius: 0.25rem;
            font-size: 0.75rem;
            font-weight: 600;
            display: inline-block;
          }

          .sidebar-nav {
            flex: 1;
            padding: 1.5rem 1rem;
          }

          .nav-item {
            display: flex;
            align-items: center;
            gap: 0.75rem;
            width: 100%;
            padding: 0.75rem 1rem;
            margin-bottom: 0.5rem;
            background: none;
            border: none;
            border-radius: 0.5rem;
            color: #000000;
            font-size: 0.9375rem;
            font-weight: 500;
            cursor: pointer;
            transition: all 0.2s;
          }

          .nav-item:hover {
            background: #f9f9f9;
            color: #000000;
          }

          .nav-item.active {
            background: #ffa500;
            color: #ffffff;
          }

          .sidebar-footer {
            padding: 1.5rem;
            border-top: 1px solid #e5e7eb;
          }

          .github-link {
            display: flex;
            align-items: center;
            gap: 0.75rem;
            color: #000000;
            text-decoration: none;
            transition: color 0.2s;
          }

          .github-link:hover {
            color: #ffa500;
          }

          .main-content {
            flex: 1;
            overflow-y: auto;
            background: #ffffff;
          }
        `}</style>

      <FirstTimeSetup onSetupComplete={() => {
        // Refresh the current view or trigger any necessary updates
        setCurrentTab('dashboard');
      }} />

      {/* Keyboard Shortcuts Help Modal */}
      <KeyboardShortcutsHelp
        shortcuts={shortcuts}
        isOpen={showShortcutsHelp}
        onClose={() => setShowShortcutsHelp(false)}
      />
    </div>
    </>
  );
}

/**
 * App Component (Outer)
 *
 * Wraps the app with AppProvider and ErrorBoundary
 */
function App() {
  return (
    <ErrorBoundary>
      <AppProvider>
        <ThemeProvider>
          <AppInner />
        </ThemeProvider>
      </AppProvider>
    </ErrorBoundary>
  );
}

export default App;
