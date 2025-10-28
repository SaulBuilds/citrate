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
import { Settings as SettingsView } from './components/Settings';
import { FirstTimeSetup } from './components/FirstTimeSetup';
import ErrorBoundary from './components/ErrorBoundary';
import {
  LayoutDashboard,
  Wallet as WalletIcon,
  Network,
  Brain,
  ShoppingBag,
  MessageSquare,
  Database,
  Settings,
  Github
} from 'lucide-react';

type View = 'dashboard' | 'wallet' | 'dag' | 'models' | 'marketplace' | 'chat' | 'ipfs' | 'settings';

function App() {
  const [currentView, setCurrentView] = useState<View>('dashboard');
  const isNativeApp = typeof window !== 'undefined' && window.__TAURI__ !== undefined;

  useEffect(() => {
    const handler = (e: any) => {
      const hash = e?.detail?.hash;
      if (hash) {
        try { localStorage.setItem('dag_focus_hash', hash); } catch {}
      }
      setCurrentView('dag');
    };
    window.addEventListener('open-dag-for-hash' as any, handler);

    // Auto-initialize on app start
    if (isNativeApp) {
      initializeApp();
    }

    return () => window.removeEventListener('open-dag-for-hash' as any, handler);
  }, [isNativeApp]);

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
    switch (currentView) {
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
      case 'settings':
        return <SettingsView />;
      default:
        return <Dashboard />;
    }
  };

  return (
    <ErrorBoundary>
      <div className="app">
        <div className="sidebar">
          <div className="sidebar-header">
            <img src={citrateLogo} alt="Citrate" className="app-logo" />
            <p className="app-version">v3.0.0</p>
            <p className="app-mode">{isNativeApp ? '🖥️ Native' : '🌐 Web Mode'}</p>
          </div>

          <nav className="sidebar-nav">
            <button
              className={`nav-item ${currentView === 'dashboard' ? 'active' : ''}`}
              onClick={() => setCurrentView('dashboard')}
            >
              <LayoutDashboard size={20} />
              <span>Dashboard</span>
            </button>

            <button
              className={`nav-item ${currentView === 'wallet' ? 'active' : ''}`}
              onClick={() => setCurrentView('wallet')}
            >
              <WalletIcon size={20} />
              <span>Wallet</span>
            </button>

            <button
              className={`nav-item ${currentView === 'dag' ? 'active' : ''}`}
              onClick={() => setCurrentView('dag')}
            >
              <Network size={20} />
              <span>DAG Explorer</span>
            </button>

            <button
              className={`nav-item ${currentView === 'models' ? 'active' : ''}`}
              onClick={() => setCurrentView('models')}
            >
              <Brain size={20} />
              <span>AI Models</span>
            </button>

            <button
              className={`nav-item ${currentView === 'marketplace' ? 'active' : ''}`}
              onClick={() => setCurrentView('marketplace')}
            >
              <ShoppingBag size={20} />
              <span>Marketplace</span>
            </button>

            <button
              className={`nav-item ${currentView === 'chat' ? 'active' : ''}`}
              onClick={() => setCurrentView('chat')}
            >
              <MessageSquare size={20} />
              <span>AI Chat</span>
            </button>

            <button
              className={`nav-item ${currentView === 'ipfs' ? 'active' : ''}`}
              onClick={() => setCurrentView('ipfs')}
            >
              <Database size={20} />
              <span>IPFS Storage</span>
            </button>

            <button
              className={`nav-item ${currentView === 'settings' ? 'active' : ''}`}
              onClick={() => setCurrentView('settings')}
            >
              <Settings size={20} />
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

        <main className="main-content">
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
          setCurrentView('dashboard');
        }} />
      </div>
    </ErrorBoundary>
  );
}

// Settings moved to dedicated component

export default App;
