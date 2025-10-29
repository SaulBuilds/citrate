/**
 * Pagination Component
 *
 * Pagination controls for navigating search results.
 * Features:
 * - Previous/Next buttons
 * - Page number buttons with ellipsis
 * - Go to first/last page
 * - Total results count
 * - Responsive mobile layout
 * - Keyboard accessible
 * - Disabled state handling
 */

import React from 'react';

export interface PaginationProps {
  currentPage: number;      // Zero-indexed
  totalPages: number;
  onPageChange: (page: number) => void;
  totalResults?: number;
  resultsPerPage?: number;
  className?: string;
  disabled?: boolean;
  maxPageButtons?: number;  // Max page number buttons to show (default: 7)
}

export const Pagination: React.FC<PaginationProps> = ({
  currentPage,
  totalPages,
  onPageChange,
  totalResults,
  resultsPerPage = 20,
  className = '',
  disabled = false,
  maxPageButtons = 7
}) => {
  // If no pages, don't render
  if (totalPages <= 0) {
    return null;
  }

  // Calculate page range to display
  const getPageRange = (): (number | 'ellipsis')[] => {
    if (totalPages <= maxPageButtons) {
      // Show all pages
      return Array.from({ length: totalPages }, (_, i) => i);
    }

    const sideButtons = Math.floor((maxPageButtons - 3) / 2); // Reserve 3 for first, last, and current
    const pages: (number | 'ellipsis')[] = [];

    // Always show first page
    pages.push(0);

    // Calculate range around current page
    let rangeStart = Math.max(1, currentPage - sideButtons);
    let rangeEnd = Math.min(totalPages - 2, currentPage + sideButtons);

    // Adjust if we're near the start or end
    if (currentPage <= sideButtons) {
      rangeEnd = Math.min(totalPages - 2, maxPageButtons - 2);
    }
    if (currentPage >= totalPages - sideButtons - 1) {
      rangeStart = Math.max(1, totalPages - maxPageButtons + 1);
    }

    // Add ellipsis after first if needed
    if (rangeStart > 1) {
      pages.push('ellipsis');
    }

    // Add middle pages
    for (let i = rangeStart; i <= rangeEnd; i++) {
      pages.push(i);
    }

    // Add ellipsis before last if needed
    if (rangeEnd < totalPages - 1) {
      pages.push('ellipsis');
    }

    // Always show last page
    pages.push(totalPages - 1);

    return pages;
  };

  const pageRange = getPageRange();
  const hasPrev = currentPage > 0;
  const hasNext = currentPage < totalPages - 1;

  const handlePageClick = (page: number) => {
    if (!disabled && page !== currentPage && page >= 0 && page < totalPages) {
      onPageChange(page);
    }
  };

  const handlePrev = () => {
    if (hasPrev) {
      handlePageClick(currentPage - 1);
    }
  };

  const handleNext = () => {
    if (hasNext) {
      handlePageClick(currentPage + 1);
    }
  };

  const handleFirst = () => {
    handlePageClick(0);
  };

  const handleLast = () => {
    handlePageClick(totalPages - 1);
  };

  // Calculate result range
  const startResult = currentPage * resultsPerPage + 1;
  const endResult = Math.min((currentPage + 1) * resultsPerPage, totalResults || 0);

  return (
    <div className={`pagination ${className}`}>
      {totalResults !== undefined && (
        <div className="pagination-info">
          Showing {startResult.toLocaleString()}-{endResult.toLocaleString()} of{' '}
          {totalResults.toLocaleString()} results
        </div>
      )}

      <div className="pagination-controls">
        {/* Previous button */}
        <button
          type="button"
          onClick={handlePrev}
          disabled={!hasPrev || disabled}
          className="pagination-button pagination-prev"
          aria-label="Previous page"
        >
          <svg
            xmlns="http://www.w3.org/2000/svg"
            width="18"
            height="18"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            strokeWidth="2"
            strokeLinecap="round"
            strokeLinejoin="round"
          >
            <polyline points="15 18 9 12 15 6"></polyline>
          </svg>
          <span className="button-label">Previous</span>
        </button>

        {/* Page numbers */}
        <div className="pagination-pages">
          {pageRange.map((page, index) => {
            if (page === 'ellipsis') {
              return (
                <span key={`ellipsis-${index}`} className="pagination-ellipsis">
                  ...
                </span>
              );
            }

            const isActive = page === currentPage;
            return (
              <button
                key={page}
                type="button"
                onClick={() => handlePageClick(page)}
                disabled={disabled}
                className={`pagination-button pagination-page ${isActive ? 'active' : ''}`}
                aria-label={`Go to page ${page + 1}`}
                aria-current={isActive ? 'page' : undefined}
              >
                {page + 1}
              </button>
            );
          })}
        </div>

        {/* Next button */}
        <button
          type="button"
          onClick={handleNext}
          disabled={!hasNext || disabled}
          className="pagination-button pagination-next"
          aria-label="Next page"
        >
          <span className="button-label">Next</span>
          <svg
            xmlns="http://www.w3.org/2000/svg"
            width="18"
            height="18"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            strokeWidth="2"
            strokeLinecap="round"
            strokeLinejoin="round"
          >
            <polyline points="9 18 15 12 9 6"></polyline>
          </svg>
        </button>
      </div>

      <style jsx>{`
        .pagination {
          display: flex;
          flex-direction: column;
          align-items: center;
          gap: 16px;
          padding: 24px 0;
        }

        .pagination-info {
          font-size: 14px;
          color: #6b7280;
          font-weight: 500;
        }

        .pagination-controls {
          display: flex;
          align-items: center;
          gap: 8px;
        }

        .pagination-button {
          display: flex;
          align-items: center;
          justify-content: center;
          gap: 6px;
          min-width: 40px;
          height: 40px;
          padding: 0 12px;
          background: white;
          border: 1px solid #e5e7eb;
          border-radius: 6px;
          font-size: 14px;
          font-weight: 500;
          color: #374151;
          cursor: pointer;
          transition: all 0.2s;
        }

        .pagination-button:hover:not(:disabled) {
          border-color: #3b82f6;
          background: #f9fafb;
          color: #3b82f6;
        }

        .pagination-button:disabled {
          opacity: 0.4;
          cursor: not-allowed;
        }

        .pagination-button.active {
          background: #3b82f6;
          border-color: #3b82f6;
          color: white;
          font-weight: 600;
        }

        .pagination-button.active:hover {
          background: #2563eb;
          border-color: #2563eb;
        }

        .pagination-prev,
        .pagination-next {
          padding: 0 16px;
        }

        .pagination-pages {
          display: flex;
          align-items: center;
          gap: 4px;
        }

        .pagination-page {
          min-width: 40px;
          padding: 0 8px;
        }

        .pagination-ellipsis {
          display: flex;
          align-items: center;
          justify-content: center;
          min-width: 40px;
          height: 40px;
          color: #9ca3af;
          font-weight: 600;
          user-select: none;
        }

        /* Mobile responsive */
        @media (max-width: 640px) {
          .pagination {
            padding: 16px 0;
          }

          .pagination-info {
            font-size: 13px;
          }

          .pagination-controls {
            width: 100%;
            justify-content: space-between;
            gap: 4px;
          }

          .button-label {
            display: none;
          }

          .pagination-prev,
          .pagination-next {
            padding: 0 12px;
            min-width: 40px;
          }

          .pagination-pages {
            gap: 2px;
          }

          .pagination-page {
            min-width: 36px;
            height: 36px;
            font-size: 13px;
          }

          .pagination-ellipsis {
            min-width: 28px;
            height: 36px;
            font-size: 13px;
          }

          .pagination-button {
            height: 36px;
            min-width: 36px;
          }
        }

        /* Tablet */
        @media (min-width: 641px) and (max-width: 1024px) {
          .pagination-pages {
            gap: 4px;
          }
        }
      `}</style>
    </div>
  );
};

export default Pagination;
