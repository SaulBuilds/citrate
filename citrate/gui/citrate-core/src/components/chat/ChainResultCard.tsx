/**
 * ChainResultCard Component
 *
 * Displays blockchain query results in a formatted, readable way.
 * Supports balance, block, transaction, receipt, and account info.
 */

import React, { useState } from 'react';
import {
  Wallet,
  Box,
  ArrowRightLeft,
  Receipt,
  User,
  FileCode,
  Copy,
  Check,
  ChevronDown,
  ChevronUp,
  RefreshCw,
  CheckCircle,
  XCircle,
  Clock,
} from 'lucide-react';
import {
  ChainResult,
  ChainResultType,
  BalanceResult,
  BlockResult,
  TransactionResult,
  ReceiptResult,
  AccountResult,
  ContractResult,
} from '../../types/agent';

interface ChainResultCardProps {
  result: ChainResult;
  onRefresh?: () => void;
}

export const ChainResultCard: React.FC<ChainResultCardProps> = ({
  result,
  onRefresh,
}) => {
  const [expanded, setExpanded] = useState(false);
  const [copied, setCopied] = useState<string | null>(null);

  const copyValue = async (value: string, field: string) => {
    await navigator.clipboard.writeText(value);
    setCopied(field);
    setTimeout(() => setCopied(null), 2000);
  };

  const getIcon = (type: ChainResultType) => {
    switch (type) {
      case 'balance':
        return <Wallet size={20} />;
      case 'block':
        return <Box size={20} />;
      case 'transaction':
        return <ArrowRightLeft size={20} />;
      case 'receipt':
        return <Receipt size={20} />;
      case 'account':
        return <User size={20} />;
      case 'contract':
        return <FileCode size={20} />;
      default:
        return <Box size={20} />;
    }
  };

  const getTitle = (type: ChainResultType) => {
    switch (type) {
      case 'balance':
        return 'Balance Query';
      case 'block':
        return 'Block Information';
      case 'transaction':
        return 'Transaction Details';
      case 'receipt':
        return 'Transaction Receipt';
      case 'account':
        return 'Account Information';
      case 'contract':
        return 'Contract Information';
      default:
        return 'Query Result';
    }
  };

  const renderContent = () => {
    switch (result.type) {
      case 'balance':
        return <BalanceContent data={result.data as BalanceResult} copyValue={copyValue} copied={copied} />;
      case 'block':
        return <BlockContent data={result.data as BlockResult} copyValue={copyValue} copied={copied} expanded={expanded} />;
      case 'transaction':
        return <TransactionContent data={result.data as TransactionResult} copyValue={copyValue} copied={copied} expanded={expanded} />;
      case 'receipt':
        return <ReceiptContent data={result.data as ReceiptResult} copyValue={copyValue} copied={copied} expanded={expanded} />;
      case 'account':
        return <AccountContent data={result.data as AccountResult} copyValue={copyValue} copied={copied} />;
      case 'contract':
        return <ContractContent data={result.data as ContractResult} copyValue={copyValue} copied={copied} expanded={expanded} />;
      default:
        return <div>Unknown result type</div>;
    }
  };

  return (
    <div className="chain-result-card">
      <div className="card-header">
        <div className="header-title">
          {getIcon(result.type)}
          <span>{getTitle(result.type)}</span>
        </div>
        <div className="header-actions">
          {onRefresh && (
            <button className="btn-icon" onClick={onRefresh} title="Refresh">
              <RefreshCw size={16} />
            </button>
          )}
          {['block', 'transaction', 'receipt', 'contract'].includes(result.type) && (
            <button
              className="btn-icon"
              onClick={() => setExpanded(!expanded)}
              title={expanded ? 'Collapse' : 'Expand'}
            >
              {expanded ? <ChevronUp size={16} /> : <ChevronDown size={16} />}
            </button>
          )}
        </div>
      </div>

      <div className="card-content">{renderContent()}</div>

      <div className="card-footer">
        <span className="timestamp">
          Queried at {new Date(result.timestamp).toLocaleTimeString()}
        </span>
      </div>

      <style jsx>{`
        .chain-result-card {
          background: white;
          border: 1px solid #e5e7eb;
          border-radius: 0.75rem;
          overflow: hidden;
          margin: 0.5rem 0;
        }

        .card-header {
          display: flex;
          justify-content: space-between;
          align-items: center;
          padding: 0.75rem 1rem;
          background: #f9fafb;
          border-bottom: 1px solid #e5e7eb;
        }

        .header-title {
          display: flex;
          align-items: center;
          gap: 0.5rem;
          font-weight: 600;
          color: #374151;
        }

        .header-actions {
          display: flex;
          gap: 0.25rem;
        }

        .btn-icon {
          display: flex;
          align-items: center;
          justify-content: center;
          width: 28px;
          height: 28px;
          background: none;
          border: none;
          border-radius: 0.25rem;
          cursor: pointer;
          color: #6b7280;
          transition: all 0.2s;
        }

        .btn-icon:hover {
          background: #e5e7eb;
          color: #374151;
        }

        .card-content {
          padding: 1rem;
        }

        .card-footer {
          padding: 0.5rem 1rem;
          background: #f9fafb;
          border-top: 1px solid #e5e7eb;
        }

        .timestamp {
          font-size: 0.75rem;
          color: #9ca3af;
        }
      `}</style>
    </div>
  );
};

