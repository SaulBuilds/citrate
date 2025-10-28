/**
 * Glassmorphic Skeleton Loading Components
 *
 * Beautiful loading placeholders with glassmorphism design:
 * - Semi-transparent backgrounds with backdrop blur
 * - Subtle borders and shadows
 * - Smooth shimmer animation
 * - Multiple variants for different content types
 *
 * Usage:
 * <SkeletonText width="200px" />
 * <SkeletonCard height="100px" />
 * <SkeletonCircle size={40} />
 * <SkeletonTable rows={5} columns={3} />
 */

import { CSSProperties } from 'react';

interface SkeletonBaseProps {
  className?: string;
  style?: CSSProperties;
}

interface SkeletonTextProps extends SkeletonBaseProps {
  width?: string;
  height?: string;
  lines?: number;
}

interface SkeletonCardProps extends SkeletonBaseProps {
  width?: string;
  height?: string;
}

interface SkeletonCircleProps extends SkeletonBaseProps {
  size?: number;
}

interface SkeletonTableProps extends SkeletonBaseProps {
  rows?: number;
  columns?: number;
}

/**
 * Skeleton Text - For text content, addresses, balances
 */
export function SkeletonText({
  width = '100%',
  height = '1rem',
  lines = 1,
  className = '',
  style = {}
}: SkeletonTextProps) {
  return (
    <div className={className} style={style}>
      {Array.from({ length: lines }).map((_, i) => (
        <div
          key={i}
          className="skeleton-base"
          style={{
            width: lines > 1 && i === lines - 1 ? `${Math.random() * 40 + 60}%` : width,
            height,
            marginBottom: lines > 1 && i < lines - 1 ? '0.5rem' : 0
          }}
        />
      ))}
      <style jsx>{`
        .skeleton-base {
          position: relative;
          overflow: hidden;
          border-radius: 0.5rem;
          background: rgba(255, 255, 255, 0.1);
          backdrop-filter: blur(10px);
          border: 1px solid rgba(255, 255, 255, 0.2);
          box-shadow:
            0 4px 6px rgba(0, 0, 0, 0.05),
            inset 0 1px 0 rgba(255, 255, 255, 0.1);
        }

        .skeleton-base::after {
          content: '';
          position: absolute;
          top: 0;
          left: -100%;
          width: 100%;
          height: 100%;
          background: linear-gradient(
            90deg,
            transparent 0%,
            rgba(255, 255, 255, 0.3) 50%,
            transparent 100%
          );
          animation: shimmer 2s infinite;
        }

        @keyframes shimmer {
          0% {
            left: -100%;
          }
          100% {
            left: 100%;
          }
        }
      `}</style>
    </div>
  );
}

/**
 * Skeleton Card - For card components, blocks, transactions
 */
export function SkeletonCard({
  width = '100%',
  height = '200px',
  className = '',
  style = {}
}: SkeletonCardProps) {
  return (
    <>
      <div
        className={`skeleton-card ${className}`}
        style={{
          width,
          height,
          ...style
        }}
      />
      <style jsx>{`
        .skeleton-card {
          position: relative;
          overflow: hidden;
          border-radius: 1rem;
          background: rgba(255, 255, 255, 0.05);
          backdrop-filter: blur(12px);
          border: 1px solid rgba(255, 255, 255, 0.15);
          box-shadow:
            0 8px 32px rgba(0, 0, 0, 0.1),
            inset 0 1px 0 rgba(255, 255, 255, 0.1),
            inset 0 -1px 0 rgba(0, 0, 0, 0.1);
        }

        .skeleton-card::before {
          content: '';
          position: absolute;
          inset: 0;
          background: linear-gradient(
            135deg,
            rgba(255, 255, 255, 0.1) 0%,
            rgba(255, 255, 255, 0.05) 50%,
            rgba(255, 255, 255, 0.1) 100%
          );
          animation: pulse 2s ease-in-out infinite;
        }

        .skeleton-card::after {
          content: '';
          position: absolute;
          top: 0;
          left: -100%;
          width: 50%;
          height: 100%;
          background: linear-gradient(
            90deg,
            transparent 0%,
            rgba(255, 255, 255, 0.2) 50%,
            transparent 100%
          );
          animation: shimmer 2.5s infinite;
          animation-delay: 0.5s;
        }

        @keyframes pulse {
          0%, 100% {
            opacity: 1;
          }
          50% {
            opacity: 0.5;
          }
        }

        @keyframes shimmer {
          0% {
            left: -100%;
          }
          100% {
            left: 150%;
          }
        }
      `}</style>
    </>
  );
}

/**
 * Skeleton Circle - For avatars, icons, status indicators
 */
export function SkeletonCircle({
  size = 48,
  className = '',
  style = {}
}: SkeletonCircleProps) {
  return (
    <>
      <div
        className={`skeleton-circle ${className}`}
        style={{
          width: `${size}px`,
          height: `${size}px`,
          ...style
        }}
      />
      <style jsx>{`
        .skeleton-circle {
          position: relative;
          overflow: hidden;
          border-radius: 50%;
          background: rgba(255, 255, 255, 0.1);
          backdrop-filter: blur(10px);
          border: 1px solid rgba(255, 255, 255, 0.2);
          box-shadow:
            0 4px 8px rgba(0, 0, 0, 0.1),
            inset 0 1px 0 rgba(255, 255, 255, 0.2);
        }

        .skeleton-circle::after {
          content: '';
          position: absolute;
          top: -50%;
          left: -50%;
          width: 200%;
          height: 200%;
          background: linear-gradient(
            135deg,
            transparent 0%,
            rgba(255, 255, 255, 0.3) 50%,
            transparent 100%
          );
          animation: rotate-shimmer 3s linear infinite;
        }

        @keyframes rotate-shimmer {
          0% {
            transform: rotate(0deg);
          }
          100% {
            transform: rotate(360deg);
          }
        }
      `}</style>
    </>
  );
}

