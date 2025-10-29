/**
 * Metrics Dashboard Demo
 *
 * Example usage of the MetricsDashboard and QualityScoreBreakdown components.
 * This file demonstrates how to integrate the metrics components with mock data.
 */

import React, { useState, useEffect } from 'react';
import MetricsDashboard from '../components/marketplace/MetricsDashboard';
import QualityScoreBreakdownComponent from '../components/marketplace/QualityScoreBreakdown';
import {
  MetricsTracker,
  getMockMetricsHistory,
  QualityScoreBreakdown,
  PerformanceMetrics,
  MetricsHistory,
} from '../utils/metrics';

/**
 * Demo component showing metrics dashboard usage
 */
export const MetricsDemo: React.FC = () => {
  const [metricsTracker] = useState(() => new MetricsTracker('demo-model-001'));
  const [performanceMetrics, setPerformanceMetrics] = useState<PerformanceMetrics | null>(null);
  const [metricsHistory, setMetricsHistory] = useState<MetricsHistory | null>(null);
  const [qualityScore, setQualityScore] = useState<QualityScoreBreakdown | null>(null);

  useEffect(() => {
    // Simulate collecting metrics
    const collectMetrics = () => {
      // Record some sample inference measurements
      for (let i = 0; i < 100; i++) {
        const latency = 150 + Math.random() * 200; // 150-350ms
        const success = Math.random() > 0.02; // 98% success rate

        metricsTracker.recordInference({
          latencyMs: latency,
          success,
          tokensUsed: Math.floor(100 + Math.random() * 500),
          responseSize: Math.floor(1000 + Math.random() * 5000),
        });
      }

      // Get current metrics
      const metrics = metricsTracker.getPerformanceMetrics();
      setPerformanceMetrics(metrics);

      // Get metrics history (or use mock data for demo)
      const history = getMockMetricsHistory(30); // 30 days of data
      setMetricsHistory(history);

      // Calculate quality score
      const quality = metricsTracker.calculateQualityScore(
        4.5, // Average rating (out of 5)
        127, // Total reviews
        1542, // Total sales
        45231 // Total inferences
      );
      setQualityScore(quality);
    };

    // Initial collection
    collectMetrics();

    // Simulate real-time updates every 10 seconds
    const interval = setInterval(() => {
      // Record a new measurement
      metricsTracker.recordInference({
        latencyMs: 150 + Math.random() * 200,
        success: Math.random() > 0.02,
        tokensUsed: Math.floor(100 + Math.random() * 500),
        responseSize: Math.floor(1000 + Math.random() * 5000),
      });

      // Update metrics
      const metrics = metricsTracker.getPerformanceMetrics();
      setPerformanceMetrics(metrics);
    }, 10000);

    return () => clearInterval(interval);
  }, [metricsTracker]);

  if (!performanceMetrics || !metricsHistory || !qualityScore) {
    return (
      <div style={{ padding: '40px', textAlign: 'center' }}>
        <h2>Loading metrics...</h2>
      </div>
    );
  }

  return (
    <div style={{ padding: '20px', background: '#f9fafb', minHeight: '100vh' }}>
      <div style={{ maxWidth: '1400px', margin: '0 auto' }}>
        <h1 style={{ fontSize: '32px', fontWeight: '700', marginBottom: '24px' }}>
          Metrics Dashboard Demo
        </h1>

        {/* Full Dashboard */}
        <MetricsDashboard
          modelId="demo-model-001"
          modelName="GPT-4 Turbo Demo Model"
          performanceMetrics={performanceMetrics}
          metricsHistory={metricsHistory}
          qualityScore={qualityScore}
          onExport={(format) => {
            console.log(`Exporting metrics in ${format} format`);
          }}
        />

        {/* Standalone Quality Score Breakdown */}
        <div style={{ marginTop: '32px' }}>
          <h2 style={{ fontSize: '24px', fontWeight: '700', marginBottom: '16px' }}>
            Standalone Quality Score Component
          </h2>
          <QualityScoreBreakdownComponent qualityScore={qualityScore} showDetails={true} />
        </div>

        {/* Usage Instructions */}
        <div
          style={{
            marginTop: '32px',
            padding: '24px',
            background: 'white',
            borderRadius: '12px',
            border: '1px solid #e5e7eb',
          }}
        >
          <h2 style={{ fontSize: '20px', fontWeight: '700', marginBottom: '16px' }}>
            Usage Example
          </h2>
          <pre
            style={{
              background: '#f3f4f6',
              padding: '16px',
              borderRadius: '8px',
              overflow: 'auto',
              fontSize: '14px',
              fontFamily: 'monospace',
            }}
          >
            {`import { MetricsTracker, getMockMetricsHistory } from './utils/metrics';
import MetricsDashboard from './components/marketplace/MetricsDashboard';

// 1. Create a metrics tracker
const tracker = new MetricsTracker('your-model-id');

// 2. Record inference measurements
tracker.recordInference({
  latencyMs: 250,
  success: true,
  tokensUsed: 150,
  responseSize: 2048,
});

// 3. Get performance metrics
const performanceMetrics = tracker.getPerformanceMetrics();

// 4. Get metrics history (or use mock data)
const metricsHistory = getMockMetricsHistory(30); // 30 days

// 5. Calculate quality score
const qualityScore = tracker.calculateQualityScore(
  4.5,   // Average rating (0-5)
  127,   // Total reviews
  1542,  // Total sales
  45231  // Total inferences
);

// 6. Render the dashboard
<MetricsDashboard
  modelId="your-model-id"
  modelName="Your Model Name"
  performanceMetrics={performanceMetrics}
  metricsHistory={metricsHistory}
  qualityScore={qualityScore}
  onExport={(format) => console.log(\`Export as \${format}\`)}
/>`}
          </pre>
        </div>
      </div>
    </div>
  );
};

export default MetricsDemo;
