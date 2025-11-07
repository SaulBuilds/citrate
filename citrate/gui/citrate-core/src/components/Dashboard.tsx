import React, { useState, useEffect } from 'react';
import { nodeService } from '../services/tauri';
import { NodeStatus } from '../types';
import {
  Server,
  Users,
  Box,
  Wifi,
  WifiOff,
  Play,
  Square,
  Hash,
  Activity,
  Pickaxe
} from 'lucide-react';
import { Network as NetworkIcon } from 'lucide-react';
import { SkeletonStats, SkeletonTable } from './Skeleton';

export const Dashboard: React.FC = () => {
  const [nodeStatus, setNodeStatus] = useState<NodeStatus | null>(null);
  const [prevStatus, setPrevStatus] = useState<{ ts: number; height: number } | null>(null);
  const [loading, setLoading] = useState(false);
  const [initialLoading, setInitialLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [successMessage, setSuccessMessage] = useState<string | null>(null);
  const [txOverview, setTxOverview] = useState<{ pending: number; last_block: number }>({ pending: 0, last_block: 0 });
  const [mempool, setMempool] = useState<Array<{hash: string; from: string; to?: string; value: string; nonce: number}>>([]);
  const [rewardAddress, setRewardAddress] = useState<string | null>(null);

  useEffect(() => {
    // Initial status fetch
    fetchStatus();

    // Subscribe to status updates
    const unsubscribe = nodeService.onStatusUpdate((status) => {
      setNodeStatus(prev => {
        if (prev) {
          setPrevStatus({ ts: Date.now(), height: prev.blockHeight });
        }
        return status;
      });
    });

    // Fallback polling in case events are missed
    const poll = setInterval(fetchStatus, 1000);

    // Cleanup
    return () => {
      unsubscribe.then((fn: () => void) => fn());
      clearInterval(poll);
    };
  }, []);

  const fetchStatus = async () => {
    try {
      const status = await nodeService.getStatus();
      setNodeStatus(status);
      try {
        setTxOverview(await nodeService.getTxOverview());
        const mp = await (nodeService as any).getMempoolPending?.(10);
        if (mp) setMempool(mp);
        const addr = await (nodeService as any).getRewardAddress?.();
        if (addr) setRewardAddress(addr);
      } catch {}
    } catch (err) {
      console.error('Failed to fetch node status:', err);
    } finally {
      // Mark initial loading as complete
      if (initialLoading) {
        setInitialLoading(false);
      }
    }
  };

  const handleStartNode = async () => {
    console.log('Start node button clicked');
    setLoading(true);
    setError(null);
    setSuccessMessage(null);
    try {
      console.log('Calling nodeService.start()');
      const result = await nodeService.start();
      console.log('Node start result:', result);

      // Show success message
      setSuccessMessage(result);

      // Clear success message after 5 seconds
      setTimeout(() => setSuccessMessage(null), 5000);

      await fetchStatus();
    } catch (err: any) {
      console.error('Failed to start node:', err);
      setError(err.toString());
    } finally {
      setLoading(false);
    }
  };

  const handleStopNode = async () => {
    setLoading(true);
    setError(null);
    try {
      await nodeService.stop();
      await fetchStatus();
    } catch (err: any) {
      setError(err.toString());
    } finally {
      setLoading(false);
    }
  };

  const formatUptime = (seconds: number) => {
    const hours = Math.floor(seconds / 3600);
    const minutes = Math.floor((seconds % 3600) / 60);
    return `${hours}h ${minutes}m`;
  };

  const formatHash = (hash?: string | null) => {
    if (!hash) return '—';
    return `${hash.slice(0, 8)}...${hash.slice(-6)}`;
  };

  const formatTime = (ms?: number | null) => {
    if (!ms) return '—';
    const d = new Date(ms);
    return d.toLocaleTimeString();
  };

  const productionRate = (() => {
    if (!nodeStatus || !prevStatus) return '—';
    const dh = nodeStatus.blockHeight - prevStatus.height;
    const dtSec = Math.max(1, (Date.now() - prevStatus.ts) / 1000);
    const perMin = (dh / dtSec) * 60;
    return perMin.toFixed(2) + ' blk/min';
  })();

  return (
    <div className="dashboard">
      <header className="dashboard-header">
        <h1>Dashboard</h1>
        <div className="node-controls" role="group" aria-label="Node controls">
          {nodeStatus?.running ? (
            <button
              onClick={handleStopNode}
              disabled={loading}
              className="btn btn-danger"
              aria-label="Stop node"
              aria-busy={loading}
              style={{ opacity: loading ? 0.6 : 1, cursor: loading ? 'wait' : 'pointer' }}
            >
              {loading ? '⏳' : <Square size={16} aria-hidden="true" />}
              {loading ? 'Stopping...' : 'Stop Node'}
            </button>
          ) : (
            <button
              onClick={handleStartNode}
              disabled={loading}
              className="btn btn-primary"
              aria-label="Start node"
              aria-busy={loading}
              style={{ opacity: loading ? 0.6 : 1, cursor: loading ? 'wait' : 'pointer' }}
            >
              {loading ? '⏳' : <Play size={16} aria-hidden="true" />}
              {loading ? 'Connecting...' : 'Start Node'}
            </button>
          )}
        </div>
      </header>
      {/* Mempool Snapshot */}
      <section className="card" style={{ marginTop: '1rem' }} aria-label="Mempool transactions">
        <div className="card-header"><h3>Mempool {!initialLoading && `(top ${Math.min(10, mempool.length)} of ${txOverview.pending})`}</h3></div>
        <div className="card-body">
          {initialLoading ? (
            <SkeletonTable rows={5} columns={4} />
          ) : (
            <>
              {mempool.length === 0 && <div className="muted">No pending transactions</div>}
              {mempool.length > 0 && (
                <div className="mono" style={{ display: 'grid', gridTemplateColumns: 'auto auto auto auto', gap: '0.5rem' }}>
                  <div>Hash</div><div>From</div><div>To</div><div>Value</div>
                  {mempool.map((t, i) => (
                    <React.Fragment key={t.hash + i}>
                      <div>{t.hash.slice(0, 10)}…</div>
                      <div>{t.from.slice(0, 10)}…</div>
                      <div>{t.to ? t.to.slice(0, 10) + '…' : '—'}</div>
                      <div>{Number(t.value || '0').toLocaleString()}</div>
                    </React.Fragment>
                  ))}
                </div>
              )}
            </>
          )}
        </div>
      </section>

      {error && (
        <div className="alert alert-error" role="alert" aria-live="assertive">
          {error}
        </div>
      )}

      {successMessage && (
        <div className="alert alert-success" role="alert" aria-live="polite" style={{
          background: '#10b981',
          color: 'white',
          padding: '1rem',
          borderRadius: '0.5rem',
          marginBottom: '1rem'
        }}>
          ✓ {successMessage}
        </div>
      )}

      {initialLoading ? (
        <SkeletonStats count={8} />
      ) : (
        <section className="stats-grid" role="region" aria-label="Node statistics">
          <div className="stat-card">
          <div className="stat-icon">
            {nodeStatus?.running ? (
              <Wifi className="text-green" />
            ) : (
              <WifiOff className="text-gray" />
            )}
          </div>
          <div className="stat-content">
            <h3>Node Status</h3>
            <p className={nodeStatus?.running ? 'text-green' : 'text-gray'}>
              {nodeStatus?.running ? 'Online' : 'Offline'}
            </p>
            {nodeStatus?.running && (
              <span className="stat-detail">
                Uptime: {formatUptime(nodeStatus.uptime)}
              </span>
            )}
          </div>
        </div>

        <div className="stat-card">
          <div className="stat-icon">
            <Box className="text-blue" />
          </div>
          <div className="stat-content">
            <h3>Block Height</h3>
            <p className="stat-value">
              {nodeStatus?.blockHeight?.toLocaleString() || '0'}
            </p>
            {nodeStatus?.syncing && (
              <span className="stat-detail text-yellow">Syncing...</span>
            )}
          </div>
        </div>

        <div className="stat-card">
          <div className="stat-icon">
            <Hash className="text-purple" />
          </div>
          <div className="stat-content">
            <h3>Last Block</h3>
            <p className="stat-value mono">{formatHash(nodeStatus?.lastBlockHash || null)}</p>
            <span className="stat-detail">{formatTime(nodeStatus?.lastBlockTimestamp || null)}</span>
          </div>
        </div>

        <div className="stat-card">
          <div className="stat-icon">
            <Activity className="text-green" />
          </div>
          <div className="stat-content">
            <h3>Production</h3>
            <p className="stat-value">{productionRate}</p>
            <span className="stat-detail">Avg over last tick</span>
          </div>
        </div>

        <div className="stat-card">
          <div className="stat-icon">
            <Users className="text-purple" />
          </div>
          <div className="stat-content">
            <h3>Peers</h3>
            <p className="stat-value">{nodeStatus?.peerCount || 0}</p>
            <span className="stat-detail">Connected</span>
          </div>
        </div>

        <div className="stat-card">
          <div className="stat-icon">
            <Server className="text-orange" />
          </div>
          <div className="stat-content">
            <h3>Network</h3>
            <p className="stat-value">{nodeStatus?.networkId || 'Unknown'}</p>
            <span className="stat-detail">v{nodeStatus?.version || '0.0.0'}</span>
          </div>
        </div>

        <div className="stat-card">
          <div className="stat-icon">
            <NetworkIcon className="text-blue" />
          </div>
          <div className="stat-content">
            <h3>DAG Tips</h3>
            <p className="stat-value">{nodeStatus?.dagTips ?? 0}</p>
            <span className="stat-detail">Current tips</span>
          </div>
        </div>

        <div className="stat-card">
          <div className="stat-icon">
            <Activity className="text-green" />
          </div>
          <div className="stat-content">
            <h3>Txs</h3>
            <p className="stat-value">{txOverview.pending} pending</p>
            <span className="stat-detail">{txOverview.last_block} in last block</span>
          </div>
        </div>

        <div className="stat-card">
          <div className="stat-icon">
            <Pickaxe className={rewardAddress && nodeStatus?.running ? "text-yellow" : "text-gray"} />
          </div>
          <div className="stat-content">
            <h3>Mining Status</h3>
            <p className={`stat-value ${rewardAddress && nodeStatus?.running ? 'text-green' : 'text-gray'}`}>
              {rewardAddress && nodeStatus?.running ? '⛏️ Active' : '⏸️ Inactive'}
            </p>
            {rewardAddress && (
              <span className="stat-detail mono" title={rewardAddress}>
                {rewardAddress.slice(0, 10)}...{rewardAddress.slice(-8)}
              </span>
            )}
            {!rewardAddress && (
              <span className="stat-detail">No reward address set</span>
            )}
          </div>
        </div>
        </section>
      )}

      <style jsx>{`
        .dashboard {
          padding: 2rem;
        }

        .dashboard-header {
          display: flex;
          justify-content: space-between;
          align-items: center;
          margin-bottom: 2rem;
        }

        .dashboard-header h1 {
          margin: 0;
          font-size: 2rem;
          font-weight: 600;
        }

        .node-controls {
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

        .btn:disabled {
          opacity: 0.5;
          cursor: not-allowed;
        }

        .btn-primary {
          background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
          color: white;
        }

        .btn-primary:hover:not(:disabled) {
          transform: translateY(-2px);
          box-shadow: 0 10px 20px rgba(102, 126, 234, 0.4);
        }

        .btn-danger {
          background: linear-gradient(135deg, #f43f5e 0%, #ef4444 100%);
          color: white;
        }

        .btn-danger:hover:not(:disabled) {
          transform: translateY(-2px);
          box-shadow: 0 10px 20px rgba(244, 63, 94, 0.4);
        }

        .alert {
          padding: 1rem;
          border-radius: 0.5rem;
          margin-bottom: 1.5rem;
        }

        .alert-error {
          background: #fee;
          color: #c00;
          border: 1px solid #fcc;
        }

        .stats-grid {
          display: grid;
          grid-template-columns: repeat(auto-fit, minmax(250px, 1fr));
          gap: 1.5rem;
        }

        .stat-card {
          background: var(--bg-primary);
          border-radius: 1rem;
          padding: 1.5rem;
          display: flex;
          align-items: flex-start;
          gap: 1rem;
          box-shadow: 0 4px 6px rgba(0, 0, 0, 0.1);
          transition: transform 0.2s;
        }

        .stat-card:hover {
          transform: translateY(-4px);
          box-shadow: 0 8px 16px rgba(0, 0, 0, 0.15);
        }

        .stat-icon {
          padding: 0.75rem;
          border-radius: 0.75rem;
          background: var(--bg-tertiary);
        }

        .stat-content h3 {
          margin: 0 0 0.5rem 0;
          font-size: 0.875rem;
          font-weight: 500;
          color: var(--text-secondary);
        }

        .stat-value {
          margin: 0;
          font-size: 1.5rem;
          font-weight: 600;
          color: var(--text-primary);
        }

        .stat-detail {
          font-size: 0.75rem;
          color: var(--text-muted);
        }

        .text-green { color: #10b981; }
        .text-gray { color: var(--text-secondary); }
        .text-blue { color: #3b82f6; }
        .text-purple { color: #8b5cf6; }
        .text-orange { color: #f97316; }
        .text-yellow { color: #eab308; }
        .card { background: var(--bg-primary); border: 1px solid var(--border-primary); border-radius: 0.5rem; }
        .card-header { padding: 1rem 1.5rem; border-bottom: 1px solid var(--border-primary); }
        .card-body { padding: 1rem 1.5rem; }
        .mono { font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, 'Liberation Mono', 'Courier New', monospace; font-size: 0.85rem; }
        .muted { color: var(--text-muted); }
      `}</style>
    </div>
  );
};