/**
 * Skeleton Table - For table rows (peers, transactions, blocks)
 */
export function SkeletonTable({
  rows = 5,
  columns = 4,
  className = '',
  style = {}
}: SkeletonTableProps) {
  return (
    <div className={`skeleton-table ${className}`} style={style}>
      {Array.from({ length: rows }).map((_, rowIndex) => (
        <div key={rowIndex} className="skeleton-row">
          {Array.from({ length: columns }).map((_, colIndex) => (
            <div
              key={colIndex}
              className="skeleton-cell"
              style={{
                width: `${100 / columns}%`,
                flex: columns === 1 ? 1 : undefined
              }}
            >
              <div className="skeleton-cell-content" />
            </div>
          ))}
        </div>
      ))}
      <style jsx>{`
        .skeleton-table {
          display: flex;
          flex-direction: column;
          gap: 0.75rem;
        }

        .skeleton-row {
          display: flex;
          gap: 1rem;
          padding: 1rem;
          border-radius: 0.75rem;
          background: rgba(255, 255, 255, 0.03);
          backdrop-filter: blur(8px);
          border: 1px solid rgba(255, 255, 255, 0.1);
          transition: all 0.3s ease;
        }

        .skeleton-row:hover {
          background: rgba(255, 255, 255, 0.05);
          border-color: rgba(255, 255, 255, 0.15);
        }

        .skeleton-cell {
          flex: 1;
          min-width: 0;
        }

        .skeleton-cell-content {
          position: relative;
          overflow: hidden;
          height: 1rem;
          border-radius: 0.375rem;
          background: rgba(255, 255, 255, 0.1);
          box-shadow: inset 0 1px 2px rgba(0, 0, 0, 0.05);
        }

        .skeleton-cell-content::after {
          content: '';
          position: absolute;
          top: 0;
          left: -100%;
          width: 100%;
          height: 100%;
          background: linear-gradient(
            90deg,
            transparent 0%,
            rgba(255, 255, 255, 0.25) 50%,
            transparent 100%
          );
          animation: shimmer 2s infinite;
          animation-delay: calc(var(--row-index) * 0.1s);
        }

        @keyframes shimmer {
          0% {
            left: -100%;
          }
          100% {
            left: 100%;
          }
        }
      `}</style>
    </div>
  );
}

/**
 * Skeleton List - For lists of items
 */
export function SkeletonList({
  items = 5,
  itemHeight = '60px',
  className = '',
  style = {}
}: {
  items?: number;
  itemHeight?: string;
  className?: string;
  style?: CSSProperties;
}) {
  return (
    <div className={`skeleton-list ${className}`} style={style}>
      {Array.from({ length: items }).map((_, i) => (
        <div key={i} className="skeleton-list-item">
          <SkeletonCircle size={40} />
          <div className="skeleton-list-content">
            <SkeletonText width="60%" height="1rem" />
            <SkeletonText width="40%" height="0.875rem" />
          </div>
        </div>
      ))}
      <style jsx>{`
        .skeleton-list {
          display: flex;
          flex-direction: column;
          gap: 0.75rem;
        }

        .skeleton-list-item {
          display: flex;
          align-items: center;
          gap: 1rem;
          padding: 1rem;
          height: ${itemHeight};
          border-radius: 0.75rem;
          background: rgba(255, 255, 255, 0.03);
          backdrop-filter: blur(8px);
          border: 1px solid rgba(255, 255, 255, 0.1);
          transition: all 0.3s ease;
        }

        .skeleton-list-item:hover {
          background: rgba(255, 255, 255, 0.05);
          border-color: rgba(255, 255, 255, 0.15);
        }

        .skeleton-list-content {
          flex: 1;
          display: flex;
          flex-direction: column;
          gap: 0.5rem;
        }
      `}</style>
    </div>
  );
}

/**
 * Skeleton Stats Grid - For dashboard statistics
 */
export function SkeletonStats({
  count = 4,
  className = '',
  style = {}
}: {
  count?: number;
  className?: string;
  style?: CSSProperties;
}) {
  return (
    <div className={`skeleton-stats ${className}`} style={style}>
      {Array.from({ length: count }).map((_, i) => (
        <div key={i} className="skeleton-stat-card">
          <SkeletonText width="50%" height="0.875rem" />
          <SkeletonText width="70%" height="2rem" />
          <SkeletonText width="40%" height="0.75rem" />
        </div>
      ))}
      <style jsx>{`
        .skeleton-stats {
          display: grid;
          grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
          gap: 1.5rem;
        }

        .skeleton-stat-card {
          display: flex;
          flex-direction: column;
          gap: 0.75rem;
          padding: 1.5rem;
          border-radius: 1rem;
          background: rgba(255, 255, 255, 0.05);
          backdrop-filter: blur(12px);
          border: 1px solid rgba(255, 255, 255, 0.15);
          box-shadow:
            0 8px 32px rgba(0, 0, 0, 0.1),
            inset 0 1px 0 rgba(255, 255, 255, 0.1);
        }
      `}</style>
    </div>
  );
}

export default {
  Text: SkeletonText,
  Card: SkeletonCard,
  Circle: SkeletonCircle,
  Table: SkeletonTable,
  List: SkeletonList,
  Stats: SkeletonStats
};
