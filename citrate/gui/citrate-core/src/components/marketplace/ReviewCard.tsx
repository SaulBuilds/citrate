/**
 * ReviewCard Component
 *
 * Displays an individual review with:
 * - Star rating display
 * - Reviewer information (address or name)
 * - Verified purchase badge
 * - Review text with "Read more" expansion
 * - Helpful/Unhelpful voting buttons
 * - Report abuse functionality
 * - Relative timestamp
 */

import React, { useState } from 'react';
import { Review } from '../../utils/types/reviews';

export interface ReviewCardProps {
  review: Review;
  onVote?: (reviewId: string, isHelpful: boolean) => void;
  onReport?: (reviewId: string, reason: string) => void;
  className?: string;
}

export const ReviewCard: React.FC<ReviewCardProps> = ({
  review,
  onVote,
  onReport,
  className = '',
}) => {
  const [isExpanded, setIsExpanded] = useState(false);
  const [showReportDialog, setShowReportDialog] = useState(false);
  const [hasVoted, setHasVoted] = useState(false);

  const MAX_LENGTH = 300;
  const needsExpansion = review.text.length > MAX_LENGTH;
  const displayText = !isExpanded && needsExpansion
    ? review.text.slice(0, MAX_LENGTH) + '...'
    : review.text;

  const handleVote = (isHelpful: boolean) => {
    if (hasVoted || !onVote) return;
    onVote(review.reviewId, isHelpful);
    setHasVoted(true);
  };

  const handleReport = (reason: string) => {
    if (onReport) {
      onReport(review.reviewId, reason);
      setShowReportDialog(false);
    }
  };

  const renderStars = (rating: number) => {
    return (
      <div className="stars">
        {Array.from({ length: 5 }).map((_, i) => (
          <span key={i} className={`star ${i < rating ? 'filled' : ''}`}>
            ★
          </span>
        ))}
      </div>
    );
  };

  const formatTimestamp = (timestamp: number) => {
    const now = Date.now();
    const diff = now - timestamp * 1000;
    const seconds = Math.floor(diff / 1000);
    const minutes = Math.floor(seconds / 60);
    const hours = Math.floor(minutes / 60);
    const days = Math.floor(hours / 24);
    const months = Math.floor(days / 30);
    const years = Math.floor(days / 365);

    if (years > 0) return `${years} year${years > 1 ? 's' : ''} ago`;
    if (months > 0) return `${months} month${months > 1 ? 's' : ''} ago`;
    if (days > 0) return `${days} day${days > 1 ? 's' : ''} ago`;
    if (hours > 0) return `${hours} hour${hours > 1 ? 's' : ''} ago`;
    if (minutes > 0) return `${minutes} minute${minutes > 1 ? 's' : ''} ago`;
    return 'Just now';
  };

  const truncateAddress = (address: string) => {
    return `${address.slice(0, 6)}...${address.slice(-4)}`;
  };

  return (
    <div className={`review-card ${className}`}>
      <div className="review-header">
        <div className="reviewer-info">
          <div className="reviewer-name">
            {review.reviewerName || truncateAddress(review.reviewer)}
          </div>
          {review.verifiedPurchase && (
            <div className="verified-badge">
              <svg
                width="14"
                height="14"
                viewBox="0 0 24 24"
                fill="none"
                xmlns="http://www.w3.org/2000/svg"
              >
                <path
                  d="M9 12L11 14L15 10M21 12C21 16.9706 16.9706 21 12 21C7.02944 21 3 16.9706 3 12C3 7.02944 7.02944 3 12 3C16.9706 3 21 7.02944 21 12Z"
                  stroke="currentColor"
                  strokeWidth="2"
                  strokeLinecap="round"
                  strokeLinejoin="round"
                />
              </svg>
              Verified Purchase
            </div>
          )}
        </div>

        <div className="review-meta">
          {renderStars(review.rating)}
          <span className="review-date">{formatTimestamp(review.timestamp)}</span>
        </div>
      </div>

      <div className="review-body">
        <p className="review-text">{displayText}</p>
        {needsExpansion && (
          <button
            className="read-more-btn"
            onClick={() => setIsExpanded(!isExpanded)}
          >
            {isExpanded ? 'Show less' : 'Read more'}
          </button>
        )}
      </div>

      <div className="review-actions">
        <div className="vote-buttons">
          <button
            className={`vote-btn ${hasVoted ? 'disabled' : ''}`}
            onClick={() => handleVote(true)}
            disabled={hasVoted}
            aria-label="Mark as helpful"
          >
            <svg
              width="16"
              height="16"
              viewBox="0 0 24 24"
              fill="none"
              xmlns="http://www.w3.org/2000/svg"
            >
              <path
                d="M7 22V11M2 13V20C2 21.1046 2.89543 22 4 22H17.4262C18.907 22 20.1662 20.9197 20.3914 19.4562L21.4683 12.4562C21.7479 10.6389 20.3418 9 18.5032 9H15V4C15 2.89543 14.1046 2 13 2C12.4477 2 12 2.44772 12 3V3.93551C12 4.3733 11.8684 4.80057 11.6213 5.16318L7.78885 10.4432C7.43394 10.9628 7 11.3967 7 12V12Z"
                stroke="currentColor"
                strokeWidth="2"
                strokeLinecap="round"
                strokeLinejoin="round"
              />
            </svg>
            Helpful ({review.helpfulVotes})
          </button>

          <button
            className={`vote-btn ${hasVoted ? 'disabled' : ''}`}
            onClick={() => handleVote(false)}
            disabled={hasVoted}
            aria-label="Mark as unhelpful"
          >
            <svg
              width="16"
              height="16"
              viewBox="0 0 24 24"
              fill="none"
              xmlns="http://www.w3.org/2000/svg"
            >
              <path
                d="M17 2V13M22 11V4C22 2.89543 21.1046 2 20 2H6.57376C5.09297 2 3.83375 3.08027 3.60864 4.54383L2.53174 11.5438C2.25208 13.3611 3.65824 15 5.49681 15H9V20C9 21.1046 9.89543 22 11 22C11.5523 22 12 21.5523 12 21V20.0645C12 19.6267 12.1316 19.1994 12.3787 18.8368L16.2111 13.5568C16.5661 13.0372 17 12.6033 17 12V12Z"
                stroke="currentColor"
                strokeWidth="2"
                strokeLinecap="round"
                strokeLinejoin="round"
              />
            </svg>
            ({review.unhelpfulVotes})
          </button>
        </div>

        <button
          className="report-btn"
          onClick={() => setShowReportDialog(true)}
          aria-label="Report review"
        >
          <svg
            width="16"
            height="16"
            viewBox="0 0 24 24"
            fill="none"
            xmlns="http://www.w3.org/2000/svg"
          >
            <path
              d="M3 5C3 3.89543 3.89543 3 5 3H19C20.1046 3 21 3.89543 21 5V19C21 20.1046 20.1046 21 19 21H5C3.89543 21 3 20.1046 3 19V5Z"
              stroke="currentColor"
              strokeWidth="2"
            />
            <path
              d="M12 7V13M12 17H12.01"
              stroke="currentColor"
              strokeWidth="2"
              strokeLinecap="round"
            />
          </svg>
          Report
        </button>
      </div>

      {showReportDialog && (
        <div className="report-dialog-overlay" onClick={() => setShowReportDialog(false)}>
          <div className="report-dialog" onClick={(e) => e.stopPropagation()}>
            <h3>Report Review</h3>
            <p>Why are you reporting this review?</p>
            <div className="report-reasons">
              {[
                'Spam or misleading content',
                'Offensive language',
                'False information',
                'Duplicate review',
                'Not relevant to the model',
                'Other',
              ].map((reason) => (
                <button
                  key={reason}
                  className="reason-btn"
                  onClick={() => handleReport(reason)}
                >
                  {reason}
                </button>
              ))}
            </div>
            <button className="cancel-btn" onClick={() => setShowReportDialog(false)}>
              Cancel
            </button>
          </div>
        </div>
      )}

      <style jsx>{`
        .review-card {
          background: white;
          border: 1px solid #e5e7eb;
          border-radius: 12px;
          padding: 20px;
          transition: box-shadow 0.2s ease;
        }

        .review-card:hover {
          box-shadow: 0 4px 12px rgba(0, 0, 0, 0.08);
        }

        .review-header {
          display: flex;
          justify-content: space-between;
          align-items: flex-start;
          margin-bottom: 16px;
          gap: 16px;
        }

        .reviewer-info {
          display: flex;
          align-items: center;
          gap: 10px;
        }

        .reviewer-name {
          font-weight: 600;
          color: #111827;
          font-size: 15px;
        }

        .verified-badge {
          display: flex;
          align-items: center;
          gap: 4px;
          background: #d1fae5;
          color: #065f46;
          padding: 3px 8px;
          border-radius: 6px;
          font-size: 12px;
          font-weight: 600;
        }

        .verified-badge svg {
          color: #10b981;
        }

        .review-meta {
          display: flex;
          flex-direction: column;
          align-items: flex-end;
          gap: 4px;
        }

        .stars {
          display: flex;
          gap: 2px;
        }

        .star {
          color: #d1d5db;
          font-size: 16px;
        }

        .star.filled {
          color: #fbbf24;
        }

        .review-date {
          font-size: 13px;
          color: #9ca3af;
        }

        .review-body {
          margin-bottom: 16px;
        }

        .review-text {
          color: #374151;
          line-height: 1.6;
          margin: 0;
          white-space: pre-wrap;
          word-wrap: break-word;
        }

        .read-more-btn {
          background: none;
          border: none;
          color: #3b82f6;
          font-weight: 600;
          cursor: pointer;
          padding: 4px 0;
          margin-top: 8px;
          font-size: 14px;
        }

        .read-more-btn:hover {
          color: #2563eb;
          text-decoration: underline;
        }

        .review-actions {
          display: flex;
          justify-content: space-between;
          align-items: center;
          padding-top: 16px;
          border-top: 1px solid #f3f4f6;
        }

        .vote-buttons {
          display: flex;
          gap: 12px;
        }

        .vote-btn {
          display: flex;
          align-items: center;
          gap: 6px;
          background: #f9fafb;
          border: 1px solid #e5e7eb;
          border-radius: 8px;
          padding: 6px 12px;
          color: #6b7280;
          font-size: 14px;
          font-weight: 500;
          cursor: pointer;
          transition: all 0.2s ease;
        }

        .vote-btn:hover:not(.disabled) {
          background: #f3f4f6;
          border-color: #d1d5db;
          color: #374151;
        }

        .vote-btn.disabled {
          cursor: not-allowed;
          opacity: 0.5;
        }

        .report-btn {
          display: flex;
          align-items: center;
          gap: 6px;
          background: none;
          border: none;
          color: #9ca3af;
          font-size: 14px;
          font-weight: 500;
          cursor: pointer;
          padding: 6px;
          transition: color 0.2s ease;
        }

        .report-btn:hover {
          color: #ef4444;
        }

        .report-dialog-overlay {
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

        .report-dialog {
          background: white;
          border-radius: 16px;
          padding: 24px;
          max-width: 400px;
          width: 90%;
          box-shadow: 0 20px 25px -5px rgba(0, 0, 0, 0.1);
        }

        .report-dialog h3 {
          margin: 0 0 8px 0;
          font-size: 18px;
          font-weight: 700;
          color: #111827;
        }

        .report-dialog p {
          margin: 0 0 16px 0;
          color: #6b7280;
          font-size: 14px;
        }

        .report-reasons {
          display: flex;
          flex-direction: column;
          gap: 8px;
          margin-bottom: 16px;
        }

        .reason-btn {
          background: #f9fafb;
          border: 1px solid #e5e7eb;
          border-radius: 8px;
          padding: 12px 16px;
          text-align: left;
          cursor: pointer;
          font-size: 14px;
          color: #374151;
          transition: all 0.2s ease;
        }

        .reason-btn:hover {
          background: #f3f4f6;
          border-color: #d1d5db;
        }

        .cancel-btn {
          width: 100%;
          background: white;
          border: 1px solid #e5e7eb;
          border-radius: 8px;
          padding: 10px;
          cursor: pointer;
          font-weight: 600;
          color: #6b7280;
        }

        .cancel-btn:hover {
          background: #f9fafb;
        }

        @media (max-width: 640px) {
          .review-header {
            flex-direction: column;
            gap: 12px;
          }

          .review-meta {
            align-items: flex-start;
          }

          .review-actions {
            flex-direction: column;
            align-items: stretch;
            gap: 12px;
          }

          .vote-buttons {
            justify-content: space-between;
          }
        }
      `}</style>
    </div>
  );
};

export default ReviewCard;
