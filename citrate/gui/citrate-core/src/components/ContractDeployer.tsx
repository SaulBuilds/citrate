/**
 * ContractDeployer Component
 *
 * UI for deploying smart contracts to Citrate.
 * Features:
 * - Contract source editor
 * - Compilation with error display
 * - Constructor parameter inputs
 * - Gas estimation
 * - Deployment transaction preview
 * - Deployment tracking
 */

import React, { useState, useEffect } from 'react';
import { ContractEditor } from './ContractEditor';
import {
  compileContract,
  CompilationResult,
  validateContractSize,
  formatBytecodeSize,
} from '../utils/contractCompiler';
import { deployContract } from '../utils/contractDeployment';
import { invoke } from '@tauri-apps/api/core';
import { Loader2, CheckCircle, AlertCircle, Rocket, Code } from 'lucide-react';

interface ConstructorParam {
  name: string;
  type: string;
  value: string;
}

interface Account {
  address: string;
  label: string;
  balance: string;
}

export const ContractDeployer: React.FC = () => {
  const [sourceCode, setSourceCode] = useState('');
  const [contractName, setContractName] = useState('MyContract');
  const [compiling, setCompiling] = useState(false);
  const [compilationResult, setCompilationResult] = useState<CompilationResult | null>(null);
  const [constructorParams, setConstructorParams] = useState<ConstructorParam[]>([]);
  const [deploying, setDeploying] = useState(false);
  const [deploymentTxHash, setDeploymentTxHash] = useState<string | null>(null);
  const [deploymentAddress, setDeploymentAddress] = useState<string | null>(null);
  const [accounts, setAccounts] = useState<Account[]>([]);
  const [selectedAccount, setSelectedAccount] = useState<string>('');
  const [password, setPassword] = useState('');
  const [showPasswordPrompt, setShowPasswordPrompt] = useState(false);

  // Load accounts on mount
  useEffect(() => {
    loadAccounts();
  }, []);

  const loadAccounts = async () => {
    try {
      const accountsList = await invoke<Account[]>('get_accounts');
      setAccounts(accountsList);
      if (accountsList.length > 0 && !selectedAccount) {
        setSelectedAccount(accountsList[0].address);
      }
    } catch (error) {
      console.error('[ContractDeployer] Failed to load accounts:', error);
    }
  };

  const handleCompile = async () => {
    if (!sourceCode.trim()) {
      alert('Please enter contract source code');
      return;
    }

    if (!contractName.trim()) {
      alert('Please enter contract name');
      return;
    }

    setCompiling(true);
    setCompilationResult(null);
    setConstructorParams([]);

    try {
      const result = await compileContract(sourceCode, contractName);
      setCompilationResult(result);

      // If compilation succeeded, extract constructor parameters
      if (result.success && result.abi) {
        const constructor = result.abi.find((item: any) => item.type === 'constructor');
        if (constructor && constructor.inputs) {
          const params = constructor.inputs.map((input: any) => ({
            name: input.name || 'param',
            type: input.type,
            value: '',
          }));
          setConstructorParams(params);
        }
      }
    } catch (error: any) {
      console.error('[ContractDeployer] Compilation error:', error);
      setCompilationResult({
        success: false,
        errors: [{
          severity: 'error',
          message: error.message || 'Unknown compilation error',
        }],
      });
    } finally {
      setCompiling(false);
    }
  };

  const handleParamChange = (index: number, value: string) => {
    const updated = [...constructorParams];
    updated[index].value = value;
    setConstructorParams(updated);
  };

  const handleDeploy = async () => {
    if (!compilationResult?.success || !compilationResult.bytecode) {
      alert('Please compile the contract first');
      return;
    }

    if (!selectedAccount) {
      alert('Please select an account to deploy from');
      return;
    }

    // Validate constructor parameters
    const hasEmptyParams = constructorParams.some(p => !p.value.trim());
    if (hasEmptyParams && constructorParams.length > 0) {
      const confirmed = window.confirm(
        'Some constructor parameters are empty. Continue anyway?'
      );
      if (!confirmed) return;
    }

    // Show password prompt
    setShowPasswordPrompt(true);
  };

  const handleConfirmDeployment = async () => {
    if (!password) {
      alert('Please enter your password');
      return;
    }

    setShowPasswordPrompt(false);
    setDeploying(true);

    try {
      console.log('[ContractDeployer] Deploying contract...');

      const result = await deployContract({
        from: selectedAccount,
        bytecode: compilationResult!.bytecode!,
        abi: compilationResult!.abi || [],
        constructorArgs: constructorParams.map(p => p.value),
        password: password,
      });

      setDeploymentTxHash(result.txHash);
      setDeploymentAddress(result.contractAddress || null);
      setPassword(''); // Clear password

      alert(`Contract deployed!\nTransaction: ${result.txHash.slice(0, 10)}...`);
    } catch (error: any) {
      console.error('[ContractDeployer] Deployment error:', error);
      alert(`Deployment failed: ${error.message}`);
    } finally {
      setDeploying(false);
    }
  };

  const handleCancelDeployment = () => {
    setShowPasswordPrompt(false);
    setPassword('');
  };

  const isReadyToDeploy = compilationResult?.success && compilationResult.bytecode;
  const sizeValidation = compilationResult?.bytecode
    ? validateContractSize(compilationResult.bytecode)
    : null;

  return (
    <div className="contract-deployer">
      <div className="deployer-header">
        <h2>Deploy Smart Contract</h2>
        <p>Write, compile, and deploy Solidity contracts to Citrate</p>
      </div>

      {/* Contract Name Input */}
      <div className="form-group">
        <label htmlFor="contract-name">Contract Name</label>
        <input
          id="contract-name"
          type="text"
          value={contractName}
          onChange={(e) => setContractName(e.target.value)}
          placeholder="MyContract"
          className="contract-name-input"
        />
      </div>

      {/* Account Selection */}
      <div className="form-group">
        <label htmlFor="account-select">Deploy From Account</label>
        <select
          id="account-select"
          value={selectedAccount}
          onChange={(e) => setSelectedAccount(e.target.value)}
          className="account-select"
        >
          {accounts.length === 0 ? (
            <option value="">No accounts available</option>
          ) : (
            accounts.map((account) => (
              <option key={account.address} value={account.address}>
                {account.label} ({account.address.slice(0, 10)}...)
              </option>
            ))
          )}
        </select>
      </div>

      {/* Source Code Editor */}
      <div className="editor-section">
        <h3>Contract Source Code</h3>
        <ContractEditor
          value={sourceCode}
          onChange={setSourceCode}
          height="400px"
          storageKey="deployer-source-code"
        />
      </div>

      {/* Compile Button */}
      <div className="action-section">
        <button
          onClick={handleCompile}
          disabled={compiling || !sourceCode.trim()}
          className="btn btn-primary"
        >
          {compiling ? (
            <>
              <Loader2 size={16} className="spinner" />
              <span>Compiling...</span>
            </>
          ) : (
            <>
              <Code size={16} />
              <span>Compile Contract</span>
            </>
          )}
        </button>
      </div>

      {/* Compilation Results */}
      {compilationResult && (
        <div className="compilation-results">
          {compilationResult.success ? (
            <div className="result-success">
              <div className="result-header">
                <CheckCircle size={20} className="success-icon" />
                <h3>Compilation Successful</h3>
              </div>

              <div className="result-details">
                <div className="detail-item">
                  <span className="label">Bytecode Size:</span>
                  <span className="value">
                    {formatBytecodeSize(compilationResult.contractSize || 0)}
                    {sizeValidation && !sizeValidation.isValid && (
                      <span className="error-text"> (Exceeds 24KB limit!)</span>
                    )}
                  </span>
                </div>

                <div className="detail-item">
                  <span className="label">Estimated Gas:</span>
                  <span className="value">
                    {compilationResult.gasEstimate?.toLocaleString()} gas
                  </span>
                </div>

                {compilationResult.warnings && compilationResult.warnings.length > 0 && (
                  <div className="warnings">
                    <h4>Warnings:</h4>
                    {compilationResult.warnings.map((warning, i) => (
                      <div key={i} className="warning-item">{warning}</div>
                    ))}
                  </div>
                )}
              </div>
            </div>
          ) : (
            <div className="result-error">
              <div className="result-header">
                <AlertCircle size={20} className="error-icon" />
                <h3>Compilation Failed</h3>
              </div>

              <div className="errors">
                {compilationResult.errors?.map((error, i) => (
                  <div key={i} className="error-item">
                    <span className="error-severity">{error.severity}:</span>
                    <span className="error-message">{error.message}</span>
                    {error.line && <span className="error-line">Line {error.line}</span>}
                  </div>
                ))}
              </div>
            </div>
          )}
        </div>
      )}

      {/* Constructor Parameters */}
      {isReadyToDeploy && constructorParams.length > 0 && (
        <div className="constructor-params">
          <h3>Constructor Parameters</h3>
          <p className="help-text">
            Enter the values for the constructor parameters
          </p>

          {constructorParams.map((param, index) => (
            <div key={index} className="param-input-group">
              <label htmlFor={`param-${index}`}>
                {param.name}
                <span className="param-type">({param.type})</span>
              </label>
              <input
                id={`param-${index}`}
                type="text"
                value={param.value}
                onChange={(e) => handleParamChange(index, e.target.value)}
                placeholder={`Enter ${param.type}`}
                className="param-input"
              />
            </div>
          ))}
        </div>
      )}

      {/* Deploy Button */}
      {isReadyToDeploy && (
        <div className="deploy-section">
          <button
            onClick={handleDeploy}
            disabled={deploying || (sizeValidation !== null && !sizeValidation.isValid)}
            className="btn btn-success btn-large"
          >
            {deploying ? (
              <>
                <Loader2 size={20} className="spinner" />
                <span>Deploying Contract...</span>
              </>
            ) : (
              <>
                <Rocket size={20} />
                <span>Deploy to Citrate</span>
              </>
            )}
          </button>

          {sizeValidation && !sizeValidation.isValid && (
            <p className="error-text">
              Contract exceeds 24KB size limit and cannot be deployed
            </p>
          )}
        </div>
      )}

      {/* Password Prompt Modal */}
      {showPasswordPrompt && (
        <div className="modal-overlay">
          <div className="modal-content">
            <h3>Confirm Deployment</h3>
            <p>Enter your password to sign the deployment transaction</p>
            <div className="form-group">
              <label htmlFor="password-input">Password</label>
              <input
                id="password-input"
                type="password"
                value={password}
                onChange={(e) => setPassword(e.target.value)}
                placeholder="Enter your password"
                className="password-input"
                autoFocus
                onKeyPress={(e) => {
                  if (e.key === 'Enter') handleConfirmDeployment();
                }}
              />
            </div>
            <div className="modal-actions">
              <button
                onClick={handleCancelDeployment}
                className="btn btn-secondary"
              >
                Cancel
              </button>
              <button
                onClick={handleConfirmDeployment}
                className="btn btn-primary"
                disabled={!password}
              >
                Deploy Contract
              </button>
            </div>
          </div>
        </div>
      )}

      {/* Deployment Result */}
      {deploymentTxHash && (
        <div className="deployment-result">
          <div className="result-header">
            <CheckCircle size={20} className="success-icon" />
            <h3>Contract Deployed!</h3>
          </div>
          <div className="tx-hash">
            <span className="label">Transaction Hash:</span>
            <code className="hash">{deploymentTxHash}</code>
          </div>
          {deploymentAddress && (
            <div className="tx-hash">
              <span className="label">Contract Address:</span>
              <code className="hash">{deploymentAddress}</code>
            </div>
          )}
          {!deploymentAddress && (
            <p className="help-text">
              Waiting for transaction confirmation to retrieve contract address...
            </p>
          )}
        </div>
      )}

      <style jsx>{`
        .contract-deployer {
          max-width: 1200px;
        }

        .deployer-header {
          margin-bottom: 2rem;
        }

        .deployer-header h2 {
          margin: 0 0 0.5rem 0;
          font-size: 1.5rem;
          font-weight: 600;
          color: var(--text-primary);
        }

        .deployer-header p {
          margin: 0;
          color: var(--text-secondary);
        }

        .form-group {
          margin-bottom: 1.5rem;
        }

        .form-group label {
          display: block;
          margin-bottom: 0.5rem;
          font-weight: 500;
          color: var(--text-primary);
        }

        .contract-name-input {
          width: 100%;
          max-width: 400px;
          padding: 0.75rem;
          border: 1px solid var(--border-primary);
          border-radius: 0.375rem;
          background: var(--bg-primary);
          color: var(--text-primary);
          font-size: 1rem;
        }

        .editor-section {
          margin-bottom: 1.5rem;
        }

        .editor-section h3 {
          margin: 0 0 1rem 0;
          font-size: 1.1rem;
          font-weight: 600;
          color: var(--text-primary);
        }

        .action-section {
          margin: 1.5rem 0;
        }

        .btn {
          display: inline-flex;
          align-items: center;
          gap: 0.5rem;
          padding: 0.75rem 1.5rem;
          border: none;
          border-radius: 0.5rem;
          font-size: 1rem;
          font-weight: 500;
          cursor: pointer;
          transition: all 200ms ease;
        }

        .btn:disabled {
          opacity: 0.5;
          cursor: not-allowed;
        }

        .btn-primary {
          background: var(--brand-primary);
          color: white;
        }

        .btn-primary:hover:not(:disabled) {
          background: var(--brand-hover);
          transform: translateY(-1px);
          box-shadow: 0 4px 12px rgba(255, 165, 0, 0.3);
        }

        .btn-success {
          background: linear-gradient(135deg, #10b981 0%, #059669 100%);
          color: white;
        }

        .btn-success:hover:not(:disabled) {
          transform: translateY(-2px);
          box-shadow: 0 6px 16px rgba(16, 185, 129, 0.4);
        }

        .btn-large {
          padding: 1rem 2rem;
          font-size: 1.1rem;
        }

        .spinner {
          animation: spin 1s linear infinite;
        }

        @keyframes spin {
          from { transform: rotate(0deg); }
          to { transform: rotate(360deg); }
        }

        .compilation-results {
          margin: 1.5rem 0;
          padding: 1.5rem;
          border-radius: 0.5rem;
          border: 1px solid var(--border-primary);
        }

        .result-success {
          background: rgba(16, 185, 129, 0.05);
          border-color: #10b981;
        }

        .result-error {
          background: rgba(239, 68, 68, 0.05);
          border-color: #ef4444;
        }

        .result-header {
          display: flex;
          align-items: center;
          gap: 0.75rem;
          margin-bottom: 1rem;
        }

        .result-header h3 {
          margin: 0;
          font-size: 1.1rem;
          font-weight: 600;
        }

        .success-icon {
          color: #10b981;
        }

        .error-icon {
          color: #ef4444;
        }

        .result-details {
          display: flex;
          flex-direction: column;
          gap: 0.75rem;
        }

        .detail-item {
          display: flex;
          gap: 0.5rem;
        }

        .label {
          font-weight: 500;
          color: var(--text-secondary);
        }

        .value {
          color: var(--text-primary);
        }

        .errors, .warnings {
          margin-top: 1rem;
        }

        .errors h4, .warnings h4 {
          margin: 0 0 0.5rem 0;
          font-size: 0.9rem;
          font-weight: 600;
          color: var(--text-secondary);
        }

        .error-item, .warning-item {
          padding: 0.5rem;
          margin-bottom: 0.5rem;
          background: var(--bg-secondary);
          border-radius: 0.25rem;
          font-family: monospace;
          font-size: 0.875rem;
        }

        .error-severity {
          font-weight: 600;
          margin-right: 0.5rem;
          color: #ef4444;
        }

        .error-text {
          color: #ef4444;
          font-size: 0.875rem;
          margin-top: 0.5rem;
        }

        .constructor-params {
          margin: 2rem 0;
          padding: 1.5rem;
          background: var(--bg-secondary);
          border-radius: 0.5rem;
          border: 1px solid var(--border-primary);
        }

        .constructor-params h3 {
          margin: 0 0 0.5rem 0;
          font-size: 1.1rem;
          font-weight: 600;
          color: var(--text-primary);
        }

        .help-text {
          margin: 0 0 1rem 0;
          font-size: 0.875rem;
          color: var(--text-secondary);
        }

        .param-input-group {
          margin-bottom: 1rem;
        }

        .param-input-group label {
          display: block;
          margin-bottom: 0.375rem;
          font-weight: 500;
          color: var(--text-primary);
        }

        .param-type {
          color: var(--text-muted);
          font-weight: 400;
          margin-left: 0.5rem;
        }

        .param-input {
          width: 100%;
          padding: 0.75rem;
          border: 1px solid var(--border-primary);
          border-radius: 0.375rem;
          background: var(--bg-primary);
          color: var(--text-primary);
          font-size: 1rem;
        }

        .deploy-section {
          margin: 2rem 0;
          text-align: center;
        }

        .deployment-result {
          margin: 2rem 0;
          padding: 1.5rem;
          background: rgba(16, 185, 129, 0.05);
          border: 1px solid #10b981;
          border-radius: 0.5rem;
        }

        .tx-hash {
          margin: 1rem 0;
          padding: 1rem;
          background: var(--bg-primary);
          border-radius: 0.375rem;
        }

        .hash {
          display: block;
          margin-top: 0.5rem;
          font-family: monospace;
          font-size: 0.875rem;
          color: var(--brand-primary);
          word-break: break-all;
        }

        .account-select {
          width: 100%;
          max-width: 400px;
          padding: 0.75rem;
          border: 1px solid var(--border-primary);
          border-radius: 0.375rem;
          background: var(--bg-primary);
          color: var(--text-primary);
          font-size: 1rem;
          cursor: pointer;
        }

        .modal-overlay {
          position: fixed;
          top: 0;
          left: 0;
          right: 0;
          bottom: 0;
          background: rgba(0, 0, 0, 0.6);
          display: flex;
          align-items: center;
          justify-content: center;
          z-index: 1000;
        }

        .modal-content {
          background: var(--bg-primary);
          border: 1px solid var(--border-primary);
          border-radius: 0.5rem;
          padding: 2rem;
          max-width: 500px;
          width: 90%;
          box-shadow: 0 10px 40px rgba(0, 0, 0, 0.3);
        }

        .modal-content h3 {
          margin: 0 0 1rem 0;
          font-size: 1.25rem;
          font-weight: 600;
          color: var(--text-primary);
        }

        .modal-content > p {
          margin: 0 0 1.5rem 0;
          color: var(--text-secondary);
        }

        .password-input {
          width: 100%;
          padding: 0.75rem;
          border: 1px solid var(--border-primary);
          border-radius: 0.375rem;
          background: var(--bg-primary);
          color: var(--text-primary);
          font-size: 1rem;
        }

        .modal-actions {
          display: flex;
          gap: 1rem;
          margin-top: 1.5rem;
          justify-content: flex-end;
        }

        .btn-secondary {
          background: var(--bg-secondary);
          color: var(--text-primary);
          border: 1px solid var(--border-primary);
        }

        .btn-secondary:hover:not(:disabled) {
          background: var(--bg-tertiary);
        }
      `}</style>
    </div>
  );
};

export default ContractDeployer;
