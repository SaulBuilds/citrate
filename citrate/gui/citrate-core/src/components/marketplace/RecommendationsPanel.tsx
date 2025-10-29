/**
 * RecommendationsPanel Component
 *
 * Unified recommendations panel combining all recommendation widgets.
 * Provides tabbed interface with multiple recommendation types.
 */

import React, { useState } from 'react';
import { SearchDocument } from '../../utils/search/types';
import { SimilarModels } from './SimilarModels';
import { TrendingModels } from './TrendingModels';
import { RecentlyViewed } from './RecentlyViewed';
import { CollaborativeRecommendations } from './CollaborativeRecommendations';
import { useRecommendations } from '../../hooks/useRecommendations';
import { clearHistory } from '../../utils/recommendations';

interface RecommendationsPanelProps {
  modelId?: string;
  userAddress?: string;
  models: SearchDocument[];
  onModelClick: (modelId: string) => void;
  layout?: 'tabs' | 'sections' | 'sidebar';
  collapsible?: boolean;
}

type TabType = 'similar' | 'trending' | 'recent' | 'personalized';

export const RecommendationsPanel: React.FC<RecommendationsPanelProps> = ({
  modelId,
  userAddress,
  models,
  onModelClick,
  layout = 'sections',
  collapsible = false
}) => {
  const [activeTab, setActiveTab] = useState<TabType>('similar');
  const [collapsed, setCollapsed] = useState(false);

  const {
    similarModels,
    trendingModels,
    recentlyViewed,
    collaborative,
    personalized,
    isLoading,
    refreshRecommendations
  } = useRecommendations({
    modelId,
    userAddress,
    models,
    enabled: !collapsed
  });

  const handleClearHistory = () => {
    if (confirm('Are you sure you want to clear your viewing history?')) {
      clearHistory();
      refreshRecommendations();
    }
  };

  if (collapsed) {
    return (
      <div className="recommendations-panel collapsed">
        <button className="expand-button" onClick={() => setCollapsed(false)}>
          <span className="icon">📊</span>
          <span className="text">Show Recommendations</span>
        </button>

        <style jsx>{`
          .recommendations-panel.collapsed {
            padding: 12px;
            background: #f5f5f5;
            border-radius: 8px;
          }

          .expand-button {
            width: 100%;
            display: flex;
            align-items: center;
            justify-content: center;
            gap: 8px;
            padding: 12px;
            background: white;
            border: 1px solid #e0e0e0;
            border-radius: 6px;
            cursor: pointer;
            transition: all 0.2s;
          }

          .expand-button:hover {
            background: #f0f7ff;
            border-color: #0066cc;
          }

          .icon {
            font-size: 20px;
          }

          .text {
            font-size: 14px;
            font-weight: 600;
            color: #1a1a1a;
          }
        `}</style>
      </div>
    );
  }

  // Tabs Layout
  if (layout === 'tabs') {
    return (
      <div className="recommendations-panel tabs-layout">
        <div className="panel-header">
          <h2>Recommendations</h2>
          {collapsible && (
            <button className="collapse-button" onClick={() => setCollapsed(true)}>
              ✕
            </button>
          )}
        </div>

        <div className="tabs">
          <button
            className={`tab ${activeTab === 'similar' ? 'active' : ''}`}
            onClick={() => setActiveTab('similar')}
            disabled={!modelId}
          >
            Similar
          </button>
          <button
            className={`tab ${activeTab === 'trending' ? 'active' : ''}`}
            onClick={() => setActiveTab('trending')}
          >
            Trending
          </button>
          <button
            className={`tab ${activeTab === 'recent' ? 'active' : ''}`}
            onClick={() => setActiveTab('recent')}
          >
            Recent
          </button>
          {userAddress && (
            <button
              className={`tab ${activeTab === 'personalized' ? 'active' : ''}`}
              onClick={() => setActiveTab('personalized')}
            >
              For You
            </button>
          )}
        </div>

        <div className="tab-content">
          {activeTab === 'similar' && modelId && (
            <SimilarModels
              modelId={modelId}
              similarModels={similarModels}
              isLoading={isLoading}
              onModelClick={onModelClick}
            />
          )}
          {activeTab === 'trending' && (
            <TrendingModels
              trendingModels={trendingModels}
              isLoading={isLoading}
              onModelClick={onModelClick}
            />
          )}
          {activeTab === 'recent' && (
            <RecentlyViewed
              recentModels={recentlyViewed}
              isLoading={isLoading}
              onModelClick={onModelClick}
              onClearHistory={handleClearHistory}
            />
          )}
          {activeTab === 'personalized' && userAddress && (
            <div className="personalized-section">
              <SimilarModels
                modelId={modelId || ''}
                similarModels={personalized}
                isLoading={isLoading}
                onModelClick={onModelClick}
              />
            </div>
          )}
        </div>

        <style jsx>{`
          .recommendations-panel {
            background: #ffffff;
            border-radius: 12px;
            box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
            overflow: hidden;
          }

          .panel-header {
            display: flex;
            justify-content: space-between;
            align-items: center;
            padding: 20px 24px;
            border-bottom: 1px solid #e0e0e0;
          }

          h2 {
            margin: 0;
            font-size: 20px;
            font-weight: 700;
            color: #1a1a1a;
          }

          .collapse-button {
            background: none;
            border: none;
            font-size: 20px;
            color: #666;
            cursor: pointer;
            padding: 4px 8px;
            border-radius: 4px;
            transition: all 0.2s;
          }

          .collapse-button:hover {
            background: #f5f5f5;
            color: #333;
          }

          .tabs {
            display: flex;
            gap: 4px;
            padding: 16px 24px 0 24px;
            background: #f9f9f9;
            border-bottom: 2px solid #e0e0e0;
          }

          .tab {
            padding: 12px 20px;
            background: transparent;
            border: none;
            border-radius: 8px 8px 0 0;
            font-size: 14px;
            font-weight: 600;
            color: #666;
            cursor: pointer;
            transition: all 0.2s;
            position: relative;
          }

          .tab:disabled {
            opacity: 0.4;
            cursor: not-allowed;
          }

          .tab:not(:disabled):hover {
            background: rgba(0, 102, 204, 0.05);
            color: #0066cc;
          }

          .tab.active {
            background: #ffffff;
            color: #0066cc;
          }

          .tab.active::after {
            content: '';
            position: absolute;
            bottom: -2px;
            left: 0;
            right: 0;
            height: 2px;
            background: #0066cc;
          }

          .tab-content {
            padding: 0;
          }
        `}</style>
      </div>
    );
  }

  // Sections Layout (default)
  return (
    <div className="recommendations-panel sections-layout">
      <div className="panel-header">
        <h2>Recommendations</h2>
        {collapsible && (
          <button className="collapse-button" onClick={() => setCollapsed(true)}>
            ✕
          </button>
        )}
      </div>

      <div className="sections">
        {modelId && similarModels.length > 0 && (
          <section className="section">
            <SimilarModels
              modelId={modelId}
              similarModels={similarModels}
              isLoading={isLoading}
              onModelClick={onModelClick}
            />
          </section>
        )}

        {trendingModels.length > 0 && (
          <section className="section">
            <TrendingModels
              trendingModels={trendingModels}
              isLoading={isLoading}
              onModelClick={onModelClick}
            />
          </section>
        )}

        {modelId && collaborative.length > 0 && (
          <section className="section">
            <CollaborativeRecommendations
              modelId={modelId}
              recommendations={collaborative}
              isLoading={isLoading}
              onModelClick={onModelClick}
            />
          </section>
        )}

        {recentlyViewed.length > 0 && (
          <section className="section">
            <RecentlyViewed
              recentModels={recentlyViewed}
              isLoading={isLoading}
              onModelClick={onModelClick}
              onClearHistory={handleClearHistory}
            />
          </section>
        )}

        {userAddress && personalized.length > 0 && (
          <section className="section personalized">
            <div className="personalized-header">
              <h3>
                <span className="sparkle">✨</span>
                Recommended For You
              </h3>
            </div>
            <SimilarModels
              modelId={modelId || ''}
              similarModels={personalized}
              isLoading={isLoading}
              onModelClick={onModelClick}
            />
          </section>
        )}
      </div>

      <style jsx>{`
        .recommendations-panel {
          background: #ffffff;
          border-radius: 12px;
          box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
        }

        .panel-header {
          display: flex;
          justify-content: space-between;
          align-items: center;
          padding: 20px 24px;
          border-bottom: 1px solid #e0e0e0;
        }

        h2 {
          margin: 0;
          font-size: 20px;
          font-weight: 700;
          color: #1a1a1a;
        }

        .collapse-button {
          background: none;
          border: none;
          font-size: 20px;
          color: #666;
          cursor: pointer;
          padding: 4px 8px;
          border-radius: 4px;
          transition: all 0.2s;
        }

        .collapse-button:hover {
          background: #f5f5f5;
          color: #333;
        }

        .sections {
          display: flex;
          flex-direction: column;
          gap: 0;
        }

        .section {
          padding: 24px;
          border-bottom: 1px solid #f0f0f0;
        }

        .section:last-child {
          border-bottom: none;
        }

        .section.personalized {
          background: linear-gradient(135deg, #f0f4ff 0%, #ffffff 100%);
        }

        .personalized-header {
          margin-bottom: 16px;
        }

        .personalized-header h3 {
          margin: 0;
          font-size: 18px;
          font-weight: 600;
          color: #1a1a1a;
          display: flex;
          align-items: center;
          gap: 8px;
        }

        .sparkle {
          font-size: 22px;
          animation: sparkle 2s ease-in-out infinite;
        }

        @keyframes sparkle {
          0%, 100% {
            transform: scale(1) rotate(0deg);
            filter: brightness(1);
          }
          50% {
            transform: scale(1.2) rotate(180deg);
            filter: brightness(1.3);
          }
        }

        @media (max-width: 768px) {
          .panel-header {
            padding: 16px;
          }

          .sections {
            gap: 0;
          }

          .section {
            padding: 16px;
          }
        }
      `}</style>
    </div>
  );
};
