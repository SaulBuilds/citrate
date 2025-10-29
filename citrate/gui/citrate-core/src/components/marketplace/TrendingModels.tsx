/**
 * TrendingModels Component
 *
 * Displays trending models in a horizontal scrollable carousel.
 * Shows trending badges and rank indicators.
 */

import React, { useRef, useState } from 'react';
import { SearchDocument, CATEGORY_INFO, formatPrice } from '../../utils/search/types';

interface TrendingModelsProps {
  trendingModels: SearchDocument[];
  isLoading?: boolean;
  onModelClick: (modelId: string) => void;
  timeWindow?: '24h' | '7d' | '30d' | '90d';
}

export const TrendingModels: React.FC<TrendingModelsProps> = ({
  trendingModels,
  isLoading = false,
  onModelClick,
  timeWindow = '7d'
}) => {
  const scrollRef = useRef<HTMLDivElement>(null);
  const [canScrollLeft, setCanScrollLeft] = useState(false);
  const [canScrollRight, setCanScrollRight] = useState(true);

  const handleScroll = () => {
    if (!scrollRef.current) return;

    const { scrollLeft, scrollWidth, clientWidth } = scrollRef.current;
    setCanScrollLeft(scrollLeft > 0);
    setCanScrollRight(scrollLeft < scrollWidth - clientWidth - 10);
  };

  const scroll = (direction: 'left' | 'right') => {
    if (!scrollRef.current) return;

    const scrollAmount = 300;
    const newScrollLeft = direction === 'left'
      ? scrollRef.current.scrollLeft - scrollAmount
      : scrollRef.current.scrollLeft + scrollAmount;

    scrollRef.current.scrollTo({
      left: newScrollLeft,
      behavior: 'smooth'
    });
  };

  const getRankBadge = (index: number) => {
    if (index === 0) return { emoji: '🥇', text: '#1', color: '#ffd700' };
    if (index === 1) return { emoji: '🥈', text: '#2', color: '#c0c0c0' };
    if (index === 2) return { emoji: '🥉', text: '#3', color: '#cd7f32' };
    return { emoji: '', text: `#${index + 1}`, color: '#666' };
  };

  if (isLoading) {
    return (
      <div className="trending-models">
        <div className="header">
          <h3>
            <span className="fire-emoji">🔥</span>
            Trending Models
          </h3>
          <span className="time-window">{timeWindow}</span>
        </div>
        <div className="carousel-container">
          <div className="carousel">
            {[1, 2, 3, 4, 5].map(i => (
              <div key={i} className="model-card skeleton">
                <div className="skeleton-rank" />
                <div className="skeleton-icon" />
                <div className="skeleton-text" />
                <div className="skeleton-text short" />
              </div>
            ))}
          </div>
        </div>

        <style jsx>{`
          .trending-models {
            padding: 24px;
            background: linear-gradient(135deg, #fff5f5 0%, #ffffff 100%);
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
            display: flex;
            align-items: center;
            gap: 8px;
          }

          .fire-emoji {
            font-size: 24px;
            animation: flicker 1.5s ease-in-out infinite;
          }

          .time-window {
            font-size: 12px;
            color: #666;
            background: #f5f5f5;
            padding: 4px 8px;
            border-radius: 12px;
          }

          .carousel-container {
            overflow: hidden;
          }

          .carousel {
            display: flex;
            gap: 16px;
            overflow-x: auto;
            scrollbar-width: thin;
          }

          .model-card.skeleton {
            min-width: 200px;
            padding: 16px;
            background: #f5f5f5;
            border-radius: 8px;
            animation: pulse 1.5s ease-in-out infinite;
          }

          .skeleton-rank {
            width: 32px;
            height: 20px;
            background: #e0e0e0;
            border-radius: 4px;
            margin-bottom: 8px;
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

          @keyframes flicker {
            0%, 100% {
              transform: scale(1);
            }
            50% {
              transform: scale(1.1);
            }
          }
        `}</style>
      </div>
    );
  }

  if (trendingModels.length === 0) {
    return (
      <div className="trending-models empty">
        <div className="header">
          <h3>
            <span className="fire-emoji">🔥</span>
            Trending Models
          </h3>
        </div>
        <div className="empty-state">
          <p>No trending models in the selected timeframe</p>
        </div>

        <style jsx>{`
          .trending-models {
            padding: 24px;
            background: linear-gradient(135deg, #fff5f5 0%, #ffffff 100%);
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
            display: flex;
            align-items: center;
            gap: 8px;
          }

          .fire-emoji {
            font-size: 24px;
          }

          .empty-state {
            text-align: center;
            padding: 40px 20px;
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
    <div className="trending-models">
      <div className="header">
        <h3>
          <span className="fire-emoji">🔥</span>
          Trending Models
        </h3>
        <span className="time-window">{timeWindow}</span>
      </div>

      <div className="carousel-container">
        {canScrollLeft && (
          <button className="scroll-button left" onClick={() => scroll('left')}>
            ‹
          </button>
        )}

        <div
          ref={scrollRef}
          className="carousel"
          onScroll={handleScroll}
        >
          {trendingModels.map((model, index) => {
            const categoryInfo = CATEGORY_INFO[model.category];
            const stars = Math.round((model.averageRating / 100) * 10) / 10;
            const rank = getRankBadge(index);

            return (
              <div
                key={model.modelId}
                className="model-card"
                onClick={() => onModelClick(model.modelId)}
              >
                <div className="rank-badge" style={{ background: rank.color }}>
                  {rank.emoji && <span className="rank-emoji">{rank.emoji}</span>}
                  <span className="rank-text">{rank.text}</span>
                </div>

                <div className="model-icon">{categoryInfo.icon}</div>

                <div className="model-info">
                  <h4 className="model-name">{model.name}</h4>
                  <div className="category-badge">{categoryInfo.label}</div>

                  <div className="model-stats">
                    <div className="stat">
                      <span className="stat-icon">🛒</span>
                      <span className="stat-value">{model.totalSales}</span>
                    </div>
                    <div className="stat">
                      <span className="stat-icon">⚡</span>
                      <span className="stat-value">{model.totalInferences}</span>
                    </div>
                  </div>

                  <div className="price">{formatPrice(model.discountPrice || model.basePrice)}</div>
                </div>
              </div>
            );
          })}
        </div>

        {canScrollRight && (
          <button className="scroll-button right" onClick={() => scroll('right')}>
            ›
          </button>
        )}
      </div>

      <style jsx>{`
        .trending-models {
          padding: 24px;
          background: linear-gradient(135deg, #fff5f5 0%, #ffffff 100%);
          border-radius: 12px;
          box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
          position: relative;
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
          display: flex;
          align-items: center;
          gap: 8px;
        }

        .fire-emoji {
          font-size: 24px;
          animation: flicker 1.5s ease-in-out infinite;
        }

        .time-window {
          font-size: 12px;
          color: #666;
          background: #f5f5f5;
          padding: 4px 8px;
          border-radius: 12px;
          font-weight: 500;
        }

        .carousel-container {
          position: relative;
        }

        .carousel {
          display: flex;
          gap: 16px;
          overflow-x: auto;
          scroll-behavior: smooth;
          scrollbar-width: thin;
          scrollbar-color: #ccc transparent;
          padding: 8px 0;
        }

        .carousel::-webkit-scrollbar {
          height: 6px;
        }

        .carousel::-webkit-scrollbar-track {
          background: transparent;
        }

        .carousel::-webkit-scrollbar-thumb {
          background: #ccc;
          border-radius: 3px;
        }

        .carousel::-webkit-scrollbar-thumb:hover {
          background: #999;
        }

        .scroll-button {
          position: absolute;
          top: 50%;
          transform: translateY(-50%);
          width: 40px;
          height: 40px;
          background: rgba(255, 255, 255, 0.95);
          border: 1px solid #ddd;
          border-radius: 50%;
          font-size: 24px;
          cursor: pointer;
          z-index: 10;
          display: flex;
          align-items: center;
          justify-content: center;
          box-shadow: 0 2px 8px rgba(0, 0, 0, 0.15);
          transition: all 0.2s;
        }

        .scroll-button:hover {
          background: white;
          box-shadow: 0 4px 12px rgba(0, 0, 0, 0.2);
        }

        .scroll-button.left {
          left: -12px;
        }

        .scroll-button.right {
          right: -12px;
        }

        .model-card {
          min-width: 200px;
          max-width: 200px;
          padding: 16px;
          background: white;
          border: 2px solid #ffe0e0;
          border-radius: 8px;
          cursor: pointer;
          transition: all 0.2s;
          position: relative;
        }

        .model-card:hover {
          transform: translateY(-4px);
          box-shadow: 0 6px 16px rgba(255, 87, 34, 0.2);
          border-color: #ff5722;
        }

        .rank-badge {
          position: absolute;
          top: -8px;
          right: -8px;
          padding: 4px 8px;
          border-radius: 12px;
          font-size: 12px;
          font-weight: 700;
          color: white;
          display: flex;
          align-items: center;
          gap: 4px;
          box-shadow: 0 2px 6px rgba(0, 0, 0, 0.2);
        }

        .rank-emoji {
          font-size: 14px;
        }

        .rank-text {
          font-size: 11px;
        }

        .model-icon {
          font-size: 42px;
          margin: 12px 0;
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
          background: #ffebee;
          color: #c62828;
          font-size: 11px;
          font-weight: 500;
          border-radius: 12px;
          width: fit-content;
        }

        .model-stats {
          display: flex;
          gap: 12px;
          margin-top: 4px;
        }

        .stat {
          display: flex;
          align-items: center;
          gap: 4px;
        }

        .stat-icon {
          font-size: 14px;
        }

        .stat-value {
          font-size: 12px;
          font-weight: 600;
          color: #666;
        }

        .price {
          font-size: 14px;
          font-weight: 700;
          color: #ff5722;
          margin-top: 4px;
        }

        @keyframes flicker {
          0%, 100% {
            transform: scale(1);
            filter: brightness(1);
          }
          50% {
            transform: scale(1.15);
            filter: brightness(1.2);
          }
        }

        @media (max-width: 768px) {
          .scroll-button {
            display: none;
          }

          .model-card {
            min-width: 180px;
            max-width: 180px;
          }
        }
      `}</style>
    </div>
  );
};
