/**
 * ReviewModerationPanel Component
 *
 * Admin panel for reviewing flagged reviews.
 * Features:
 * - List of flagged reviews
 * - Review details with reporter information
 * - Actions: Approve, Remove, Ban User
 * - Filter by status
 * - Admin authentication check
 */

import React, { useState } from 'react';
import { ReviewReport } from '../../utils/types/reviews';

export interface FlaggedReview extends ReviewReport {
  reviewText: string;
  reviewerAddress: string;
  reviewRating: number;
}

export interface ReviewModerationPanelProps {
  adminAddress: string;
  flaggedReviews: FlaggedReview[];
  onApprove?: (reviewId: string) => Promise<void>;
  onRemove?: (reviewId: string) => Promise<void>;
  onBanUser?: (userAddress: string) => Promise<void>;
  className?: string;
}

export const ReviewModerationPanel: React.FC<ReviewModerationPanelProps> = ({
  adminAddress,
  flaggedReviews,
  onApprove,
  onRemove,
  onBanUser,
  className = '',
}) => {
  const [statusFilter, setStatusFilter] = useState<'all' | 'pending' | 'approved' | 'removed'>('all');
  const [isProcessing, setIsProcessing] = useState(false);
  const [error, setError] = useState<string | null>(null);

  // Filter reviews
  const filteredReviews = flaggedReviews.filter((review) => {
    if (statusFilter === 'all') return true;
    return review.status === statusFilter;
  });

  const handleAction = async (
    action: 'approve' | 'remove' | 'ban',
    reviewId: string,
    userAddress?: string
  ) => {
    setIsProcessing(true);
    setError(null);

    try {
      if (action === 'approve' && onApprove) {
        await onApprove(reviewId);
      } else if (action === 'remove' && onRemove) {
        await onRemove(reviewId);
      } else if (action === 'ban' && userAddress && onBanUser) {
        await onBanUser(userAddress);
      }
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Action failed');
    } finally {
      setIsProcessing(false);
    }
  };

  const formatTimestamp = (timestamp: number) => {
    return new Date(timestamp * 1000).toLocaleString();
  };

  const renderStars = (rating: number) => {
    return (
      <div className="stars">
        {Array.from({ length: 5 }).map((_, i) => (
          <span key={i} className={`star ${i < rating ? 'filled' : ''}`}>★</span>
        ))}
      </div>
    );
  };

  return (
    <div className={`moderation-panel ${className}`}>
      <div className="panel-header">
        <h2>Review Moderation</h2>
        <div className="admin-badge">
          Admin: {adminAddress.slice(0, 6)}...{adminAddress.slice(-4)}
        </div>
      </div>

      <div className="filter-bar">
        <button
          className={`filter-btn ${statusFilter === 'all' ? 'active' : ''}`}
          onClick={() => setStatusFilter('all')}
        >
          All ({flaggedReviews.length})
        </button>
        <button
          className={`filter-btn ${statusFilter === 'pending' ? 'active' : ''}`}
          onClick={() => setStatusFilter('pending')}
        >
          Pending ({flaggedReviews.filter((r) => r.status === 'pending').length})
        </button>
        <button
          className={`filter-btn ${statusFilter === 'approved' ? 'active' : ''}`}
          onClick={() => setStatusFilter('approved')}
        >
          Approved ({flaggedReviews.filter((r) => r.status === 'approved').length})
        </button>
        <button
          className={`filter-btn ${statusFilter === 'removed' ? 'active' : ''}`}
          onClick={() => setStatusFilter('removed')}
        >
          Removed ({flaggedReviews.filter((r) => r.status === 'removed').length})
        </button>
      </div>

      {error && (
        <div className="error-message">
          <svg width="16" height="16" viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg">
            <path d="M12 8V12M12 16H12.01M22 12C22 17.5228 17.5228 22 12 22C6.47715 22 2 17.5228 2 12C2 6.47715 6.47715 2 12 2C17.5228 2 22 6.47715 22 12Z" stroke="currentColor" strokeWidth="2" strokeLinecap="round"/>
          </svg>
          {error}
        </div>
      )}

      <div className="flagged-reviews">
        {filteredReviews.length === 0 ? (
          <div className="empty-state">
            <p>No {statusFilter !== 'all' ? statusFilter : 'flagged'} reviews</p>
          </div>
        ) : (
          filteredReviews.map((review) => (
            <div key={review.reviewId} className={`flagged-review status-${review.status}`}>
              <div className="review-content">
                <div className="review-header">
                  <div className="reviewer">
                    <strong>Reviewer:</strong>{' '}
                    {review.reviewerAddress.slice(0, 6)}...{review.reviewerAddress.slice(-4)}
                  </div>
                  {renderStars(review.reviewRating)}
                </div>

                <p className="review-text">{review.reviewText}</p>

                <div className="report-info">
                  <div className="report-reason">
                    <svg width="14" height="14" viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg">
                      <path d="M3 5C3 3.89543 3.89543 3 5 3H19C20.1046 3 21 3.89543 21 5V19C21 20.1046 20.1046 21 19 21H5C3.89543 21 3 20.1046 3 19V5Z" stroke="currentColor" strokeWidth="2"/>
                      <path d="M12 7V13M12 17H12.01" stroke="currentColor" strokeWidth="2" strokeLinecap="round"/>
                    </svg>
                    <strong>Reason:</strong> {review.reason}
                  </div>
                  <div className="report-meta">
                    Reported by {review.reporter.slice(0, 6)}...{review.reporter.slice(-4)} •{' '}
                    {formatTimestamp(review.timestamp)}
                  </div>
                </div>

                <div className="status-badge">{review.status}</div>
              </div>

              {review.status === 'pending' && (
                <div className="actions">
                  <button
                    className="action-btn approve-btn"
                    onClick={() => handleAction('approve', review.reviewId)}
                    disabled={isProcessing}
                  >
                    <svg width="16" height="16" viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg">
                      <path d="M9 12L11 14L15 10M21 12C21 16.9706 16.9706 21 12 21C7.02944 21 3 16.9706 3 12C3 7.02944 7.02944 3 12 3C16.9706 3 21 7.02944 21 12Z" stroke="currentColor" strokeWidth="2" strokeLinecap="round"/>
                    </svg>
                    Approve
                  </button>

                  <button
                    className="action-btn remove-btn"
                    onClick={() => handleAction('remove', review.reviewId)}
                    disabled={isProcessing}
                  >
                    <svg width="16" height="16" viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg">
                      <path d="M19 7L18.1327 19.1425C18.0579 20.1891 17.187 21 16.1378 21H7.86224C6.81296 21 5.94208 20.1891 5.86732 19.1425L5 7M10 11V17M14 11V17M15 7V4C15 3.44772 14.5523 3 14 3H10C9.44772 3 9 3.44772 9 4V7M4 7H20" stroke="currentColor" strokeWidth="2" strokeLinecap="round"/>
                    </svg>
                    Remove
                  </button>

                  <button
                    className="action-btn ban-btn"
                    onClick={() => handleAction('ban', review.reviewId, review.reviewerAddress)}
                    disabled={isProcessing}
                  >
                    <svg width="16" height="16" viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg">
                      <path d="M18.364 18.364C21.8787 14.8492 21.8787 9.15076 18.364 5.63604C14.8492 2.12132 9.15076 2.12132 5.63604 5.63604M18.364 18.364C14.8492 21.8787 9.15076 21.8787 5.63604 18.364C2.12132 14.8492 2.12132 9.15076 5.63604 5.63604M18.364 18.364L5.63604 5.63604" stroke="currentColor" strokeWidth="2" strokeLinecap="round"/>
                    </svg>
                    Ban User
                  </button>
                </div>
              )}
            </div>
          ))
        )}
      </div>

      <style jsx>{`
        .moderation-panel {
          background: white;
          border: 1px solid #e5e7eb;
          border-radius: 12px;
          padding: 24px;
        }

        .panel-header {
          display: flex;
          justify-content: space-between;
          align-items: center;
          margin-bottom: 24px;
        }

        .panel-header h2 {
          margin: 0;
          font-size: 24px;
          font-weight: 700;
          color: #111827;
        }

        .admin-badge {
          background: #eff6ff;
          color: #1e40af;
          padding: 6px 12px;
          border-radius: 6px;
          font-size: 13px;
          font-weight: 600;
        }

        .filter-bar {
          display: flex;
          gap: 8px;
          margin-bottom: 24px;
          flex-wrap: wrap;
        }

        .filter-btn {
          padding: 8px 16px;
          background: white;
          border: 1px solid #e5e7eb;
          border-radius: 8px;
          color: #6b7280;
          font-weight: 500;
          cursor: pointer;
          transition: all 0.2s ease;
        }

        .filter-btn:hover {
          background: #f9fafb;
        }

        .filter-btn.active {
          background: #3b82f6;
          border-color: #3b82f6;
          color: white;
        }

        .error-message {
          display: flex;
          align-items: center;
          gap: 10px;
          background: #fef2f2;
          border: 1px solid #fecaca;
          border-radius: 8px;
          padding: 12px 16px;
          margin-bottom: 16px;
          color: #991b1b;
          font-size: 14px;
        }

        .flagged-reviews {
          display: flex;
          flex-direction: column;
          gap: 16px;
        }

        .flagged-review {
          border: 1px solid #e5e7eb;
          border-radius: 12px;
          padding: 20px;
          position: relative;
        }

        .flagged-review.status-pending {
          border-left: 4px solid #f59e0b;
        }

        .flagged-review.status-approved {
          border-left: 4px solid #10b981;
          opacity: 0.7;
        }

        .flagged-review.status-removed {
          border-left: 4px solid #ef4444;
          opacity: 0.7;
        }

        .review-header {
          display: flex;
          justify-content: space-between;
          align-items: center;
          margin-bottom: 12px;
        }

        .reviewer {
          color: #374151;
          font-size: 14px;
        }

        .stars {
          display: flex;
          gap: 2px;
        }

        .star {
          color: #d1d5db;
          font-size: 14px;
        }

        .star.filled {
          color: #fbbf24;
        }

        .review-text {
          color: #111827;
          line-height: 1.6;
          margin: 0 0 16px 0;
        }

        .report-info {
          background: #fef3c7;
          border: 1px solid #fde68a;
          border-radius: 8px;
          padding: 12px;
          margin-bottom: 16px;
        }

        .report-reason {
          display: flex;
          align-items: center;
          gap: 8px;
          color: #78350f;
          font-size: 14px;
          margin-bottom: 6px;
        }

        .report-meta {
          color: #92400e;
          font-size: 12px;
        }

        .status-badge {
          position: absolute;
          top: 20px;
          right: 20px;
          padding: 4px 10px;
          border-radius: 6px;
          font-size: 11px;
          font-weight: 700;
          text-transform: uppercase;
          background: #f3f4f6;
          color: #6b7280;
        }

        .status-pending .status-badge {
          background: #fef3c7;
          color: #92400e;
        }

        .status-approved .status-badge {
          background: #d1fae5;
          color: #065f46;
        }

        .status-removed .status-badge {
          background: #fee2e2;
          color: #991b1b;
        }

        .actions {
          display: flex;
          gap: 8px;
          flex-wrap: wrap;
        }

        .action-btn {
          display: flex;
          align-items: center;
          gap: 6px;
          padding: 8px 16px;
          border: none;
          border-radius: 8px;
          font-weight: 600;
          cursor: pointer;
          font-size: 14px;
          transition: all 0.2s ease;
        }

        .approve-btn {
          background: #d1fae5;
          color: #065f46;
        }

        .approve-btn:hover:not(:disabled) {
          background: #a7f3d0;
        }

        .remove-btn {
          background: #fee2e2;
          color: #991b1b;
        }

        .remove-btn:hover:not(:disabled) {
          background: #fecaca;
        }

        .ban-btn {
          background: #f3f4f6;
          color: #374151;
        }

        .ban-btn:hover:not(:disabled) {
          background: #e5e7eb;
        }

        .action-btn:disabled {
          opacity: 0.5;
          cursor: not-allowed;
        }

        .empty-state {
          padding: 40px;
          text-align: center;
          color: #9ca3af;
        }

        @media (max-width: 640px) {
          .panel-header {
            flex-direction: column;
            align-items: flex-start;
            gap: 12px;
          }

          .review-header {
            flex-direction: column;
            align-items: flex-start;
            gap: 8px;
          }

          .actions {
            flex-direction: column;
          }

          .action-btn {
            width: 100%;
          }
        }
      `}</style>
    </div>
  );
};

export default ReviewModerationPanel;