// =============================================================================
// Balance Content
// =============================================================================

interface ContentProps<T> {
  data: T;
  copyValue: (value: string, field: string) => void;
  copied: string | null;
  expanded?: boolean;
}

const BalanceContent: React.FC<ContentProps<BalanceResult>> = ({ data, copyValue, copied }) => {
  const formatBalance = (balance: string) => {
    const num = Number(BigInt(balance)) / 1e18;
    return num.toLocaleString(undefined, { minimumFractionDigits: 0, maximumFractionDigits: 6 });
  };

  return (
    <div className="balance-content">
      <div className="balance-amount">
        <span className="amount">{data.formatted || formatBalance(data.balance)}</span>
        <span className="symbol">{data.tokenSymbol || 'CIT'}</span>
      </div>
      <div className="address-row">
        <span className="label">Address:</span>
        <code>{data.address.slice(0, 10)}...{data.address.slice(-8)}</code>
        <button
          className="btn-copy"
          onClick={() => copyValue(data.address, 'address')}
        >
          {copied === 'address' ? <Check size={14} /> : <Copy size={14} />}
        </button>
      </div>

      <style jsx>{`
        .balance-content {
          text-align: center;
        }

        .balance-amount {
          display: flex;
          align-items: baseline;
          justify-content: center;
          gap: 0.5rem;
          margin-bottom: 1rem;
        }

        .amount {
          font-size: 2rem;
          font-weight: 700;
          color: #059669;
        }

        .symbol {
          font-size: 1rem;
          font-weight: 500;
          color: #6b7280;
        }

        .address-row {
          display: flex;
          align-items: center;
          justify-content: center;
          gap: 0.5rem;
          font-size: 0.875rem;
        }

        .label {
          color: #6b7280;
        }

        code {
          background: #f3f4f6;
          padding: 0.25rem 0.5rem;
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
        }

        .btn-copy:hover {
          background: #f3f4f6;
          color: #374151;
        }
      `}</style>
    </div>
  );
};

// =============================================================================
// Block Content
// =============================================================================

const BlockContent: React.FC<ContentProps<BlockResult>> = ({ data, copyValue, copied, expanded }) => (
  <div className="block-content">
    <div className="detail-grid">
      <DetailRow label="Block Number" value={data.number.toLocaleString()} />
      <DetailRow label="Transactions" value={data.transactionCount.toString()} />
      <DetailRow
        label="Block Hash"
        value={`${data.hash.slice(0, 10)}...${data.hash.slice(-8)}`}
        copyable
        fullValue={data.hash}
        onCopy={copyValue}
        copied={copied}
        copyField="hash"
      />
      <DetailRow label="Timestamp" value={new Date(data.timestamp * 1000).toLocaleString()} />
    </div>

    {expanded && (
      <div className="expanded-details">
        <DetailRow
          label="Parent Hash"
          value={`${data.parentHash.slice(0, 10)}...${data.parentHash.slice(-8)}`}
          copyable
          fullValue={data.parentHash}
          onCopy={copyValue}
          copied={copied}
          copyField="parentHash"
        />
        <DetailRow label="Gas Used" value={parseInt(data.gasUsed).toLocaleString()} />
        <DetailRow label="Gas Limit" value={parseInt(data.gasLimit).toLocaleString()} />
        {data.miner && <DetailRow label="Miner" value={`${data.miner.slice(0, 10)}...`} />}
      </div>
    )}

    <style jsx>{`
      .block-content {
        display: flex;
        flex-direction: column;
        gap: 0.75rem;
      }

      .detail-grid {
        display: grid;
        grid-template-columns: repeat(2, 1fr);
        gap: 0.75rem;
      }

      .expanded-details {
        padding-top: 0.75rem;
        border-top: 1px solid #e5e7eb;
        display: flex;
        flex-direction: column;
        gap: 0.5rem;
      }
    `}</style>
  </div>
);

// =============================================================================
// Transaction Content
// =============================================================================

