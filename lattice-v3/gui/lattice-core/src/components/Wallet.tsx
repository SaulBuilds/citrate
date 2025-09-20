import React, { useState, useEffect } from 'react';
import { walletService, nodeService } from '../services/tauri';
import { Account, TxActivity } from '../types';
import { 
  Wallet as WalletIcon,
  Plus,
  Key,
  Copy,
  CheckCircle,
  Send
} from 'lucide-react';

export const Wallet: React.FC = () => {
  const [accounts, setAccounts] = useState<Account[]>([]);
  const [selectedAccount, setSelectedAccount] = useState<Account | null>(null);
  const [showCreateModal, setShowCreateModal] = useState(false);
  const [showImportModal, setShowImportModal] = useState(false);
  const [showSendModal, setShowSendModal] = useState(false);
  const [copied, setCopied] = useState<string | null>(null);
  const [activity, setActivity] = useState<TxActivity[]>([]);
  const [expanded, setExpanded] = useState<Record<string, boolean>>({});
  const [signMessage, setSignMessage] = useState('');
  const [signPassword, setSignPassword] = useState('');
  const [signature, setSignature] = useState<string | null>(null);
  const [verifyMessage, setVerifyMessage] = useState('');
  const [verifySignature, setVerifySignature] = useState('');
  const [verifyAddress, setVerifyAddress] = useState('');
  const [verifyResult, setVerifyResult] = useState<boolean | null>(null);
  // Tracked external addresses
  const [tracked, setTracked] = useState<string[]>(() => {
    try {
      const raw = localStorage.getItem('lattice_tracked_addresses');
      return raw ? JSON.parse(raw) : [];
    } catch { return []; }
  });
  const [trackedInput, setTrackedInput] = useState('');
  const [trackedData, setTrackedData] = useState<Record<string, { balance: string; activity: TxActivity[] }>>({});

  useEffect(() => {
    loadAccounts();
    // Auto-refresh balances every ~2.5s
    const interval = setInterval(() => {
      loadAccounts();
      loadActivity();
      refreshTracked();
    }, 2500);
    return () => clearInterval(interval);
  }, []);

  const loadAccounts = async () => {
    try {
      const accs = await walletService.getAccounts();
      setAccounts(accs);
      if (accs.length > 0) {
        if (!selectedAccount) {
          setSelectedAccount(accs[0]);
        } else {
          const updated = accs.find(a => a.address.toLowerCase() === selectedAccount.address.toLowerCase());
          if (updated) setSelectedAccount(updated);
        }
      }
    } catch (err) {
      console.error('Failed to load accounts:', err);
    }
  };

  // Expose loadActivity so we can refresh after send
  const loadActivity = async () => {
    try {
      if (selectedAccount) {
        const act = await walletService.getAccountActivity(selectedAccount.address, 256, 100);
        setActivity(act);
      } else {
        setActivity([]);
      }
    } catch (err) {
      console.error('Failed to load activity:', err);
    }
  };

  const persistTracked = (list: string[]) => {
    setTracked(list);
    try { localStorage.setItem('lattice_tracked_addresses', JSON.stringify(list)); } catch {}
  };

  const addTracked = async () => {
    const addr = trackedInput.trim();
    if (!addr) return;
    if (tracked.includes(addr)) return;
    const list = [...tracked, addr];
    persistTracked(list);
    setTrackedInput('');
    await refreshTrackedOne(addr);
  };

  const removeTracked = (addr: string) => {
    const list = tracked.filter(a => a.toLowerCase() !== addr.toLowerCase());
    persistTracked(list);
    setTrackedData(prev => { const c = { ...prev }; delete c[addr]; return c; });
  };

  const refreshTrackedOne = async (addr: string) => {
    try {
      const [bal, act] = await Promise.all([
        walletService.getObservedBalance(addr, 256),
        walletService.getAccountActivity(addr, 256, 25),
      ]);
      setTrackedData(prev => ({ ...prev, [addr]: { balance: bal, activity: act } }));
    } catch (e) {
      console.error('Failed to refresh tracked address', addr, e);
    }
  };

  const refreshTracked = async () => {
    for (const addr of tracked) { await refreshTrackedOne(addr); }
  };

  const copyToClipboard = (text: string, field: string) => {
    navigator.clipboard.writeText(text);
    setCopied(field);
    setTimeout(() => setCopied(null), 2000);
  };

  const formatLat = (valueStr: string): string => {
    try {
      const wei = BigInt(valueStr || '0');
      const whole = wei / (10n ** 18n);
      const frac = wei % (10n ** 18n);
      const fracStr = (frac + (10n ** 18n)).toString().slice(1, 5);
      return `${whole.toString()}.${fracStr} LAT`;
    } catch {
      return '0.0000 LAT';
    }
  };

  const handleSign = async () => {
    if (!selectedAccount) return;
    try {
      const sig = await walletService.signMessage(signMessage, selectedAccount.address, signPassword);
      setSignature(sig);
      setSignPassword('');
    } catch (err) {
      console.error('Sign failed:', err);
    }
  };

  const handleVerify = async () => {
    try {
      const ok = await walletService.verifySignature(verifyMessage, verifySignature, verifyAddress || (selectedAccount?.address || ''));
      setVerifyResult(ok);
    } catch (err) {
      console.error('Verify failed:', err);
      setVerifyResult(false);
    }
  };

  const formatAddress = (address: string) => {
    return `${address.slice(0, 6)}...${address.slice(-4)}`;
  };

  const formatBalance = (balance: bigint) => {
    // Convert from wei to ETH (assuming 18 decimals)
    const eth = Number(balance) / 1e18;
    return eth.toFixed(4);
  };

  return (
    <div className="wallet">
      <div className="wallet-header">
        <h2>Wallet Manager</h2>
        <div className="wallet-actions">
          <button 
            className="btn btn-secondary"
            onClick={() => setShowImportModal(true)}
          >
            <Key size={16} />
            Import Account
          </button>
          <button 
            className="btn btn-primary"
            onClick={() => setShowCreateModal(true)}
          >
            <Plus size={16} />
            Create Account
          </button>
        </div>
      </div>

      <div className="accounts-grid">
        {accounts.map(account => (
          <div 
            key={account.address}
            className={`account-card ${selectedAccount?.address === account.address ? 'selected' : ''}`}
            onClick={() => setSelectedAccount(account)}
          >
            <div className="account-header">
              <WalletIcon size={20} />
              <h3>{account.label}</h3>
            </div>
            
            <div className="account-address">
              <span>{formatAddress(account.address)}</span>
              <button 
                className="copy-btn"
                onClick={(e) => {
                  e.stopPropagation();
                  copyToClipboard(account.address, account.address);
                }}
              >
                {copied === account.address ? (
                  <CheckCircle size={14} className="text-green" />
                ) : (
                  <Copy size={14} />
                )}
              </button>
            </div>

            <div className="account-balance">
              <span className="balance-label">Balance</span>
              <span className="balance-value">
                {formatBalance(account.balance)} LAT
              </span>
            </div>

            <div className="account-footer">
              <span className="nonce">Nonce: {account.nonce}</span>
              <button 
                className="btn-sm btn-primary"
                onClick={(e) => {
                  e.stopPropagation();
                  setSelectedAccount(account);
                  setShowSendModal(true);
                }}
              >
                <Send size={14} />
                Send
              </button>
              <button
                className="btn-sm btn-secondary"
                onClick={async (e) => {
                  e.stopPropagation();
                  try {
                    await nodeService.setRewardAddress(account.address);
                    copyToClipboard(account.address, 'reward');
                    alert('Reward address set to this account. Block producer will use it.');
                  } catch (err) {
                    console.error('Failed to set reward address', err);
                    alert('Failed to set reward address');
                  }
                }}
              >
                Set Rewards
              </button>
            </div>
          </div>
        ))}

        {accounts.length === 0 && (
          <div className="empty-state">
            <WalletIcon size={48} className="text-gray" />
            <p>No accounts yet</p>
            <p className="text-muted">Create or import an account to get started</p>
          </div>
        )}
      </div>

      {/* Activity */}
      {selectedAccount && (
        <div className="activity">
          <h3>Activity</h3>
          <div className="activity-list">
            {activity.length === 0 && <div className="muted">No activity yet</div>}
            {activity.map(tx => (
              <div key={tx.hash} className={`tx ${tx.status}`}>
                <div className="left">
                  <div className="hash mono">{tx.hash.slice(0, 10)}…</div>
                  <div className="meta">
                    <span className={`badge ${tx.status === 'pending' ? 'badge-yellow' : 'badge-green'}`}>{tx.status}</span>
                    {tx.blockHeight !== undefined && <span className="muted">h{tx.blockHeight}</span>}
                    {tx.timestamp && <span className="muted">{new Date(tx.timestamp).toLocaleTimeString()}</span>}
                  </div>
                </div>
                <div className="right">
                  <div className="amount">{formatLat(tx.value)}</div>
                  <div className="addr mono">→ {(tx.to || '').slice(0, 10)}…</div>
                  <button className="btn btn-secondary btn-sm" onClick={() => copyToClipboard(tx.hash, 'txhash')}>Copy</button>
                  <button className="btn btn-secondary btn-sm" onClick={() => setExpanded(prev => ({ ...prev, [tx.hash]: !prev[tx.hash] }))}>Details</button>
                  {tx.blockHash && (
                    <button className="btn btn-primary btn-sm" onClick={() => {
                      try {
                        localStorage.setItem('dag_focus_hash', tx.blockHash!);
                        window.dispatchEvent(new CustomEvent('open-dag-for-hash', { detail: { hash: tx.blockHash } }));
                      } catch {}
                    }}>View in Explorer</button>
                  )}
                </div>
                {expanded[tx.hash] && (
                  <div className="tx-details">
                    <div className="row mono"><span className="label">From</span><span>{tx.from}</span><button className="btn btn-secondary btn-sm" onClick={() => copyToClipboard(tx.from, 'from')}>Copy</button></div>
                    <div className="row mono"><span className="label">To</span><span>{tx.to || '—'}</span>
                      {tx.to && <>
                        <button className="btn btn-secondary btn-sm" onClick={() => copyToClipboard(tx.to!, 'to')}>Copy</button>
                        <button className="btn btn-secondary btn-sm" onClick={() => { setTrackedInput(tx.to!); /* user can click Track */ }}>Track</button>
                      </>}
                    </div>
                    <div className="row mono"><span className="label">Hash</span><span>{tx.hash}</span></div>
                  </div>
                )}
              </div>
            ))}
          </div>
        </div>
      )}

      {/* Tracked External Addresses */}
      <div className="activity" style={{ marginTop: '2rem' }}>
        <h3>Tracked Addresses</h3>
        <div className="form-group" style={{ display: 'flex', gap: '0.5rem' }}>
          <input type="text" placeholder="0x... external address" value={trackedInput} onChange={e => setTrackedInput(e.target.value)} />
          <button className="btn btn-secondary" onClick={addTracked} disabled={!trackedInput.trim()}>Track</button>
          <button className="btn btn-secondary" onClick={refreshTracked} disabled={tracked.length === 0}>Refresh</button>
        </div>
        <div className="activity-list">
          {tracked.length === 0 && <div className="muted">No tracked addresses</div>}
          {tracked.map(addr => (
            <div key={addr} className="tx confirmed">
              <div className="left">
                <div className="hash mono">{addr.slice(0, 12)}…</div>
                <div className="meta">
                  <span className="badge">Observed</span>
                  <span className="muted">{formatLat(trackedData[addr]?.balance || '0')}</span>
                </div>
              </div>
              <div className="right" style={{ display: 'flex', gap: '0.5rem', alignItems: 'center' }}>
                <button className="btn btn-secondary btn-sm" onClick={() => refreshTrackedOne(addr)}>Refresh</button>
                <button className="btn btn-secondary btn-sm" onClick={() => removeTracked(addr)}>Remove</button>
              </div>
              {/* Recent activity for this address */}
              {trackedData[addr]?.activity && trackedData[addr].activity.length > 0 && (
                <div style={{ marginTop: '0.5rem' }}>
                  {trackedData[addr].activity.slice(0,5).map(tx => (
                    <div key={tx.hash} className="tx-item mono" style={{ fontSize: '0.8rem' }}>
                      <span className="muted">{tx.status}</span> • {tx.hash.slice(0,10)}… • {formatLat(tx.value)}
                    </div>
                  ))}
                </div>
              )}
            </div>
          ))}
        </div>
      </div>

      {/* Sign / Verify */}
      <div className="sign-verify">
        <h3>Sign &amp; Verify</h3>
        <div className="sign-box">
          <h4>Sign Message</h4>
          <textarea placeholder="Message to sign" value={signMessage} onChange={e => setSignMessage(e.target.value)} />
          <input type="password" placeholder="Password" value={signPassword} onChange={e => setSignPassword(e.target.value)} />
          <button className="btn btn-primary" onClick={handleSign} disabled={!selectedAccount || !signMessage || !signPassword}>Sign</button>
          {signature && (
            <div className="signature mono">
              <span>{signature.slice(0, 28)}…</span>
              <button className="btn btn-secondary btn-sm" onClick={() => copyToClipboard(signature, 'signature')}>Copy</button>
            </div>
          )}
        </div>
        <div className="verify-box">
          <h4>Verify Signature</h4>
          <textarea placeholder="Message" value={verifyMessage} onChange={e => setVerifyMessage(e.target.value)} />
          <input type="text" placeholder="Signature (hex)" value={verifySignature} onChange={e => setVerifySignature(e.target.value)} />
          <input type="text" placeholder="Address (optional)" value={verifyAddress} onChange={e => setVerifyAddress(e.target.value)} />
          <button className="btn btn-secondary" onClick={handleVerify} disabled={!verifyMessage || !verifySignature}>Verify</button>
          {verifyResult !== null && (
            <div className={`verify-result ${verifyResult ? 'ok' : 'bad'}`}>{verifyResult ? 'Valid' : 'Invalid'}</div>
          )}
        </div>
      </div>

      {/* Create Account Modal */}
      {showCreateModal && (
        <CreateAccountModal 
          onClose={() => setShowCreateModal(false)}
          onCreated={() => {
            loadAccounts();
            setShowCreateModal(false);
          }}
        />
      )}

      {/* Import Account Modal */}
      {showImportModal && (
        <ImportAccountModal
          onClose={() => setShowImportModal(false)}
          onImported={() => {
            loadAccounts();
            setShowImportModal(false);
          }}
        />
      )}

      {/* Send Transaction Modal */}
      {showSendModal && selectedAccount && (
        <SendTransactionModal
          account={selectedAccount}
          onClose={() => setShowSendModal(false)}
          onSent={() => {
            loadAccounts();
            loadActivity();
            setShowSendModal(false);
          }}
        />
      )}

      <style jsx>{`
        .wallet {
          padding: 2rem;
        }

        .wallet-header {
          display: flex;
          justify-content: space-between;
          align-items: center;
          margin-bottom: 2rem;
        }

        .wallet-header h2 {
          margin: 0;
          font-size: 1.5rem;
          font-weight: 600;
        }

        .wallet-actions {
          display: flex;
          gap: 1rem;
        }

        .btn {
          display: flex;
          align-items: center;
          gap: 0.5rem;
          padding: 0.75rem 1.5rem;
          border: none;
          border-radius: 0.5rem;
          font-size: 1rem;
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

        .btn-sm {
          padding: 0.5rem 1rem;
          font-size: 0.875rem;
        }

        .accounts-grid {
          display: grid;
          grid-template-columns: repeat(auto-fill, minmax(320px, 1fr));
          gap: 1.5rem;
        }

        .account-card {
          background: white;
          border-radius: 1rem;
          padding: 1.5rem;
          box-shadow: 0 4px 6px rgba(0, 0, 0, 0.1);
          cursor: pointer;
          transition: all 0.2s;
          border: 2px solid transparent;
        }

        .account-card:hover {
          transform: translateY(-4px);
          box-shadow: 0 8px 16px rgba(0, 0, 0, 0.15);
        }

        .account-card.selected {
          border-color: #667eea;
        }

        .account-header {
          display: flex;
          align-items: center;
          gap: 0.75rem;
          margin-bottom: 1rem;
        }

        .account-header h3 {
          margin: 0;
          font-size: 1.125rem;
          font-weight: 600;
        }

        .account-address {
          display: flex;
          align-items: center;
          justify-content: space-between;
          padding: 0.75rem;
          background: #f9fafb;
          border-radius: 0.5rem;
          margin-bottom: 1rem;
          font-family: monospace;
        }

        .copy-btn {
          background: none;
          border: none;
          cursor: pointer;
          padding: 0.25rem;
          display: flex;
          align-items: center;
        }

        .account-balance {
          display: flex;
          justify-content: space-between;
          margin-bottom: 1rem;
        }

        .balance-label {
          color: #6b7280;
          font-size: 0.875rem;
        }

        .balance-value {
          font-size: 1.25rem;
          font-weight: 600;
        }

        .account-footer {
          display: flex;
          justify-content: space-between;
          align-items: center;
        }

        .nonce {
          color: #9ca3af;
          font-size: 0.875rem;
        }

        .empty-state {
          grid-column: 1 / -1;
          text-align: center;
          padding: 3rem;
        }

        .empty-state p {
          margin: 0.5rem 0;
        }

        .text-gray { color: #6b7280; }
        .text-green { color: #10b981; }
        .text-muted { color: #9ca3af; }

        /* Activity & Sign/Verify */
        .activity { margin-top: 2rem; }
        .activity h3 { margin: 0 0 0.75rem 0; }
        .activity-list { display: flex; flex-direction: column; gap: 0.5rem; }
        .tx { display: flex; justify-content: space-between; align-items: center; background: #f9fafb; border-radius: 0.5rem; padding: 0.75rem; }
        .tx .meta { display: flex; gap: 0.5rem; align-items: center; }
        .badge { padding: 0.15rem 0.5rem; border-radius: 0.25rem; font-size: 0.75rem; font-weight: 600; }
        .badge-yellow { background: #fef3c7; color: #92400e; }
        .badge-green { background: #d1fae5; color: #065f46; }
        .sign-verify { margin-top: 2rem; display: grid; grid-template-columns: 1fr 1fr; gap: 1rem; }
        .sign-verify h3 { grid-column: 1 / -1; margin: 0; }
        .sign-box, .verify-box { background: white; border-radius: 0.75rem; padding: 1rem; box-shadow: 0 1px 2px rgba(0,0,0,0.05); }
        .signature { display: flex; gap: 0.5rem; align-items: center; margin-top: 0.5rem; }
        .verify-result.ok { color: #065f46; }
        .verify-result.bad { color: #991b1b; }
      `}</style>
    </div>
  );
};

