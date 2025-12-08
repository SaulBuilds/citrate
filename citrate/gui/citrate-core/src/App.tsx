import { useEffect, useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import './App.css';
import { ChatDashboard, MinimalSidebar } from './components/layout';
import { Wallet } from './components/Wallet';
import { DAGVisualization } from './components/DAGVisualization';
import { Models } from './components/Models';
import { Marketplace } from './components/Marketplace';
import { IPFS } from './components/IPFS';
import { Contracts } from './components/Contracts';
import { Settings as SettingsView } from './components/Settings';
import { FirstTimeSetup } from './components/FirstTimeSetup';
import { OnboardingModal } from './components/OnboardingModal';
import ErrorBoundary from './components/ErrorBoundary';
import { AppProvider, useAppTab } from './contexts/AppContext';
import { ThemeProvider } from './contexts/ThemeContext';
import { ErrorProvider, useError } from './contexts/ErrorContext';
import { ErrorNotification } from './components/common/ErrorNotification';
import { DevModeIndicator } from './components/common/DevModeIndicator';
import { useKeyboardShortcuts, KeyboardShortcut } from './hooks/useKeyboardShortcuts';
import KeyboardShortcutsHelp from './components/KeyboardShortcutsHelp';

/**
 * Main App Component (Inner)
 *
 * Uses AppContext for state persistence
 */
function AppInner() {
  const { currentTab, setCurrentTab } = useAppTab();
  const { addError } = useError();
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
      description: 'Navigate to IPFS Storage',
      action: () => setCurrentTab('ipfs'),
    },
    {
      key: '7',
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
          addError({
            code: 'NETWORK_OFFLINE',
            message: 'Network connectivity issue',
            details: 'Could not establish connection to the network. Some features may be unavailable.',
            severity: 'warning',
            category: 'network',
          });
        }
      }, 3000);

    } catch (error) {
      console.error('App initialization failed:', error);
      const errorMessage = error instanceof Error ? error.message : String(error);

      // Determine error type and add appropriate error
      if (errorMessage.toLowerCase().includes('network') || errorMessage.toLowerCase().includes('connect')) {
        addError({
          code: 'RPC_UNAVAILABLE',
          message: 'Failed to start node',
          details: errorMessage,
          severity: 'error',
          category: 'network',
        });
      } else {
        addError({
          code: 'UNKNOWN_ERROR',
          message: 'App initialization failed',
          details: errorMessage,
          severity: 'error',
          category: 'unknown',
        });
      }
    }
  };

  const renderView = () => {
    switch (currentTab) {
      case 'dashboard':
        return <ChatDashboard />;
      case 'wallet':
        return <Wallet />;
      case 'dag':
        return <DAGVisualization />;
      case 'models':
        return <Models />;
      case 'marketplace':
        return <Marketplace />;
      case 'ipfs':
        return <IPFS />;
      case 'contracts':
        return <Contracts />;
      case 'settings':
        return <SettingsView />;
      default:
        return <ChatDashboard />;
    }
  };

  return (
    <>
      <div className="app">
        {/* New Minimal Sidebar - AI Chat removed, dashboard is now chat-first */}
        <MinimalSidebar
          currentTab={currentTab as any}
          onTabChange={(tab) => setCurrentTab(tab)}
        />

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

      {/* AI Onboarding Modal - shown after first-time wallet setup */}
      <OnboardingModal onComplete={() => {
        // User completed onboarding, ensure we're on dashboard
        setCurrentTab('dashboard');
      }} />

      {/* Keyboard Shortcuts Help Modal */}
      <KeyboardShortcutsHelp
        shortcuts={shortcuts}
        isOpen={showShortcutsHelp}
        onClose={() => setShowShortcutsHelp(false)}
      />

      {/* Error Notifications */}
      <ErrorNotification maxToasts={5} showCriticalModal={true} />

      {/* Dev Mode Indicator (only shown in development builds) */}
      <DevModeIndicator position="bottom-left" />
    </div>
    </>
  );
}

/**
 * App Component (Outer)
 *
 * Wraps the app with providers and ErrorBoundary
 */
function App() {
  return (
    <ErrorBoundary>
      <AppProvider>
        <ThemeProvider>
          <ErrorProvider>
            <AppInner />
          </ErrorProvider>
        </ThemeProvider>
      </AppProvider>
    </ErrorBoundary>
  );
}

export default App;