const TransactionContent: React.FC<ContentProps<TransactionResult>> = ({ data, copyValue, copied, expanded }) => {
  const formatValue = (value: string) => {
    const num = Number(BigInt(value)) / 1e18;
    return num.toLocaleString(undefined, { minimumFractionDigits: 0, maximumFractionDigits: 6 }) + ' CIT';
  };

  const getStatusBadge = () => {
    switch (data.status) {
      case 'success':
        return <span className="status success"><CheckCircle size={14} /> Success</span>;
      case 'failed':
        return <span className="status failed"><XCircle size={14} /> Failed</span>;
      case 'pending':
        return <span className="status pending"><Clock size={14} /> Pending</span>;
      default:
        return null;
    }
  };

  return (
    <div className="transaction-content">
      {data.status && <div className="status-row">{getStatusBadge()}</div>}

      <DetailRow
        label="Hash"
        value={`${data.hash.slice(0, 10)}...${data.hash.slice(-8)}`}
        copyable
        fullValue={data.hash}
        onCopy={copyValue}
        copied={copied}
        copyField="txHash"
      />
      <DetailRow
        label="From"
        value={`${data.from.slice(0, 10)}...${data.from.slice(-8)}`}
        copyable
        fullValue={data.from}
        onCopy={copyValue}
        copied={copied}
        copyField="from"
      />
      {data.to && (
        <DetailRow
          label="To"
          value={`${data.to.slice(0, 10)}...${data.to.slice(-8)}`}
          copyable
          fullValue={data.to}
          onCopy={copyValue}
          copied={copied}
          copyField="to"
        />
      )}
      <DetailRow label="Value" value={formatValue(data.value)} />

      {expanded && (
        <div className="expanded-details">
          <DetailRow label="Gas" value={parseInt(data.gas).toLocaleString()} />
          <DetailRow label="Gas Price" value={parseInt(data.gasPrice).toLocaleString() + ' wei'} />
          <DetailRow label="Nonce" value={data.nonce.toString()} />
          {data.blockNumber && <DetailRow label="Block" value={data.blockNumber.toLocaleString()} />}
        </div>
      )}

      <style jsx>{`
        .transaction-content {
          display: flex;
          flex-direction: column;
          gap: 0.5rem;
        }

        .status-row {
          margin-bottom: 0.5rem;
        }

        :global(.status) {
          display: inline-flex;
          align-items: center;
          gap: 0.375rem;
          padding: 0.25rem 0.5rem;
          border-radius: 9999px;
          font-size: 0.75rem;
          font-weight: 600;
        }

        :global(.status.success) {
          background: #d1fae5;
          color: #059669;
        }

        :global(.status.failed) {
          background: #fee2e2;
          color: #dc2626;
        }

        :global(.status.pending) {
          background: #fef3c7;
          color: #d97706;
        }

        .expanded-details {
          padding-top: 0.75rem;
          border-top: 1px solid #e5e7eb;
        }
      `}</style>
    </div>
  );
};

// =============================================================================
// Receipt Content
// =============================================================================

const ReceiptContent: React.FC<ContentProps<ReceiptResult>> = ({ data, copyValue, copied, expanded }) => (
  <div className="receipt-content">
    <div className="status-row">
      {data.status ? (
        <span className="status success"><CheckCircle size={14} /> Success</span>
      ) : (
        <span className="status failed"><XCircle size={14} /> Failed</span>
      )}
    </div>

    <DetailRow
      label="Tx Hash"
      value={`${data.transactionHash.slice(0, 10)}...${data.transactionHash.slice(-8)}`}
      copyable
      fullValue={data.transactionHash}
      onCopy={copyValue}
      copied={copied}
      copyField="txHash"
    />
    <DetailRow label="Block" value={data.blockNumber.toLocaleString()} />
    <DetailRow label="Gas Used" value={parseInt(data.gasUsed).toLocaleString()} />

    {data.contractAddress && (
      <DetailRow
        label="Contract"
        value={`${data.contractAddress.slice(0, 10)}...${data.contractAddress.slice(-8)}`}
        copyable
        fullValue={data.contractAddress}
        onCopy={copyValue}
        copied={copied}
        copyField="contract"
      />
    )}

    {expanded && data.logs.length > 0 && (
      <div className="logs-section">
        <span className="logs-title">Logs ({data.logs.length})</span>
        {data.logs.slice(0, 3).map((log, i) => (
          <div key={i} className="log-item">
            <code>{log.topics[0]?.slice(0, 20)}...</code>
          </div>
        ))}
        {data.logs.length > 3 && (
          <span className="more-logs">+{data.logs.length - 3} more</span>
        )}
      </div>
    )}

    <style jsx>{`
      .receipt-content {
        display: flex;
        flex-direction: column;
        gap: 0.5rem;
      }

      .status-row {
        margin-bottom: 0.5rem;
      }

      .logs-section {
        margin-top: 0.75rem;
        padding-top: 0.75rem;
        border-top: 1px solid #e5e7eb;
      }

      .logs-title {
        display: block;
        font-size: 0.75rem;
        font-weight: 600;
        color: #6b7280;
        margin-bottom: 0.5rem;
      }

      .log-item {
        padding: 0.375rem;
        background: #f3f4f6;
        border-radius: 0.25rem;
        margin-bottom: 0.25rem;
      }

      .log-item code {
        font-size: 0.75rem;
      }

      .more-logs {
        font-size: 0.75rem;
        color: #9ca3af;
      }
    `}</style>
  </div>
);

