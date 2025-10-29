/**
 * ContractEditor Component
 *
 * Monaco-based code editor for Solidity smart contracts.
 * Features:
 * - Syntax highlighting for Solidity
 * - Auto-save to localStorage
 * - Import/export contracts
 * - Example contract library
 * - Dark mode support
 */

import React, { useState, useEffect } from 'react';
import Editor from '@monaco-editor/react';
import { useTheme } from '../contexts/ThemeContext';
import { Download, Upload, FileText, Save } from 'lucide-react';

export interface ContractEditorProps {
  value: string;
  onChange: (value: string) => void;
  readonly?: boolean;
  height?: string;
  autoSave?: boolean;
  storageKey?: string;
}

// Example Solidity contracts
const EXAMPLE_CONTRACTS = {
  erc20: {
    name: 'ERC-20 Token',
    code: `// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;

contract SimpleToken {
    string public name;
    string public symbol;
    uint8 public decimals;
    uint256 public totalSupply;

    mapping(address => uint256) public balanceOf;
    mapping(address => mapping(address => uint256)) public allowance;

    event Transfer(address indexed from, address indexed to, uint256 value);
    event Approval(address indexed owner, address indexed spender, uint256 value);

    constructor(string memory _name, string memory _symbol, uint256 _initialSupply) {
        name = _name;
        symbol = _symbol;
        decimals = 18;
        totalSupply = _initialSupply * 10 ** uint256(decimals);
        balanceOf[msg.sender] = totalSupply;
    }

    function transfer(address _to, uint256 _value) public returns (bool success) {
        require(balanceOf[msg.sender] >= _value, "Insufficient balance");
        balanceOf[msg.sender] -= _value;
        balanceOf[_to] += _value;
        emit Transfer(msg.sender, _to, _value);
        return true;
    }

    function approve(address _spender, uint256 _value) public returns (bool success) {
        allowance[msg.sender][_spender] = _value;
        emit Approval(msg.sender, _spender, _value);
        return true;
    }

    function transferFrom(address _from, address _to, uint256 _value) public returns (bool success) {
        require(_value <= balanceOf[_from], "Insufficient balance");
        require(_value <= allowance[_from][msg.sender], "Insufficient allowance");
        balanceOf[_from] -= _value;
        balanceOf[_to] += _value;
        allowance[_from][msg.sender] -= _value;
        emit Transfer(_from, _to, _value);
        return true;
    }
}`,
  },
  storage: {
    name: 'Simple Storage',
    code: `// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;

contract SimpleStorage {
    uint256 private storedNumber;
    string private storedString;

    event NumberUpdated(uint256 newNumber);
    event StringUpdated(string newString);

    function setNumber(uint256 _number) public {
        storedNumber = _number;
        emit NumberUpdated(_number);
    }

    function getNumber() public view returns (uint256) {
        return storedNumber;
    }

    function setString(string memory _str) public {
        storedString = _str;
        emit StringUpdated(_str);
    }

    function getString() public view returns (string memory) {
        return storedString;
    }
}`,
  },
  counter: {
    name: 'Counter',
    code: `// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;

contract Counter {
    uint256 public count;

    event Incremented(uint256 newCount);
    event Decremented(uint256 newCount);
    event Reset();

    function increment() public {
        count += 1;
        emit Incremented(count);
    }

    function decrement() public {
        require(count > 0, "Count cannot be negative");
        count -= 1;
        emit Decremented(count);
    }

    function reset() public {
        count = 0;
        emit Reset();
    }

    function getCount() public view returns (uint256) {
        return count;
    }
}`,
  },
};

