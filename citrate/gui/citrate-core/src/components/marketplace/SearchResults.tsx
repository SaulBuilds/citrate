/**
 * SearchResults Component
 *
 * Displays a grid of ModelCard components with loading and empty states.
 * Features:
 * - Responsive grid layout (1-3 columns based on screen size)
 * - Loading skeleton placeholders
 * - Empty state with helpful messaging
 * - Error state handling
 * - Grid animations
 * - Result count and execution time display
 */

import React from 'react';
import { ModelCard } from './ModelCard';
import { SearchDocument } from '../../utils/search';

export interface SearchResultsProps {
  results: SearchDocument[];
  isLoading: boolean;
  error?: string | null;
  onModelClick?: (modelId: string) => void;
  className?: string;
  executionTimeMs?: number;
  showExecutionTime?: boolean;
}

export const SearchResults: React.FC<SearchResultsProps> = ({
  results,
  isLoading,
  error,
  onModelClick,
  className = '',
  executionTimeMs,
  showExecutionTime = false
}) => {
  // Loading state with skeletons
  if (isLoading && results.length === 0) {
    return (
      <div className={`search-results ${className}`}>
        <div className="results-grid">
          {Array.from({ length: 6 }).map((_, index) => (
            <div key={index} className="skeleton-card">
              <div className="skeleton-header">
                <div className="skeleton-icon"></div>
                <div className="skeleton-badge"></div>
              </div>
              <div className="skeleton-body">
                <div className="skeleton-meta">
                  <div className="skeleton-tag"></div>
                  <div className="skeleton-tag small"></div>
                </div>
                <div className="skeleton-title"></div>
                <div className="skeleton-text"></div>
                <div className="skeleton-text short"></div>
                <div className="skeleton-tags">
                  <div className="skeleton-tag"></div>
                  <div className="skeleton-tag"></div>
                  <div className="skeleton-tag"></div>
                </div>
                <div className="skeleton-stats">
                  <div className="skeleton-stars"></div>
                  <div className="skeleton-sales"></div>
                </div>
                <div className="skeleton-footer">
                  <div className="skeleton-price"></div>
                  <div className="skeleton-score"></div>
                </div>
              </div>
            </div>
          ))}
        </div>

        <style jsx>{`
          .search-results {
            width: 100%;
          }

          .results-grid {
            display: grid;
            grid-template-columns: repeat(auto-fill, minmax(320px, 1fr));
            gap: 24px;
          }

          .skeleton-card {
            background: white;
            border: 1px solid #e5e7eb;
            border-radius: 12px;
            padding: 20px;
            animation: pulse 1.5s ease-in-out infinite;
          }

          @keyframes pulse {
            0%, 100% {
              opacity: 1;
            }
            50% {
              opacity: 0.7;
            }
          }

          .skeleton-header {
            display: flex;
            justify-content: space-between;
            align-items: flex-start;
            margin-bottom: 16px;
          }

          .skeleton-icon {
            width: 60px;
            height: 60px;
            background: linear-gradient(90deg, #f3f4f6 0%, #e5e7eb 50%, #f3f4f6 100%);
            background-size: 200% 100%;
            animation: shimmer 1.5s ease-in-out infinite;
            border-radius: 12px;
          }

          .skeleton-badge {
            width: 50px;
            height: 24px;
            background: linear-gradient(90deg, #f3f4f6 0%, #e5e7eb 50%, #f3f4f6 100%);
            background-size: 200% 100%;
            animation: shimmer 1.5s ease-in-out infinite;
            border-radius: 6px;
          }

          @keyframes shimmer {
            0% {
              background-position: 200% 0;
            }
            100% {
              background-position: -200% 0;
            }
          }

          .skeleton-body {
            display: flex;
            flex-direction: column;
            gap: 12px;
          }

          .skeleton-meta {
            display: flex;
            gap: 8px;
          }

          .skeleton-tag {
            width: 80px;
            height: 22px;
            background: linear-gradient(90deg, #f3f4f6 0%, #e5e7eb 50%, #f3f4f6 100%);
            background-size: 200% 100%;
            animation: shimmer 1.5s ease-in-out infinite;
            border-radius: 6px;
          }

          .skeleton-tag.small {
            width: 60px;
          }

          .skeleton-title {
            width: 80%;
            height: 24px;
            background: linear-gradient(90deg, #f3f4f6 0%, #e5e7eb 50%, #f3f4f6 100%);
            background-size: 200% 100%;
            animation: shimmer 1.5s ease-in-out infinite;
            border-radius: 4px;
          }

          .skeleton-text {
            width: 100%;
            height: 16px;
            background: linear-gradient(90deg, #f3f4f6 0%, #e5e7eb 50%, #f3f4f6 100%);
            background-size: 200% 100%;
            animation: shimmer 1.5s ease-in-out infinite;
            border-radius: 4px;
          }

          .skeleton-text.short {
            width: 70%;
          }

          .skeleton-tags {
            display: flex;
            gap: 6px;
          }

          .skeleton-stats {
            display: flex;
            flex-direction: column;
            gap: 8px;
            padding: 12px 0;
            border-top: 1px solid #f3f4f6;
            border-bottom: 1px solid #f3f4f6;
          }

          .skeleton-stars,
          .skeleton-sales {
            width: 120px;
            height: 16px;
            background: linear-gradient(90deg, #f3f4f6 0%, #e5e7eb 50%, #f3f4f6 100%);
            background-size: 200% 100%;
            animation: shimmer 1.5s ease-in-out infinite;
            border-radius: 4px;
          }

          .skeleton-footer {
            display: flex;
            justify-content: space-between;
            align-items: flex-end;
          }

          .skeleton-price {
            width: 100px;
            height: 28px;
            background: linear-gradient(90deg, #f3f4f6 0%, #e5e7eb 50%, #f3f4f6 100%);
            background-size: 200% 100%;
            animation: shimmer 1.5s ease-in-out infinite;
            border-radius: 4px;
          }

          .skeleton-score {
            width: 60px;
            height: 50px;
            background: linear-gradient(90deg, #f3f4f6 0%, #e5e7eb 50%, #f3f4f6 100%);
            background-size: 200% 100%;
            animation: shimmer 1.5s ease-in-out infinite;
            border-radius: 8px;
          }

          @media (max-width: 640px) {
            .results-grid {
              grid-template-columns: 1fr;
              gap: 16px;
            }
          }

          @media (min-width: 641px) and (max-width: 1024px) {
            .results-grid {
              grid-template-columns: repeat(2, 1fr);
            }
          }
        `}</style>
      </div>
    );
  }

  // Error state
  if (error) {
    return (
      <div className={`search-results ${className}`}>
        <div className="empty-state error-state">
          <div className="empty-icon error-icon">
            <svg
              xmlns="http://www.w3.org/2000/svg"
              width="64"
              height="64"
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              strokeWidth="2"
              strokeLinecap="round"
              strokeLinejoin="round"
            >
              <circle cx="12" cy="12" r="10"></circle>
              <line x1="12" y1="8" x2="12" y2="12"></line>
              <line x1="12" y1="16" x2="12.01" y2="16"></line>
            </svg>
          </div>
          <h3 className="empty-title">Search Error</h3>
          <p className="empty-description">{error}</p>
          <button
            type="button"
            onClick={() => window.location.reload()}
            className="retry-button"
          >
            Retry Search
          </button>
        </div>

        <style jsx>{`
          .search-results {
            width: 100%;
          }

          .empty-state {
            display: flex;
            flex-direction: column;
            align-items: center;
            justify-content: center;
            padding: 80px 20px;
            text-align: center;
          }

          .empty-icon {
            margin-bottom: 24px;
            color: #9ca3af;
          }

          .error-icon {
            color: #ef4444;
          }

          .empty-title {
            font-size: 24px;
            font-weight: 700;
            color: #111827;
            margin: 0 0 12px 0;
          }

          .empty-description {
            font-size: 16px;
            color: #6b7280;
            line-height: 1.5;
            max-width: 500px;
            margin: 0 0 24px 0;
          }

          .retry-button {
            padding: 12px 24px;
            background: #3b82f6;
            color: white;
            border: none;
            border-radius: 8px;
            font-size: 16px;
            font-weight: 500;
            cursor: pointer;
            transition: all 0.2s;
          }

          .retry-button:hover {
            background: #2563eb;
            transform: translateY(-2px);
            box-shadow: 0 4px 12px rgba(59, 130, 246, 0.3);
          }

          .retry-button:active {
            transform: translateY(0);
          }
        `}</style>
      </div>
    );
  }

  // Empty state (no results)
  if (results.length === 0) {
    return (
      <div className={`search-results ${className}`}>
        <div className="empty-state">
          <div className="empty-icon">
            <svg
              xmlns="http://www.w3.org/2000/svg"
              width="64"
              height="64"
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              strokeWidth="2"
              strokeLinecap="round"
              strokeLinejoin="round"
            >
              <circle cx="11" cy="11" r="8"></circle>
              <path d="m21 21-4.35-4.35"></path>
            </svg>
          </div>
          <h3 className="empty-title">No Models Found</h3>
          <p className="empty-description">
            We couldn't find any models matching your search criteria.
            <br />
            Try adjusting your filters or search query.
          </p>
          <div className="empty-suggestions">
            <h4>Suggestions:</h4>
            <ul>
              <li>Check your spelling and try again</li>
              <li>Try more general keywords</li>
              <li>Remove some filters</li>
              <li>Browse featured or trending models</li>
            </ul>
          </div>
        </div>

        <style jsx>{`
          .search-results {
            width: 100%;
          }

          .empty-state {
            display: flex;
            flex-direction: column;
            align-items: center;
            justify-content: center;
            padding: 80px 20px;
            text-align: center;
          }

          .empty-icon {
            margin-bottom: 24px;
            color: #9ca3af;
          }

          .empty-title {
            font-size: 24px;
            font-weight: 700;
            color: #111827;
            margin: 0 0 12px 0;
          }

          .empty-description {
            font-size: 16px;
            color: #6b7280;
            line-height: 1.5;
            max-width: 500px;
            margin: 0 0 32px 0;
          }

          .empty-suggestions {
            background: #f9fafb;
            border: 1px solid #e5e7eb;
            border-radius: 12px;
            padding: 24px;
            max-width: 400px;
            text-align: left;
          }

          .empty-suggestions h4 {
            font-size: 16px;
            font-weight: 600;
            color: #111827;
            margin: 0 0 12px 0;
          }

          .empty-suggestions ul {
            margin: 0;
            padding: 0 0 0 20px;
            list-style-type: disc;
          }

          .empty-suggestions li {
            font-size: 14px;
            color: #6b7280;
            line-height: 1.8;
          }
        `}</style>
      </div>
    );
  }

  // Results grid
  return (
    <div className={`search-results ${className}`}>
      {showExecutionTime && executionTimeMs !== undefined && (
        <div className="results-meta">
          <span className="results-time">
            Found {results.length} model{results.length !== 1 ? 's' : ''} in{' '}
            {executionTimeMs.toFixed(0)}ms
          </span>
        </div>
      )}

      <div className="results-grid">
        {results.map((model, index) => (
          <div
            key={model.modelId}
            className="result-item"
            style={{ animationDelay: `${index * 50}ms` }}
          >
            <ModelCard model={model} onClick={onModelClick} />
          </div>
        ))}
      </div>

      {isLoading && (
        <div className="loading-overlay">
          <div className="loading-spinner">
            <svg className="spinner" viewBox="0 0 50 50">
              <circle
                className="spinner-path"
                cx="25"
                cy="25"
                r="20"
                fill="none"
                strokeWidth="5"
              ></circle>
            </svg>
            <span>Loading more results...</span>
          </div>
        </div>
      )}

      <style jsx>{`
        .search-results {
          width: 100%;
          position: relative;
        }

        .results-meta {
          margin-bottom: 16px;
          padding: 8px 0;
        }

        .results-time {
          font-size: 14px;
          color: #6b7280;
          font-weight: 500;
        }

        .results-grid {
          display: grid;
          grid-template-columns: repeat(auto-fill, minmax(320px, 1fr));
          gap: 24px;
        }

        .result-item {
          animation: fadeInUp 0.4s ease-out both;
        }

        @keyframes fadeInUp {
          from {
            opacity: 0;
            transform: translateY(20px);
          }
          to {
            opacity: 1;
            transform: translateY(0);
          }
        }

        .loading-overlay {
          position: absolute;
          top: 0;
          left: 0;
          right: 0;
          bottom: 0;
          background: rgba(255, 255, 255, 0.8);
          display: flex;
          align-items: center;
          justify-content: center;
          backdrop-filter: blur(2px);
          z-index: 10;
        }

        .loading-spinner {
          display: flex;
          flex-direction: column;
          align-items: center;
          gap: 16px;
        }

        .spinner {
          animation: rotate 2s linear infinite;
          width: 50px;
          height: 50px;
        }

        .spinner-path {
          stroke: #3b82f6;
          stroke-linecap: round;
          animation: dash 1.5s ease-in-out infinite;
        }

        @keyframes rotate {
          100% {
            transform: rotate(360deg);
          }
        }

        @keyframes dash {
          0% {
            stroke-dasharray: 1, 150;
            stroke-dashoffset: 0;
          }
          50% {
            stroke-dasharray: 90, 150;
            stroke-dashoffset: -35;
          }
          100% {
            stroke-dasharray: 90, 150;
            stroke-dashoffset: -124;
          }
        }

        .loading-spinner span {
          font-size: 14px;
          color: #6b7280;
          font-weight: 500;
        }

        @media (max-width: 640px) {
          .results-grid {
            grid-template-columns: 1fr;
            gap: 16px;
          }
        }

        @media (min-width: 641px) and (max-width: 1024px) {
          .results-grid {
            grid-template-columns: repeat(2, 1fr);
          }
        }

        @media (min-width: 1025px) and (max-width: 1440px) {
          .results-grid {
            grid-template-columns: repeat(2, 1fr);
          }
        }

        @media (min-width: 1441px) {
          .results-grid {
            grid-template-columns: repeat(3, 1fr);
          }
        }
      `}</style>
    </div>
  );
};

export default SearchResults;
