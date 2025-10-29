/**
 * RecentlyViewed Component
 *
 * Shows user's recently viewed models with ability to clear history.
 * Privacy-conscious: all data stored locally.
 */

import React from 'react';
import { SearchDocument, CATEGORY_INFO, formatPrice } from '../../utils/search/types';

interface RecentlyViewedProps {
  recentModels: SearchDocument[];
  isLoading?: boolean;
  onModelClick: (modelId: string) => void;
  onClearHistory: () => void;
  limit?: number;
}

export const RecentlyViewed: React.FC<RecentlyViewedProps> = ({
  recentModels,
  isLoading = false,
  onModelClick,
  onClearHistory,
  limit = 5
}) => {
  const displayModels = recentModels.slice(0, limit);

  if (isLoading) {
    return (
      <div className="recently-viewed">
        <div className="header">
          <h3>Recently Viewed</h3>
        </div>
        <div className="models-list">
          {[1, 2, 3].map(i => (
            <div key={i} className="model-item skeleton">
              <div className="skeleton-icon" />
              <div className="skeleton-content">
                <div className="skeleton-text" />
                <div className="skeleton-text short" />
              </div>
            </div>
          ))}
        </div>

        <style jsx>{`
          .recently-viewed {
            padding: 24px;
            background: #ffffff;
            border-radius: 12px;
            box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
          }

          .header {
            display: flex;
            justify-content: space-between;
            align-items: center;
            margin-bottom: 16px;
          }

          h3 {
            margin: 0;
            font-size: 18px;
            font-weight: 600;
            color: #1a1a1a;
          }

          .models-list {
            display: flex;
            flex-direction: column;
            gap: 12px;
          }

          .model-item.skeleton {
            display: flex;
            gap: 12px;
            padding: 12px;
            background: #f5f5f5;
            border-radius: 8px;
            animation: pulse 1.5s ease-in-out infinite;
          }

          .skeleton-icon {
            width: 48px;
            height: 48px;
            background: #e0e0e0;
            border-radius: 8px;
            flex-shrink: 0;
          }

          .skeleton-content {
            flex: 1;
            display: flex;
            flex-direction: column;
            gap: 8px;
          }

          .skeleton-text {
            height: 14px;
            background: #e0e0e0;
            border-radius: 4px;
          }

          .skeleton-text.short {
            width: 60%;
          }

          @keyframes pulse {
            0%, 100% {
              opacity: 1;
            }
            50% {
              opacity: 0.6;
            }
          }
        `}</style>
      </div>
    );
  }

  if (displayModels.length === 0) {
    return (
      <div className="recently-viewed empty">
        <div className="header">
          <h3>Recently Viewed</h3>
        </div>
        <div className="empty-state">
          <div className="empty-icon">👁️</div>
          <p>You haven't viewed any models yet</p>
          <span className="empty-hint">Browse the marketplace to see models here</span>
        </div>

        <style jsx>{`
          .recently-viewed {
            padding: 24px;
            background: #ffffff;
            border-radius: 12px;
            box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
          }

          .header {
            display: flex;
            justify-content: space-between;
            align-items: center;
            margin-bottom: 16px;
          }

          h3 {
            margin: 0;
            font-size: 18px;
            font-weight: 600;
            color: #1a1a1a;
          }

          .empty-state {
            text-align: center;
            padding: 40px 20px;
          }

          .empty-icon {
            font-size: 48px;
            margin-bottom: 12px;
            opacity: 0.5;
          }

          .empty-state p {
            margin: 0 0 8px 0;
            color: #666;
            font-size: 14px;
          }

          .empty-hint {
            display: block;
            color: #999;
            font-size: 12px;
          }
        `}</style>
      </div>
    );
  }

  return (
    <div className="recently-viewed">
      <div className="header">
        <h3>Recently Viewed</h3>
        <button className="clear-button" onClick={onClearHistory} title="Clear history">
          Clear
        </button>
      </div>

      <div className="models-list">
        {displayModels.map(model => {
          const categoryInfo = CATEGORY_INFO[model.category];
          const stars = Math.round((model.averageRating / 100) * 10) / 10;

          return (
            <div
              key={model.modelId}
              className="model-item"
              onClick={() => onModelClick(model.modelId)}
            >
              <div className="model-icon">{categoryInfo.icon}</div>

              <div className="model-info">
                <h4 className="model-name">{model.name}</h4>
                <div className="model-meta">
                  <span className="category">{categoryInfo.label}</span>
                  <span className="separator">•</span>
                  <div className="rating">
                    <span className="stars">★</span>
                    <span className="rating-value">{stars.toFixed(1)}</span>
                  </div>
                </div>
              </div>

              <div className="model-price">
                {formatPrice(model.discountPrice || model.basePrice)}
              </div>
            </div>
          );
        })}
      </div>

      <style jsx>{`
        .recently-viewed {
          padding: 24px;
          background: #ffffff;
          border-radius: 12px;
          box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
        }

        .header {
          display: flex;
          justify-content: space-between;
          align-items: center;
          margin-bottom: 16px;
        }

        h3 {
          margin: 0;
          font-size: 18px;
          font-weight: 600;
          color: #1a1a1a;
        }

        .clear-button {
          background: none;
          border: 1px solid #e0e0e0;
          color: #666;
          font-size: 12px;
          font-weight: 500;
          padding: 6px 12px;
          border-radius: 6px;
          cursor: pointer;
          transition: all 0.2s;
        }

        .clear-button:hover {
          background: #f5f5f5;
          border-color: #ccc;
          color: #333;
        }

        .models-list {
          display: flex;
          flex-direction: column;
          gap: 8px;
        }

        .model-item {
          display: flex;
          align-items: center;
          gap: 12px;
          padding: 12px;
          background: #fafafa;
          border: 1px solid #e0e0e0;
          border-radius: 8px;
          cursor: pointer;
          transition: all 0.2s;
        }

        .model-item:hover {
          background: #f0f7ff;
          border-color: #0066cc;
          transform: translateX(4px);
        }

        .model-icon {
          font-size: 36px;
          flex-shrink: 0;
        }

        .model-info {
          flex: 1;
          min-width: 0;
        }

        .model-name {
          margin: 0 0 4px 0;
          font-size: 14px;
          font-weight: 600;
          color: #1a1a1a;
          overflow: hidden;
          text-overflow: ellipsis;
          white-space: nowrap;
        }

        .model-meta {
          display: flex;
          align-items: center;
          gap: 6px;
          font-size: 12px;
          color: #666;
        }

        .category {
          font-weight: 500;
        }

        .separator {
          color: #ccc;
        }

        .rating {
          display: flex;
          align-items: center;
          gap: 2px;
        }

        .stars {
          color: #ffa500;
          font-size: 12px;
        }

        .rating-value {
          font-weight: 500;
        }

        .model-price {
          font-size: 13px;
          font-weight: 600;
          color: #2e7d32;
          flex-shrink: 0;
        }

        @media (max-width: 480px) {
          .model-item {
            flex-direction: column;
            align-items: flex-start;
          }

          .model-price {
            align-self: flex-end;
          }
        }
      `}</style>
    </div>
  );
};
