/**
 * Metrics Dashboard Component
 *
 * Comprehensive dashboard for displaying model performance metrics and analytics.
 * Features:
 * - Quality score breakdown
 * - Performance line charts (latency over time)
 * - Throughput bar charts
 * - Reliability metrics
 * - Time range selector
 * - Export functionality
 */

import React, { useState, useMemo } from 'react';
import {
  LineChart,
  Line,
  AreaChart,
  Area,
  BarChart,
  Bar,
  XAxis,
  YAxis,
  CartesianGrid,
  Tooltip,
  Legend,
  ResponsiveContainer,
} from 'recharts';
import {
  PerformanceMetrics,
  MetricsHistory,
  QualityScoreBreakdown,
  TimeRange,
  historyToChartData,
} from '../../utils/metrics';
import QualityScoreBreakdownComponent from './QualityScoreBreakdown';

export interface MetricsDashboardProps {
  modelId: string;
  modelName: string;
  performanceMetrics: PerformanceMetrics;
  metricsHistory: MetricsHistory;
  qualityScore: QualityScoreBreakdown;
  className?: string;
  onExport?: (format: 'json' | 'csv') => void;
}

export const MetricsDashboard: React.FC<MetricsDashboardProps> = ({
  modelId,
  modelName,
  performanceMetrics,
  metricsHistory,
  qualityScore,
  className = '',
  onExport,
}) => {
  const [timeRange, setTimeRange] = useState<TimeRange>('7d');
  const [activeTab, setActiveTab] = useState<'overview' | 'performance' | 'reliability' | 'quality'>('overview');

  // Convert metrics history to chart data
  const chartData = useMemo(() => {
    return historyToChartData(metricsHistory);
  }, [metricsHistory]);

  // Filter chart data by time range
  const filteredChartData = useMemo(() => {
    const now = Date.now();
    let cutoff = 0;

    switch (timeRange) {
      case '24h':
        cutoff = now - 24 * 3600000;
        break;
      case '7d':
        cutoff = now - 7 * 24 * 3600000;
        break;
      case '30d':
        cutoff = now - 30 * 24 * 3600000;
        break;
      case '90d':
        cutoff = now - 90 * 24 * 3600000;
        break;
      case 'all':
        cutoff = 0;
        break;
    }

    return chartData.filter((d) => d.timestamp >= cutoff);
  }, [chartData, timeRange]);

  const handleExport = (format: 'json' | 'csv') => {
    if (onExport) {
      onExport(format);
    } else {
      // Default export behavior
      const data = {
        modelId,
        modelName,
        timestamp: Date.now(),
        performanceMetrics,
        metricsHistory,
        qualityScore,
      };

      if (format === 'json') {
        const blob = new Blob([JSON.stringify(data, null, 2)], { type: 'application/json' });
        const url = URL.createObjectURL(blob);
        const a = document.createElement('a');
        a.href = url;
        a.download = `metrics-${modelId}-${Date.now()}.json`;
        a.click();
        URL.revokeObjectURL(url);
      } else if (format === 'csv') {
        // Simple CSV export of chart data
        const headers = ['Timestamp', 'Date', 'Latency (ms)', 'Throughput', 'Error Rate (%)', 'Success Rate (%)'];
        const rows = filteredChartData.map((d) => [
          d.timestamp,
          d.date,
          d.latency || '',
          d.throughput || '',
          d.errorRate || '',
          d.successRate || '',
        ]);

        const csv = [headers, ...rows].map((row) => row.join(',')).join('\n');
        const blob = new Blob([csv], { type: 'text/csv' });
        const url = URL.createObjectURL(blob);
        const a = document.createElement('a');
        a.href = url;
        a.download = `metrics-${modelId}-${Date.now()}.csv`;
        a.click();
        URL.revokeObjectURL(url);
      }
    }
  };

  const timeRangeOptions: { value: TimeRange; label: string }[] = [
    { value: '24h', label: '24 Hours' },
    { value: '7d', label: '7 Days' },
    { value: '30d', label: '30 Days' },
    { value: '90d', label: '90 Days' },
    { value: 'all', label: 'All Time' },
  ];

  const tabs = [
    { id: 'overview' as const, label: 'Overview', icon: '📊' },
    { id: 'performance' as const, label: 'Performance', icon: '⚡' },
    { id: 'reliability' as const, label: 'Reliability', icon: '🛡️' },
    { id: 'quality' as const, label: 'Quality Score', icon: '⭐' },
  ];

  // Custom tooltip for charts
  const CustomTooltip = ({ active, payload, label }: any) => {
    if (active && payload && payload.length) {
      return (
        <div className="custom-tooltip">
          <p className="tooltip-label">{label}</p>
          {payload.map((entry: any, index: number) => (
            <p key={index} style={{ color: entry.color }}>
              {entry.name}: {typeof entry.value === 'number' ? entry.value.toFixed(2) : entry.value}
            </p>
          ))}
        </div>
      );
    }
    return null;
  };

  return (
    <div className={`metrics-dashboard ${className}`}>
      {/* Header */}
      <div className="dashboard-header">
        <div className="header-content">
          <h2 className="dashboard-title">{modelName}</h2>
          <p className="dashboard-subtitle">Model ID: {modelId}</p>
        </div>

        <div className="header-actions">
          <div className="time-range-selector">
            {timeRangeOptions.map((option) => (
              <button
                key={option.value}
                className={`time-range-btn ${timeRange === option.value ? 'active' : ''}`}
                onClick={() => setTimeRange(option.value)}
              >
                {option.label}
              </button>
            ))}
          </div>

          <button className="export-btn" onClick={() => handleExport('json')}>
            <svg
              xmlns="http://www.w3.org/2000/svg"
              width="16"
              height="16"
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              strokeWidth="2"
              strokeLinecap="round"
              strokeLinejoin="round"
            >
              <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4" />
              <polyline points="7 10 12 15 17 10" />
              <line x1="12" y1="15" x2="12" y2="3" />
            </svg>
            Export
          </button>
        </div>
      </div>

      {/* Navigation Tabs */}
      <div className="dashboard-tabs">
        {tabs.map((tab) => (
          <button
            key={tab.id}
            className={`tab-btn ${activeTab === tab.id ? 'active' : ''}`}
            onClick={() => setActiveTab(tab.id)}
          >
            <span className="tab-icon">{tab.icon}</span>
            <span className="tab-label">{tab.label}</span>
          </button>
        ))}
      </div>

      {/* Tab Content */}
      <div className="dashboard-content">
        {activeTab === 'overview' && (
          <div className="overview-tab">
            {/* Key Metrics Cards */}
            <div className="metrics-grid">
              <div className="metric-card">
                <div className="metric-icon" style={{ background: '#eff6ff', color: '#3b82f6' }}>
                  ⚡
                </div>
                <div className="metric-content">
                  <div className="metric-label">Avg Latency</div>
                  <div className="metric-value">{performanceMetrics.avgLatency.toFixed(0)}ms</div>
                  <div className="metric-detail">
                    P95: {performanceMetrics.p95Latency.toFixed(0)}ms
                  </div>
                </div>
              </div>

              <div className="metric-card">
                <div className="metric-icon" style={{ background: '#f0fdf4', color: '#10b981' }}>
                  📈
                </div>
                <div className="metric-content">
                  <div className="metric-label">Throughput</div>
                  <div className="metric-value">
                    {performanceMetrics.throughputPerMinute.toFixed(1)}/min
                  </div>
                  <div className="metric-detail">
                    {performanceMetrics.totalInferences.toLocaleString()} total
                  </div>
                </div>
              </div>

              <div className="metric-card">
                <div className="metric-icon" style={{ background: '#fef3f2', color: '#ef4444' }}>
                  ⚠️
                </div>
                <div className="metric-content">
                  <div className="metric-label">Error Rate</div>
                  <div className="metric-value">{performanceMetrics.errorRate.toFixed(2)}%</div>
                  <div className="metric-detail">
                    {performanceMetrics.failedInferences} failures
                  </div>
                </div>
              </div>

              <div className="metric-card">
                <div className="metric-icon" style={{ background: '#faf5ff', color: '#8b5cf6' }}>
                  ⭐
                </div>
                <div className="metric-content">
                  <div className="metric-label">Quality Score</div>
                  <div className="metric-value">{qualityScore.overall}</div>
                  <div className="metric-detail">Out of 100</div>
                </div>
              </div>
            </div>

            {/* Latency Chart */}
            <div className="chart-card">
              <h3 className="chart-title">Latency Over Time</h3>
              <ResponsiveContainer width="100%" height={300}>
                <AreaChart data={filteredChartData}>
                  <defs>
                    <linearGradient id="colorLatency" x1="0" y1="0" x2="0" y2="1">
                      <stop offset="5%" stopColor="#3b82f6" stopOpacity={0.3} />
                      <stop offset="95%" stopColor="#3b82f6" stopOpacity={0} />
                    </linearGradient>
                  </defs>
                  <CartesianGrid strokeDasharray="3 3" stroke="#e5e7eb" />
                  <XAxis
                    dataKey="date"
                    stroke="#6b7280"
                    style={{ fontSize: '12px' }}
                  />
                  <YAxis
                    stroke="#6b7280"
                    style={{ fontSize: '12px' }}
                    label={{ value: 'Latency (ms)', angle: -90, position: 'insideLeft' }}
                  />
                  <Tooltip content={<CustomTooltip />} />
                  <Area
                    type="monotone"
                    dataKey="latency"
                    stroke="#3b82f6"
                    strokeWidth={2}
                    fill="url(#colorLatency)"
                  />
                </AreaChart>
              </ResponsiveContainer>
            </div>

            {/* Throughput Chart */}
            <div className="chart-card">
              <h3 className="chart-title">Throughput Over Time</h3>
              <ResponsiveContainer width="100%" height={300}>
                <BarChart data={filteredChartData}>
                  <CartesianGrid strokeDasharray="3 3" stroke="#e5e7eb" />
                  <XAxis
                    dataKey="date"
                    stroke="#6b7280"
                    style={{ fontSize: '12px' }}
                  />
                  <YAxis
                    stroke="#6b7280"
                    style={{ fontSize: '12px' }}
                    label={{ value: 'Requests/sec', angle: -90, position: 'insideLeft' }}
                  />
                  <Tooltip content={<CustomTooltip />} />
                  <Bar dataKey="throughput" fill="#10b981" radius={[4, 4, 0, 0]} />
                </BarChart>
              </ResponsiveContainer>
            </div>
          </div>
        )}

        {activeTab === 'performance' && (
          <div className="performance-tab">
            {/* Percentile Chart */}
            <div className="chart-card">
              <h3 className="chart-title">Latency Distribution</h3>
              <div className="percentile-bars">
                {[
                  { label: 'P50 (Median)', value: performanceMetrics.p50Latency, color: '#10b981' },
                  { label: 'P90', value: performanceMetrics.p90Latency, color: '#3b82f6' },
                  { label: 'P95', value: performanceMetrics.p95Latency, color: '#f59e0b' },
                  { label: 'P99', value: performanceMetrics.p99Latency, color: '#ef4444' },
                ].map((percentile) => (
                  <div key={percentile.label} className="percentile-row">
                    <div className="percentile-label">{percentile.label}</div>
                    <div className="percentile-bar-container">
                      <div
                        className="percentile-bar"
                        style={{
                          width: `${(percentile.value / performanceMetrics.maxLatency) * 100}%`,
                          backgroundColor: percentile.color,
                        }}
                      />
                    </div>
                    <div className="percentile-value">{percentile.value.toFixed(0)}ms</div>
                  </div>
                ))}
              </div>
            </div>

            {/* Latency Line Chart with P50/P95/P99 */}
            <div className="chart-card">
              <h3 className="chart-title">Latency Trends</h3>
              <ResponsiveContainer width="100%" height={400}>
                <LineChart data={filteredChartData}>
                  <CartesianGrid strokeDasharray="3 3" stroke="#e5e7eb" />
                  <XAxis
                    dataKey="date"
                    stroke="#6b7280"
                    style={{ fontSize: '12px' }}
                  />
                  <YAxis
                    stroke="#6b7280"
                    style={{ fontSize: '12px' }}
                    label={{ value: 'Latency (ms)', angle: -90, position: 'insideLeft' }}
                  />
                  <Tooltip content={<CustomTooltip />} />
                  <Legend />
                  <Line
                    type="monotone"
                    dataKey="latency"
                    stroke="#3b82f6"
                    strokeWidth={2}
                    name="Average Latency"
                    dot={false}
                  />
                </LineChart>
              </ResponsiveContainer>
            </div>

            {/* Performance Stats Table */}
            <div className="chart-card">
              <h3 className="chart-title">Performance Statistics</h3>
              <div className="stats-table">
                <div className="stat-row">
                  <span className="stat-label">Minimum Latency</span>
                  <span className="stat-value">{performanceMetrics.minLatency.toFixed(0)}ms</span>
                </div>
                <div className="stat-row">
                  <span className="stat-label">Maximum Latency</span>
                  <span className="stat-value">{performanceMetrics.maxLatency.toFixed(0)}ms</span>
                </div>
                <div className="stat-row">
                  <span className="stat-label">Average Latency</span>
                  <span className="stat-value">{performanceMetrics.avgLatency.toFixed(0)}ms</span>
                </div>
                <div className="stat-row">
                  <span className="stat-label">Total Inferences</span>
                  <span className="stat-value">{performanceMetrics.totalInferences.toLocaleString()}</span>
                </div>
                <div className="stat-row">
                  <span className="stat-label">Throughput (per hour)</span>
                  <span className="stat-value">{performanceMetrics.throughputPerHour.toFixed(1)}</span>
                </div>
              </div>
            </div>
          </div>
        )}

        {activeTab === 'reliability' && (
          <div className="reliability-tab">
            {/* Success vs Error Rate */}
            <div className="chart-card">
              <h3 className="chart-title">Success vs Error Rate</h3>
              <ResponsiveContainer width="100%" height={300}>
                <AreaChart data={filteredChartData}>
                  <defs>
                    <linearGradient id="colorSuccess" x1="0" y1="0" x2="0" y2="1">
                      <stop offset="5%" stopColor="#10b981" stopOpacity={0.3} />
                      <stop offset="95%" stopColor="#10b981" stopOpacity={0} />
                    </linearGradient>
                    <linearGradient id="colorError" x1="0" y1="0" x2="0" y2="1">
                      <stop offset="5%" stopColor="#ef4444" stopOpacity={0.3} />
                      <stop offset="95%" stopColor="#ef4444" stopOpacity={0} />
                    </linearGradient>
                  </defs>
                  <CartesianGrid strokeDasharray="3 3" stroke="#e5e7eb" />
                  <XAxis dataKey="date" stroke="#6b7280" style={{ fontSize: '12px' }} />
                  <YAxis
                    stroke="#6b7280"
                    style={{ fontSize: '12px' }}
                    label={{ value: 'Rate (%)', angle: -90, position: 'insideLeft' }}
                  />
                  <Tooltip content={<CustomTooltip />} />
                  <Legend />
                  <Area
                    type="monotone"
                    dataKey="successRate"
                    stroke="#10b981"
                    strokeWidth={2}
                    fill="url(#colorSuccess)"
                    name="Success Rate"
                  />
                  <Area
                    type="monotone"
                    dataKey="errorRate"
                    stroke="#ef4444"
                    strokeWidth={2}
                    fill="url(#colorError)"
                    name="Error Rate"
                  />
                </AreaChart>
              </ResponsiveContainer>
            </div>

            {/* Reliability Metrics Grid */}
            <div className="reliability-grid">
              <div className="reliability-card">
                <div className="reliability-icon" style={{ background: '#f0fdf4', color: '#10b981' }}>
                  ✓
                </div>
                <div className="reliability-content">
                  <div className="reliability-label">Successful Requests</div>
                  <div className="reliability-value">
                    {performanceMetrics.successfulInferences.toLocaleString()}
                  </div>
                  <div className="reliability-percentage">
                    {((performanceMetrics.successfulInferences / performanceMetrics.totalInferences) * 100).toFixed(2)}%
                  </div>
                </div>
              </div>

              <div className="reliability-card">
                <div className="reliability-icon" style={{ background: '#fef3f2', color: '#ef4444' }}>
                  ✕
                </div>
                <div className="reliability-content">
                  <div className="reliability-label">Failed Requests</div>
                  <div className="reliability-value">
                    {performanceMetrics.failedInferences.toLocaleString()}
                  </div>
                  <div className="reliability-percentage">
                    {performanceMetrics.errorRate.toFixed(2)}%
                  </div>
                </div>
              </div>

              <div className="reliability-card">
                <div className="reliability-icon" style={{ background: '#eff6ff', color: '#3b82f6' }}>
                  ⏱️
                </div>
                <div className="reliability-content">
                  <div className="reliability-label">Uptime</div>
                  <div className="reliability-value">
                    {(100 - performanceMetrics.errorRate).toFixed(2)}%
                  </div>
                  <div className="reliability-percentage">
                    {performanceMetrics.durationSeconds.toFixed(0)}s monitored
                  </div>
                </div>
              </div>
            </div>
          </div>
        )}

        {activeTab === 'quality' && (
          <div className="quality-tab">
            <QualityScoreBreakdownComponent qualityScore={qualityScore} showDetails={true} />
          </div>
        )}
      </div>

      <style jsx>{`
        .metrics-dashboard {
          display: flex;
          flex-direction: column;
          gap: 24px;
          padding: 24px;
          background: #f9fafb;
          border-radius: 12px;
          min-height: 600px;
        }

        .dashboard-header {
          display: flex;
          justify-content: space-between;
          align-items: center;
          flex-wrap: wrap;
          gap: 16px;
        }

        .header-content {
          flex: 1;
        }

        .dashboard-title {
          font-size: 24px;
          font-weight: 700;
          color: #111827;
          margin: 0 0 4px 0;
        }

        .dashboard-subtitle {
          font-size: 14px;
          color: #6b7280;
          margin: 0;
          font-family: monospace;
        }

        .header-actions {
          display: flex;
          align-items: center;
          gap: 12px;
          flex-wrap: wrap;
        }

        .time-range-selector {
          display: flex;
          gap: 4px;
          background: white;
          padding: 4px;
          border-radius: 8px;
          border: 1px solid #e5e7eb;
        }

        .time-range-btn {
          padding: 6px 12px;
          border: none;
          background: transparent;
          color: #6b7280;
          font-size: 13px;
          font-weight: 600;
          border-radius: 6px;
          cursor: pointer;
          transition: all 0.2s;
        }

        .time-range-btn:hover {
          background: #f3f4f6;
          color: #111827;
        }

        .time-range-btn.active {
          background: #3b82f6;
          color: white;
        }

        .export-btn {
          display: flex;
          align-items: center;
          gap: 6px;
          padding: 8px 16px;
          background: white;
          border: 1px solid #e5e7eb;
          border-radius: 8px;
          color: #374151;
          font-size: 14px;
          font-weight: 600;
          cursor: pointer;
          transition: all 0.2s;
        }

        .export-btn:hover {
          background: #f9fafb;
          border-color: #3b82f6;
          color: #3b82f6;
        }

        .dashboard-tabs {
          display: flex;
          gap: 8px;
          background: white;
          padding: 6px;
          border-radius: 10px;
          border: 1px solid #e5e7eb;
          overflow-x: auto;
        }

        .tab-btn {
          display: flex;
          align-items: center;
          gap: 8px;
          padding: 10px 20px;
          border: none;
          background: transparent;
          color: #6b7280;
          font-size: 14px;
          font-weight: 600;
          border-radius: 8px;
          cursor: pointer;
          transition: all 0.2s;
          white-space: nowrap;
        }

        .tab-btn:hover {
          background: #f3f4f6;
          color: #111827;
        }

        .tab-btn.active {
          background: #3b82f6;
          color: white;
        }

        .tab-icon {
          font-size: 18px;
        }

        .dashboard-content {
          background: white;
          border-radius: 12px;
          padding: 24px;
          border: 1px solid #e5e7eb;
        }

        .overview-tab,
        .performance-tab,
        .reliability-tab,
        .quality-tab {
          display: flex;
          flex-direction: column;
          gap: 24px;
        }

        .metrics-grid {
          display: grid;
          grid-template-columns: repeat(auto-fit, minmax(240px, 1fr));
          gap: 16px;
        }

        .metric-card {
          display: flex;
          gap: 16px;
          padding: 20px;
          background: white;
          border: 1px solid #e5e7eb;
          border-radius: 12px;
          transition: all 0.2s;
        }

        .metric-card:hover {
          border-color: #3b82f6;
          box-shadow: 0 4px 12px rgba(59, 130, 246, 0.1);
        }

        .metric-icon {
          width: 48px;
          height: 48px;
          display: flex;
          align-items: center;
          justify-content: center;
          border-radius: 10px;
          font-size: 24px;
        }

        .metric-content {
          flex: 1;
        }

        .metric-label {
          font-size: 13px;
          color: #6b7280;
          font-weight: 600;
          margin-bottom: 4px;
        }

        .metric-value {
          font-size: 28px;
          font-weight: 800;
          color: #111827;
          line-height: 1;
          margin-bottom: 4px;
        }

        .metric-detail {
          font-size: 12px;
          color: #9ca3af;
        }

        .chart-card {
          padding: 24px;
          background: white;
          border: 1px solid #e5e7eb;
          border-radius: 12px;
        }

        .chart-title {
          font-size: 16px;
          font-weight: 700;
          color: #111827;
          margin: 0 0 20px 0;
        }

        .custom-tooltip {
          background: white;
          padding: 12px;
          border: 1px solid #e5e7eb;
          border-radius: 8px;
          box-shadow: 0 4px 12px rgba(0, 0, 0, 0.1);
        }

        .tooltip-label {
          font-size: 12px;
          font-weight: 600;
          color: #111827;
          margin: 0 0 8px 0;
        }

        .custom-tooltip p {
          font-size: 12px;
          margin: 4px 0;
          font-weight: 600;
        }

        .percentile-bars {
          display: flex;
          flex-direction: column;
          gap: 16px;
        }

        .percentile-row {
          display: flex;
          align-items: center;
          gap: 12px;
        }

        .percentile-label {
          width: 120px;
          font-size: 14px;
          font-weight: 600;
          color: #374151;
        }

        .percentile-bar-container {
          flex: 1;
          height: 32px;
          background: #f3f4f6;
          border-radius: 6px;
          overflow: hidden;
        }

        .percentile-bar {
          height: 100%;
          border-radius: 6px;
          transition: width 0.6s ease-in-out;
        }

        .percentile-value {
          width: 80px;
          text-align: right;
          font-size: 16px;
          font-weight: 700;
          color: #111827;
        }

        .stats-table {
          display: flex;
          flex-direction: column;
          gap: 12px;
        }

        .stat-row {
          display: flex;
          justify-content: space-between;
          align-items: center;
          padding: 12px;
          background: #f9fafb;
          border-radius: 8px;
        }

        .stat-label {
          font-size: 14px;
          color: #6b7280;
          font-weight: 600;
        }

        .stat-value {
          font-size: 16px;
          color: #111827;
          font-weight: 700;
        }

        .reliability-grid {
          display: grid;
          grid-template-columns: repeat(auto-fit, minmax(260px, 1fr));
          gap: 16px;
        }

        .reliability-card {
          display: flex;
          gap: 16px;
          padding: 24px;
          background: white;
          border: 1px solid #e5e7eb;
          border-radius: 12px;
        }

        .reliability-icon {
          width: 56px;
          height: 56px;
          display: flex;
          align-items: center;
          justify-content: center;
          border-radius: 12px;
          font-size: 28px;
        }

        .reliability-content {
          flex: 1;
        }

        .reliability-label {
          font-size: 14px;
          color: #6b7280;
          font-weight: 600;
          margin-bottom: 8px;
        }

        .reliability-value {
          font-size: 32px;
          font-weight: 800;
          color: #111827;
          line-height: 1;
          margin-bottom: 4px;
        }

        .reliability-percentage {
          font-size: 13px;
          color: #9ca3af;
          font-weight: 600;
        }

        @media (max-width: 768px) {
          .metrics-dashboard {
            padding: 16px;
          }

          .dashboard-header {
            flex-direction: column;
            align-items: flex-start;
          }

          .header-actions {
            width: 100%;
            flex-direction: column;
          }

          .time-range-selector {
            width: 100%;
            overflow-x: auto;
          }

          .export-btn {
            width: 100%;
            justify-content: center;
          }

          .dashboard-tabs {
            overflow-x: auto;
          }

          .dashboard-content {
            padding: 16px;
          }

          .metrics-grid {
            grid-template-columns: 1fr;
          }

          .metric-value {
            font-size: 24px;
          }
        }
      `}</style>
    </div>
  );
};

export default MetricsDashboard;
