/**
 * ReviewList Component
 *
 * Displays a list of reviews with sorting, filtering, and pagination.
 * Features:
 * - Sort dropdown (Most Helpful, Recent, Highest/Lowest Rating)
 * - Filter by rating (5 stars, 4 stars, etc.)
 * - Pagination
 * - Empty state
 * - Loading skeleton states
 */

import React, { useState } from 'react';
import { Review, ReviewSortOption } from '../../utils/types/reviews';
import ReviewCard from './ReviewCard';

export interface ReviewListProps {
  modelId: string;
  reviews: Review[];
  isLoading: boolean;
  onSort?: (sortBy: ReviewSortOption) => void;
  onFilterRating?: (rating: number | null) => void;
  onVote?: (reviewId: string, isHelpful: boolean) => void;
  onReport?: (reviewId: string, reason: string) => void;
  className?: string;
}

const REVIEWS_PER_PAGE = 10;

export const ReviewList: React.FC<ReviewListProps> = ({
  modelId,
  reviews,
  isLoading,
  onSort,
  onFilterRating,
  onVote,
  onReport,
  className = '',
}) => {
  const [currentPage, setCurrentPage] = useState(0);
  const [sortBy, setSortBy] = useState<ReviewSortOption>('mostHelpful');
  const [filterRating, setFilterRating] = useState<number | null>(null);

  const handleSortChange = (newSort: ReviewSortOption) => {
    setSortBy(newSort);
    setCurrentPage(0);
    if (onSort) onSort(newSort);
  };

  const handleFilterChange = (rating: number | null) => {
    setFilterRating(rating);
    setCurrentPage(0);
    if (onFilterRating) onFilterRating(rating);
  };

  const startIndex = currentPage * REVIEWS_PER_PAGE;
  const endIndex = startIndex + REVIEWS_PER_PAGE;
  const paginatedReviews = reviews.slice(startIndex, endIndex);
  const totalPages = Math.ceil(reviews.length / REVIEWS_PER_PAGE);

  const renderRatingDistribution = () => {
    const distribution = { 5: 0, 4: 0, 3: 0, 2: 0, 1: 0 };
    reviews.forEach((review) => {
      distribution[review.rating as keyof typeof distribution]++;
    });

    return (
      <div className="rating-distribution">
        {[5, 4, 3, 2, 1].map((rating) => {
          const count = distribution[rating as keyof typeof distribution];
          const percentage = reviews.length > 0 ? (count / reviews.length) * 100 : 0;

          return (
            <button
              key={rating}
              className={`rating-bar ${filterRating === rating ? 'active' : ''}`}
              onClick={() => handleFilterChange(filterRating === rating ? null : rating)}
            >
              <div className="rating-label">
                <span className="stars">
                  {'★'.repeat(rating)}
                  {'☆'.repeat(5 - rating)}
                </span>
                <span className="count">({count})</span>
              </div>
              <div className="bar-container">
                <div className="bar-fill" style={{ width: `${percentage}%` }} />
              </div>
            </button>
          );
        })}
      </div>
    );
  };

  const renderSkeleton = () => (
    <div className="skeleton-container">
      {Array.from({ length: 3 }).map((_, i) => (
        <div key={i} className="skeleton-card">
          <div className="skeleton-header">
            <div className="skeleton-avatar" />
            <div className="skeleton-text-short" />
          </div>
          <div className="skeleton-text-long" />
          <div className="skeleton-text-medium" />
        </div>
      ))}
    </div>
  );

  const renderEmpty = () => (
    <div className="empty-state">
      <svg
        width="64"
        height="64"
        viewBox="0 0 24 24"
        fill="none"
        xmlns="http://www.w3.org/2000/svg"
      >
        <path
          d="M9 12L11 14L15 10M21 12C21 16.9706 16.9706 21 12 21C7.02944 21 3 16.9706 3 12C3 7.02944 7.02944 3 12 3C16.9706 3 21 7.02944 21 12Z"
          stroke="#d1d5db"
          strokeWidth="2"
          strokeLinecap="round"
          strokeLinejoin="round"
        />
      </svg>
      <h3>No reviews yet</h3>
      <p>Be the first to review this model</p>
    </div>
  );

  return (
    <div className={`review-list ${className}`}>
      <div className="review-list-header">
        <h2>Reviews ({reviews.length})</h2>

        <div className="controls">
          <select
            className="sort-select"
            value={sortBy}
            onChange={(e) => handleSortChange(e.target.value as ReviewSortOption)}
          >
            <option value="mostHelpful">Most Helpful</option>
            <option value="recent">Most Recent</option>
            <option value="highestRating">Highest Rating</option>
            <option value="lowestRating">Lowest Rating</option>
          </select>
        </div>
      </div>

      {reviews.length > 0 && renderRatingDistribution()}

      {filterRating && (
        <div className="active-filter">
          <span>Showing {reviews.length} {reviews.length === 1 ? 'review' : 'reviews'} with {filterRating} stars</span>
          <button onClick={() => handleFilterChange(null)}>Clear filter</button>
        </div>
      )}

      <div className="reviews-container">
        {isLoading ? (
          renderSkeleton()
        ) : reviews.length === 0 ? (
          renderEmpty()
        ) : (
          <>
            {paginatedReviews.map((review) => (
              <ReviewCard
                key={review.reviewId}
                review={review}
                onVote={onVote}
                onReport={onReport}
              />
            ))}

            {totalPages > 1 && (
              <div className="pagination">
                <button
                  className="page-btn"
                  onClick={() => setCurrentPage(Math.max(0, currentPage - 1))}
                  disabled={currentPage === 0}
                >
                  Previous
                </button>

                <div className="page-numbers">
                  {Array.from({ length: totalPages }).map((_, i) => (
                    <button
                      key={i}
                      className={`page-number ${i === currentPage ? 'active' : ''}`}
                      onClick={() => setCurrentPage(i)}
                    >
                      {i + 1}
                    </button>
                  ))}
                </div>

                <button
                  className="page-btn"
                  onClick={() => setCurrentPage(Math.min(totalPages - 1, currentPage + 1))}
                  disabled={currentPage === totalPages - 1}
                >
                  Next
                </button>
              </div>
            )}
          </>
        )}
      </div>

      <style jsx>{`
        .review-list {
          width: 100%;
        }

        .review-list-header {
          display: flex;
          justify-content: space-between;
          align-items: center;
          margin-bottom: 24px;
        }

        .review-list-header h2 {
          margin: 0;
          font-size: 24px;
          font-weight: 700;
          color: #111827;
        }

        .controls {
          display: flex;
          gap: 12px;
        }

        .sort-select {
          padding: 8px 32px 8px 12px;
          border: 1px solid #e5e7eb;
          border-radius: 8px;
          background: white;
          font-size: 14px;
          color: #374151;
          cursor: pointer;
          appearance: none;
          background-image: url("data:image/svg+xml,%3Csvg width='12' height='8' viewBox='0 0 12 8' fill='none' xmlns='http://www.w3.org/2000/svg'%3E%3Cpath d='M1 1L6 6L11 1' stroke='%236b7280' stroke-width='2' stroke-linecap='round'/%3E%3C/svg%3E");
          background-repeat: no-repeat;
          background-position: right 12px center;
        }

        .rating-distribution {
          background: #f9fafb;
          border: 1px solid #e5e7eb;
          border-radius: 12px;
          padding: 20px;
          margin-bottom: 24px;
        }

        .rating-bar {
          display: flex;
          align-items: center;
          gap: 16px;
          padding: 8px 0;
          background: none;
          border: none;
          width: 100%;
          cursor: pointer;
          transition: opacity 0.2s ease;
        }

        .rating-bar:hover {
          opacity: 0.8;
        }

        .rating-bar.active {
          font-weight: 600;
        }

        .rating-label {
          display: flex;
          align-items: center;
          gap: 8px;
          min-width: 120px;
        }

        .stars {
          color: #fbbf24;
          font-size: 14px;
        }

        .count {
          color: #6b7280;
          font-size: 13px;
        }

        .bar-container {
          flex: 1;
          height: 8px;
          background: #e5e7eb;
          border-radius: 4px;
          overflow: hidden;
        }

        .bar-fill {
          height: 100%;
          background: #3b82f6;
          transition: width 0.3s ease;
        }

        .active-filter {
          display: flex;
          align-items: center;
          justify-content: space-between;
          background: #eff6ff;
          border: 1px solid #bfdbfe;
          border-radius: 8px;
          padding: 12px 16px;
          margin-bottom: 20px;
          color: #1e40af;
          font-size: 14px;
        }

        .active-filter button {
          background: none;
          border: none;
          color: #3b82f6;
          font-weight: 600;
          cursor: pointer;
          text-decoration: underline;
        }

        .reviews-container {
          display: flex;
          flex-direction: column;
          gap: 16px;
        }

        .skeleton-container {
          display: flex;
          flex-direction: column;
          gap: 16px;
        }

        .skeleton-card {
          background: white;
          border: 1px solid #e5e7eb;
          border-radius: 12px;
          padding: 20px;
        }

        .skeleton-header {
          display: flex;
          align-items: center;
          gap: 12px;
          margin-bottom: 16px;
        }

        .skeleton-avatar {
          width: 40px;
          height: 40px;
          background: #f3f4f6;
          border-radius: 50%;
          animation: pulse 1.5s ease-in-out infinite;
        }

        .skeleton-text-short {
          width: 120px;
          height: 16px;
          background: #f3f4f6;
          border-radius: 4px;
          animation: pulse 1.5s ease-in-out infinite;
        }

        .skeleton-text-medium {
          width: 60%;
          height: 14px;
          background: #f3f4f6;
          border-radius: 4px;
          margin-top: 12px;
          animation: pulse 1.5s ease-in-out infinite;
        }

        .skeleton-text-long {
          width: 100%;
          height: 14px;
          background: #f3f4f6;
          border-radius: 4px;
          margin-top: 12px;
          animation: pulse 1.5s ease-in-out infinite;
        }

        @keyframes pulse {
          0%, 100% {
            opacity: 1;
          }
          50% {
            opacity: 0.5;
          }
        }

        .empty-state {
          display: flex;
          flex-direction: column;
          align-items: center;
          justify-content: center;
          padding: 60px 20px;
          text-align: center;
        }

        .empty-state svg {
          margin-bottom: 16px;
        }

        .empty-state h3 {
          margin: 0 0 8px 0;
          font-size: 18px;
          font-weight: 600;
          color: #111827;
        }

        .empty-state p {
          margin: 0;
          color: #6b7280;
          font-size: 14px;
        }

        .pagination {
          display: flex;
          justify-content: center;
          align-items: center;
          gap: 8px;
          margin-top: 32px;
        }

        .page-btn {
          padding: 8px 16px;
          background: white;
          border: 1px solid #e5e7eb;
          border-radius: 8px;
          color: #374151;
          font-weight: 500;
          cursor: pointer;
          transition: all 0.2s ease;
        }

        .page-btn:hover:not(:disabled) {
          background: #f9fafb;
          border-color: #d1d5db;
        }

        .page-btn:disabled {
          opacity: 0.5;
          cursor: not-allowed;
        }

        .page-numbers {
          display: flex;
          gap: 4px;
        }

        .page-number {
          min-width: 40px;
          padding: 8px 12px;
          background: white;
          border: 1px solid #e5e7eb;
          border-radius: 8px;
          color: #374151;
          font-weight: 500;
          cursor: pointer;
          transition: all 0.2s ease;
        }

        .page-number:hover {
          background: #f9fafb;
        }

        .page-number.active {
          background: #3b82f6;
          border-color: #3b82f6;
          color: white;
        }

        @media (max-width: 640px) {
          .review-list-header {
            flex-direction: column;
            align-items: flex-start;
            gap: 16px;
          }

          .rating-bar {
            flex-direction: column;
            align-items: flex-start;
          }

          .bar-container {
            width: 100%;
          }

          .pagination {
            flex-wrap: wrap;
          }
        }
      `}</style>
    </div>
  );
};

export default ReviewList;