// =============================================================================
// Account Content
// =============================================================================

const AccountContent: React.FC<ContentProps<AccountResult>> = ({ data, copyValue, copied }) => {
  const formatBalance = (balance: string) => {
    const num = Number(BigInt(balance)) / 1e18;
    return num.toLocaleString(undefined, { minimumFractionDigits: 0, maximumFractionDigits: 6 }) + ' CIT';
  };

  return (
    <div className="account-content">
      <DetailRow
        label="Address"
        value={`${data.address.slice(0, 10)}...${data.address.slice(-8)}`}
        copyable
        fullValue={data.address}
        onCopy={copyValue}
        copied={copied}
        copyField="address"
      />
      <DetailRow label="Balance" value={formatBalance(data.balance)} />
      <DetailRow label="Nonce" value={data.nonce.toString()} />
      <DetailRow label="Contract" value={data.hasCode ? 'Yes' : 'No'} />

      <style jsx>{`
        .account-content {
          display: flex;
          flex-direction: column;
          gap: 0.5rem;
        }
      `}</style>
    </div>
  );
};

// =============================================================================
// Contract Content
// =============================================================================

const ContractContent: React.FC<ContentProps<ContractResult>> = ({ data, copyValue, copied, expanded }) => (
  <div className="contract-content">
    <DetailRow
      label="Address"
      value={`${data.address.slice(0, 10)}...${data.address.slice(-8)}`}
      copyable
      fullValue={data.address}
      onCopy={copyValue}
      copied={copied}
      copyField="address"
    />
    {data.name && <DetailRow label="Name" value={data.name} />}

    {expanded && data.abi && (
      <div className="abi-section">
        <span className="abi-title">ABI Functions ({(data.abi as any[]).filter(f => f.type === 'function').length})</span>
        {(data.abi as any[])
          .filter(f => f.type === 'function')
          .slice(0, 5)
          .map((fn, i) => (
            <div key={i} className="abi-item">
              <code>{fn.name}()</code>
            </div>
          ))}
      </div>
    )}

    <style jsx>{`
      .contract-content {
        display: flex;
        flex-direction: column;
        gap: 0.5rem;
      }

      .abi-section {
        margin-top: 0.75rem;
        padding-top: 0.75rem;
        border-top: 1px solid #e5e7eb;
      }

      .abi-title {
        display: block;
        font-size: 0.75rem;
        font-weight: 600;
        color: #6b7280;
        margin-bottom: 0.5rem;
      }

      .abi-item {
        padding: 0.375rem;
        background: #f3f4f6;
        border-radius: 0.25rem;
        margin-bottom: 0.25rem;
      }

      .abi-item code {
        font-size: 0.75rem;
      }
    `}</style>
  </div>
);

// =============================================================================
// DetailRow Component
// =============================================================================

interface DetailRowProps {
  label: string;
  value: string;
  copyable?: boolean;
  fullValue?: string;
  onCopy?: (value: string, field: string) => void;
  copied?: string | null;
  copyField?: string;
}

const DetailRow: React.FC<DetailRowProps> = ({
  label,
  value,
  copyable,
  fullValue,
  onCopy,
  copied,
  copyField,
}) => (
  <div className="detail-row">
    <span className="label">{label}</span>
    <div className="value-wrapper">
      <span className="value">{value}</span>
      {copyable && onCopy && copyField && (
        <button
          className="btn-copy"
          onClick={() => onCopy(fullValue || value, copyField)}
        >
          {copied === copyField ? <Check size={14} /> : <Copy size={14} />}
        </button>
      )}
    </div>

    <style jsx>{`
      .detail-row {
        display: flex;
        justify-content: space-between;
        align-items: center;
        padding: 0.375rem 0;
      }

      .label {
        font-size: 0.875rem;
        color: #6b7280;
      }

      .value-wrapper {
        display: flex;
        align-items: center;
        gap: 0.375rem;
      }

      .value {
        font-size: 0.875rem;
        font-weight: 500;
        color: #111827;
        font-family: monospace;
      }

      .btn-copy {
        display: flex;
        align-items: center;
        justify-content: center;
        width: 22px;
        height: 22px;
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
    `}</style>
  </div>
);

export default ChainResultCard;
