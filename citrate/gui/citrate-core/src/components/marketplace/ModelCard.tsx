/**
 * ModelCard Component
 *
 * Displays a single model in the search results grid.
 * Features:
 * - Model thumbnail/icon
 * - Name, creator, and description
 * - Category badge and tags
 * - Price with discount indicator
 * - Rating stars and review count
 * - Quality score badge
 * - Featured indicator
 * - Hover effects and animations
 * - Click to view model details
 */

import React from 'react';
import {
  SearchDocument,
  CATEGORY_INFO,
  SIZE_INFO,
  formatPrice,
  formatModelSize
} from '../../utils/search';

export interface ModelCardProps {
  model: SearchDocument;
  onClick?: (modelId: string) => void;
  className?: string;
}

export const ModelCard: React.FC<ModelCardProps> = ({
  model,
  onClick,
  className = ''
}) => {
  const handleClick = () => {
    if (onClick) {
      onClick(model.modelId);
    }
  };

  const renderStars = (rating: number) => {
    const stars = Math.round(rating / 100); // Convert 0-500 to 0-5
    return (
      <div className="star-rating">
        {Array.from({ length: 5 }).map((_, i) => (
          <span key={i} className={`star ${i < stars ? 'filled' : ''}`}>
            ★
          </span>
        ))}
      </div>
    );
  };

  const effectivePrice = Math.min(model.basePrice, model.discountPrice);
  const hasDiscount = model.discountPrice < model.basePrice;
  const discountPercent = hasDiscount
    ? Math.round(((model.basePrice - model.discountPrice) / model.basePrice) * 100)
    : 0;

  const categoryInfo = CATEGORY_INFO[model.category];
  const sizeInfo = model.modelSize ? SIZE_INFO[model.modelSize] : null;

  return (
    <div className={`model-card ${className}`} onClick={handleClick}>
      {model.featured && (
        <div className="featured-badge">
          <svg
            xmlns="http://www.w3.org/2000/svg"
            width="16"
            height="16"
            viewBox="0 0 24 24"
            fill="currentColor"
          >
            <polygon points="12 2 15.09 8.26 22 9.27 17 14.14 18.18 21.02 12 17.77 5.82 21.02 7 14.14 2 9.27 8.91 8.26 12 2"></polygon>
          </svg>
          Featured
        </div>
      )}

      <div className="model-card-header">
        <div className="model-icon">
          <span className="category-emoji">{categoryInfo.icon}</span>
        </div>

        {hasDiscount && (
          <div className="discount-badge">-{discountPercent}%</div>
        )}
      </div>

      <div className="model-card-body">
        <div className="model-meta">
          <span className="category-badge">{categoryInfo.label}</span>
          {sizeInfo && (
            <span
              className="size-badge"
              style={{ backgroundColor: sizeInfo.color }}
            >
              {sizeInfo.label}
            </span>
          )}
        </div>

        <h3 className="model-name">{model.name}</h3>

        <p className="model-creator">
          by {model.creatorName || `${model.creatorAddress.slice(0, 6)}...${model.creatorAddress.slice(-4)}`}
        </p>

        <p className="model-description">
          {model.description.length > 120
            ? `${model.description.slice(0, 120)}...`
            : model.description}
        </p>

        {model.tags.length > 0 && (
          <div className="model-tags">
            {model.tags.slice(0, 3).map((tag, index) => (
              <span key={index} className="tag">
                {tag}
              </span>
            ))}
            {model.tags.length > 3 && (
              <span className="tag-more">+{model.tags.length - 3}</span>
            )}
          </div>
        )}

        <div className="model-stats">
          <div className="stat-item">
            {renderStars(model.averageRating)}
            <span className="review-count">
              ({model.reviewCount})
            </span>
          </div>

          <div className="stat-item">
            <svg
              xmlns="http://www.w3.org/2000/svg"
              width="14"
              height="14"
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              strokeWidth="2"
              strokeLinecap="round"
              strokeLinejoin="round"
            >
              <path d="M16 21v-2a4 4 0 0 0-4-4H6a4 4 0 0 0-4 4v2"></path>
              <circle cx="9" cy="7" r="4"></circle>
              <path d="M22 21v-2a4 4 0 0 0-3-3.87"></path>
              <path d="M16 3.13a4 4 0 0 1 0 7.75"></path>
            </svg>
            <span>{model.totalSales.toLocaleString()} sales</span>
          </div>
        </div>

        <div className="model-footer">
          <div className="price-section">
            {hasDiscount && (
              <span className="original-price">{formatPrice(model.basePrice)}</span>
            )}
            <span className="current-price">{formatPrice(effectivePrice)}</span>
            <span className="price-label">per inference</span>
          </div>

          <div className="quality-score-badge" data-score={model.qualityScore}>
            <span className="score-value">{model.qualityScore}</span>
            <span className="score-label">Quality</span>
          </div>
        </div>

        {model.framework && (
          <div className="framework-badge">
            <svg
              xmlns="http://www.w3.org/2000/svg"
              width="12"
              height="12"
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              strokeWidth="2"
              strokeLinecap="round"
              strokeLinejoin="round"
            >
              <polyline points="16 18 22 12 16 6"></polyline>
              <polyline points="8 6 2 12 8 18"></polyline>
            </svg>
            {model.framework}
          </div>
        )}
      </div>

      <style jsx>{`
        .model-card {
          position: relative;
          background: white;
          border: 1px solid #e5e7eb;
          border-radius: 12px;
          padding: 20px;
          cursor: pointer;
          transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
          overflow: hidden;
        }

        .model-card:hover {
          border-color: #3b82f6;
          box-shadow: 0 10px 30px rgba(59, 130, 246, 0.15);
          transform: translateY(-4px);
        }

        .featured-badge {
          position: absolute;
          top: 12px;
          right: 12px;
          background: linear-gradient(135deg, #fbbf24 0%, #f59e0b 100%);
          color: white;
          padding: 4px 10px;
          border-radius: 12px;
          font-size: 11px;
          font-weight: 600;
          display: flex;
          align-items: center;
          gap: 4px;
          box-shadow: 0 2px 8px rgba(251, 191, 36, 0.3);
        }

        .model-card-header {
          display: flex;
          justify-content: space-between;
          align-items: flex-start;
          margin-bottom: 16px;
        }

        .model-icon {
          width: 60px;
          height: 60px;
          background: linear-gradient(135deg, #f3f4f6 0%, #e5e7eb 100%);
          border-radius: 12px;
          display: flex;
          align-items: center;
          justify-content: center;
          font-size: 32px;
        }

        .discount-badge {
          background: #ef4444;
          color: white;
          padding: 4px 8px;
          border-radius: 6px;
          font-size: 12px;
          font-weight: 700;
        }

        .model-card-body {
          display: flex;
          flex-direction: column;
          gap: 12px;
        }

        .model-meta {
          display: flex;
          align-items: center;
          gap: 8px;
        }

        .category-badge {
          background: #eff6ff;
          color: #3b82f6;
          padding: 4px 10px;
          border-radius: 6px;
          font-size: 11px;
          font-weight: 600;
        }

        .size-badge {
          color: white;
          padding: 4px 8px;
          border-radius: 6px;
          font-size: 10px;
          font-weight: 700;
          text-transform: uppercase;
        }

        .model-name {
          font-size: 18px;
          font-weight: 700;
          color: #111827;
          margin: 0;
          line-height: 1.3;
        }

        .model-creator {
          font-size: 13px;
          color: #6b7280;
          margin: 0;
        }

        .model-description {
          font-size: 14px;
          color: #374151;
          line-height: 1.5;
          margin: 0;
        }

        .model-tags {
          display: flex;
          flex-wrap: wrap;
          gap: 6px;
        }

        .tag {
          background: #f3f4f6;
          color: #4b5563;
          padding: 3px 8px;
          border-radius: 4px;
          font-size: 12px;
          font-weight: 500;
        }

        .tag-more {
          background: #e5e7eb;
          color: #6b7280;
          padding: 3px 8px;
          border-radius: 4px;
          font-size: 11px;
          font-weight: 600;
        }

        .model-stats {
          display: flex;
          flex-direction: column;
          gap: 8px;
          padding: 12px 0;
          border-top: 1px solid #f3f4f6;
          border-bottom: 1px solid #f3f4f6;
        }

        .stat-item {
          display: flex;
          align-items: center;
          gap: 6px;
          font-size: 13px;
          color: #6b7280;
        }

        .star-rating {
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

        .review-count {
          font-size: 12px;
          color: #9ca3af;
        }

        .model-footer {
          display: flex;
          justify-content: space-between;
          align-items: flex-end;
        }

        .price-section {
          display: flex;
          flex-direction: column;
          gap: 2px;
        }

        .original-price {
          font-size: 12px;
          color: #9ca3af;
          text-decoration: line-through;
        }

        .current-price {
          font-size: 20px;
          font-weight: 700;
          color: #111827;
        }

        .price-label {
          font-size: 11px;
          color: #6b7280;
        }

        .quality-score-badge {
          display: flex;
          flex-direction: column;
          align-items: center;
          background: linear-gradient(135deg, #3b82f6 0%, #2563eb 100%);
          color: white;
          padding: 8px 12px;
          border-radius: 8px;
          min-width: 50px;
        }

        .quality-score-badge[data-score^="9"],
        .quality-score-badge[data-score="100"] {
          background: linear-gradient(135deg, #10b981 0%, #059669 100%);
        }

        .quality-score-badge[data-score^="8"] {
          background: linear-gradient(135deg, #3b82f6 0%, #2563eb 100%);
        }

        .quality-score-badge[data-score^="7"] {
          background: linear-gradient(135deg, #8b5cf6 0%, #7c3aed 100%);
        }

        .quality-score-badge[data-score^="6"],
        .quality-score-badge[data-score^="5"] {
          background: linear-gradient(135deg, #f59e0b 0%, #d97706 100%);
        }

        .score-value {
          font-size: 18px;
          font-weight: 700;
        }

        .score-label {
          font-size: 9px;
          font-weight: 600;
          text-transform: uppercase;
          opacity: 0.9;
        }

        .framework-badge {
          position: absolute;
          bottom: 12px;
          right: 12px;
          background: rgba(0, 0, 0, 0.05);
          color: #6b7280;
          padding: 4px 8px;
          border-radius: 6px;
          font-size: 11px;
          font-weight: 500;
          display: flex;
          align-items: center;
          gap: 4px;
        }

        @media (max-width: 640px) {
          .model-card {
            padding: 16px;
          }

          .model-name {
            font-size: 16px;
          }

          .current-price {
            font-size: 18px;
          }
        }
      `}</style>
    </div>
  );
};

export default ModelCard;