export const ContractEditor: React.FC<ContractEditorProps> = ({
  value,
  onChange,
  readonly = false,
  height = '500px',
  autoSave = true,
  storageKey = 'contract-source',
}) => {
  const { currentTheme } = useTheme();
  const [showExamples, setShowExamples] = useState(false);
  const [lastSaved, setLastSaved] = useState<Date | null>(null);

  // Auto-save to localStorage
  useEffect(() => {
    if (!autoSave || !value) return;

    const timeoutId = setTimeout(() => {
      try {
        localStorage.setItem(storageKey, value);
        setLastSaved(new Date());
        console.log('[ContractEditor] Auto-saved to localStorage');
      } catch (error) {
        console.error('[ContractEditor] Failed to auto-save:', error);
      }
    }, 1000); // Debounce: save 1 second after last edit

    return () => clearTimeout(timeoutId);
  }, [value, autoSave, storageKey]);

  // Load from localStorage on mount
  useEffect(() => {
    if (!autoSave) return;

    try {
      const saved = localStorage.getItem(storageKey);
      if (saved && !value) {
        onChange(saved);
        console.log('[ContractEditor] Loaded from localStorage');
      }
    } catch (error) {
      console.error('[ContractEditor] Failed to load from storage:', error);
    }
  }, []);

  const handleEditorChange = (newValue: string | undefined) => {
    onChange(newValue || '');
  };

  const handleLoadExample = (exampleKey: keyof typeof EXAMPLE_CONTRACTS) => {
    const example = EXAMPLE_CONTRACTS[exampleKey];
    onChange(example.code);
    setShowExamples(false);
  };

  const handleExport = () => {
    const blob = new Blob([value], { type: 'text/plain' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = 'Contract.sol';
    a.click();
    URL.revokeObjectURL(url);
  };

  const handleImport = () => {
    const input = document.createElement('input');
    input.type = 'file';
    input.accept = '.sol';
    input.onchange = (e) => {
      const file = (e.target as HTMLInputElement).files?.[0];
      if (!file) return;

      const reader = new FileReader();
      reader.onload = (event) => {
        const content = event.target?.result as string;
        onChange(content);
      };
      reader.readAsText(file);
    };
    input.click();
  };

  const handleManualSave = () => {
    if (autoSave) {
      localStorage.setItem(storageKey, value);
      setLastSaved(new Date());
    }
  };

  return (
    <div className="contract-editor">
      <div className="editor-toolbar">
        <div className="toolbar-left">
          <button
            className="toolbar-button"
            onClick={() => setShowExamples(!showExamples)}
            aria-label="Load example contract"
          >
            <FileText size={16} />
            <span>Examples</span>
          </button>

          <button
            className="toolbar-button"
            onClick={handleImport}
            aria-label="Import contract from file"
          >
            <Upload size={16} />
            <span>Import</span>
          </button>

          <button
            className="toolbar-button"
            onClick={handleExport}
            disabled={!value}
            aria-label="Export contract to file"
          >
            <Download size={16} />
            <span>Export</span>
          </button>

          {autoSave && (
            <button
              className="toolbar-button"
              onClick={handleManualSave}
              aria-label="Save contract"
            >
              <Save size={16} />
              <span>Save</span>
            </button>
          )}
        </div>

        <div className="toolbar-right">
          {lastSaved && (
            <span className="last-saved">
              Saved {lastSaved.toLocaleTimeString()}
            </span>
          )}
        </div>
      </div>

      {showExamples && (
        <div className="examples-dropdown">
          <h4>Example Contracts</h4>
          {Object.entries(EXAMPLE_CONTRACTS).map(([key, example]) => (
            <button
              key={key}
              className="example-button"
              onClick={() => handleLoadExample(key as keyof typeof EXAMPLE_CONTRACTS)}
            >
              {example.name}
            </button>
          ))}
        </div>
      )}

      <div className="editor-container">
        <Editor
          height={height}
          defaultLanguage="sol"
          language="sol"
          theme={currentTheme === 'dark' ? 'vs-dark' : 'vs-light'}
          value={value}
          onChange={handleEditorChange}
          options={{
            readOnly: readonly,
            minimap: { enabled: false },
            fontSize: 14,
            lineNumbers: 'on',
            folding: true,
            wordWrap: 'on',
            automaticLayout: true,
            scrollBeyondLastLine: false,
            tabSize: 4,
            insertSpaces: true,
          }}
        />
      </div>

      <style jsx>{`
        .contract-editor {
          display: flex;
          flex-direction: column;
          border: 1px solid var(--border-primary);
          border-radius: 0.5rem;
          overflow: hidden;
        }

        .editor-toolbar {
          display: flex;
          justify-content: space-between;
          align-items: center;
          padding: 0.5rem;
          background: var(--bg-secondary);
          border-bottom: 1px solid var(--border-primary);
        }

        .toolbar-left {
          display: flex;
          gap: 0.5rem;
        }

        .toolbar-button {
          display: flex;
          align-items: center;
          gap: 0.375rem;
          padding: 0.5rem 0.75rem;
          background: var(--bg-primary);
          border: 1px solid var(--border-primary);
          border-radius: 0.375rem;
          color: var(--text-primary);
          font-size: 0.875rem;
          cursor: pointer;
          transition: all 200ms ease;
        }

        .toolbar-button:hover:not(:disabled) {
          background: var(--bg-tertiary);
          border-color: var(--brand-primary);
        }

        .toolbar-button:disabled {
          opacity: 0.5;
          cursor: not-allowed;
        }

        .toolbar-right {
          font-size: 0.8rem;
          color: var(--text-muted);
        }

        .last-saved {
          font-style: italic;
        }

        .examples-dropdown {
          position: absolute;
          top: 3rem;
          left: 0.5rem;
          background: var(--bg-primary);
          border: 1px solid var(--border-primary);
          border-radius: 0.5rem;
          padding: 0.75rem;
          box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
          z-index: 10;
          min-width: 200px;
        }

        .examples-dropdown h4 {
          margin: 0 0 0.5rem 0;
          font-size: 0.875rem;
          font-weight: 600;
          color: var(--text-secondary);
          text-transform: uppercase;
        }

        .example-button {
          display: block;
          width: 100%;
          padding: 0.5rem 0.75rem;
          background: transparent;
          border: none;
          text-align: left;
          color: var(--text-primary);
          cursor: pointer;
          border-radius: 0.25rem;
          transition: background 200ms ease;
        }

        .example-button:hover {
          background: var(--bg-secondary);
        }

        .editor-container {
          position: relative;
        }
      `}</style>
    </div>
  );
};

export default ContractEditor;
