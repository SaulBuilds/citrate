/**
 * Quality Score Breakdown Component
 *
 * Displays a visual breakdown of the quality score components with detailed metrics.
 * Shows 4 components: Rating (40%), Performance (30%), Reliability (20%), Engagement (10%)
 */

import React from 'react';
import { QualityScoreBreakdown } from '../../utils/metrics';

export interface QualityScoreBreakdownProps {
  qualityScore: QualityScoreBreakdown;
  className?: string;
  showDetails?: boolean;
}

export const QualityScoreBreakdownComponent: React.FC<QualityScoreBreakdownProps> = ({
  qualityScore,
  className = '',
  showDetails = true,
}) => {
  const getScoreColor = (score: number): string => {
    if (score >= 90) return '#10b981'; // green
    if (score >= 80) return '#3b82f6'; // blue
    if (score >= 70) return '#8b5cf6'; // purple
    if (score >= 60) return '#f59e0b'; // amber
    return '#ef4444'; // red
  };

  const getScoreGrade = (score: number): string => {
    if (score >= 90) return 'Excellent';
    if (score >= 80) return 'Very Good';
    if (score >= 70) return 'Good';
    if (score >= 60) return 'Fair';
    return 'Needs Improvement';
  };

  const components = [
    {
      name: 'Rating',
      data: qualityScore.rating,
      color: '#10b981',
      icon: '⭐',
      description: 'User ratings and reviews',
    },
    {
      name: 'Performance',
      data: qualityScore.performance,
      color: '#3b82f6',
      icon: '⚡',
      description: 'Speed and consistency',
    },
    {
      name: 'Reliability',
      data: qualityScore.reliability,
      color: '#8b5cf6',
      icon: '🛡️',
      description: 'Uptime and error rate',
    },
    {
      name: 'Engagement',
      data: qualityScore.engagement,
      color: '#f59e0b',
      icon: '📈',
      description: 'Usage and adoption',
    },
  ];

  return (
    <div className={`quality-breakdown ${className}`}>
      {/* Overall Score */}
      <div className="overall-score-section">
        <div className="score-ring-container">
          <svg width="160" height="160" viewBox="0 0 160 160">
            <circle
              cx="80"
              cy="80"
              r="70"
              fill="none"
              stroke="#e5e7eb"
              strokeWidth="12"
            />
            <circle
              cx="80"
              cy="80"
              r="70"
              fill="none"
              stroke={getScoreColor(qualityScore.overall)}
              strokeWidth="12"
              strokeDasharray={`${(qualityScore.overall / 100) * 439.8} 439.8`}
              strokeLinecap="round"
              transform="rotate(-90 80 80)"
              className="score-progress"
            />
          </svg>
          <div className="score-content">
            <div className="score-value">{qualityScore.overall}</div>
            <div className="score-max">/100</div>
          </div>
        </div>
        <div className="score-details">
          <h3 className="score-grade">{getScoreGrade(qualityScore.overall)}</h3>
          <p className="score-description">Overall Quality Score</p>
        </div>
      </div>

      {/* Component Breakdown */}
      <div className="components-grid">
        {components.map((component) => (
          <div key={component.name} className="component-card">
            <div className="component-header">
              <span className="component-icon">{component.icon}</span>
              <div className="component-info">
                <h4 className="component-name">{component.name}</h4>
                <p className="component-description">{component.description}</p>
              </div>
              <div className="component-score" style={{ color: component.color }}>
                {component.data.score}
              </div>
            </div>

            {/* Progress Bar */}
            <div className="progress-bar-container">
              <div
                className="progress-bar"
                style={{
                  width: `${component.data.score}%`,
                  backgroundColor: component.color,
                }}
              />
            </div>

            <div className="component-weight">
              Weight: {Math.round(component.data.weight * 100)}%
            </div>

            {/* Detailed Metrics */}
            {showDetails && (
              <div className="component-details">
                {component.name === 'Rating' && 'averageStars' in component.data.details && (
                  <>
                    <div className="detail-item">
                      <span className="detail-label">Average:</span>
                      <span className="detail-value">
                        {component.data.details.averageStars.toFixed(1)} ⭐
                      </span>
                    </div>
                    <div className="detail-item">
                      <span className="detail-label">Reviews:</span>
                      <span className="detail-value">
                        {component.data.details.totalReviews.toLocaleString()}
                      </span>
                    </div>
                    <div className="detail-item">
                      <span className="detail-label">Trend:</span>
                      <span className="detail-value detail-trend">
                        {component.data.details.recentTrend === 'up' && '↗️ Up'}
                        {component.data.details.recentTrend === 'down' && '↘️ Down'}
                        {component.data.details.recentTrend === 'stable' && '→ Stable'}
                      </span>
                    </div>
                  </>
                )}

                {component.name === 'Performance' && 'avgLatency' in component.data.details && (
                  <>
                    <div className="detail-item">
                      <span className="detail-label">Avg Latency:</span>
                      <span className="detail-value">
                        {component.data.details.avgLatency.toFixed(0)}ms
                      </span>
                    </div>
                    <div className="detail-item">
                      <span className="detail-label">Reliability:</span>
                      <span className="detail-value">
                        {component.data.details.reliability.toFixed(1)}%
                      </span>
                    </div>
                    <div className="detail-item">
                      <span className="detail-label">Consistency:</span>
                      <span className="detail-value">
                        {component.data.details.consistencyScore.toFixed(1)}%
                      </span>
                    </div>
                  </>
                )}

                {component.name === 'Reliability' && 'uptime' in component.data.details && (
                  <>
                    <div className="detail-item">
                      <span className="detail-label">Uptime:</span>
                      <span className="detail-value">
                        {component.data.details.uptime.toFixed(2)}%
                      </span>
                    </div>
                    <div className="detail-item">
                      <span className="detail-label">Error Rate:</span>
                      <span className="detail-value">
                        {component.data.details.errorRate.toFixed(2)}%
                      </span>
                    </div>
                    <div className="detail-item">
                      <span className="detail-label">MTBF:</span>
                      <span className="detail-value">
                        {component.data.details.meanTimeBetweenFailures.toFixed(1)}h
                      </span>
                    </div>
                  </>
                )}

                {component.name === 'Engagement' && 'totalSales' in component.data.details && (
                  <>
                    <div className="detail-item">
                      <span className="detail-label">Sales:</span>
                      <span className="detail-value">
                        {component.data.details.totalSales.toLocaleString()}
                      </span>
                    </div>
                    <div className="detail-item">
                      <span className="detail-label">Inferences:</span>
                      <span className="detail-value">
                        {component.data.details.totalInferences.toLocaleString()}
                      </span>
                    </div>
                    <div className="detail-item">
                      <span className="detail-label">Active Users:</span>
                      <span className="detail-value">
                        {component.data.details.activeUsers.toLocaleString()}
                      </span>
                    </div>
                  </>
                )}
              </div>
            )}
          </div>
        ))}
      </div>

      <style jsx>{`
        .quality-breakdown {
          display: flex;
          flex-direction: column;
          gap: 32px;
          padding: 24px;
          background: white;
          border-radius: 12px;
          border: 1px solid #e5e7eb;
        }

        .overall-score-section {
          display: flex;
          align-items: center;
          gap: 32px;
          padding: 24px;
          background: linear-gradient(135deg, #f9fafb 0%, #f3f4f6 100%);
          border-radius: 12px;
        }

        .score-ring-container {
          position: relative;
          width: 160px;
          height: 160px;
        }

        .score-progress {
          transition: stroke-dasharray 1s ease-in-out;
        }

        .score-content {
          position: absolute;
          top: 50%;
          left: 50%;
          transform: translate(-50%, -50%);
          text-align: center;
        }

        .score-value {
          font-size: 48px;
          font-weight: 800;
          color: #111827;
          line-height: 1;
        }

        .score-max {
          font-size: 18px;
          color: #6b7280;
          font-weight: 600;
        }

        .score-details {
          flex: 1;
        }

        .score-grade {
          font-size: 28px;
          font-weight: 700;
          color: #111827;
          margin: 0 0 8px 0;
        }

        .score-description {
          font-size: 16px;
          color: #6b7280;
          margin: 0;
        }

        .components-grid {
          display: grid;
          grid-template-columns: repeat(auto-fit, minmax(280px, 1fr));
          gap: 20px;
        }

        .component-card {
          background: white;
          border: 1px solid #e5e7eb;
          border-radius: 12px;
          padding: 20px;
          transition: all 0.2s ease;
        }

        .component-card:hover {
          border-color: #3b82f6;
          box-shadow: 0 4px 12px rgba(59, 130, 246, 0.1);
        }

        .component-header {
          display: flex;
          align-items: flex-start;
          gap: 12px;
          margin-bottom: 16px;
        }

        .component-icon {
          font-size: 28px;
          line-height: 1;
        }

        .component-info {
          flex: 1;
        }

        .component-name {
          font-size: 16px;
          font-weight: 700;
          color: #111827;
          margin: 0 0 4px 0;
        }

        .component-description {
          font-size: 13px;
          color: #6b7280;
          margin: 0;
        }

        .component-score {
          font-size: 32px;
          font-weight: 800;
          line-height: 1;
        }

        .progress-bar-container {
          width: 100%;
          height: 8px;
          background: #e5e7eb;
          border-radius: 4px;
          overflow: hidden;
          margin-bottom: 12px;
        }

        .progress-bar {
          height: 100%;
          border-radius: 4px;
          transition: width 0.6s ease-in-out;
        }

        .component-weight {
          font-size: 12px;
          color: #6b7280;
          font-weight: 600;
          margin-bottom: 12px;
        }

        .component-details {
          display: flex;
          flex-direction: column;
          gap: 8px;
          padding-top: 12px;
          border-top: 1px solid #f3f4f6;
        }

        .detail-item {
          display: flex;
          justify-content: space-between;
          align-items: center;
          font-size: 13px;
        }

        .detail-label {
          color: #6b7280;
          font-weight: 500;
        }

        .detail-value {
          color: #111827;
          font-weight: 600;
        }

        .detail-trend {
          font-size: 14px;
        }

        @media (max-width: 768px) {
          .quality-breakdown {
            padding: 16px;
            gap: 24px;
          }

          .overall-score-section {
            flex-direction: column;
            text-align: center;
            padding: 20px;
          }

          .components-grid {
            grid-template-columns: 1fr;
          }

          .score-value {
            font-size: 42px;
          }

          .score-grade {
            font-size: 24px;
          }
        }
      `}</style>
    </div>
  );
};

export default QualityScoreBreakdownComponent;
