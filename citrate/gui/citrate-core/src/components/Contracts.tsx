/**
 * Contracts Component
 *
 * Smart contract development and interaction interface.
 * Provides tools for deploying, interacting with, and managing smart contracts.
 */

import React, { useState } from 'react';
import { Upload, Play, BookOpen } from 'lucide-react';
import { ContractDeployer } from './ContractDeployer';
import { ContractInteraction } from './ContractInteraction';

type ContractsTab = 'deploy' | 'interact' | 'my-contracts';

export const Contracts: React.FC = () => {
  const [activeTab, setActiveTab] = useState<ContractsTab>('deploy');

  return (
    <div className="contracts">
      <header className="contracts-header">
        <h1>Smart Contracts</h1>
        <p className="subtitle">Deploy and interact with smart contracts on Citrate</p>
      </header>

      <nav className="contracts-tabs" role="tablist" aria-label="Contract tools">
        <button
          role="tab"
          aria-selected={activeTab === 'deploy'}
          aria-controls="deploy-panel"
          className={`tab-button ${activeTab === 'deploy' ? 'active' : ''}`}
          onClick={() => setActiveTab('deploy')}
        >
          <Upload size={18} aria-hidden="true" />
          <span>Deploy Contract</span>
        </button>

        <button
          role="tab"
          aria-selected={activeTab === 'interact'}
          aria-controls="interact-panel"
          className={`tab-button ${activeTab === 'interact' ? 'active' : ''}`}
          onClick={() => setActiveTab('interact')}
        >
          <Play size={18} aria-hidden="true" />
          <span>Interact</span>
        </button>

        <button
          role="tab"
          aria-selected={activeTab === 'my-contracts'}
          aria-controls="my-contracts-panel"
          className={`tab-button ${activeTab === 'my-contracts' ? 'active' : ''}`}
          onClick={() => setActiveTab('my-contracts')}
        >
          <BookOpen size={18} aria-hidden="true" />
          <span>My Contracts</span>
        </button>
      </nav>

      <div className="contracts-content">
        {activeTab === 'deploy' && (
          <div id="deploy-panel" role="tabpanel" aria-labelledby="deploy-tab">
            <ContractDeployer />
          </div>
        )}

        {activeTab === 'interact' && (
          <div id="interact-panel" role="tabpanel" aria-labelledby="interact-tab">
            <ContractInteraction />
          </div>
        )}

        {activeTab === 'my-contracts' && (
          <div id="my-contracts-panel" role="tabpanel" aria-labelledby="my-contracts-tab">
            <MyContractsPlaceholder />
          </div>
        )}
      </div>

      <style jsx>{`
        .contracts {
          padding: 2rem;
          max-width: 1400px;
          margin: 0 auto;
        }

        .contracts-header {
          margin-bottom: 2rem;
        }

        .contracts-header h1 {
          margin: 0 0 0.5rem 0;
          font-size: 2rem;
          font-weight: 600;
          color: var(--text-primary);
        }

        .subtitle {
          margin: 0;
          color: var(--text-secondary);
          font-size: 1rem;
        }

        .contracts-tabs {
          display: flex;
          gap: 0.5rem;
          border-bottom: 2px solid var(--border-primary);
          margin-bottom: 2rem;
        }

        .tab-button {
          display: flex;
          align-items: center;
          gap: 0.5rem;
          padding: 0.75rem 1.5rem;
          background: transparent;
          border: none;
          border-bottom: 3px solid transparent;
          color: var(--text-secondary);
          font-size: 0.95rem;
          font-weight: 500;
          cursor: pointer;
          transition: all 200ms ease;
          position: relative;
          top: 2px;
        }

        .tab-button:hover {
          color: var(--text-primary);
          background: var(--bg-secondary);
        }

        .tab-button.active {
          color: var(--brand-primary);
          border-bottom-color: var(--brand-primary);
          font-weight: 600;
        }

        .tab-button:focus-visible {
          outline: 3px solid var(--brand-primary);
          outline-offset: 2px;
          border-radius: 4px;
        }

        .contracts-content {
          min-height: 500px;
        }
      `}</style>
    </div>
  );
};

/**
 * Placeholder components for each tab
 * These will be replaced with actual implementations in later tasks
 */

const MyContractsPlaceholder: React.FC = () => {
  return (
    <div className="placeholder">
      <div className="placeholder-icon">
        <BookOpen size={48} />
      </div>
      <h2>My Contracts</h2>
      <p>Manage your deployed contracts and saved ABIs.</p>
      <div className="placeholder-features">
        <div className="feature">✓ List of deployed contracts</div>
        <div className="feature">✓ Contract metadata and labels</div>
        <div className="feature">✓ ABI storage and export</div>
        <div className="feature">✓ Quick access to interact mode</div>
      </div>
      <p className="coming-soon">Implementation coming in Day 2</p>

      <style jsx>{`
        .placeholder {
          text-align: center;
          padding: 4rem 2rem;
          color: var(--text-secondary);
        }

        .placeholder-icon {
          color: var(--brand-primary);
          margin-bottom: 1.5rem;
          opacity: 0.6;
        }

        .placeholder h2 {
          margin: 0 0 1rem 0;
          color: var(--text-primary);
          font-size: 1.5rem;
        }

        .placeholder > p {
          margin: 0 0 2rem 0;
          font-size: 1rem;
        }

        .placeholder-features {
          max-width: 500px;
          margin: 0 auto 2rem;
          text-align: left;
        }

        .feature {
          padding: 0.5rem 0;
          color: var(--text-secondary);
        }

        .coming-soon {
          color: var(--text-muted);
          font-size: 0.875rem;
          font-style: italic;
        }
      `}</style>
    </div>
  );
};

export default Contracts;
