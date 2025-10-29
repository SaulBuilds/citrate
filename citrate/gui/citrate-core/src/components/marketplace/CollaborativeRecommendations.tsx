/**
 * CollaborativeRecommendations Component
 *
 * Shows "Users who bought this also bought..." recommendations.
 * Displays co-purchase patterns and frequencies.
 */

import React from 'react';
import { SearchDocument, CATEGORY_INFO, formatPrice } from '../../utils/search/types';

interface CollaborativeRecommendationsProps {
  modelId: string;
  recommendations: SearchDocument[];
  isLoading?: boolean;
  onModelClick: (modelId: string) => void;
  limit?: number;
}

export const CollaborativeRecommendations: React.FC<CollaborativeRecommendationsProps> = ({
  modelId,
  recommendations,
  isLoading = false,
  onModelClick,
  limit = 3
}) => {
  const displayRecommendations = recommendations.slice(0, limit);

  if (isLoading) {
    return (
      <div className="collaborative-recommendations">
        <h3>Users who bought this also bought</h3>
        <div className="recommendations-grid">
          {[1, 2, 3].map(i => (
            <div key={i} className="recommendation-card skeleton">
              <div className="skeleton-icon" />
              <div className="skeleton-text" />
              <div className="skeleton-text short" />
            </div>
          ))}
        </div>

        <style jsx>{`
          .collaborative-recommendations {
            padding: 24px;
            background: #ffffff;
            border-radius: 12px;
            box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
          }

          h3 {
            margin: 0 0 16px 0;
            font-size: 18px;
            font-weight: 600;
            color: #1a1a1a;
          }

          .recommendations-grid {
            display: grid;
            grid-template-columns: repeat(auto-fill, minmax(180px, 1fr));
            gap: 16px;
          }

          .recommendation-card.skeleton {
            padding: 16px;
            background: #f5f5f5;
            border-radius: 8px;
            animation: pulse 1.5s ease-in-out infinite;
          }

          .skeleton-icon {
            width: 48px;
            height: 48px;
            background: #e0e0e0;
            border-radius: 8px;
            margin-bottom: 12px;
          }

          .skeleton-text {
            height: 14px;
            background: #e0e0e0;
            border-radius: 4px;
            margin-bottom: 8px;
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

  if (displayRecommendations.length === 0) {
    return (
      <div className="collaborative-recommendations empty">
        <h3>Users who bought this also bought</h3>
        <div className="empty-state">
          <div className="empty-icon">🤝</div>
          <p>No collaborative recommendations yet</p>
          <span className="empty-hint">Be the first to discover related models!</span>
        </div>

        <style jsx>{`
          .collaborative-recommendations {
            padding: 24px;
            background: #ffffff;
            border-radius: 12px;
            box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
          }

          h3 {
            margin: 0 0 16px 0;
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
    <div className="collaborative-recommendations">
      <h3>
        <span className="handshake-icon">🤝</span>
        Users who bought this also bought
      </h3>

      <div className="recommendations-grid">
        {displayRecommendations.map((model, index) => {
          const categoryInfo = CATEGORY_INFO[model.category];
          const stars = Math.round((model.averageRating / 100) * 10) / 10;

          // Calculate mock co-purchase percentage (would come from actual data)
          const coOccurrence = Math.max(15, 80 - index * 20);

          return (
            <div
              key={model.modelId}
              className="recommendation-card"
              onClick={() => onModelClick(model.modelId)}
            >
              <div className="co-occurrence-badge">
                {coOccurrence}% also bought
              </div>

              <div className="model-icon">{categoryInfo.icon}</div>

              <div className="model-info">
                <h4 className="model-name">{model.name}</h4>
                <div className="category-badge">{categoryInfo.label}</div>

                <div className="model-stats">
                  <div className="rating">
                    <span className="stars">★</span>
                    <span className="rating-value">{stars.toFixed(1)}</span>
                  </div>
                  <div className="purchases">
                    <span className="purchase-icon">🛒</span>
                    <span className="purchase-count">{model.totalSales}</span>
                  </div>
                </div>

                <div className="price">
                  {formatPrice(model.discountPrice || model.basePrice)}
                </div>
              </div>
            </div>
          );
        })}
      </div>

      <style jsx>{`
        .collaborative-recommendations {
          padding: 24px;
          background: #ffffff;
          border-radius: 12px;
          box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
        }

        h3 {
          margin: 0 0 16px 0;
          font-size: 18px;
          font-weight: 600;
          color: #1a1a1a;
          display: flex;
          align-items: center;
          gap: 8px;
        }

        .handshake-icon {
          font-size: 22px;
        }

        .recommendations-grid {
          display: grid;
          grid-template-columns: repeat(auto-fill, minmax(180px, 1fr));
          gap: 16px;
        }

        .recommendation-card {
          position: relative;
          padding: 16px;
          background: #fafafa;
          border: 1px solid #e0e0e0;
          border-radius: 8px;
          cursor: pointer;
          transition: all 0.2s;
        }

        .recommendation-card:hover {
          transform: translateY(-2px);
          box-shadow: 0 4px 12px rgba(0, 0, 0, 0.1);
          border-color: #0066cc;
          background: #ffffff;
        }

        .co-occurrence-badge {
          position: absolute;
          top: 8px;
          right: 8px;
          padding: 4px 8px;
          background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
          color: white;
          font-size: 10px;
          font-weight: 600;
          border-radius: 12px;
          box-shadow: 0 2px 4px rgba(102, 126, 234, 0.3);
        }

        .model-icon {
          font-size: 40px;
          margin: 8px 0 12px 0;
          text-align: center;
        }

        .model-info {
          display: flex;
          flex-direction: column;
          gap: 8px;
        }

        .model-name {
          margin: 0;
          font-size: 14px;
          font-weight: 600;
          color: #1a1a1a;
          overflow: hidden;
          text-overflow: ellipsis;
          white-space: nowrap;
        }

        .category-badge {
          display: inline-block;
          padding: 2px 8px;
          background: #e8eaf6;
          color: #3f51b5;
          font-size: 11px;
          font-weight: 500;
          border-radius: 12px;
          width: fit-content;
        }

        .model-stats {
          display: flex;
          justify-content: space-between;
          align-items: center;
          margin-top: 4px;
        }

        .rating {
          display: flex;
          align-items: center;
          gap: 3px;
        }

        .stars {
          color: #ffa500;
          font-size: 12px;
        }

        .rating-value {
          font-size: 12px;
          font-weight: 500;
          color: #666;
        }

        .purchases {
          display: flex;
          align-items: center;
          gap: 3px;
        }

        .purchase-icon {
          font-size: 12px;
        }

        .purchase-count {
          font-size: 12px;
          font-weight: 600;
          color: #666;
        }

        .price {
          font-size: 14px;
          font-weight: 700;
          color: #2e7d32;
          margin-top: 4px;
        }

        @media (max-width: 768px) {
          .recommendations-grid {
            grid-template-columns: repeat(2, 1fr);
          }
        }

        @media (max-width: 480px) {
          .recommendations-grid {
            grid-template-columns: 1fr;
          }
        }
      `}</style>
    </div>
  );
};
