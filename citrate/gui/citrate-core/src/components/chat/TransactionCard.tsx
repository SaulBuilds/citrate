/**
 * TransactionCard Component
 *
 * Displays pending transaction details with approve/reject actions.
 * Shows transaction type, addresses, value, and gas estimate.
 */

import React, { useState, useEffect } from 'react';
import {
  ArrowUpRight,
  FileCode,
  Play,
  Check,
  X,
  Copy,
  AlertTriangle,
  Clock,
  Loader,
  Shield,
} from 'lucide-react';
import { PendingTransaction, TransactionSimulation } from '../../types/agent';

interface TransactionCardProps {
  transaction: PendingTransaction;
  onApprove: () => Promise<void>;
  onReject: () => Promise<void>;
  simulation?: TransactionSimulation;
}

export const TransactionCard: React.FC<TransactionCardProps> = ({
  transaction,
  onApprove,
  onReject,
  simulation,
}) => {
  const [isApproving, setIsApproving] = useState(false);
  const [isRejecting, setIsRejecting] = useState(false);
  const [copied, setCopied] = useState<string | null>(null);
  const [timeLeft, setTimeLeft] = useState<number | null>(null);

  // Countdown timer if there's a timeout
  useEffect(() => {
    if (!transaction.timeoutAt) return;

    const updateTimeLeft = () => {
      const remaining = Math.max(0, transaction.timeoutAt! - Date.now());
      setTimeLeft(remaining);
    };

    updateTimeLeft();
    const interval = setInterval(updateTimeLeft, 1000);
    return () => clearInterval(interval);
  }, [transaction.timeoutAt]);

  const handleApprove = async () => {
    setIsApproving(true);
    try {
      await onApprove();
    } finally {
      setIsApproving(false);
    }
  };

  const handleReject = async () => {
    setIsRejecting(true);
    try {
      await onReject();
    } finally {
      setIsRejecting(false);
    }
  };

  const copyAddress = async (address: string, field: string) => {
    await navigator.clipboard.writeText(address);
    setCopied(field);
    setTimeout(() => setCopied(null), 2000);
  };

  const truncateAddress = (address: string) => {
    if (!address) return '';
    return `${address.slice(0, 6)}...${address.slice(-4)}`;
  };

  const formatValue = (value: string | undefined) => {
    if (!value) return '0';
    // Convert from wei to ETH (simplified)
    const numValue = BigInt(value);
    const ethValue = Number(numValue) / 1e18;
    return ethValue.toLocaleString(undefined, {
      minimumFractionDigits: 0,
      maximumFractionDigits: 6,
    });
  };

  const formatGas = (gas: string | undefined) => {
    if (!gas) return 'Estimating...';
    return parseInt(gas).toLocaleString();
  };

  const formatTimeLeft = (ms: number) => {
    const seconds = Math.floor(ms / 1000);
    const minutes = Math.floor(seconds / 60);
    const remainingSeconds = seconds % 60;
    return `${minutes}:${remainingSeconds.toString().padStart(2, '0')}`;
  };

  const getTypeIcon = () => {
    switch (transaction.type) {
      case 'send':
        return <ArrowUpRight size={20} />;
      case 'deploy':
        return <FileCode size={20} />;
      case 'call':
        return <Play size={20} />;
      default:
        return <ArrowUpRight size={20} />;
    }
  };

  const getTypeLabel = () => {
    switch (transaction.type) {
      case 'send':
        return 'Send Transaction';
      case 'deploy':
        return 'Deploy Contract';
      case 'call':
        return 'Contract Call';
      default:
        return 'Transaction';
    }
  };

  const isProcessing = isApproving || isRejecting;

  return (
    <div className={`transaction-card ${simulation?.success === false ? 'simulation-failed' : ''}`}>
      {/* Header */}
      <div className="card-header">
        <div className="type-badge">
          {getTypeIcon()}
          <span>{getTypeLabel()}</span>
        </div>

        {timeLeft !== null && timeLeft > 0 && (
          <div className="timeout-badge">
            <Clock size={14} />
            <span>{formatTimeLeft(timeLeft)}</span>
          </div>
        )}
      </div>

      {/* Transaction Details */}
      <div className="card-details">
        {/* From Address */}
        <div className="detail-row">
          <span className="detail-label">From</span>
          <div className="detail-value address">
            <span>{truncateAddress(transaction.from)}</span>
            <button
              className="btn-copy"
              onClick={() => copyAddress(transaction.from, 'from')}
              title="Copy address"
            >
              {copied === 'from' ? <Check size={14} /> : <Copy size={14} />}
            </button>
          </div>
        </div>

        {/* To Address */}
        {transaction.to && (
          <div className="detail-row">
            <span className="detail-label">To</span>
            <div className="detail-value address">
              <span>{truncateAddress(transaction.to)}</span>
              <button
                className="btn-copy"
                onClick={() => copyAddress(transaction.to!, 'to')}
                title="Copy address"
              >
                {copied === 'to' ? <Check size={14} /> : <Copy size={14} />}
              </button>
            </div>
          </div>
        )}

        {/* Value */}
        {transaction.value && (
          <div className="detail-row">
            <span className="detail-label">Value</span>
            <div className="detail-value amount">
              <span>{formatValue(transaction.value)} CIT</span>
            </div>
          </div>
        )}

        {/* Gas */}
        <div className="detail-row">
          <span className="detail-label">Gas Limit</span>
          <div className="detail-value">
            <span>{formatGas(transaction.gas)}</span>
          </div>
        </div>

        {/* Contract Info */}
        {transaction.contractName && (
          <div className="detail-row">
            <span className="detail-label">Contract</span>
            <div className="detail-value">
              <span>{transaction.contractName}</span>
            </div>
          </div>
        )}

        {/* Method Info */}
        {transaction.methodName && (
          <div className="detail-row">
            <span className="detail-label">Method</span>
            <div className="detail-value">
              <code>{transaction.methodName}</code>
            </div>
          </div>
        )}
      </div>

      {/* Simulation Result */}
      {simulation && (
        <div className={`simulation-result ${simulation.success ? 'success' : 'failed'}`}>
          {simulation.success ? (
            <>
              <Shield size={16} />
              <span>Simulation passed</span>
              {simulation.gasUsed && (
                <span className="gas-used">Gas used: {formatGas(simulation.gasUsed)}</span>
              )}
            </>
          ) : (
            <>
              <AlertTriangle size={16} />
              <span>Simulation failed: {simulation.error || 'Unknown error'}</span>
            </>
          )}
        </div>
      )}

      {/* Action Buttons */}
      <div className="card-actions">
        <button
          className="btn-reject"
          onClick={handleReject}
          disabled={isProcessing}
          title="Reject transaction (Esc)"
        >
          {isRejecting ? <Loader size={18} className="spinning" /> : <X size={18} />}
          <span>Reject</span>
        </button>

        <button
          className="btn-approve"
          onClick={handleApprove}
          disabled={isProcessing}
          title="Approve transaction (Enter)"
        >
          {isApproving ? <Loader size={18} className="spinning" /> : <Check size={18} />}
          <span>Approve</span>
        </button>
      </div>

      <style jsx>{`
        .transaction-card {
          background: white;
          border: 1px solid #e5e7eb;
          border-radius: 0.75rem;
          overflow: hidden;
          margin: 0.5rem 0;
        }

        .transaction-card.simulation-failed {
          border-color: #fecaca;
        }

        .card-header {
          display: flex;
          justify-content: space-between;
          align-items: center;
          padding: 0.75rem 1rem;
          background: #f9fafb;
          border-bottom: 1px solid #e5e7eb;
        }

        .type-badge {
          display: flex;
          align-items: center;
          gap: 0.5rem;
          font-weight: 600;
          color: #374151;
        }

        .timeout-badge {
          display: flex;
          align-items: center;
          gap: 0.375rem;
          padding: 0.25rem 0.5rem;
          background: #fef3c7;
          color: #d97706;
          border-radius: 9999px;
          font-size: 0.75rem;
          font-weight: 500;
        }

        .card-details {
          padding: 1rem;
        }

        .detail-row {
          display: flex;
          justify-content: space-between;
          align-items: center;
          padding: 0.5rem 0;
          border-bottom: 1px solid #f3f4f6;
        }

        .detail-row:last-child {
          border-bottom: none;
        }

        .detail-label {
          font-size: 0.875rem;
          color: #6b7280;
        }

        .detail-value {
          display: flex;
          align-items: center;
          gap: 0.5rem;
          font-size: 0.875rem;
          font-weight: 500;
          color: #111827;
        }

        .detail-value.address {
          font-family: monospace;
        }

        .detail-value.amount {
          color: #059669;
          font-weight: 600;
        }

        .detail-value code {
          background: #f3f4f6;
          padding: 0.125rem 0.375rem;
          border-radius: 0.25rem;
          font-size: 0.8125rem;
        }

        .btn-copy {
          display: flex;
          align-items: center;
          justify-content: center;
          width: 24px;
          height: 24px;
          background: none;
          border: none;
          border-radius: 0.25rem;
          cursor: pointer;
          color: #9ca3af;
          transition: all 0.2s;
        }

        .btn-copy:hover {
          background: #f3f4f6;
          color: #374151;
        }

        .simulation-result {
          display: flex;
          align-items: center;
          gap: 0.5rem;
          padding: 0.75rem 1rem;
          font-size: 0.875rem;
          font-weight: 500;
        }

        .simulation-result.success {
          background: #d1fae5;
          color: #059669;
        }

        .simulation-result.failed {
          background: #fee2e2;
          color: #dc2626;
        }

        .gas-used {
          margin-left: auto;
          font-weight: 400;
          opacity: 0.8;
        }

        .card-actions {
          display: flex;
          gap: 0.75rem;
          padding: 1rem;
          background: #f9fafb;
          border-top: 1px solid #e5e7eb;
        }

        .btn-reject,
        .btn-approve {
          flex: 1;
          display: flex;
          align-items: center;
          justify-content: center;
          gap: 0.5rem;
          padding: 0.75rem 1rem;
          border: none;
          border-radius: 0.5rem;
          font-size: 0.875rem;
          font-weight: 600;
          cursor: pointer;
          transition: all 0.2s;
        }

        .btn-reject {
          background: white;
          border: 1px solid #e5e7eb;
          color: #374151;
        }

        .btn-reject:hover:not(:disabled) {
          background: #fee2e2;
          border-color: #fecaca;
          color: #dc2626;
        }

        .btn-approve {
          background: #059669;
          color: white;
        }

        .btn-approve:hover:not(:disabled) {
          background: #047857;
        }

        .btn-reject:disabled,
        .btn-approve:disabled {
          opacity: 0.6;
          cursor: not-allowed;
        }

        .spinning {
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
      `}</style>
    </div>
  );
};

export default TransactionCard;
