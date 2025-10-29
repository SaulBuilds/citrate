/**
 * ContractInteraction Component
 *
 * UI for interacting with deployed smart contracts.
 * Features:
 * - Load contract by address
 * - Import/paste ABI
 * - Call read functions (view/pure)
 * - Call write functions (with transaction)
 * - Display results and events
 */

import React, { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { Play, FileUp, Loader2, CheckCircle, AlertCircle, Eye, Edit } from 'lucide-react';
import {
  callContractFunction,
  sendContractTransaction,
  formatFunctionOutput,
  parseUserInput,
} from '../utils/contractInteraction';

interface Account {
  address: string;
  label: string;
  balance: string;
}

interface AbiFunction {
  name: string;
  type: string;
  stateMutability?: string;
  inputs: Array<{
    name: string;
    type: string;
    internalType?: string;
  }>;
  outputs: Array<{
    name: string;
    type: string;
    internalType?: string;
  }>;
}

export const ContractInteraction: React.FC = () => {
  const [contractAddress, setContractAddress] = useState('');
  const [abiInput, setAbiInput] = useState('');
  const [loadedContract, setLoadedContract] = useState<string | null>(null);
  const [readFunctions, setReadFunctions] = useState<AbiFunction[]>([]);
  const [writeFunctions, setWriteFunctions] = useState<AbiFunction[]>([]);
  const [selectedFunction, setSelectedFunction] = useState<AbiFunction | null>(null);
  const [functionInputs, setFunctionInputs] = useState<Record<string, string>>({});
  const [executing, setExecuting] = useState(false);
  const [result, setResult] = useState<any>(null);
  const [error, setError] = useState<string | null>(null);
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
      console.error('[ContractInteraction] Failed to load accounts:', error);
    }
  };

  const handleLoadContract = () => {
    if (!contractAddress.trim()) {
      setError('Please enter a contract address');
      return;
    }

    if (!abiInput.trim()) {
      setError('Please enter the contract ABI');
      return;
    }

    try {
      // Parse ABI
      const parsedAbi = JSON.parse(abiInput);

      if (!Array.isArray(parsedAbi)) {
        setError('ABI must be a JSON array');
        return;
      }

      // Filter functions only
      const functionAbi = parsedAbi.filter(
        (item: any) => item.type === 'function'
      ) as AbiFunction[];

      // Separate read and write functions
      const readOnly = functionAbi.filter(
        (fn) =>
          fn.stateMutability === 'view' ||
          fn.stateMutability === 'pure'
      );

      const writeOnly = functionAbi.filter(
        (fn) =>
          fn.stateMutability !== 'view' &&
          fn.stateMutability !== 'pure'
      );

      setReadFunctions(readOnly);
      setWriteFunctions(writeOnly);
      setLoadedContract(contractAddress);
      setError(null);
      setResult(null);

      console.log('[ContractInteraction] Contract loaded:', {
        address: contractAddress,
        functions: functionAbi.length,
        readFunctions: readOnly.length,
        writeFunctions: writeOnly.length,
      });
    } catch (err: any) {
      setError(`Failed to parse ABI: ${err.message}`);
      console.error('[ContractInteraction] ABI parse error:', err);
    }
  };

  const handleSelectFunction = (fn: AbiFunction) => {
    setSelectedFunction(fn);
    setFunctionInputs({});
    setResult(null);
    setError(null);

    // Initialize input values
    const inputs: Record<string, string> = {};
    fn.inputs.forEach((input, index) => {
      inputs[input.name || `param${index}`] = '';
    });
    setFunctionInputs(inputs);
  };

  const handleInputChange = (name: string, value: string) => {
    setFunctionInputs((prev) => ({ ...prev, [name]: value }));
  };

  const handleExecuteFunction = () => {
    if (!selectedFunction) return;

    const isReadOnly =
      selectedFunction.stateMutability === 'view' ||
      selectedFunction.stateMutability === 'pure';

    if (isReadOnly) {
      // Execute read function directly
      executeReadFunction();
    } else {
      // Show password prompt for write function
      if (!selectedAccount) {
        setError('Please select an account');
        return;
      }
      setShowPasswordPrompt(true);
    }
  };

  const executeReadFunction = async () => {
    if (!selectedFunction || !loadedContract) return;

    setExecuting(true);
    setError(null);
    setResult(null);

    try {
      console.log('[ContractInteraction] Executing read function:', selectedFunction.name);

      // Parse user inputs
      const parsedArgs = selectedFunction.inputs.map((input, index) => {
        const inputName = input.name || `param${index}`;
        const rawValue = functionInputs[inputName];
        return parseUserInput(input.type, rawValue);
      });

      // Call the contract function
      const callResult = await callContractFunction({
        contractAddress: loadedContract,
        functionName: selectedFunction.name,
        inputs: selectedFunction.inputs,
        outputs: selectedFunction.outputs,
        args: parsedArgs,
      });

      // Format the result
      const formattedOutput = formatFunctionOutput(
        selectedFunction.outputs,
        callResult.outputs
      );

      setResult({
        success: true,
        outputs: callResult.outputs,
        decodedOutputs: callResult.decodedOutputs,
        formattedOutput,
      });

      // Note: eth_call is not yet implemented in backend
      setError('⚠️  Note: eth_call not yet implemented. Using mock result.');
    } catch (err: any) {
      console.error('[ContractInteraction] Read function error:', err);
      setError(`Failed to execute function: ${err.message}`);
    } finally {
      setExecuting(false);
    }
  };

  const handleConfirmWriteFunction = async () => {
    if (!password) {
      setError('Please enter your password');
      return;
    }

    setShowPasswordPrompt(false);
    setExecuting(true);
    setError(null);
    setResult(null);

    try {
      if (!selectedFunction || !loadedContract) return;

      console.log('[ContractInteraction] Executing write function:', selectedFunction.name);

      // Parse user inputs
      const parsedArgs = selectedFunction.inputs.map((input, index) => {
        const inputName = input.name || `param${index}`;
        const rawValue = functionInputs[inputName];
        return parseUserInput(input.type, rawValue);
      });

      // Send transaction
      const txResult = await sendContractTransaction({
        from: selectedAccount,
        contractAddress: loadedContract,
        functionName: selectedFunction.name,
        inputs: selectedFunction.inputs,
        args: parsedArgs,
        password: password,
      });

      setResult({
        success: true,
        txHash: txResult.txHash,
      });

      setPassword('');
      console.log('[ContractInteraction] Transaction sent:', txResult.txHash);
    } catch (err: any) {
      console.error('[ContractInteraction] Write function error:', err);
      setError(`Failed to execute function: ${err.message}`);
      setPassword('');
    } finally {
      setExecuting(false);
    }
  };

  const handleCancelWriteFunction = () => {
    setShowPasswordPrompt(false);
    setPassword('');
  };

  const handleImportAbi = () => {
    const input = document.createElement('input');
    input.type = 'file';
    input.accept = '.json';
    input.onchange = (e) => {
      const file = (e.target as HTMLInputElement).files?.[0];
      if (!file) return;

      const reader = new FileReader();
      reader.onload = (event) => {
        const content = event.target?.result as string;
        setAbiInput(content);
      };
      reader.readAsText(file);
    };
    input.click();
  };

  return (
    <div className="contract-interaction">
      <div className="interaction-header">
        <h2>Interact with Smart Contract</h2>
        <p>Load a deployed contract and call its functions</p>
      </div>

      {/* Contract Loading Section */}
      {!loadedContract && (
        <div className="load-contract-section">
          <div className="form-group">
            <label htmlFor="contract-address">Contract Address</label>
            <input
              id="contract-address"
              type="text"
              value={contractAddress}
              onChange={(e) => setContractAddress(e.target.value)}
              placeholder="0x..."
              className="address-input"
            />
          </div>

          <div className="form-group">
            <label htmlFor="abi-input">Contract ABI</label>
            <div className="abi-input-actions">
              <button onClick={handleImportAbi} className="btn-import">
                <FileUp size={16} />
                <span>Import from file</span>
              </button>
            </div>
            <textarea
              id="abi-input"
              value={abiInput}
              onChange={(e) => setAbiInput(e.target.value)}
              placeholder='[{"type":"function","name":"balanceOf",...}]'
              className="abi-textarea"
              rows={10}
            />
          </div>

          {error && (
            <div className="error-message">
              <AlertCircle size={16} />
              <span>{error}</span>
            </div>
          )}

          <div className="action-section">
            <button
              onClick={handleLoadContract}
              disabled={!contractAddress.trim() || !abiInput.trim()}
              className="btn btn-primary btn-large"
            >
              <Play size={20} />
              <span>Load Contract</span>
            </button>
          </div>
        </div>
      )}

      {/* Loaded Contract Interface */}
      {loadedContract && (
        <div className="loaded-contract">
          <div className="contract-info">
            <div className="info-item">
              <span className="label">Contract Address:</span>
              <code className="value">{loadedContract}</code>
            </div>
            <div className="info-item">
              <span className="label">Functions:</span>
              <span className="value">
                {readFunctions.length} read, {writeFunctions.length} write
              </span>
            </div>
            <button
              onClick={() => {
                setLoadedContract(null);
                setSelectedFunction(null);
                setResult(null);
                setError(null);
              }}
              className="btn-reset"
            >
              Load Different Contract
            </button>
          </div>

          {/* Account Selection for Write Functions */}
          {writeFunctions.length > 0 && (
            <div className="form-group">
              <label htmlFor="account-select">From Account</label>
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
          )}

          {/* Function Lists */}
          <div className="functions-section">
            {/* Read Functions */}
            {readFunctions.length > 0 && (
              <div className="function-category">
                <h3>
                  <Eye size={18} />
                  Read Functions
                </h3>
                <div className="function-list">
                  {readFunctions.map((fn, index) => (
                    <button
                      key={`read-${index}`}
                      onClick={() => handleSelectFunction(fn)}
                      className={`function-button ${
                        selectedFunction === fn ? 'active' : ''
                      }`}
                    >
                      <span className="function-name">{fn.name}</span>
                      {fn.inputs.length > 0 && (
                        <span className="function-params">
                          ({fn.inputs.map((i) => i.type).join(', ')})
                        </span>
                      )}
                    </button>
                  ))}
                </div>
              </div>
            )}

            {/* Write Functions */}
            {writeFunctions.length > 0 && (
              <div className="function-category">
                <h3>
                  <Edit size={18} />
                  Write Functions
                </h3>
                <div className="function-list">
                  {writeFunctions.map((fn, index) => (
                    <button
                      key={`write-${index}`}
                      onClick={() => handleSelectFunction(fn)}
                      className={`function-button write ${
                        selectedFunction === fn ? 'active' : ''
                      }`}
                    >
                      <span className="function-name">{fn.name}</span>
                      {fn.inputs.length > 0 && (
                        <span className="function-params">
                          ({fn.inputs.map((i) => i.type).join(', ')})
                        </span>
                      )}
                    </button>
                  ))}
                </div>
              </div>
            )}
          </div>

          {/* Function Execution Panel */}
          {selectedFunction && (
            <div className="execution-panel">
              <h3>
                Execute: {selectedFunction.name}
                {selectedFunction.stateMutability && (
                  <span className="mutability-badge">
                    {selectedFunction.stateMutability}
                  </span>
                )}
              </h3>

              {/* Function Inputs */}
              {selectedFunction.inputs.length > 0 && (
                <div className="function-inputs">
                  <h4>Parameters</h4>
                  {selectedFunction.inputs.map((input, index) => (
                    <div key={index} className="input-group">
                      <label>
                        {input.name || `param${index}`}
                        <span className="input-type">({input.type})</span>
                      </label>
                      <input
                        type="text"
                        value={functionInputs[input.name || `param${index}`] || ''}
                        onChange={(e) =>
                          handleInputChange(
                            input.name || `param${index}`,
                            e.target.value
                          )
                        }
                        placeholder={`Enter ${input.type}`}
                        className="param-input"
                      />
                    </div>
                  ))}
                </div>
              )}

              {/* Execute Button */}
              <div className="execute-section">
                <button
                  onClick={handleExecuteFunction}
                  disabled={executing}
                  className={`btn btn-large ${
                    selectedFunction.stateMutability === 'view' ||
                    selectedFunction.stateMutability === 'pure'
                      ? 'btn-secondary'
                      : 'btn-primary'
                  }`}
                >
                  {executing ? (
                    <>
                      <Loader2 size={20} className="spinner" />
                      <span>Executing...</span>
                    </>
                  ) : (
                    <>
                      <Play size={20} />
                      <span>
                        {selectedFunction.stateMutability === 'view' ||
                        selectedFunction.stateMutability === 'pure'
                          ? 'Call Function'
                          : 'Send Transaction'}
                      </span>
                    </>
                  )}
                </button>
              </div>

              {/* Result Display */}
              {result && (
                <div className="result-display">
                  <div className="result-header">
                    <CheckCircle size={20} className="success-icon" />
                    <h4>Result</h4>
                  </div>
                  <div className="result-content">
                    <pre>{JSON.stringify(result, null, 2)}</pre>
                  </div>
                </div>
              )}

              {/* Error Display */}
              {error && (
                <div className="error-display">
                  <AlertCircle size={16} />
                  <span>{error}</span>
                </div>
              )}
            </div>
          )}
        </div>
      )}

      {/* Password Prompt Modal */}
      {showPasswordPrompt && (
        <div className="modal-overlay">
          <div className="modal-content">
            <h3>Confirm Transaction</h3>
            <p>Enter your password to sign and send the transaction</p>
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
                  if (e.key === 'Enter') handleConfirmWriteFunction();
                }}
              />
            </div>
            <div className="modal-actions">
              <button
                onClick={handleCancelWriteFunction}
                className="btn btn-secondary"
              >
                Cancel
              </button>
              <button
                onClick={handleConfirmWriteFunction}
                className="btn btn-primary"
                disabled={!password}
              >
                Send Transaction
              </button>
            </div>
          </div>
        </div>
      )}

      <style jsx>{`
        .contract-interaction {
          max-width: 1200px;
        }

        .interaction-header {
          margin-bottom: 2rem;
        }

        .interaction-header h2 {
          margin: 0 0 0.5rem 0;
          font-size: 1.5rem;
          font-weight: 600;
          color: var(--text-primary);
        }

        .interaction-header p {
          margin: 0;
          color: var(--text-secondary);
        }

        .load-contract-section {
          background: var(--bg-secondary);
          padding: 2rem;
          border-radius: 0.5rem;
          border: 1px solid var(--border-primary);
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

        .address-input {
          width: 100%;
          padding: 0.75rem;
          border: 1px solid var(--border-primary);
          border-radius: 0.375rem;
          background: var(--bg-primary);
          color: var(--text-primary);
          font-size: 1rem;
          font-family: monospace;
        }

        .abi-input-actions {
          margin-bottom: 0.5rem;
        }

        .btn-import {
          display: inline-flex;
          align-items: center;
          gap: 0.5rem;
          padding: 0.5rem 1rem;
          background: var(--bg-primary);
          border: 1px solid var(--border-primary);
          border-radius: 0.375rem;
          color: var(--text-primary);
          font-size: 0.875rem;
          cursor: pointer;
          transition: all 200ms ease;
        }

        .btn-import:hover {
          background: var(--bg-tertiary);
          border-color: var(--brand-primary);
        }

        .abi-textarea {
          width: 100%;
          padding: 0.75rem;
          border: 1px solid var(--border-primary);
          border-radius: 0.375rem;
          background: var(--bg-primary);
          color: var(--text-primary);
          font-size: 0.875rem;
          font-family: monospace;
          resize: vertical;
        }

        .error-message {
          display: flex;
          align-items: center;
          gap: 0.5rem;
          padding: 1rem;
          background: rgba(239, 68, 68, 0.1);
          border: 1px solid #ef4444;
          border-radius: 0.375rem;
          color: #ef4444;
          margin-bottom: 1rem;
        }

        .action-section {
          text-align: center;
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

        .btn-secondary {
          background: var(--bg-tertiary);
          color: var(--text-primary);
          border: 1px solid var(--border-primary);
        }

        .btn-secondary:hover:not(:disabled) {
          background: var(--bg-secondary);
        }

        .btn-large {
          padding: 1rem 2rem;
          font-size: 1.1rem;
        }

        .spinner {
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

        .loaded-contract {
          margin-top: 2rem;
        }

        .contract-info {
          display: flex;
          align-items: center;
          justify-content: space-between;
          padding: 1.5rem;
          background: var(--bg-secondary);
          border: 1px solid var(--border-primary);
          border-radius: 0.5rem;
          margin-bottom: 2rem;
        }

        .info-item {
          display: flex;
          flex-direction: column;
          gap: 0.25rem;
        }

        .info-item .label {
          font-size: 0.875rem;
          color: var(--text-secondary);
        }

        .info-item .value {
          font-size: 1rem;
          color: var(--text-primary);
          font-weight: 500;
        }

        .info-item code {
          font-family: monospace;
          font-size: 0.875rem;
          color: var(--brand-primary);
        }

        .btn-reset {
          padding: 0.5rem 1rem;
          background: var(--bg-primary);
          border: 1px solid var(--border-primary);
          border-radius: 0.375rem;
          color: var(--text-primary);
          font-size: 0.875rem;
          cursor: pointer;
          transition: all 200ms ease;
        }

        .btn-reset:hover {
          background: var(--bg-tertiary);
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

        .functions-section {
          display: grid;
          grid-template-columns: repeat(auto-fit, minmax(300px, 1fr));
          gap: 2rem;
          margin-bottom: 2rem;
        }

        .function-category h3 {
          display: flex;
          align-items: center;
          gap: 0.5rem;
          margin: 0 0 1rem 0;
          font-size: 1.1rem;
          font-weight: 600;
          color: var(--text-primary);
        }

        .function-list {
          display: flex;
          flex-direction: column;
          gap: 0.5rem;
        }

        .function-button {
          display: flex;
          flex-direction: column;
          align-items: flex-start;
          padding: 0.75rem 1rem;
          background: var(--bg-secondary);
          border: 1px solid var(--border-primary);
          border-radius: 0.375rem;
          color: var(--text-primary);
          cursor: pointer;
          transition: all 200ms ease;
          text-align: left;
        }

        .function-button:hover {
          background: var(--bg-tertiary);
          border-color: var(--brand-primary);
        }

        .function-button.active {
          background: rgba(255, 165, 0, 0.1);
          border-color: var(--brand-primary);
          border-width: 2px;
        }

        .function-button.write {
          border-left: 3px solid #10b981;
        }

        .function-name {
          font-weight: 600;
          font-size: 0.95rem;
        }

        .function-params {
          font-size: 0.8rem;
          color: var(--text-muted);
          font-family: monospace;
          margin-top: 0.25rem;
        }

        .execution-panel {
          padding: 2rem;
          background: var(--bg-secondary);
          border: 1px solid var(--border-primary);
          border-radius: 0.5rem;
        }

        .execution-panel > h3 {
          display: flex;
          align-items: center;
          gap: 0.75rem;
          margin: 0 0 1.5rem 0;
          font-size: 1.25rem;
          font-weight: 600;
          color: var(--text-primary);
        }

        .mutability-badge {
          padding: 0.25rem 0.75rem;
          background: rgba(16, 185, 129, 0.1);
          border: 1px solid #10b981;
          border-radius: 1rem;
          font-size: 0.75rem;
          font-weight: 500;
          color: #10b981;
          text-transform: uppercase;
        }

        .function-inputs {
          margin-bottom: 2rem;
        }

        .function-inputs h4 {
          margin: 0 0 1rem 0;
          font-size: 1rem;
          font-weight: 600;
          color: var(--text-secondary);
        }

        .input-group {
          margin-bottom: 1rem;
        }

        .input-group label {
          display: block;
          margin-bottom: 0.375rem;
          font-weight: 500;
          color: var(--text-primary);
        }

        .input-type {
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

        .execute-section {
          margin: 2rem 0;
          text-align: center;
        }

        .result-display {
          margin-top: 2rem;
          padding: 1.5rem;
          background: rgba(16, 185, 129, 0.05);
          border: 1px solid #10b981;
          border-radius: 0.5rem;
        }

        .result-header {
          display: flex;
          align-items: center;
          gap: 0.5rem;
          margin-bottom: 1rem;
        }

        .result-header h4 {
          margin: 0;
          font-size: 1.1rem;
          font-weight: 600;
          color: var(--text-primary);
        }

        .success-icon {
          color: #10b981;
        }

        .result-content pre {
          margin: 0;
          padding: 1rem;
          background: var(--bg-primary);
          border-radius: 0.375rem;
          overflow-x: auto;
          font-family: monospace;
          font-size: 0.875rem;
          color: var(--text-primary);
        }

        .error-display {
          display: flex;
          align-items: center;
          gap: 0.5rem;
          padding: 1rem;
          background: rgba(239, 68, 68, 0.1);
          border: 1px solid #ef4444;
          border-radius: 0.375rem;
          color: #ef4444;
          margin-top: 1rem;
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
      `}</style>
    </div>
  );
};

export default ContractInteraction;
