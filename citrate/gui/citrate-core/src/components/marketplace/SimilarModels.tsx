/**
 * SimilarModels Component
 *
 * Displays a grid of similar models based on content-based filtering.
 * Shows mini model cards with key information.
 */

import React from 'react';
import { SearchDocument, CATEGORY_INFO, formatPrice } from '../../utils/search/types';

interface SimilarModelsProps {
  modelId: string;
  similarModels: SearchDocument[];
  isLoading?: boolean;
  onModelClick: (modelId: string) => void;
  onViewAll?: () => void;
}

export const SimilarModels: React.FC<SimilarModelsProps> = ({
  modelId,
  similarModels,
  isLoading = false,
  onModelClick,
  onViewAll
}) => {
  if (isLoading) {
    return (
      <div className="similar-models">
        <h3>Similar Models</h3>
        <div className="models-grid">
          {[1, 2, 3, 4].map(i => (
            <div key={i} className="model-card skeleton">
              <div className="skeleton-icon" />
              <div className="skeleton-text" />
              <div className="skeleton-text short" />
            </div>
          ))}
        </div>

        <style jsx>{`
          .similar-models {
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

          .models-grid {
            display: grid;
            grid-template-columns: repeat(auto-fill, minmax(200px, 1fr));
            gap: 16px;
          }

          .model-card.skeleton {
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

  if (similarModels.length === 0) {
    return (
      <div className="similar-models empty">
        <h3>Similar Models</h3>
        <div className="empty-state">
          <div className="empty-icon">🔍</div>
          <p>No similar models found</p>
        </div>

        <style jsx>{`
          .similar-models {
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
          }

          .empty-state p {
            margin: 0;
            color: #666;
            font-size: 14px;
          }
        `}</style>
      </div>
    );
  }

  return (
    <div className="similar-models">
      <div className="header">
        <h3>Similar Models</h3>
        {onViewAll && similarModels.length > 4 && (
          <button className="view-all" onClick={onViewAll}>
            View All →
          </button>
        )}
      </div>

      <div className="models-grid">
        {similarModels.slice(0, 4).map(model => {
          const categoryInfo = CATEGORY_INFO[model.category];
          const stars = Math.round((model.averageRating / 100) * 10) / 10;

          return (
            <div
              key={model.modelId}
              className="model-card"
              onClick={() => onModelClick(model.modelId)}
            >
              <div className="model-icon">{categoryInfo.icon}</div>
              <div className="model-info">
                <h4 className="model-name">{model.name}</h4>
                <div className="model-category">
                  <span className="category-badge">{categoryInfo.label}</span>
                </div>
                <div className="model-stats">
                  <div className="rating">
                    <span className="stars">{'★'.repeat(Math.round(stars))}{'☆'.repeat(5 - Math.round(stars))}</span>
                    <span className="rating-value">{stars.toFixed(1)}</span>
                  </div>
                  <div className="price">{formatPrice(model.discountPrice || model.basePrice)}</div>
                </div>
              </div>
            </div>
          );
        })}
      </div>

      <style jsx>{`
        .similar-models {
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

        .view-all {
          background: none;
          border: none;
          color: #0066cc;
          font-size: 14px;
          font-weight: 500;
          cursor: pointer;
          padding: 4px 8px;
          border-radius: 4px;
          transition: all 0.2s;
        }

        .view-all:hover {
          background: #f0f7ff;
          color: #0052a3;
        }

        .models-grid {
          display: grid;
          grid-template-columns: repeat(auto-fill, minmax(200px, 1fr));
          gap: 16px;
        }

        .model-card {
          padding: 16px;
          background: #fafafa;
          border: 1px solid #e0e0e0;
          border-radius: 8px;
          cursor: pointer;
          transition: all 0.2s;
        }

        .model-card:hover {
          transform: translateY(-2px);
          box-shadow: 0 4px 12px rgba(0, 0, 0, 0.1);
          border-color: #0066cc;
        }

        .model-icon {
          font-size: 36px;
          margin-bottom: 12px;
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

        .model-category {
          display: flex;
          gap: 4px;
        }

        .category-badge {
          display: inline-block;
          padding: 2px 8px;
          background: #e3f2fd;
          color: #1976d2;
          font-size: 11px;
          font-weight: 500;
          border-radius: 12px;
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
          gap: 4px;
        }

        .stars {
          color: #ffa500;
          font-size: 12px;
          line-height: 1;
        }

        .rating-value {
          font-size: 12px;
          font-weight: 500;
          color: #666;
        }

        .price {
          font-size: 13px;
          font-weight: 600;
          color: #2e7d32;
        }

        @media (max-width: 768px) {
          .models-grid {
            grid-template-columns: repeat(2, 1fr);
          }
        }

        @media (max-width: 480px) {
          .models-grid {
            grid-template-columns: 1fr;
          }
        }
      `}</style>
    </div>
  );
};
