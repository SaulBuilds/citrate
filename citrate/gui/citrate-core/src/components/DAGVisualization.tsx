import React, { useState, useEffect, useCallback, useRef, useMemo } from 'react';
import { dagService } from '../services/tauri';
import { DAGData, DAGNode } from '../types';
import ForceGraph2D, { ForceGraphMethods } from 'react-force-graph-2d';
import {
  Network,
  Info,
  RefreshCw,
  Layers,
  GitBranch,
  Box,
  Activity,
  Clock,
  Hash,
  Grid3X3,
  Share2
} from 'lucide-react';

// Graph data types for force-graph
interface GraphNode {
  id: string;
  hash: string;
  height: number;
  isBlue: boolean;
  isTip: boolean | undefined;
  blueScore: number;
  transactions: number;
  x?: number;
  y?: number;
}

interface GraphLink {
  source: string;
  target: string;
  isSelected: boolean;
}

export const DAGVisualization: React.FC = () => {
  const [dagData, setDagData] = useState<DAGData | null>(null);
  const [selectedNode, setSelectedNode] = useState<DAGNode | null>(null);
  const [blockDetails, setBlockDetails] = useState<any | null>(null);
  const [showTxs, setShowTxs] = useState(false);
  const [loading, setLoading] = useState(false);
  const [autoRefresh, setAutoRefresh] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [viewMode, setViewMode] = useState<'table' | 'graph'>('graph');
  const graphRef = useRef<ForceGraphMethods>(null);
  const containerRef = useRef<HTMLDivElement>(null);
  const [dimensions, setDimensions] = useState({ width: 800, height: 600 });

  // Convert DAG data to force-graph format
  const graphData = useMemo(() => {
    if (!dagData || !dagData.nodes.length) {
      return { nodes: [], links: [] };
    }

    const nodes: GraphNode[] = dagData.nodes.map(node => ({
      id: node.hash || node.id,
      hash: node.hash,
      height: node.height,
      isBlue: node.isBlue,
      isTip: node.isTip,
      blueScore: node.blueScore,
      transactions: node.transactions || 0,
    }));

    const links: GraphLink[] = dagData.links?.map(link => ({
      source: typeof link.source === 'string' ? link.source : (link.source as any).id,
      target: typeof link.target === 'string' ? link.target : (link.target as any).id,
      isSelected: link.isSelected,
    })) || [];

    return { nodes, links };
  }, [dagData]);

  // Handle container resize
  useEffect(() => {
    const updateDimensions = () => {
      if (containerRef.current) {
        setDimensions({
          width: containerRef.current.offsetWidth,
          height: containerRef.current.offsetHeight - 50,
        });
      }
    };

    updateDimensions();
    window.addEventListener('resize', updateDimensions);
    return () => window.removeEventListener('resize', updateDimensions);
  }, []);

  // Node canvas rendering
  const nodeCanvasObject = useCallback((node: any, ctx: CanvasRenderingContext2D, globalScale: number) => {
    const size = node.isTip ? 12 : 8;
    const fontSize = 10 / globalScale;

    // Draw node circle
    ctx.beginPath();
    ctx.arc(node.x || 0, node.y || 0, size, 0, 2 * Math.PI);

    // Color based on status
    if (node.isTip) {
      ctx.fillStyle = '#a78bfa'; // Purple for tips
    } else if (node.isBlue) {
      ctx.fillStyle = '#3b82f6'; // Blue
    } else {
      ctx.fillStyle = '#ef4444'; // Red
    }
    ctx.fill();

    // Draw border
    ctx.strokeStyle = node.isTip ? '#7c3aed' : '#fff';
    ctx.lineWidth = 2;
    ctx.stroke();

    // Draw height label
    ctx.font = `${fontSize}px Sans-Serif`;
    ctx.textAlign = 'center';
    ctx.textBaseline = 'top';
    ctx.fillStyle = '#374151';
    ctx.fillText(`H:${node.height}`, node.x || 0, (node.y || 0) + size + 2);
  }, []);

  // Link rendering
  const linkCanvasObject = useCallback((link: any, ctx: CanvasRenderingContext2D) => {
    const start = link.source;
    const end = link.target;

    if (!start || !end || typeof start.x === 'undefined') return;

    ctx.beginPath();
    ctx.moveTo(start.x, start.y);
    ctx.lineTo(end.x, end.y);
    ctx.strokeStyle = link.isSelected ? '#667eea' : '#9ca3af';
    ctx.lineWidth = link.isSelected ? 2 : 1;
    ctx.stroke();

    // Draw arrow
    const angle = Math.atan2(end.y - start.y, end.x - start.x);
    const arrowLength = 8;
    ctx.beginPath();
    ctx.moveTo(end.x, end.y);
    ctx.lineTo(
      end.x - arrowLength * Math.cos(angle - Math.PI / 6),
      end.y - arrowLength * Math.sin(angle - Math.PI / 6)
    );
    ctx.lineTo(
      end.x - arrowLength * Math.cos(angle + Math.PI / 6),
      end.y - arrowLength * Math.sin(angle + Math.PI / 6)
    );
    ctx.closePath();
    ctx.fillStyle = link.isSelected ? '#667eea' : '#9ca3af';
    ctx.fill();
  }, []);

  const handleGraphNodeClick = useCallback((node: any) => {
    if (dagData) {
      const dagNode = dagData.nodes.find(n => n.hash === node.hash || n.id === node.id);
      if (dagNode) {
        setSelectedNode(dagNode);
        dagService.getBlockDetails(dagNode.hash)
          .then(setBlockDetails)
          .catch(() => setBlockDetails(null));
      }
    }
  }, [dagData]);

  useEffect(() => {
    loadDAGData();
    
    if (autoRefresh) {
      const interval = setInterval(loadDAGData, 5000);
      return () => clearInterval(interval);
    }
  }, [autoRefresh]);

  const loadDAGData = async () => {
    try {
      setLoading(true);
      setError(null);

      try {
        // Try to get DAG data from embedded node
        const data = await dagService.getData(100);
        setDagData(data);

        // If a focus hash is set, try to focus it
        try {
          const focus = localStorage.getItem('dag_focus_hash');
          if (focus && data && data.nodes.length > 0) {
            const n = data.nodes.find(n => n.hash.toLowerCase() === focus.toLowerCase());
            if (n) {
              setSelectedNode(n);
              dagService.getBlockDetails(n.hash).then(setBlockDetails).catch(() => setBlockDetails(null));
            }
          }
        } catch {}
      } catch (dagError) {
        // Fallback: use RPC to create simple block list for external connections
        console.log('DAG service unavailable (external RPC mode), using block list fallback');
        const { invoke } = await import('@tauri-apps/api/core');

        try {
          const status: any = await invoke('get_status');
          const blockHeight = status.block_height || 0;

          // Create fallback DAG data from recent blocks
          const fallbackNodes: DAGNode[] = [];
          const startHeight = Math.max(0, blockHeight - 20);

          for (let i = startHeight; i <= blockHeight; i++) {
            fallbackNodes.push({
              id: `block_${i}`,
              hash: `block_${i}`,
              height: i,
              timestamp: Date.now() - ((blockHeight - i) * 2000),
              selectedParent: i > 0 ? `block_${i-1}` : undefined,
              mergeParents: [],
              blueScore: i,
              isBlue: true,
              isTip: i === blockHeight,
              transactions: 0,
              proposer: '0x',
              size: 0
            });
          }

          setDagData({
            nodes: fallbackNodes.reverse(),
            links: [],
            tips: [],
            statistics: {
              totalBlocks: blockHeight + 1,
              maxHeight: blockHeight,
              currentTips: 1,
              blueBlocks: blockHeight + 1,
              redBlocks: 0,
              averageBlueScore: blockHeight / 2
            }
          });

          setError('Using external RPC - showing recent blocks (DAG visualization unavailable)');
        } catch (rpcError) {
          throw dagError; // Re-throw original error if fallback also fails
        }
      }
    } catch (err: any) {
      console.error('Failed to load DAG data:', err);
      setError(err.toString());
      setDagData(null);
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    const handleOpen = (e: any) => {
      const hash = e?.detail?.hash;
      if (hash && dagData) {
        const n = dagData.nodes.find(n => n.hash.toLowerCase() === String(hash).toLowerCase());
        if (n) {
          setSelectedNode(n);
          dagService.getBlockDetails(n.hash).then(setBlockDetails).catch(() => setBlockDetails(null));
        }
      }
    };
    window.addEventListener('open-dag-for-hash' as any, handleOpen);
    return () => window.removeEventListener('open-dag-for-hash' as any, handleOpen);
  }, [dagData]);

  const handleNodeClick = (node: DAGNode) => {
    setSelectedNode(node);
    dagService.getBlockDetails(node.hash)
      .then(setBlockDetails)
      .catch(() => setBlockDetails(null));
  };

  const formatHash = (hash: string) => {
    if (!hash) return '...';
    return `${hash.slice(0, 8)}...${hash.slice(-6)}`;
  };

  const formatTime = (timestamp: number) => {
    const date = new Date(timestamp);
    return date.toLocaleTimeString();
  };

  return (
    <div className="dag-visualization">
      <div className="dag-header">
        <h2>DAG Visualization</h2>
        <div className="dag-controls">
          <div className="view-toggle">
            <button
              className={`toggle-btn ${viewMode === 'graph' ? 'active' : ''}`}
              onClick={() => setViewMode('graph')}
              title="Graph View"
            >
              <Share2 size={16} />
            </button>
            <button
              className={`toggle-btn ${viewMode === 'table' ? 'active' : ''}`}
              onClick={() => setViewMode('table')}
              title="Table View"
            >
              <Grid3X3 size={16} />
            </button>
          </div>
          <button
            className={`btn ${autoRefresh ? 'btn-primary' : 'btn-secondary'}`}
            onClick={() => setAutoRefresh(!autoRefresh)}
          >
            <RefreshCw size={16} className={autoRefresh ? 'spinning' : ''} />
            Auto Refresh
          </button>
          <button className="btn btn-secondary" onClick={loadDAGData}>
            <RefreshCw size={16} />
            Refresh
          </button>
        </div>
      </div>

      {dagData && (
        <div className="dag-stats">
          <div className="stat">
            <Layers size={16} />
            <span>Height: {dagData.statistics.maxHeight}</span>
          </div>
          <div className="stat">
            <GitBranch size={16} />
            <span>Tips: {dagData.statistics.currentTips}</span>
          </div>
          <div className="stat blue">
            <Network size={16} />
            <span>Blue: {dagData.statistics.blueBlocks}</span>
          </div>
          <div className="stat red">
            <Network size={16} />
            <span>Red: {dagData.statistics.redBlocks}</span>
          </div>
          <div className="stat">
            <Info size={16} />
            <span>Avg Blue Score: {dagData.statistics.averageBlueScore.toFixed(2)}</span>
          </div>
        </div>
      )}

      <div className="dag-container" ref={containerRef}>
        {error && (
          <div className="error-message">
            <Info size={20} />
            <span>Note: DAG service is initializing. Showing genesis block.</span>
          </div>
        )}

        {loading && !dagData ? (
          <div className="loading-state">
            <RefreshCw size={48} className="spinning" />
            <p>Loading DAG data...</p>
          </div>
        ) : dagData && dagData.nodes.length > 0 ? (
          viewMode === 'graph' ? (
            <div className="graph-container">
              <ForceGraph2D
                ref={graphRef as any}
                graphData={graphData}
                width={dimensions.width}
                height={dimensions.height}
                nodeCanvasObject={nodeCanvasObject}
                linkCanvasObject={linkCanvasObject}
                onNodeClick={handleGraphNodeClick}
                nodeLabel={(node: any) => `Block ${node.height}\nHash: ${node.hash?.slice(0, 16)}...\nBlue Score: ${node.blueScore}\nTxs: ${node.transactions}`}
                linkDirectionalArrowLength={6}
                linkDirectionalArrowRelPos={1}
                d3AlphaDecay={0.02}
                d3VelocityDecay={0.3}
                cooldownTicks={100}
                enableZoomInteraction={true}
                enablePanInteraction={true}
                minZoom={0.5}
                maxZoom={4}
              />
              <div className="graph-legend">
                <div className="legend-item">
                  <span className="legend-dot blue"></span>
                  <span>Blue Block</span>
                </div>
                <div className="legend-item">
                  <span className="legend-dot red"></span>
                  <span>Red Block</span>
                </div>
                <div className="legend-item">
                  <span className="legend-dot tip"></span>
                  <span>Current Tip</span>
                </div>
              </div>
            </div>
          ) : (
          <div className="blocks-table">
            <table>
              <thead>
                <tr>
                  <th><Box size={16} /> Height</th>
                  <th><Hash size={16} /> Block Hash</th>
                  <th><Activity size={16} /> Blue Score</th>
                  <th><Network size={16} /> Status</th>
                  <th><Clock size={16} /> Time</th>
                  <th>Txs</th>
                  <th>Size</th>
                </tr>
              </thead>
              <tbody>
                {dagData.nodes.map(node => (
                  <tr
                    key={node.id}
                    onClick={() => handleNodeClick(node)}
                    className={`block-row ${node.isBlue ? 'blue' : 'red'} ${node.isTip ? 'tip' : ''}`}
                  >
                    <td className="height">{node.height}</td>
                    <td className="hash mono">{formatHash(node.hash)}</td>
                    <td className="blue-score">{node.blueScore}</td>
                    <td className="status">
                      <span className={`badge ${node.isBlue ? 'badge-blue' : 'badge-red'}`}>
                        {node.isBlue ? 'Blue' : 'Red'}
                      </span>
                      {node.isTip && <span className="badge badge-tip">Tip</span>}
                    </td>
                    <td className="time">{formatTime(node.timestamp || Date.now())}</td>
                    <td className="txs">{node.transactions}</td>
                    <td className="size">{node.size} B</td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
          )
        ) : (
          <div className="empty-state">
            <Network size={48} />
            <h3>Node Not Running</h3>
            <p>Start the embedded node to view the DAG visualization.</p>
            <p className="hint">The DAG will populate once blocks are produced.</p>
          </div>
        )}
      </div>

      {selectedNode && (
        <div className="node-details">
          <h3>Block Details</h3>
          <button className="close-btn" onClick={() => setSelectedNode(null)}>×</button>
          
          <div className="detail-row">
            <span className="label">Height:</span>
            <span className="value">{selectedNode.height}</span>
          </div>
          <div className="detail-row">
            <span className="label">Hash:</span>
            <span className="value mono">{selectedNode.hash.slice(0, 16)}...</span>
          </div>
          <div className="detail-row">
            <span className="label">Blue Score:</span>
            <span className="value">{selectedNode.blueScore}</span>
          </div>
          <div className="detail-row">
            <span className="label">Status:</span>
            <span className={`value ${selectedNode.isBlue ? 'blue' : 'red'}`}>
              {selectedNode.isBlue ? 'Blue' : 'Red'}
            </span>
          </div>
          <div className="detail-row">
            <span className="label">Transactions:</span>
            <span className="value">{blockDetails?.transactions?.length ?? selectedNode.transactions}</span>
            {blockDetails?.transactions?.length > 0 && (
              <button className="btn btn-secondary btn-sm" onClick={() => setShowTxs(v => !v)}>
                {showTxs ? 'Hide' : 'View'}
              </button>
            )}
          </div>
          <div className="detail-row">
            <span className="label">Size:</span>
            <span className="value">{selectedNode.size} bytes</span>
          </div>
          {blockDetails && (
            <>
              <div className="detail-row">
                <span className="label">State Root:</span>
                <span className="value mono">{(blockDetails.state_root || blockDetails.stateRoot || '').slice(0, 16)}...</span>
              </div>
              <div className="detail-row">
                <span className="label">Tx Root:</span>
                <span className="value mono">{(blockDetails.tx_root || blockDetails.txRoot || '').slice(0, 16)}...</span>
              </div>
              <div className="detail-row">
                <span className="label">Receipt Root:</span>
                <span className="value mono">{(blockDetails.receipt_root || blockDetails.receiptRoot || '').slice(0, 16)}...</span>
              </div>
              <div className="detail-row">
                <span className="label">Children:</span>
                <span className="value">{(blockDetails.children || []).length}</span>
              </div>
            </>
          )}
          {showTxs && blockDetails?.transactions && (
            <div className="tx-popover">
              <div className="tx-header">Tx Details</div>
              <div className="tx-list">
                {blockDetails.transactions.map((tx: any) => (
                  <div key={tx.hash} className="tx-item">
                    <div className="tx-row"><span className="tx-label">Hash:</span><span className="tx-mono">{tx.hash.slice(0, 16)}…</span></div>
                    <div className="tx-row"><span className="tx-label">From:</span><span className="tx-mono">{tx.from_addr || tx.fromAddr || tx.from?.slice(0,16) + '…'}</span></div>
                    <div className="tx-row"><span className="tx-label">To:</span><span className="tx-mono">{tx.to_addr || tx.toAddr || (tx.to ? tx.to.slice(0,16) + '…' : '—')}</span></div>
                    <div className="tx-row"><span className="tx-label">Value:</span><span>{(Number(tx.value)/1e18).toFixed(4)} SALT</span></div>
                  </div>
                ))}
              </div>
            </div>
          )}
          <div className="detail-row">
            <span className="label">Proposer:</span>
            <span className="value mono">{selectedNode.proposer.slice(0, 12)}...</span>
          </div>
        </div>
      )}

      <style jsx>{`
        .dag-visualization {
          padding: 2rem;
          height: 100vh;
          display: flex;
          flex-direction: column;
        }

        .dag-header {
          display: flex;
          justify-content: space-between;
          align-items: center;
          margin-bottom: 1.5rem;
        }

        .dag-header h2 {
          margin: 0;
          font-size: 1.5rem;
          font-weight: 600;
        }

        .dag-controls {
          display: flex;
          gap: 1rem;
          align-items: center;
        }

        .view-toggle {
          display: flex;
          background: #f3f4f6;
          border-radius: 0.5rem;
          padding: 0.25rem;
        }

        .toggle-btn {
          display: flex;
          align-items: center;
          justify-content: center;
          padding: 0.5rem 0.75rem;
          border: none;
          border-radius: 0.375rem;
          background: transparent;
          color: #6b7280;
          cursor: pointer;
          transition: all 0.2s;
        }

        .toggle-btn:hover {
          color: #374151;
        }

        .toggle-btn.active {
          background: white;
          color: #667eea;
          box-shadow: 0 1px 3px rgba(0, 0, 0, 0.1);
        }

        .graph-container {
          position: relative;
          width: 100%;
          height: 100%;
        }

        .graph-legend {
          position: absolute;
          bottom: 1rem;
          left: 1rem;
          background: white;
          padding: 0.75rem 1rem;
          border-radius: 0.5rem;
          box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
          display: flex;
          gap: 1rem;
          font-size: 0.875rem;
        }

        .legend-item {
          display: flex;
          align-items: center;
          gap: 0.5rem;
        }

        .legend-dot {
          width: 12px;
          height: 12px;
          border-radius: 50%;
        }

        .legend-dot.blue {
          background: #3b82f6;
        }

        .legend-dot.red {
          background: #ef4444;
        }

        .legend-dot.tip {
          background: #a78bfa;
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

        .dag-stats {
          display: flex;
          gap: 1.5rem;
          margin-bottom: 1.5rem;
          flex-wrap: wrap;
        }

        .stat {
          display: flex;
          align-items: center;
          gap: 0.5rem;
          padding: 0.75rem 1.25rem;
          background: white;
          border-radius: 0.5rem;
          box-shadow: 0 2px 4px rgba(0, 0, 0, 0.1);
        }

        .stat.blue {
          background: #dbeafe;
          color: #1e40af;
        }

        .stat.red {
          background: #fee2e2;
          color: #991b1b;
        }

        .dag-container {
          flex: 1;
          position: relative;
          background: white;
          border-radius: 1rem;
          overflow: auto;
          box-shadow: 0 4px 8px rgba(0, 0, 0, 0.1);
        }

        .error-message {
          display: flex;
          align-items: center;
          gap: 0.5rem;
          padding: 1rem;
          background: #fef3c7;
          color: #92400e;
          border-bottom: 1px solid #fde68a;
        }

        .blocks-table {
          width: 100%;
        }

        .blocks-table table {
          width: 100%;
          border-collapse: collapse;
        }

        .blocks-table th {
          text-align: left;
          padding: 1rem;
          background: #f9fafb;
          border-bottom: 2px solid #e5e7eb;
          font-weight: 600;
          color: #374151;
          display: flex;
          align-items: center;
          gap: 0.5rem;
        }

        .blocks-table thead tr {
          display: grid;
          grid-template-columns: 80px 1fr 120px 150px 120px 80px 100px;
        }

        .blocks-table tbody tr {
          display: grid;
          grid-template-columns: 80px 1fr 120px 150px 120px 80px 100px;
          cursor: pointer;
          transition: all 0.2s;
          border-bottom: 1px solid #f3f4f6;
        }

        .blocks-table td {
          padding: 1rem;
          color: #6b7280;
        }

        .block-row:hover {
          background: #f9fafb;
        }

        .block-row.blue td.height {
          color: #3b82f6;
          font-weight: 600;
        }

        .block-row.red td.height {
          color: #ef4444;
          font-weight: 600;
        }

        .block-row.tip {
          background: #fef3c7;
        }

        .badge {
          padding: 0.25rem 0.5rem;
          border-radius: 0.25rem;
          font-size: 0.75rem;
          font-weight: 600;
          margin-right: 0.5rem;
        }

        .badge-blue {
          background: #dbeafe;
          color: #1e40af;
        }

        .badge-red {
          background: #fee2e2;
          color: #991b1b;
        }

        .badge-tip {
          background: #a78bfa;
          color: white;
        }

        .empty-state {
          display: flex;
          flex-direction: column;
          align-items: center;
          justify-content: center;
          padding: 4rem;
          color: #9ca3af;
        }

        .empty-state h3 {
          margin: 1rem 0 0.5rem;
          color: #374151;
        }

        .empty-state p {
          margin: 0.25rem 0;
          font-size: 0.9rem;
        }

        .empty-state .hint {
          font-size: 0.8rem;
          color: #9ca3af;
        }

        .loading-state {
          display: flex;
          flex-direction: column;
          align-items: center;
          justify-content: center;
          min-height: 400px;
          color: #6b7280;
        }

        .loading-state p {
          margin-top: 1rem;
          font-size: 1.125rem;
        }

        .node-details {
          position: absolute;
          bottom: 2rem;
          left: 2rem;
          background: white;
          border-radius: 1rem;
          padding: 1.5rem;
          width: 320px;
          box-shadow: 0 8px 16px rgba(0, 0, 0, 0.2);
        }
        .tx-popover { position: absolute; bottom: 2rem; right: 2rem; width: 360px; max-height: 300px; overflow: auto; background: white; border-radius: 0.75rem; box-shadow: 0 8px 16px rgba(0,0,0,0.2); padding: 1rem; }
        .tx-header { font-weight: 600; margin-bottom: 0.5rem; }
        .tx-item { border: 1px solid #e5e7eb; border-radius: 0.5rem; padding: 0.5rem; margin-bottom: 0.5rem; }
        .tx-row { display: flex; justify-content: space-between; gap: 0.5rem; font-size: 0.875rem; }
        .tx-label { color: #6b7280; }
        .tx-mono { font-family: monospace; }

        .node-details h3 {
          margin: 0 0 1rem 0;
          font-size: 1.125rem;
        }

        .close-btn {
          position: absolute;
          top: 1rem;
          right: 1rem;
          background: none;
          border: none;
          font-size: 1.5rem;
          cursor: pointer;
          color: #6b7280;
        }

        .detail-row {
          display: flex;
          justify-content: space-between;
          padding: 0.5rem 0;
          border-bottom: 1px solid #f3f4f6;
        }

        .detail-row:last-child {
          border-bottom: none;
        }

        .label {
          color: #6b7280;
          font-size: 0.875rem;
        }

        .value {
          font-weight: 500;
        }

        .value.blue {
          color: #3b82f6;
        }

        .value.red {
          color: #ef4444;
        }

        .mono {
          font-family: monospace;
          font-size: 0.875rem;
        }

        @keyframes spin {
          from { transform: rotate(0deg); }
          to { transform: rotate(360deg); }
        }

        .spinning {
          animation: spin 2s linear infinite;
        }
      `}</style>
    </div>
  );
};