// Modal Components
const CreateAccountModal: React.FC<{
  onClose: () => void;
  onCreated: () => void;
}> = ({ onClose, onCreated }) => {
  const [label, setLabel] = useState('');
  const [password, setPassword] = useState('');
  const [confirmPassword, setConfirmPassword] = useState('');
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [credentials, setCredentials] = useState<{ priv: string; mnemonic: string; address: string } | null>(null);
  const [revealed, setRevealed] = useState(false);
  const [pressTimer, setPressTimer] = useState<number | null>(null);

  const startReveal = () => {
    if (pressTimer) return;
    const id = window.setTimeout(() => {
      setRevealed(true);
      setPressTimer(null);
    }, 2000);
    setPressTimer(id);
  };
  const cancelReveal = () => {
    if (pressTimer) {
      window.clearTimeout(pressTimer);
      setPressTimer(null);
    }
  };

  const handleCreate = async () => {
    if (password !== confirmPassword) {
      setError('Passwords do not match');
      return;
    }
    
    if (password.length < 8) {
      setError('Password must be at least 8 characters');
      return;
    }

    setLoading(true);
    setError(null);
    
    try {
      const [account, privateKey, mnemonic] = await walletService.createAccountExtended(label, password);
      setCredentials({ priv: privateKey, mnemonic, address: account.address });
    } catch (err: any) {
      setError(err.toString());
    } finally {
      setLoading(false);
    }
  };

  return (
    <div className="modal-overlay">
      <div className="modal" onClick={e => e.stopPropagation()}>
        <h3>Create New Account</h3>
        
        {error && <div className="error-message">{error}</div>}
        {!credentials ? (
          <>
            <div className="form-group">
              <label>Account Label</label>
              <input
                type="text"
                value={label}
                onChange={e => setLabel(e.target.value)}
                placeholder="Main Account"
              />
            </div>

            <div className="form-group">
              <label>Password</label>
              <input
                type="password"
                value={password}
                onChange={e => setPassword(e.target.value)}
                placeholder="Enter password"
              />
            </div>

            <div className="form-group">
              <label>Confirm Password</label>
              <input
                type="password"
                value={confirmPassword}
                onChange={e => setConfirmPassword(e.target.value)}
                placeholder="Confirm password"
              />
            </div>

            <div className="modal-actions">
              <button className="btn btn-secondary" onClick={onClose}>
                Cancel
              </button>
              <button 
                className="btn btn-primary" 
                onClick={handleCreate}
                disabled={loading || !label || !password}
              >
                {loading ? 'Creating...' : 'Create Account'}
              </button>
            </div>
          </>
        ) : (
          <>
            <div className="warning">
              Save these credentials now. They are shown once. Anyone with these can control your funds.
            </div>
            <div className="reveal" 
                 onMouseDown={startReveal} onMouseUp={cancelReveal} onMouseLeave={cancelReveal}
                 onTouchStart={startReveal} onTouchEnd={cancelReveal}>
              {revealed ? 'Credentials revealed' : 'Press and hold here for 2 seconds to reveal credentials'}
            </div>
            {revealed && (
              <div className="cred">
                <label>Address</label>
                <div className="mono row">
                  <span>{credentials.address}</span>
                  <button className="btn btn-secondary btn-sm" onClick={() => navigator.clipboard.writeText(credentials.address)}>Copy</button>
                </div>
                <label>12‑word Seed (Mnemonic)</label>
                <div className="mono row">
                  <span className="seed">{credentials.mnemonic}</span>
                  <button className="btn btn-secondary btn-sm" onClick={() => navigator.clipboard.writeText(credentials.mnemonic)}>Copy</button>
                </div>
                <label>Private Key (hex)</label>
                <div className="mono row">
                  <span>{credentials.priv}</span>
                  <button className="btn btn-secondary btn-sm" onClick={() => navigator.clipboard.writeText(credentials.priv)}>Copy</button>
                </div>
              </div>
            )}
            <div className="modal-actions">
              <button className="btn btn-primary" disabled={!revealed} onClick={() => { onCreated(); onClose(); }}>I saved it</button>
            </div>
          </>
        )}
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
          padding: 2rem;
          width: 90%;
          max-width: 500px;
        }

        .modal h3 {
          margin: 0 0 1.5rem 0;
        }

        .form-group {
          margin-bottom: 1.5rem;
        }

        .form-group label {
          display: block;
          margin-bottom: 0.5rem;
          font-weight: 500;
        }

        .form-group input {
          width: 100%;
          padding: 0.75rem;
          border: 1px solid #e5e7eb;
          border-radius: 0.5rem;
          font-size: 1rem;
        }

        .error-message {
          background: #fee;
          color: #c00;
          padding: 0.75rem;
          border-radius: 0.5rem;
          margin-bottom: 1rem;
        }

        .modal-actions {
          display: flex;
          gap: 1rem;
          justify-content: flex-end;
        }

        .btn {
          padding: 0.75rem 1.5rem;
          border: none;
          border-radius: 0.5rem;
          font-size: 1rem;
          font-weight: 500;
          cursor: pointer;
        }

        .btn-primary {
          background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
          color: white;
        }

        .btn-secondary {
          background: #f3f4f6;
          color: #374151;
        }

        .btn:disabled {
          opacity: 0.5;
          cursor: not-allowed;
        }
        .warning { background: #fff7ed; color: #7c2d12; border: 1px solid #fdba74; padding: 0.75rem; border-radius: 0.5rem; }
        .reveal { margin: 1rem 0; padding: 1rem; background: #fef9c3; border: 1px dashed #f59e0b; border-radius: 0.5rem; text-align: center; color: #92400e; }
        .cred { display: flex; flex-direction: column; gap: 0.5rem; }
        .row { display: flex; align-items: center; justify-content: space-between; gap: 0.5rem; }
      `}</style>
    </div>
  );
};

const ImportAccountModal: React.FC<{
  onClose: () => void;
  onImported: () => void;
}> = ({ onClose, onImported }) => {
  const [privateKey, setPrivateKey] = useState('');
  const [mnemonic, setMnemonic] = useState('');
  const [label, setLabel] = useState('');
  const [password, setPassword] = useState('');
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const handleImport = async () => {
    setLoading(true);
    setError(null);
    
    try {
      if (mnemonic.trim()) {
        await walletService.importAccountFromMnemonic(mnemonic.trim(), label, password);
      } else {
        await walletService.importAccount(privateKey, label, password);
      }
      onImported();
    } catch (err: any) {
      setError(err.toString());
    } finally {
      setLoading(false);
    }
  };

  return (
    <div className="modal-overlay" onClick={onClose}>
      <div className="modal" onClick={e => e.stopPropagation()}>
        <h3>Import Account</h3>
        
        {error && <div className="error-message">{error}</div>}
        
        <div className="form-group">
          <label>Private Key (Hex)</label>
          <input
            type="password"
            value={privateKey}
            onChange={e => setPrivateKey(e.target.value)}
            placeholder="Enter private key (or leave empty to use mnemonic)"
          />
        </div>

        <div className="form-group">
          <label>Mnemonic (12/24 words)</label>
          <input
            type="text"
            value={mnemonic}
            onChange={e => setMnemonic(e.target.value)}
            placeholder="Enter seed phrase (optional)"
          />
        </div>

        <div className="form-group">
          <label>Account Label</label>
          <input
            type="text"
            value={label}
            onChange={e => setLabel(e.target.value)}
            placeholder="Imported Account"
          />
        </div>

        <div className="form-group">
          <label>Password</label>
          <input
            type="password"
            value={password}
            onChange={e => setPassword(e.target.value)}
            placeholder="Enter password to encrypt key"
          />
        </div>

        <div className="modal-actions">
          <button className="btn btn-secondary" onClick={onClose}>
            Cancel
          </button>
          <button 
            className="btn btn-primary" 
            onClick={handleImport}
            disabled={loading || (!privateKey && !mnemonic) || !label || !password}
          >
            {loading ? 'Importing...' : 'Import Account'}
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
          padding: 2rem;
          width: 90%;
          max-width: 500px;
        }

        .modal h3 {
          margin: 0 0 1.5rem 0;
        }

        .form-group {
          margin-bottom: 1.5rem;
        }

        .form-group label {
          display: block;
          margin-bottom: 0.5rem;
          font-weight: 500;
        }

        .form-group input {
          width: 100%;
          padding: 0.75rem;
          border: 1px solid #e5e7eb;
          border-radius: 0.5rem;
          font-size: 1rem;
        }

        .error-message {
          background: #fee;
          color: #c00;
          padding: 0.75rem;
          border-radius: 0.5rem;
          margin-bottom: 1rem;
        }

        .modal-actions {
          display: flex;
          gap: 1rem;
          justify-content: flex-end;
        }

        .btn {
          padding: 0.75rem 1.5rem;
          border: none;
          border-radius: 0.5rem;
          font-size: 1rem;
          font-weight: 500;
          cursor: pointer;
        }

        .btn-primary {
          background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
          color: white;
        }

        .btn-secondary {
          background: #f3f4f6;
          color: #374151;
        }

        .btn:disabled {
          opacity: 0.5;
          cursor: not-allowed;
        }
      `}</style>
    </div>
  );
};

