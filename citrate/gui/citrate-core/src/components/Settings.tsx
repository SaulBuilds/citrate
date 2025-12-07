import React, { useEffect, useMemo, useState } from 'react';
import { nodeService } from '../services/tauri';
import type { NodeConfig, NodeStatus, PeerInfoSummary } from '../types';
import { validateIPv4, validatePort, ValidationResult } from '../utils/validation';
import { useTheme } from '../contexts/ThemeContext';
import { Sun, Moon, Monitor } from 'lucide-react';

export const Settings: React.FC = () => {
  const { themeMode, setThemeMode } = useTheme();
  const [config, setConfig] = useState<NodeConfig | null>(null);
  const [status, setStatus] = useState<NodeStatus | null>(null);
  const [loading, setLoading] = useState(true);
  const [saving, setSaving] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [success, setSuccess] = useState<string | null>(null);
  // Network/peering UI state
  const [bootnodes, setBootnodes] = useState<string[]>([]);
  const [newBootnode, setNewBootnode] = useState('');
  const [peers, setPeers] = useState<PeerInfoSummary[]>([]);
  const [connectEntry, setConnectEntry] = useState('');
  const [netLoading, setNetLoading] = useState(false);
  const [importHost, setImportHost] = useState('127.0.0.1');
  const [seedPath, setSeedPath] = useState('');

  // Validation error states
  const [bootnodeError, setBootnodeError] = useState('');
  const [peerError, setPeerError] = useState('');

  useEffect(() => {
    const load = async () => {
      try {
        setLoading(true);
        const [cfg, stat, bn, ps] = await Promise.all([
          nodeService.getConfig().catch(() => null),
          nodeService.getStatus().catch(() => null),
          nodeService.getBootnodes().catch(() => []),
          nodeService.getPeers().catch(() => []),
        ]);
        if (cfg) {
          // Ensure consensus field exists with defaults
          if (!cfg.consensus) {
            cfg.consensus = {
              kParameter: 18,
              pruningWindow: 100000,
              blockTimeSeconds: 2,
              finalityDepth: 12
            };
          }
          // Ensure mempool field exists with defaults
          // SECURITY: Preserve existing chainId if present, only default for new configs
          // This prevents accidental chain ID reintroduction
          if (!cfg.mempool) {
            cfg.mempool = {
              minGasPrice: 1000000000,
              maxPerSender: 16,
              chainId: 31337, // Citrate devnet default - use network-specific ID in production
              maxSize: 10000,
              replacementFactor: 110,
              txExpirySecs: 3600,
              allowReplacement: true,
              requireValidSignature: true
            };
          } else if (cfg.mempool.chainId === undefined) {
            // Only set default if chainId specifically missing, don't override
            cfg.mempool.chainId = 31337;
          }
          setConfig(cfg);
        }
        if (stat) setStatus(stat);
        setBootnodes(bn || []);
        setPeers(ps || []);
      } catch (e: any) {
        setError(e?.message || String(e));
      } finally {
        setLoading(false);
      }
    };
    load();
  }, []);

  // Minimal helper used after mutations
  const loadConfig = async () => {
    try {
      const cfg = await nodeService.getConfig();
      if (cfg) {
        // Ensure consensus field exists with defaults
        if (!cfg.consensus) {
          cfg.consensus = {
            kParameter: 18,
            pruningWindow: 100000,
            blockTimeSeconds: 2,
            finalityDepth: 12
          };
        }
        // Ensure mempool field exists with defaults
        // SECURITY: Preserve existing chainId if present, only default for new configs
        if (!cfg.mempool) {
          cfg.mempool = {
            minGasPrice: 1000000000,
            maxPerSender: 16,
            chainId: 31337, // Citrate devnet default
            maxSize: 10000,
            replacementFactor: 110,
            txExpirySecs: 3600,
            allowReplacement: true,
            requireValidSignature: true
          };
        } else if (cfg.mempool.chainId === undefined) {
          cfg.mempool.chainId = 31337;
        }
      }
      setConfig(cfg);
    } catch {}
  };

  const disabled = !!status?.running;
  const networkOptions = useMemo(() => ['devnet', 'testnet', 'mainnet'], []);

  const reloadPeers = async () => {
    try { setPeers(await nodeService.getPeers()); } catch {}
  };
  const reloadBootnodes = async () => {
    try { setBootnodes(await nodeService.getBootnodes()); } catch {}
  };

  // Validate bootnode/peer format: "ip:port" or "peerId@ip:port"
  const validateBootnodePeer = (entry: string): ValidationResult => {
    if (!entry || entry.trim() === '') {
      return { isValid: false, error: 'Entry is required' };
    }

    const trimmed = entry.trim();

    // Check for peerId@ip:port format
    if (trimmed.includes('@')) {
      const parts = trimmed.split('@');
      if (parts.length !== 2) {
        return { isValid: false, error: 'Invalid format. Use peerId@ip:port' };
      }
      // Validate the ip:port part
      const ipPort = parts[1];
      return validateIpPort(ipPort);
    }

    // Otherwise validate as ip:port
    return validateIpPort(trimmed);
  };

  const validateIpPort = (entry: string): ValidationResult => {
    const parts = entry.split(':');
    if (parts.length !== 2) {
      return { isValid: false, error: 'Invalid format. Use ip:port' };
    }

    const [ip, portStr] = parts;

    // Validate IP
    const ipValidation = validateIPv4(ip);
    if (!ipValidation.isValid) {
      return ipValidation;
    }

    // Validate port
    const portValidation = validatePort(portStr);
    if (!portValidation.isValid) {
      return portValidation;
    }

    return { isValid: true };
  };

  const updateField = async (path: string, value: any) => {
    if (!config) return;
    
    // Special handling for network changes
    if (path === 'network') {
      setNetLoading(true);
      setError(null); 
      setSuccess(null);
      
      try {
        // Stop the current node
        if (status?.running) {
          await nodeService.stop();
        }
        
        // Configure based on selected network
        let chainId: number;
        let rpcPort: number;
        let wsPort: number;
        let p2pPort: number;
        let bootnodes: string[] = [];
        
        switch (value) {
          case 'testnet':
            chainId = 42069;
            rpcPort = 18545;
            wsPort = 18546;
            p2pPort = 30304;
            bootnodes = ['127.0.0.1:30303']; // Main testnet node
            break;
          case 'mainnet':
            chainId = 1; // Placeholder for mainnet
            rpcPort = 8545;
            wsPort = 8546;
            p2pPort = 30303;
            // Mainnet bootnodes would go here
            break;
          case 'devnet':
          default:
            chainId = 1337;
            rpcPort = 8545;
            wsPort = 8546;
            p2pPort = 30303;
            break;
        }
        
        // Update config with network-specific settings
        const updated = { 
          ...config,
          network: value,
          mempool: { ...config.mempool, chainId },
          rpcPort,
          wsPort,
          p2pPort,
          bootnodes,
          enableNetwork: value !== 'devnet',
          discovery: value !== 'devnet'
        } as any;
        
        setConfig(updated);
        
        // Apply configuration and restart
        await nodeService.updateConfig(updated);
        
        if (value !== 'devnet') {
          // For testnet/mainnet, clear chain data and restart
          await (nodeService as any).joinTestnet?.({
            chainId,
            rpcPort,
            wsPort,
            p2pPort,
            bootnodes,
            clearChain: true
          });
        } else {
          // For devnet, just restart with new config
          await nodeService.start();
        }
        
        // Refresh status after a short delay
        setTimeout(async () => {
          try { 
            setStatus(await nodeService.getStatus());
            await loadConfig();
          } catch {}
        }, 1500);
        
        setSuccess(`Switched to ${value} - Chain ID: ${chainId}`);
      } catch (e: any) {
        setError(`Failed to switch network: ${e?.message || String(e)}`);
        // Revert config on error
        await loadConfig();
      } finally {
        setNetLoading(false);
      }
      return;
    }
    
    // Default handling for other fields
    const updated = { ...config } as any;
    const parts = path.split('.');
    let curr = updated;
    for (let i = 0; i < parts.length - 1; i++) {
      curr[parts[i]] = { ...(curr[parts[i]] || {}) };
      curr = curr[parts[i]];
    }
    curr[parts[parts.length - 1]] = value;
    setConfig(updated);
  };

  const handleSave = async () => {
    if (!config) return;
    setSaving(true);
    setError(null);
    setSuccess(null);
    try {
      const msg = await nodeService.updateConfig(config);
      setSuccess(msg || 'Configuration updated');
    } catch (e: any) {
      setError(e?.message || String(e));
    } finally {
      setSaving(false);
    }
  };

  // Bootnode actions
  const handleAddBootnode = async () => {
    if (!newBootnode.trim()) return;
    setNetLoading(true);
    setError(null); setSuccess(null);
    try {
      await nodeService.addBootnode(newBootnode.trim());
      setNewBootnode('');
      await reloadBootnodes();
      setSuccess('Bootnode added');
    } catch (e: any) {
      setError(e?.message || String(e));
    } finally { setNetLoading(false); }
  };
  const handleRemoveBootnode = async (entry: string) => {
    setNetLoading(true);
    setError(null); setSuccess(null);
    try {
      await nodeService.removeBootnode(entry);
      await reloadBootnodes();
      setSuccess('Bootnode removed');
    } catch (e: any) {
      setError(e?.message || String(e));
    } finally { setNetLoading(false); }
  };
  const handleConnectBootnodes = async () => {
    setNetLoading(true);
    setError(null); setSuccess(null);
    try {
      const count = await nodeService.connectBootnodes();
      await reloadPeers();
      setSuccess(`Attempted connections to bootnodes (${count} initiated)`);
    } catch (e: any) { setError(e?.message || String(e)); } finally { setNetLoading(false); }
  };
  const handleAutoAddMyBootnodes = async () => {
    setNetLoading(true);
    setError(null); setSuccess(null);
    try {
      const added = await (nodeService as any).autoAddBootnodes?.();
      await reloadBootnodes();
      await reloadPeers();
      setSuccess(`Auto-added bootnodes for your IP: ${(added || []).join(', ')}`);
    } catch (e: any) {
      setError(e?.message || String(e));
    } finally { setNetLoading(false); }
  };

  const handleJoinTestnet = async () => {
    setNetLoading(true);
    setError(null); setSuccess(null);
    try {
      const args: any = {
        chainId: 42069,
        // Let backend choose a safe dataDir if the current one is relative or under src-tauri
        rpcPort: 18545,
        wsPort: 18546,
        p2pPort: 30304,
        restPort: 3001,
        bootnodes: bootnodes,
        clearChain: true,
        seedFrom: seedPath || undefined,
      };
      await (nodeService as any).joinTestnet?.(args);
      // Best-effort: refresh status after a short delay
      setTimeout(async () => {
        try { setStatus(await nodeService.getStatus()); } catch {}
      }, 1500);
      setSuccess('Joined testnet and started node. Connect bootnodes to begin syncing.');
    } catch (e: any) {
      setError(e?.message || String(e));
    } finally {
      setNetLoading(false);
    }
  };
  const handleConnectPeer = async () => {
    if (!connectEntry.trim()) return;
    setNetLoading(true);
    setError(null); setSuccess(null);
    try {
      await nodeService.connectPeer(connectEntry.trim());
      setConnectEntry('');
      await reloadPeers();
      setSuccess('Peer connection initiated');
    } catch (e: any) { setError(e?.message || String(e)); } finally { setNetLoading(false); }
  };
  const handleDisconnectPeer = async (peerId: string) => {
    setNetLoading(true);
    setError(null); setSuccess(null);
    try {
      await nodeService.disconnectPeer(peerId);
      await reloadPeers();
      setSuccess('Peer disconnected');
    } catch (e: any) { setError(e?.message || String(e)); } finally { setNetLoading(false); }
  };

  const importScaffold = async (host: string) => {
    const ports = [30303, 30304, 30305, 30306, 30307];
    setNetLoading(true);
    setError(null); setSuccess(null);
    try {
      for (const p of ports) {
        await nodeService.addBootnode(`${host}:${p}`);
      }
      await reloadBootnodes();
      setSuccess(`Imported ${ports.length} bootnodes for host ${host}`);
    } catch (e: any) {
      setError(e?.message || String(e));
    } finally {
      setNetLoading(false);
    }
  };

  if (loading) {
    return (
      <div className="settings" style={{ padding: '2rem' }}>
        <p>Loading settings...</p>
      </div>
    );
  }

  if (!config) {
    return (
      <div className="settings" style={{ padding: '2rem' }}>
        <p>Unable to load configuration.</p>
      </div>
    );
  }

  return (
    <div className="settings">
      <h2>Settings</h2>
      {status?.running && (
        <div className="alert alert-warning">
          Node is running. Stop it to edit settings.
        </div>
      )}
      {error && <div className="alert alert-error">{error}</div>}
      {success && <div className="alert alert-success">{success}</div>}

      {/* Appearance Section */}
      <div className="settings-section">
        <h3>Appearance</h3>
        <div className="theme-selector">
          <label>Theme</label>
          <div className="theme-buttons">
            <button
              type="button"
              className={`theme-button ${themeMode === 'light' ? 'active' : ''}`}
              onClick={() => setThemeMode('light')}
              title="Light theme"
            >
              <Sun size={20} />
              <span>Light</span>
            </button>
            <button
              type="button"
              className={`theme-button ${themeMode === 'dark' ? 'active' : ''}`}
              onClick={() => setThemeMode('dark')}
              title="Dark theme"
            >
              <Moon size={20} />
              <span>Dark</span>
            </button>
            <button
              type="button"
              className={`theme-button ${themeMode === 'system' ? 'active' : ''}`}
              onClick={() => setThemeMode('system')}
              title="Follow system preference"
            >
              <Monitor size={20} />
              <span>System</span>
            </button>
          </div>
        </div>
      </div>

      <div className="settings-section">
        <h3>Node Configuration</h3>
        <div className="form-grid">
          <label>
            <span>Data Directory</span>
            <input type="text" value={config.dataDir}
              disabled={disabled}
              onChange={e => updateField('dataDir', e.target.value)} />
          </label>
          <label>
            <span>Network</span>
            <select value={config.network}
              disabled={disabled || netLoading}
              onChange={e => updateField('network', e.target.value)}>
              {networkOptions.map(opt => (
                <option key={opt} value={opt}>{opt}</option>
              ))}
            </select>
          </label>
          <label>
            <span>Reward Address</span>
            <input type="text" value={config.rewardAddress || ''}
              disabled={disabled}
              onChange={e => updateField('rewardAddress', e.target.value)} />
          </label>
          <label>
            <span>RPC Port</span>
            <input type="number" value={config.rpcPort}
              disabled={disabled}
              onChange={e => updateField('rpcPort', Number(e.target.value))} />
          </label>
          <label>
            <span>WS Port</span>
            <input type="number" value={config.wsPort}
              disabled={disabled}
              onChange={e => updateField('wsPort', Number(e.target.value))} />
          </label>
          <label>
            <span>P2P Port</span>
            <input type="number" value={config.p2pPort}
              disabled={disabled}
              onChange={e => updateField('p2pPort', Number(e.target.value))} />
          </label>
          <label>
            <span>REST Port</span>
            <input type="number" value={config.restPort}
              disabled={disabled}
              onChange={e => updateField('restPort', Number(e.target.value))} />
          </label>
          <label>
            <span>Max Peers</span>
            <input type="number" value={config.maxPeers}
              disabled={disabled}
              onChange={e => updateField('maxPeers', Number(e.target.value))} />
          </label>
          <label className="full">
            <span>Bootnodes (one per line)</span>
            <textarea value={(config.bootnodes || []).join('\n')}
              disabled={disabled}
              onChange={e => updateField('bootnodes', e.target.value.split('\n').filter(Boolean))} />
          </label>
          <label>
            <span>Enable Network</span>
            <input type="checkbox" checked={!!config.enableNetwork}
              disabled={disabled}
              onChange={e => updateField('enableNetwork', e.target.checked)} />
          </label>
          <label>
            <span>Peer Discovery</span>
            <input type="checkbox" checked={!!config.discovery}
              disabled={disabled}
              onChange={e => updateField('discovery', e.target.checked)} />
          </label>
        </div>
      </div>

      <div className="settings-section">
        <h3>Consensus</h3>
        <div className="form-grid">
          <label>
            <span>k-Parameter</span>
            <input type="number" value={config.consensus.kParameter}
              disabled={disabled}
              onChange={e => updateField('consensus.kParameter', Number(e.target.value))} />
          </label>
          <label>
            <span>Pruning Window</span>
            <input type="number" value={config.consensus.pruningWindow}
              disabled={disabled}
              onChange={e => updateField('consensus.pruningWindow', Number(e.target.value))} />
          </label>
          <label>
            <span>Block Time (sec)</span>
            <input type="number" value={config.consensus.blockTimeSeconds}
              disabled={disabled}
              onChange={e => updateField('consensus.blockTimeSeconds', Number(e.target.value))} />
          </label>
          <label>
            <span>Finality Depth</span>
            <input type="number" value={config.consensus.finalityDepth}
              disabled={disabled}
              onChange={e => updateField('consensus.finalityDepth', Number(e.target.value))} />
          </label>
        </div>
      </div>

      <div className="settings-section">
        <h3>Network &amp; Peering</h3>
        <div className="oneclick">
          <div className="row">
            <button className="btn btn-primary" disabled={netLoading} onClick={handleJoinTestnet}>
              {netLoading ? 'Joining Testnet…' : 'Join Testnet (one-click)'}
            </button>
          </div>
          <div className="row">
            <input type="text" placeholder="Optional: seed from core chain dir (e.g., /abs/path/.citrate-testnet)"
              value={seedPath} onChange={e => setSeedPath(e.target.value)} disabled={netLoading} />
          </div>
          <div className="hint muted">One-click configures testnet, resets GUI chain data, optionally seeds from your core chain dir, starts the node, then you can connect bootnodes.</div>
          <div className="row" style={{ marginTop: '0.5rem' }}>
            <button className="btn btn-secondary" disabled={netLoading} onClick={handleAutoAddMyBootnodes}>
              {netLoading ? 'Generating…' : 'Add My Bootnodes (Auto)'}
            </button>
          </div>
        </div>
        {config.enableNetwork ? (
          <>
            <div className="form-grid">
              <label className="full">
                <span>Bootnodes</span>
                <div className="bootnodes">
                  {(bootnodes || []).length === 0 && (
                    <div className="muted">No bootnodes configured</div>
                  )}
                  {(bootnodes || []).map((bn) => (
                    <div className="bootnode" key={bn}>
                      <span className="mono">{bn}</span>
                      <button className="btn btn-secondary btn-sm" disabled={disabled || netLoading}
                        onClick={() => handleRemoveBootnode(bn)}>
                        Remove
                      </button>
                    </div>
                  ))}
                  <div className="bootnode-add">
                    <div style={{ flex: 1 }}>
                      <input
                        type="text"
                        placeholder="peerId@ip:port or ip:port"
                        value={newBootnode}
                        disabled={disabled}
                        onChange={e => {
                          const value = e.target.value;
                          setNewBootnode(value);

                          // Validate in real-time
                          if (value.trim()) {
                            const validation = validateBootnodePeer(value);
                            setBootnodeError(validation.isValid ? '' : validation.error || '');
                          } else {
                            setBootnodeError('');
                          }
                        }}
                        className={bootnodeError ? 'input-error' : ''}
                      />
                      {bootnodeError && <div className="error-text">{bootnodeError}</div>}
                    </div>
                    <button className="btn btn-primary btn-sm" onClick={handleAddBootnode} disabled={disabled || netLoading || !newBootnode.trim() || !!bootnodeError}>
                      Add Bootnode
                    </button>
                  </div>
                </div>
              </label>
            </div>
            <div className="actions" style={{ marginTop: '0.5rem' }}>
              <button className="btn btn-primary" onClick={handleConnectBootnodes} disabled={netLoading || !status?.running}>
                {netLoading ? 'Connecting...' : 'Connect Bootnodes Now'}
              </button>
            </div>
            {config.network === 'devnet' && (
              <div className="import-scaffold">
                <div className="hint">Devnet detected. Quickly import local scaffold bootnodes:</div>
                <div className="import-actions">
                  <button className="btn btn-secondary btn-sm" disabled={disabled || netLoading} onClick={() => importScaffold('127.0.0.1')}>
                    Import 127.0.0.1:30303–30307
                  </button>
                  <div className="inline">
                    <input type="text" value={importHost} placeholder="192.168.1.50" onChange={e => setImportHost(e.target.value)} disabled={disabled} />
                    <button className="btn btn-secondary btn-sm" disabled={disabled || netLoading || !importHost.trim()} onClick={() => importScaffold(importHost.trim())}>
                      Import Using Host IP
                    </button>
                  </div>
                </div>
                {disabled && <div className="muted" style={{ marginTop: '0.25rem' }}>Stop the node to modify bootnodes.</div>}
              </div>
            )}
            <div className="peers" style={{ marginTop: '1rem' }}>
              <div className="peers-header">
                <span>Connected Peers ({peers.length})</span>
                <div className="peer-actions">
                  <div style={{ flex: 1 }}>
                    <input
                      type="text"
                      placeholder="peerId@ip:port or ip:port"
                      value={connectEntry}
                      onChange={e => {
                        const value = e.target.value;
                        setConnectEntry(value);

                        // Validate in real-time
                        if (value.trim()) {
                          const validation = validateBootnodePeer(value);
                          setPeerError(validation.isValid ? '' : validation.error || '');
                        } else {
                          setPeerError('');
                        }
                      }}
                      className={peerError ? 'input-error' : ''}
                    />
                    {peerError && <div className="error-text">{peerError}</div>}
                  </div>
                  <button className="btn btn-secondary btn-sm" onClick={handleConnectPeer} disabled={netLoading || !status?.running || !connectEntry.trim() || !!peerError}>
                    Connect Peer
                  </button>
                  <button className="btn btn-secondary btn-sm" onClick={reloadPeers} disabled={netLoading}>Refresh</button>
                </div>
              </div>
              <div className="peer-list">
                {peers.length === 0 && <div className="muted">No connected peers</div>}
                {peers.map(p => (
                  <div className="peer" key={p.id + p.addr}>
                    <span className="mono">{p.id}</span>
                    <span className="mono">{p.addr}</span>
                    <span className={`badge ${p.direction === 'inbound' ? 'badge-blue' : 'badge-purple'}`}>{p.direction}</span>
                    <span className="badge">{p.state}</span>
                    <span className="muted">score {p.score} • last {p.lastSeenSecs}s</span>
                    <button className="btn btn-secondary btn-sm" onClick={() => handleDisconnectPeer(p.id)} disabled={netLoading || !status?.running}>Disconnect</button>
                  </div>
                ))}
              </div>
            </div>
          </>
        ) : (
          <div className="alert alert-warning">Networking disabled — enable to use peering.</div>
        )}
      </div>

      <div className="settings-section">
        <h3>Mempool</h3>
        <div className="form-grid">
          <label>
            <span>Min Gas Price (wei)</span>
            <input type="number" value={config.mempool.minGasPrice}
              disabled={disabled}
              onChange={e => updateField('mempool.minGasPrice', Number(e.target.value))} />
          </label>
          <label>
            <span>Max Per Sender</span>
            <input type="number" value={config.mempool.maxPerSender}
              disabled={disabled}
              onChange={e => updateField('mempool.maxPerSender', Number(e.target.value))} />
          </label>
          <label>
            <span>Chain ID</span>
            <input type="number" value={config.mempool.chainId}
              disabled={disabled}
              onChange={e => updateField('mempool.chainId', Number(e.target.value))} />
          </label>
          <label>
            <span>Pool Max Size</span>
            <input type="number" value={config.mempool.maxSize}
              disabled={disabled}
              onChange={e => updateField('mempool.maxSize', Number(e.target.value))} />
          </label>
          <label>
            <span>Replacement Factor (%)</span>
            <input type="number" value={config.mempool.replacementFactor}
              disabled={disabled}
              onChange={e => updateField('mempool.replacementFactor', Number(e.target.value))} />
          </label>
          <label>
            <span>TX Expiry (secs)</span>
            <input type="number" value={config.mempool.txExpirySecs}
              disabled={disabled}
              onChange={e => updateField('mempool.txExpirySecs', Number(e.target.value))} />
          </label>
          <label>
            <span>Allow Replacement</span>
            <input type="checkbox" checked={config.mempool.allowReplacement}
              disabled={disabled}
              onChange={e => updateField('mempool.allowReplacement', e.target.checked)} />
          </label>
          <label>
            <span>Require Valid Signature</span>
            <input type="checkbox" checked={config.mempool.requireValidSignature}
              disabled={disabled}
              onChange={e => updateField('mempool.requireValidSignature', e.target.checked)} />
          </label>
        </div>
      </div>

      <div className="actions">
        <button className="btn btn-primary" onClick={handleSave} disabled={disabled || saving}>
          {saving ? 'Saving...' : 'Save Configuration'}
        </button>
      </div>

      <style jsx>{`
        .settings { padding: 2rem; }
        .settings h2 { margin: 0 0 1rem 0; font-size: 1.5rem; font-weight: 600; }
        .settings-section { background: white; border-radius: 1rem; padding: 1.25rem; margin-bottom: 1rem; box-shadow: 0 2px 4px rgba(0,0,0,0.05); }
        .settings-section h3 { margin: 0 0 1rem 0; font-size: 1.125rem; }
        .form-grid { display: grid; grid-template-columns: repeat(auto-fit,minmax(220px,1fr)); gap: 1rem; }
        label { display: flex; flex-direction: column; gap: 0.5rem; font-size: 0.9rem; color: #374151; }
        label.full { grid-column: 1 / -1; }
        input, textarea, select { border: 1px solid #e5e7eb; border-radius: 0.5rem; padding: 0.6rem 0.75rem; font-size: 0.95rem; }
        .input-error { border-color: #ef4444 !important; border-width: 2px !important; }
        .error-text { color: #ef4444; font-size: 0.875rem; margin-top: 0.25rem; }
        textarea { min-height: 80px; resize: vertical; }
        .actions { display: flex; gap: 1rem; }
        .btn { display: inline-flex; align-items: center; gap: 0.5rem; padding: 0.75rem 1.25rem; border: none; border-radius: 0.5rem; cursor: pointer; }
        .btn-primary { background: linear-gradient(135deg,#667eea 0%,#764ba2 100%); color: #fff; }
        .btn-sm { padding: 0.4rem 0.6rem; font-size: 0.85rem; }
        .alert { padding: 0.75rem 1rem; border-radius: 0.5rem; margin-bottom: 1rem; }
        .alert-warning { background: #fffbeb; color: #92400e; border: 1px solid #fef3c7; }
        .alert-error { background: #fee2e2; color: #991b1b; border: 1px solid #fecaca; }
        .alert-success { background: #ecfdf5; color: #065f46; border: 1px solid #a7f3d0; }
        .bootnodes { display: flex; flex-direction: column; gap: 0.5rem; }
        .bootnode { display: flex; gap: 0.5rem; align-items: center; justify-content: space-between; background: #f9fafb; padding: 0.5rem 0.75rem; border-radius: 0.5rem; }
        .bootnode-add { display: flex; gap: 0.5rem; align-items: center; }
        .peers-header { display: flex; justify-content: space-between; align-items: center; margin-bottom: 0.5rem; }
        .peer-actions { display: flex; gap: 0.5rem; align-items: center; }
        .peer-list { display: flex; flex-direction: column; gap: 0.5rem; }
        .peer { display: grid; grid-template-columns: 1fr 1fr auto auto auto auto; gap: 0.5rem; align-items: center; padding: 0.5rem; background: #f9fafb; border-radius: 0.5rem; }
        .badge { padding: 0.25rem 0.5rem; border-radius: 0.25rem; font-size: 0.75rem; font-weight: 600; }
        .badge-blue { background: #dbeafe; color: #1e40af; }
        .badge-purple { background: #ede9fe; color: #5b21b6; }
        .mono { font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, "Liberation Mono", "Courier New", monospace; font-size: 0.875rem; }
        .muted { color: #6b7280; font-size: 0.875rem; }
        .import-scaffold { margin-top: 0.75rem; background: #f9fafb; border-radius: 0.5rem; padding: 0.75rem; }
        .import-actions { display: flex; gap: 0.5rem; align-items: center; flex-wrap: wrap; }
        .import-actions .inline { display: flex; gap: 0.5rem; align-items: center; }

        /* Theme Selector Styles */
        .theme-selector { display: flex; flex-direction: column; gap: 0.5rem; }
        .theme-selector > label { font-weight: 500; color: var(--text-primary); }
        .theme-buttons { display: flex; gap: 0.75rem; }
        .theme-button {
          display: flex;
          align-items: center;
          gap: 0.5rem;
          padding: 0.75rem 1rem;
          border: 2px solid var(--border-primary);
          border-radius: 0.5rem;
          background: var(--bg-primary);
          color: var(--text-primary);
          cursor: pointer;
          transition: all 200ms ease;
          font-weight: 500;
        }
        .theme-button:hover {
          border-color: var(--brand-primary);
          background: var(--bg-secondary);
        }
        .theme-button.active {
          border-color: var(--brand-primary);
          background: var(--brand-primary);
          color: white;
        }
        .theme-button svg {
          flex-shrink: 0;
        }
      `}</style>
    </div>
  );
};

export default Settings;