const SendTransactionModal: React.FC<{
  account: Account;
  onClose: () => void;
  onSent: () => void;
}> = ({ account, onClose, onSent }) => {
  const [to, setTo] = useState('');
  const [amount, setAmount] = useState('');
  const [password, setPassword] = useState('');
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [txHash, setTxHash] = useState<string | null>(null);

  const handleSend = async () => {
    setLoading(true);
    setError(null);
    
    try {
      const value = BigInt(Math.floor(parseFloat(amount) * 1e18));
      const txRequest = {
        from: account.address,
        to,
        value,
        gasLimit: 21000,
        gasPrice: BigInt(20e9), // 20 gwei
        data: [],
      };
      
      const hash = await walletService.sendTransaction(txRequest, password);
      setTxHash(hash);
      setTimeout(() => {
        onSent();
      }, 2000);
    } catch (err: any) {
      setError(err.toString());
    } finally {
      setLoading(false);
    }
  };

  return (
    <div className="modal-overlay" onClick={onClose}>
      <div className="modal" onClick={e => e.stopPropagation()}>
        <h3>Send Transaction</h3>
        
        {error && <div className="error-message">{error}</div>}
        {txHash && (
          <div className="success-message">
            Transaction sent! Hash: {txHash.slice(0, 10)}...
          </div>
        )}
        
        <div className="form-group">
          <label>From</label>
          <input
            type="text"
            value={account.address}
            disabled
          />
        </div>

        <div className="form-group">
          <label>To Address</label>
          <input
            type="text"
            value={to}
            onChange={e => setTo(e.target.value)}
            placeholder="0x..."
          />
        </div>

        <div className="form-group">
          <label>Amount (LAT)</label>
          <input
            type="number"
            value={amount}
            onChange={e => setAmount(e.target.value)}
            placeholder="0.0"
            step="0.0001"
          />
        </div>

        <div className="form-group">
          <label>Password</label>
          <input
            type="password"
            value={password}
            onChange={e => setPassword(e.target.value)}
            placeholder="Enter password to sign"
          />
        </div>

        <div className="modal-actions">
          <button className="btn btn-secondary" onClick={onClose}>
            Cancel
          </button>
          <button 
            className="btn btn-primary" 
            onClick={handleSend}
            disabled={loading || !to || !amount || !password || !!txHash}
          >
            {loading ? 'Sending...' : 'Send Transaction'}
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
          padding: 2rem;
          width: 90%;
          max-width: 500px;
        }

        .modal h3 {
          margin: 0 0 1.5rem 0;
        }

        .form-group {
          margin-bottom: 1.5rem;
        }

        .form-group label {
          display: block;
          margin-bottom: 0.5rem;
          font-weight: 500;
        }

        .form-group input {
          width: 100%;
          padding: 0.75rem;
          border: 1px solid #e5e7eb;
          border-radius: 0.5rem;
          font-size: 1rem;
        }

        .form-group input:disabled {
          background: #f9fafb;
          color: #6b7280;
        }

        .error-message {
          background: #fee;
          color: #c00;
          padding: 0.75rem;
          border-radius: 0.5rem;
          margin-bottom: 1rem;
        }

        .success-message {
          background: #d4edda;
          color: #155724;
          padding: 0.75rem;
          border-radius: 0.5rem;
          margin-bottom: 1rem;
        }

        .modal-actions {
          display: flex;
          gap: 1rem;
          justify-content: flex-end;
        }

        .btn {
          padding: 0.75rem 1.5rem;
          border: none;
          border-radius: 0.5rem;
          font-size: 1rem;
          font-weight: 500;
          cursor: pointer;
        }

        .btn-primary {
          background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
          color: white;
        }

        .btn-secondary {
          background: #f3f4f6;
          color: #374151;
        }

        .btn:disabled {
          opacity: 0.5;
          cursor: not-allowed;
        }
      `}</style>
    </div>
  );
};
