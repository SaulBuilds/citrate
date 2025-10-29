# Sprint 5: Technical Tasks - Advanced Marketplace Features & Optimization

**Sprint Goal:** Build advanced marketplace features and optimize performance

---

## Day 1: Model Comparison Tool (6.5 hours)

### Task 1.1: Design Comparison UI/UX and Data Structure
**Estimated Time:** 1 hour
**Assigned To:** Developer
**Priority:** P0

#### Implementation Steps
1. Design comparison table layout
2. Define comparison data structure
3. Plan state management approach
4. Design export formats

#### Data Structure
```typescript
// gui/citrate-core/src/types/comparison.ts
export interface ComparisonSelection {
  modelIds: string[];
  maxModels: number; // 5
  timestamp: number;
}

export interface ComparisonData {
  models: ComparisonModel[];
  categories: ComparisonCategory[];
}

export interface ComparisonModel {
  modelId: string;
  name: string;
  creator: string;
  category: ModelCategory;
  framework: string;
  basePrice: bigint;
  discountPrice: bigint;
  averageRating: number;
  reviewCount: number;
  qualityScore: number;
  latencyP50: number;
  latencyP95: number;
  latencyP99: number;
  successRate: number;
  totalInferences: number;
  totalSales: number;
  inputFormats: string[];
  outputFormats: string[];
  modelSize: string;
  parameters: number;
  metadata: ModelMetadata;
}

export interface ComparisonCategory {
  name: string;
  rows: ComparisonRow[];
}

export interface ComparisonRow {
  label: string;
  key: string;
  type: 'text' | 'number' | 'currency' | 'percentage' | 'array';
  bestIndex?: number; // Index of best value
  worstIndex?: number; // Index of worst value
  values: any[];
}
```

#### Acceptance Criteria
- [ ] Data structure supports all comparison fields
- [ ] State management plan documented
- [ ] UI mockup created
- [ ] Export format defined

---

### Task 1.2: Build ModelComparisonTable Component
**Estimated Time:** 2 hours
**Assigned To:** Developer
**Priority:** P0

#### Implementation Steps
1. Create ComparisonTable component
2. Implement responsive table layout
3. Add visual indicators (best/worst)
4. Implement expand/collapse sections

#### Code Structure
```typescript
// gui/citrate-core/src/components/comparison/ComparisonTable.tsx
import React, { useMemo, useState } from 'react';
import { ChevronDown, ChevronUp } from 'lucide-react';
import { ComparisonData, ComparisonModel } from '../../types/comparison';

interface ComparisonTableProps {
  models: ComparisonModel[];
  onRemoveModel: (modelId: string) => void;
}

export const ComparisonTable: React.FC<ComparisonTableProps> = ({
  models,
  onRemoveModel,
}) => {
  const [expandedSections, setExpandedSections] = useState<Set<string>>(
    new Set(['basic', 'pricing', 'quality'])
  );

  const comparisonData = useMemo(() => {
    return buildComparisonData(models);
  }, [models]);

  const toggleSection = (sectionName: string) => {
    setExpandedSections(prev => {
      const next = new Set(prev);
      if (next.has(sectionName)) {
        next.delete(sectionName);
      } else {
        next.add(sectionName);
      }
      return next;
    });
  };

  return (
    <div className="comparison-table-container">
      <table className="comparison-table">
        <thead>
          <tr>
            <th className="sticky-header category-header">Feature</th>
            {models.map(model => (
              <th key={model.modelId} className="sticky-header model-header">
                <div className="model-header-content">
                  <span className="model-name">{model.name}</span>
                  <button
                    onClick={() => onRemoveModel(model.modelId)}
                    className="remove-model-btn"
                    aria-label={`Remove ${model.name} from comparison`}
                  >
                    ×
                  </button>
                </div>
              </th>
            ))}
          </tr>
        </thead>
        <tbody>
          {comparisonData.categories.map(category => (
            <React.Fragment key={category.name}>
              <tr className="category-row">
                <td
                  className="category-title"
                  colSpan={models.length + 1}
                  onClick={() => toggleSection(category.name)}
                >
                  <span className="category-title-text">{category.name}</span>
                  {expandedSections.has(category.name) ? (
                    <ChevronUp size={16} />
                  ) : (
                    <ChevronDown size={16} />
                  )}
                </td>
              </tr>
              {expandedSections.has(category.name) &&
                category.rows.map(row => (
                  <tr key={row.key} className="comparison-row">
                    <td className="row-label">{row.label}</td>
                    {row.values.map((value, index) => (
                      <td
                        key={index}
                        className={getCellClassName(row, index)}
                      >
                        {formatValue(value, row.type)}
                      </td>
                    ))}
                  </tr>
                ))}
            </React.Fragment>
          ))}
        </tbody>
      </table>
    </div>
  );
};

function buildComparisonData(models: ComparisonModel[]): ComparisonData {
  const categories = [
    {
      name: 'Basic Information',
      rows: [
        {
          label: 'Model Name',
          key: 'name',
          type: 'text' as const,
          values: models.map(m => m.name),
        },
        {
          label: 'Creator',
          key: 'creator',
          type: 'text' as const,
          values: models.map(m => m.creator),
        },
        {
          label: 'Category',
          key: 'category',
          type: 'text' as const,
          values: models.map(m => m.category),
        },
        {
          label: 'Framework',
          key: 'framework',
          type: 'text' as const,
          values: models.map(m => m.framework),
        },
      ],
    },
    {
      name: 'Pricing',
      rows: [
        {
          label: 'Base Price',
          key: 'basePrice',
          type: 'currency' as const,
          values: models.map(m => m.basePrice),
          bestIndex: findLowestIndex(models.map(m => Number(m.basePrice))),
          worstIndex: findHighestIndex(models.map(m => Number(m.basePrice))),
        },
        {
          label: 'Discount Price',
          key: 'discountPrice',
          type: 'currency' as const,
          values: models.map(m => m.discountPrice),
          bestIndex: findLowestIndex(models.map(m => Number(m.discountPrice))),
          worstIndex: findHighestIndex(models.map(m => Number(m.discountPrice))),
        },
      ],
    },
    {
      name: 'Quality Metrics',
      rows: [
        {
          label: 'Quality Score',
          key: 'qualityScore',
          type: 'number' as const,
          values: models.map(m => m.qualityScore),
          bestIndex: findHighestIndex(models.map(m => m.qualityScore)),
          worstIndex: findLowestIndex(models.map(m => m.qualityScore)),
        },
        {
          label: 'Average Rating',
          key: 'averageRating',
          type: 'number' as const,
          values: models.map(m => m.averageRating),
          bestIndex: findHighestIndex(models.map(m => m.averageRating)),
          worstIndex: findLowestIndex(models.map(m => m.averageRating)),
        },
        {
          label: 'Review Count',
          key: 'reviewCount',
          type: 'number' as const,
          values: models.map(m => m.reviewCount),
          bestIndex: findHighestIndex(models.map(m => m.reviewCount)),
        },
      ],
    },
    {
      name: 'Performance',
      rows: [
        {
          label: 'Latency (p50)',
          key: 'latencyP50',
          type: 'number' as const,
          values: models.map(m => m.latencyP50),
          bestIndex: findLowestIndex(models.map(m => m.latencyP50)),
          worstIndex: findHighestIndex(models.map(m => m.latencyP50)),
        },
        {
          label: 'Latency (p95)',
          key: 'latencyP95',
          type: 'number' as const,
          values: models.map(m => m.latencyP95),
          bestIndex: findLowestIndex(models.map(m => m.latencyP95)),
          worstIndex: findHighestIndex(models.map(m => m.latencyP95)),
        },
        {
          label: 'Success Rate',
          key: 'successRate',
          type: 'percentage' as const,
          values: models.map(m => m.successRate),
          bestIndex: findHighestIndex(models.map(m => m.successRate)),
          worstIndex: findLowestIndex(models.map(m => m.successRate)),
        },
      ],
    },
    {
      name: 'Usage Statistics',
      rows: [
        {
          label: 'Total Inferences',
          key: 'totalInferences',
          type: 'number' as const,
          values: models.map(m => m.totalInferences),
          bestIndex: findHighestIndex(models.map(m => m.totalInferences)),
        },
        {
          label: 'Total Sales',
          key: 'totalSales',
          type: 'number' as const,
          values: models.map(m => m.totalSales),
          bestIndex: findHighestIndex(models.map(m => m.totalSales)),
        },
      ],
    },
    {
      name: 'Technical Specifications',
      rows: [
        {
          label: 'Input Formats',
          key: 'inputFormats',
          type: 'array' as const,
          values: models.map(m => m.inputFormats),
        },
        {
          label: 'Output Formats',
          key: 'outputFormats',
          type: 'array' as const,
          values: models.map(m => m.outputFormats),
        },
        {
          label: 'Model Size',
          key: 'modelSize',
          type: 'text' as const,
          values: models.map(m => m.modelSize),
        },
        {
          label: 'Parameters',
          key: 'parameters',
          type: 'number' as const,
          values: models.map(m => m.parameters),
        },
      ],
    },
  ];

  return { models, categories };
}

function findLowestIndex(values: number[]): number {
  return values.indexOf(Math.min(...values));
}

function findHighestIndex(values: number[]): number {
  return values.indexOf(Math.max(...values));
}

function getCellClassName(row: ComparisonRow, index: number): string {
  const classes = ['comparison-cell'];
  if (row.bestIndex === index) classes.push('best-value');
  if (row.worstIndex === index) classes.push('worst-value');
  return classes.join(' ');
}

function formatValue(value: any, type: string): React.ReactNode {
  switch (type) {
    case 'currency':
      return `${Number(value) / 1e18} ETH`;
    case 'percentage':
      return `${value.toFixed(2)}%`;
    case 'array':
      return Array.isArray(value) ? value.join(', ') : value;
    case 'number':
      return typeof value === 'number' ? value.toLocaleString() : value;
    default:
      return value?.toString() || 'N/A';
  }
}
```

#### Acceptance Criteria
- [ ] Table renders correctly with 2-5 models
- [ ] Visual indicators show best/worst values
- [ ] Expand/collapse sections work
- [ ] Responsive layout (horizontal scroll on mobile)
- [ ] Remove model from comparison works

---

### Task 1.3: Create CompareButton and Comparison State Management
**Estimated Time:** 1.5 hours
**Assigned To:** Developer
**Priority:** P0

#### Implementation Steps
1. Create CompareButton component
2. Implement useComparison hook
3. Add session storage persistence
4. Create comparison context

#### Code Structure
```typescript
// gui/citrate-core/src/hooks/useComparison.ts
import { useState, useEffect, useCallback } from 'react';
import { ComparisonSelection } from '../types/comparison';

const STORAGE_KEY = 'citrate_comparison_selection';
const MAX_MODELS = 5;

export function useComparison() {
  const [selection, setSelection] = useState<ComparisonSelection>(() => {
    if (typeof window === 'undefined') return { modelIds: [], maxModels: MAX_MODELS, timestamp: Date.now() };

    const stored = sessionStorage.getItem(STORAGE_KEY);
    if (stored) {
      try {
        return JSON.parse(stored);
      } catch (e) {
        console.error('Failed to parse comparison selection:', e);
      }
    }
    return { modelIds: [], maxModels: MAX_MODELS, timestamp: Date.now() };
  });

  useEffect(() => {
    sessionStorage.setItem(STORAGE_KEY, JSON.stringify(selection));
  }, [selection]);

  const addModel = useCallback((modelId: string) => {
    setSelection(prev => {
      if (prev.modelIds.includes(modelId)) return prev;
      if (prev.modelIds.length >= MAX_MODELS) {
        console.warn(`Cannot add more than ${MAX_MODELS} models to comparison`);
        return prev;
      }
      return {
        ...prev,
        modelIds: [...prev.modelIds, modelId],
        timestamp: Date.now(),
      };
    });
  }, []);

  const removeModel = useCallback((modelId: string) => {
    setSelection(prev => ({
      ...prev,
      modelIds: prev.modelIds.filter(id => id !== modelId),
      timestamp: Date.now(),
    }));
  }, []);

  const clearComparison = useCallback(() => {
    setSelection({ modelIds: [], maxModels: MAX_MODELS, timestamp: Date.now() });
  }, []);

  const isSelected = useCallback((modelId: string) => {
    return selection.modelIds.includes(modelId);
  }, [selection.modelIds]);

  const canAddMore = selection.modelIds.length < MAX_MODELS;

  return {
    modelIds: selection.modelIds,
    count: selection.modelIds.length,
    maxModels: MAX_MODELS,
    canAddMore,
    addModel,
    removeModel,
    clearComparison,
    isSelected,
  };
}
```

```typescript
// gui/citrate-core/src/components/comparison/CompareButton.tsx
import React from 'react';
import { Scale, Check } from 'lucide-react';
import { useComparison } from '../../hooks/useComparison';

interface CompareButtonProps {
  modelId: string;
  variant?: 'icon' | 'button';
  className?: string;
}

export const CompareButton: React.FC<CompareButtonProps> = ({
  modelId,
  variant = 'button',
  className = '',
}) => {
  const { addModel, removeModel, isSelected, canAddMore } = useComparison();
  const selected = isSelected(modelId);

  const handleClick = () => {
    if (selected) {
      removeModel(modelId);
    } else {
      if (canAddMore) {
        addModel(modelId);
      } else {
        alert('Maximum 5 models can be compared at once');
      }
    }
  };

  if (variant === 'icon') {
    return (
      <button
        onClick={handleClick}
        className={`compare-icon-btn ${selected ? 'selected' : ''} ${className}`}
        aria-label={selected ? 'Remove from comparison' : 'Add to comparison'}
        disabled={!selected && !canAddMore}
      >
        {selected ? <Check size={18} /> : <Scale size={18} />}
      </button>
    );
  }

  return (
    <button
      onClick={handleClick}
      className={`compare-btn ${selected ? 'selected' : ''} ${className}`}
      disabled={!selected && !canAddMore}
    >
      {selected ? (
        <>
          <Check size={18} />
          <span>Added to Compare</span>
        </>
      ) : (
        <>
          <Scale size={18} />
          <span>Add to Compare</span>
        </>
      )}
    </button>
  );
};
```

#### Acceptance Criteria
- [ ] Add/remove models from comparison
- [ ] Selection persists across page navigation
- [ ] Maximum 5 models enforced
- [ ] Visual indication when model is selected
- [ ] Cannot add more than 5 models

---

### Task 1.4: Implement Side-by-Side Metrics Visualization
**Estimated Time:** 1.5 hours
**Assigned To:** Developer
**Priority:** P0

#### Implementation Steps
1. Create ComparisonPage component
2. Integrate ComparisonTable
3. Add filter options (show only differences)
4. Implement shareable link generation

#### Code Structure
```typescript
// gui/citrate-core/src/pages/ComparisonPage.tsx
import React, { useEffect, useState } from 'react';
import { useSearchParams } from 'react-router-dom';
import { ComparisonTable } from '../components/comparison/ComparisonTable';
import { useComparison } from '../hooks/useComparison';
import { ComparisonModel } from '../types/comparison';
import { fetchModelDetails } from '../services/api';

export const ComparisonPage: React.FC = () => {
  const [searchParams] = useSearchParams();
  const { modelIds, clearComparison, removeModel } = useComparison();
  const [models, setModels] = useState<ComparisonModel[]>([]);
  const [loading, setLoading] = useState(true);
  const [showOnlyDifferences, setShowOnlyDifferences] = useState(false);

  useEffect(() => {
    // Load from URL if present
    const compareParam = searchParams.get('compare');
    if (compareParam) {
      const urlModelIds = compareParam.split(',');
      // Load these models instead of from state
      loadModels(urlModelIds);
    } else {
      loadModels(modelIds);
    }
  }, [searchParams, modelIds]);

  const loadModels = async (ids: string[]) => {
    setLoading(true);
    try {
      const modelData = await Promise.all(
        ids.map(id => fetchModelDetails(id))
      );
      setModels(modelData);
    } catch (error) {
      console.error('Failed to load comparison models:', error);
    } finally {
      setLoading(false);
    }
  };

  const handleShareComparison = () => {
    const url = `${window.location.origin}/compare?compare=${modelIds.join(',')}`;
    navigator.clipboard.writeText(url);
    alert('Comparison link copied to clipboard!');
  };

  if (loading) {
    return <div className="loading-spinner">Loading comparison...</div>;
  }

  if (models.length === 0) {
    return (
      <div className="empty-comparison">
        <h2>No Models Selected</h2>
        <p>Add models to compare by clicking "Add to Compare" on model cards.</p>
      </div>
    );
  }

  return (
    <div className="comparison-page">
      <header className="comparison-header">
        <h1>Model Comparison</h1>
        <div className="comparison-actions">
          <label>
            <input
              type="checkbox"
              checked={showOnlyDifferences}
              onChange={e => setShowOnlyDifferences(e.target.checked)}
            />
            Show only differences
          </label>
          <button onClick={handleShareComparison} className="share-btn">
            Share Comparison
          </button>
          <button onClick={clearComparison} className="clear-btn">
            Clear All
          </button>
        </div>
      </header>

      <ComparisonTable
        models={models}
        onRemoveModel={removeModel}
        showOnlyDifferences={showOnlyDifferences}
      />
    </div>
  );
};
```

#### Acceptance Criteria
- [ ] Page loads comparison from state or URL
- [ ] Show only differences filter works
- [ ] Share link generation works
- [ ] Clear all comparison works

---

### Task 1.5: Add Export Comparison Report Feature
**Estimated Time:** 0.5 hours
**Assigned To:** Developer
**Priority:** P1

#### Implementation Steps
1. Implement CSV export
2. Implement PDF export
3. Add export buttons to UI

#### Code Structure
```typescript
// gui/citrate-core/src/utils/exportComparison.ts
import Papa from 'papaparse';
import jsPDF from 'jspdf';
import autoTable from 'jspdf-autotable';
import { ComparisonData } from '../types/comparison';

export function exportComparisonCSV(data: ComparisonData): void {
  const rows = [];

  // Header row
  rows.push(['Feature', ...data.models.map(m => m.name)]);

  // Data rows
  data.categories.forEach(category => {
    rows.push([category.name, ...Array(data.models.length).fill('')]);
    category.rows.forEach(row => {
      rows.push([row.label, ...row.values.map(v => formatValueForExport(v, row.type))]);
    });
  });

  const csv = Papa.unparse(rows);
  const blob = new Blob([csv], { type: 'text/csv;charset=utf-8;' });
  const link = document.createElement('a');
  link.href = URL.createObjectURL(blob);
  link.download = `model-comparison-${Date.now()}.csv`;
  link.click();
}

export function exportComparisonPDF(data: ComparisonData): void {
  const doc = new jsPDF('landscape');

  doc.setFontSize(18);
  doc.text('Model Comparison', 14, 20);

  let yPos = 30;

  data.categories.forEach(category => {
    if (yPos > 180) {
      doc.addPage();
      yPos = 20;
    }

    doc.setFontSize(14);
    doc.text(category.name, 14, yPos);
    yPos += 10;

    const tableData = category.rows.map(row => [
      row.label,
      ...row.values.map(v => formatValueForExport(v, row.type)),
    ]);

    autoTable(doc, {
      startY: yPos,
      head: [['Feature', ...data.models.map(m => m.name)]],
      body: tableData,
      theme: 'grid',
      styles: { fontSize: 10 },
      headStyles: { fillColor: [66, 139, 202] },
      didParseCell: (data) => {
        // Highlight best/worst values
        const row = category.rows[data.row.index];
        if (row && row.bestIndex === data.column.index - 1) {
          data.cell.styles.fillColor = [200, 255, 200];
        }
        if (row && row.worstIndex === data.column.index - 1) {
          data.cell.styles.fillColor = [255, 200, 200];
        }
      },
    });

    yPos = (doc as any).lastAutoTable.finalY + 10;
  });

  doc.save(`model-comparison-${Date.now()}.pdf`);
}

function formatValueForExport(value: any, type: string): string {
  if (type === 'currency') {
    return `${Number(value) / 1e18} ETH`;
  }
  if (type === 'percentage') {
    return `${value.toFixed(2)}%`;
  }
  if (type === 'array') {
    return Array.isArray(value) ? value.join(', ') : value;
  }
  return value?.toString() || 'N/A';
}
```

#### Acceptance Criteria
- [ ] CSV export works
- [ ] PDF export works with formatting
- [ ] Export buttons functional
- [ ] Filenames include timestamp

---

## Day 2: Creator Analytics Dashboard (7 hours)

### Task 2.1: Design Analytics Data Structure and API
**Estimated Time:** 1 hour
**Assigned To:** Developer
**Priority:** P0

#### Data Structure
```typescript
// gui/citrate-core/src/types/analytics.ts
export interface CreatorAnalytics {
  creatorAddress: string;
  timeWindow: TimeWindow;

  // Revenue metrics
  revenue: RevenueMetrics;

  // Usage metrics
  usage: UsageMetrics;

  // User demographics
  demographics: DemographicsMetrics;

  // Model performance
  modelPerformance: ModelPerformance[];
}

export interface RevenueMetrics {
  totalRevenue: bigint;
  revenueByModel: { modelId: string; revenue: bigint }[];
  revenueOverTime: { timestamp: number; revenue: bigint }[];
  revenueTrend: number; // Percentage change
  averageRevenuePerInference: bigint;
  projectedRevenue30Days: bigint;
}

export interface UsageMetrics {
  totalInferences: number;
  inferencesByModel: { modelId: string; inferences: number }[];
  inferencesOverTime: { timestamp: number; inferences: number }[];
  usageTrend: number; // Percentage change
  peakUsageTimes: HeatmapData;
  successRate: number;
  errorBreakdown: { errorType: string; count: number }[];
}

export interface DemographicsMetrics {
  totalUniqueUsers: number;
  newUsers: number;
  returningUsers: number;
  retentionRate: number;
  averageInferencesPerUser: number;
  userGrowthOverTime: { timestamp: number; newUsers: number; totalUsers: number }[];
  geographicDistribution?: { country: string; users: number }[];
  topUsers: { address: string; inferences: number }[];
}

export interface ModelPerformance {
  modelId: string;
  modelName: string;
  qualityScore: number;
  rating: number;
  reviewCount: number;
  totalInferences: number;
  totalRevenue: bigint;
  latencyP50: number;
  latencyP95: number;
  successRate: number;
  categoryRank: number;
}

export interface HeatmapData {
  data: number[][]; // [dayOfWeek][hourOfDay]
  maxValue: number;
}

export type TimeWindow = '7d' | '30d' | '90d' | 'all';
```

#### Acceptance Criteria
- [ ] Data structure comprehensive
- [ ] API endpoint designed
- [ ] Types documented
- [ ] Aggregation strategy planned

---

### Task 2.2: Build CreatorDashboard Component
**Estimated Time:** 2.5 hours
**Assigned To:** Developer
**Priority:** P0

#### Code Structure
```typescript
// gui/citrate-core/src/components/analytics/CreatorDashboard.tsx
import React, { useState } from 'react';
import { useCreatorAnalytics } from '../../hooks/useCreatorAnalytics';
import { TimeWindow } from '../../types/analytics';
import { RevenueSection } from './RevenueSection';
import { UsageSection } from './UsageSection';
import { DemographicsSection } from './DemographicsSection';
import { ModelPerformanceSection } from './ModelPerformanceSection';

export const CreatorDashboard: React.FC = () => {
  const [timeWindow, setTimeWindow] = useState<TimeWindow>('30d');
  const { data: analytics, loading, error, refetch } = useCreatorAnalytics(timeWindow);

  if (loading) return <div className="loading-spinner">Loading analytics...</div>;
  if (error) return <div className="error-message">Failed to load analytics: {error.message}</div>;
  if (!analytics) return null;

  return (
    <div className="creator-dashboard">
      <header className="dashboard-header">
        <h1>Creator Analytics</h1>
        <div className="dashboard-controls">
          <select
            value={timeWindow}
            onChange={e => setTimeWindow(e.target.value as TimeWindow)}
            className="time-window-selector"
          >
            <option value="7d">Last 7 Days</option>
            <option value="30d">Last 30 Days</option>
            <option value="90d">Last 90 Days</option>
            <option value="all">All Time</option>
          </select>
          <button onClick={refetch} className="refresh-btn">
            Refresh
          </button>
          <button onClick={() => exportAnalytics(analytics)} className="export-btn">
            Export Report
          </button>
        </div>
      </header>

      <div className="dashboard-content">
        <RevenueSection revenue={analytics.revenue} timeWindow={timeWindow} />
        <UsageSection usage={analytics.usage} timeWindow={timeWindow} />
        <DemographicsSection demographics={analytics.demographics} timeWindow={timeWindow} />
        <ModelPerformanceSection models={analytics.modelPerformance} />
      </div>
    </div>
  );
};
```

#### Acceptance Criteria
- [ ] Dashboard layout complete
- [ ] Time window selector works
- [ ] Refresh button works
- [ ] Export button functional
- [ ] Responsive design

---

### Task 2.3: Implement Revenue Charts and Graphs
**Estimated Time:** 1.5 hours
**Assigned To:** Developer
**Priority:** P0

#### Code Structure
```typescript
// gui/citrate-core/src/components/analytics/RevenueSection.tsx
import React from 'react';
import { LineChart, Line, BarChart, Bar, XAxis, YAxis, CartesianGrid, Tooltip, ResponsiveContainer } from 'recharts';
import { RevenueMetrics, TimeWindow } from '../../types/analytics';
import { formatEther, formatCurrency } from '../../utils/format';

interface RevenueSectionProps {
  revenue: RevenueMetrics;
  timeWindow: TimeWindow;
}

export const RevenueSection: React.FC<RevenueSectionProps> = ({ revenue, timeWindow }) => {
  const revenueOverTimeData = revenue.revenueOverTime.map(item => ({
    date: new Date(item.timestamp).toLocaleDateString(),
    revenue: Number(item.revenue) / 1e18,
  }));

  const revenueByModelData = revenue.revenueByModel
    .sort((a, b) => Number(b.revenue - a.revenue))
    .slice(0, 5)
    .map(item => ({
      modelId: item.modelId.substring(0, 8),
      revenue: Number(item.revenue) / 1e18,
    }));

  return (
    <section className="revenue-section analytics-section">
      <h2>Revenue Analytics</h2>

      <div className="metrics-grid">
        <div className="metric-card">
          <span className="metric-label">Total Revenue</span>
          <span className="metric-value">{formatEther(revenue.totalRevenue)} ETH</span>
          <span className={`metric-trend ${revenue.revenueTrend >= 0 ? 'positive' : 'negative'}`}>
            {revenue.revenueTrend >= 0 ? '▲' : '▼'} {Math.abs(revenue.revenueTrend).toFixed(1)}%
          </span>
        </div>

        <div className="metric-card">
          <span className="metric-label">Avg Revenue Per Inference</span>
          <span className="metric-value">{formatEther(revenue.averageRevenuePerInference)} ETH</span>
        </div>

        <div className="metric-card">
          <span className="metric-label">Projected Revenue (30d)</span>
          <span className="metric-value">{formatEther(revenue.projectedRevenue30Days)} ETH</span>
        </div>
      </div>

      <div className="charts-grid">
        <div className="chart-container">
          <h3>Revenue Over Time</h3>
          <ResponsiveContainer width="100%" height={300}>
            <LineChart data={revenueOverTimeData}>
              <CartesianGrid strokeDasharray="3 3" />
              <XAxis dataKey="date" />
              <YAxis />
              <Tooltip formatter={(value) => `${value} ETH`} />
              <Line type="monotone" dataKey="revenue" stroke="#8884d8" strokeWidth={2} />
            </LineChart>
          </ResponsiveContainer>
        </div>

        <div className="chart-container">
          <h3>Revenue by Model (Top 5)</h3>
          <ResponsiveContainer width="100%" height={300}>
            <BarChart data={revenueByModelData}>
              <CartesianGrid strokeDasharray="3 3" />
              <XAxis dataKey="modelId" />
              <YAxis />
              <Tooltip formatter={(value) => `${value} ETH`} />
              <Bar dataKey="revenue" fill="#82ca9d" />
            </BarChart>
          </ResponsiveContainer>
        </div>
      </div>
    </section>
  );
};
```

#### Acceptance Criteria
- [ ] Revenue metrics displayed
- [ ] Line chart shows revenue over time
- [ ] Bar chart shows revenue by model
- [ ] Trend indicators correct
- [ ] Responsive charts

---

### Task 2.4: Add Usage Pattern Analysis
**Estimated Time:** 1 hour
**Assigned To:** Developer
**Priority:** P0

#### Code Structure
```typescript
// gui/citrate-core/src/components/analytics/UsageSection.tsx
import React from 'react';
import { LineChart, Line, PieChart, Pie, Cell, Tooltip, ResponsiveContainer } from 'recharts';
import { UsageMetrics } from '../../types/analytics';
import { UsageHeatmap } from './UsageHeatmap';

interface UsageSectionProps {
  usage: UsageMetrics;
  timeWindow: TimeWindow;
}

export const UsageSection: React.FC<UsageSectionProps> = ({ usage, timeWindow }) => {
  const inferencesOverTimeData = usage.inferencesOverTime.map(item => ({
    date: new Date(item.timestamp).toLocaleDateString(),
    inferences: item.inferences,
  }));

  const errorBreakdownData = usage.errorBreakdown.map(item => ({
    name: item.errorType,
    value: item.count,
  }));

  const COLORS = ['#0088FE', '#00C49F', '#FFBB28', '#FF8042'];

  return (
    <section className="usage-section analytics-section">
      <h2>Usage Patterns</h2>

      <div className="metrics-grid">
        <div className="metric-card">
          <span className="metric-label">Total Inferences</span>
          <span className="metric-value">{usage.totalInferences.toLocaleString()}</span>
          <span className={`metric-trend ${usage.usageTrend >= 0 ? 'positive' : 'negative'}`}>
            {usage.usageTrend >= 0 ? '▲' : '▼'} {Math.abs(usage.usageTrend).toFixed(1)}%
          </span>
        </div>

        <div className="metric-card">
          <span className="metric-label">Success Rate</span>
          <span className="metric-value">{usage.successRate.toFixed(2)}%</span>
        </div>
      </div>

      <div className="charts-grid">
        <div className="chart-container">
          <h3>Inferences Over Time</h3>
          <ResponsiveContainer width="100%" height={300}>
            <LineChart data={inferencesOverTimeData}>
              <CartesianGrid strokeDasharray="3 3" />
              <XAxis dataKey="date" />
              <YAxis />
              <Tooltip />
              <Line type="monotone" dataKey="inferences" stroke="#8884d8" strokeWidth={2} />
            </LineChart>
          </ResponsiveContainer>
        </div>

        <div className="chart-container">
          <h3>Error Breakdown</h3>
          <ResponsiveContainer width="100%" height={300}>
            <PieChart>
              <Pie
                data={errorBreakdownData}
                cx="50%"
                cy="50%"
                labelLine={false}
                label={(entry) => entry.name}
                outerRadius={80}
                fill="#8884d8"
                dataKey="value"
              >
                {errorBreakdownData.map((entry, index) => (
                  <Cell key={`cell-${index}`} fill={COLORS[index % COLORS.length]} />
                ))}
              </Pie>
              <Tooltip />
            </PieChart>
          </ResponsiveContainer>
        </div>
      </div>

      <div className="heatmap-container">
        <h3>Peak Usage Times</h3>
        <UsageHeatmap data={usage.peakUsageTimes} />
      </div>
    </section>
  );
};
```

#### Acceptance Criteria
- [ ] Usage metrics displayed
- [ ] Inferences over time chart
- [ ] Error breakdown pie chart
- [ ] Peak usage heatmap
- [ ] Responsive design

---

### Task 2.5: Create User Demographics Visualization
**Estimated Time:** 1 hour
**Assigned To:** Developer
**Priority:** P1

#### Code Structure
```typescript
// gui/citrate-core/src/components/analytics/DemographicsSection.tsx
import React from 'react';
import { PieChart, Pie, Cell, Tooltip, ResponsiveContainer, LineChart, Line, XAxis, YAxis } from 'recharts';
import { DemographicsMetrics } from '../../types/analytics';

interface DemographicsSectionProps {
  demographics: DemographicsMetrics;
  timeWindow: TimeWindow;
}

export const DemographicsSection: React.FC<DemographicsSectionProps> = ({ demographics }) => {
  const userTypeData = [
    { name: 'New Users', value: demographics.newUsers },
    { name: 'Returning Users', value: demographics.returningUsers },
  ];

  const userGrowthData = demographics.userGrowthOverTime.map(item => ({
    date: new Date(item.timestamp).toLocaleDateString(),
    newUsers: item.newUsers,
    totalUsers: item.totalUsers,
  }));

  return (
    <section className="demographics-section analytics-section">
      <h2>User Demographics</h2>

      <div className="metrics-grid">
        <div className="metric-card">
          <span className="metric-label">Total Unique Users</span>
          <span className="metric-value">{demographics.totalUniqueUsers.toLocaleString()}</span>
        </div>

        <div className="metric-card">
          <span className="metric-label">Retention Rate</span>
          <span className="metric-value">{demographics.retentionRate.toFixed(2)}%</span>
        </div>

        <div className="metric-card">
          <span className="metric-label">Avg Inferences/User</span>
          <span className="metric-value">{demographics.averageInferencesPerUser.toFixed(1)}</span>
        </div>
      </div>

      <div className="charts-grid">
        <div className="chart-container">
          <h3>User Type Distribution</h3>
          <ResponsiveContainer width="100%" height={300}>
            <PieChart>
              <Pie
                data={userTypeData}
                cx="50%"
                cy="50%"
                labelLine={false}
                label={(entry) => `${entry.name}: ${entry.value}`}
                outerRadius={80}
                fill="#8884d8"
                dataKey="value"
              >
                <Cell fill="#0088FE" />
                <Cell fill="#00C49F" />
              </Pie>
              <Tooltip />
            </PieChart>
          </ResponsiveContainer>
        </div>

        <div className="chart-container">
          <h3>User Growth</h3>
          <ResponsiveContainer width="100%" height={300}>
            <LineChart data={userGrowthData}>
              <CartesianGrid strokeDasharray="3 3" />
              <XAxis dataKey="date" />
              <YAxis />
              <Tooltip />
              <Line type="monotone" dataKey="newUsers" stroke="#8884d8" name="New Users" />
              <Line type="monotone" dataKey="totalUsers" stroke="#82ca9d" name="Total Users" />
            </LineChart>
          </ResponsiveContainer>
        </div>
      </div>

      <div className="top-users-table">
        <h3>Top Users by Inferences</h3>
        <table>
          <thead>
            <tr>
              <th>Rank</th>
              <th>Address</th>
              <th>Inferences</th>
            </tr>
          </thead>
          <tbody>
            {demographics.topUsers.map((user, index) => (
              <tr key={user.address}>
                <td>{index + 1}</td>
                <td>{formatAddress(user.address)}</td>
                <td>{user.inferences.toLocaleString()}</td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>
    </section>
  );
};

function formatAddress(address: string): string {
  return `${address.substring(0, 6)}...${address.substring(address.length - 4)}`;
}
```

#### Acceptance Criteria
- [ ] Demographics metrics displayed
- [ ] User type pie chart
- [ ] User growth line chart
- [ ] Top users table
- [ ] Responsive design

---

## Day 3: Batch Operations (6 hours)

### Task 3.1: Design Batch Operation UI and State Management
**Estimated Time:** 0.5 hours
**Assigned To:** Developer
**Priority:** P0

#### Data Structure
```typescript
// gui/citrate-core/src/types/batch.ts
export interface BatchSelection {
  selectedIds: string[];
  allSelected: boolean;
}

export interface BatchOperation {
  type: 'updatePrice' | 'toggleStatus' | 'updateCategory' | 'updateTags' | 'delete';
  params: any;
}

export interface BatchOperationResult {
  success: number;
  failed: number;
  errors: { modelId: string; error: string }[];
}

export interface BatchUpdateParams {
  basePrice?: bigint;
  discountPrice?: bigint;
  active?: boolean;
  category?: ModelCategory;
  addTags?: string[];
  removeTags?: string[];
}
```

#### Acceptance Criteria
- [ ] Data structure defined
- [ ] State management approach planned
- [ ] UI mockup created

---

### Task 3.2: Build BatchActionsToolbar Component
**Estimated Time:** 1.5 hours
**Assigned To:** Developer
**Priority:** P0

#### Code Structure
```typescript
// gui/citrate-core/src/components/batch/BatchActionsToolbar.tsx
import React, { useState } from 'react';
import { BatchOperation, BatchSelection } from '../../types/batch';
import { BatchUpdateModal } from './BatchUpdateModal';

interface BatchActionsToolbarProps {
  selection: BatchSelection;
  onClearSelection: () => void;
  onExecuteBatch: (operation: BatchOperation) => Promise<void>;
}

export const BatchActionsToolbar: React.FC<BatchActionsToolbarProps> = ({
  selection,
  onClearSelection,
  onExecuteBatch,
}) => {
  const [showUpdateModal, setShowUpdateModal] = useState(false);
  const [operationType, setOperationType] = useState<string | null>(null);

  const handleBatchAction = (type: string) => {
    setOperationType(type);
    setShowUpdateModal(true);
  };

  return (
    <div className="batch-actions-toolbar">
      <div className="selection-info">
        <span className="selection-count">
          {selection.selectedIds.length} model(s) selected
        </span>
        <button onClick={onClearSelection} className="clear-selection-btn">
          Clear Selection
        </button>
      </div>

      <div className="batch-actions">
        <button
          onClick={() => handleBatchAction('updatePrice')}
          className="batch-action-btn"
          disabled={selection.selectedIds.length === 0}
        >
          Update Price
        </button>

        <button
          onClick={() => handleBatchAction('toggleStatus')}
          className="batch-action-btn"
          disabled={selection.selectedIds.length === 0}
        >
          Toggle Active/Inactive
        </button>

        <button
          onClick={() => handleBatchAction('updateCategory')}
          className="batch-action-btn"
          disabled={selection.selectedIds.length === 0}
        >
          Change Category
        </button>

        <button
          onClick={() => handleBatchAction('updateTags')}
          className="batch-action-btn"
          disabled={selection.selectedIds.length === 0}
        >
          Update Tags
        </button>

        <button
          onClick={() => handleBatchAction('delete')}
          className="batch-action-btn danger"
          disabled={selection.selectedIds.length === 0}
        >
          Delete Models
        </button>
      </div>

      {showUpdateModal && operationType && (
        <BatchUpdateModal
          operationType={operationType}
          selection={selection}
          onClose={() => setShowUpdateModal(false)}
          onExecute={onExecuteBatch}
        />
      )}
    </div>
  );
};
```

#### Acceptance Criteria
- [ ] Toolbar appears when models selected
- [ ] All action buttons functional
- [ ] Selection count accurate
- [ ] Clear selection works
- [ ] Modal opens on action click

---

### Task 3.3: Implement Multi-Select Functionality
**Estimated Time:** 1 hour
**Assigned To:** Developer
**Priority:** P0

#### Code Structure
```typescript
// gui/citrate-core/src/hooks/useBatchSelection.ts
import { useState, useCallback } from 'react';
import { BatchSelection } from '../types/batch';

export function useBatchSelection() {
  const [selection, setSelection] = useState<BatchSelection>({
    selectedIds: [],
    allSelected: false,
  });

  const toggleModel = useCallback((modelId: string) => {
    setSelection(prev => {
      const isSelected = prev.selectedIds.includes(modelId);
      if (isSelected) {
        return {
          ...prev,
          selectedIds: prev.selectedIds.filter(id => id !== modelId),
          allSelected: false,
        };
      } else {
        return {
          ...prev,
          selectedIds: [...prev.selectedIds, modelId],
        };
      }
    });
  }, []);

  const selectAll = useCallback((allIds: string[]) => {
    setSelection({
      selectedIds: allIds,
      allSelected: true,
    });
  }, []);

  const deselectAll = useCallback(() => {
    setSelection({
      selectedIds: [],
      allSelected: false,
    });
  }, []);

  const isSelected = useCallback((modelId: string) => {
    return selection.selectedIds.includes(modelId);
  }, [selection.selectedIds]);

  return {
    selection,
    toggleModel,
    selectAll,
    deselectAll,
    isSelected,
  };
}
```

```typescript
// Add checkbox to ModelCard component
<div className="model-card-checkbox">
  <input
    type="checkbox"
    checked={isSelected(model.modelId)}
    onChange={() => toggleModel(model.modelId)}
    aria-label={`Select ${model.name}`}
  />
</div>
```

#### Acceptance Criteria
- [ ] Checkbox on each model card
- [ ] Select all checkbox in header
- [ ] Selection state managed correctly
- [ ] Keyboard shortcuts (Shift+click) work

---

### Task 3.4: Create Batch Update Forms and Dialogs
**Estimated Time:** 1.5 hours
**Assigned To:** Developer
**Priority:** P0

#### Code Structure
```typescript
// gui/citrate-core/src/components/batch/BatchUpdateModal.tsx
import React, { useState } from 'react';
import { BatchOperation, BatchSelection, BatchUpdateParams } from '../../types/batch';

interface BatchUpdateModalProps {
  operationType: string;
  selection: BatchSelection;
  onClose: () => void;
  onExecute: (operation: BatchOperation) => Promise<void>;
}

export const BatchUpdateModal: React.FC<BatchUpdateModalProps> = ({
  operationType,
  selection,
  onClose,
  onExecute,
}) => {
  const [params, setParams] = useState<BatchUpdateParams>({});
  const [loading, setLoading] = useState(false);

  const handleSubmit = async () => {
    setLoading(true);
    try {
      await onExecute({
        type: operationType as any,
        params,
      });
      onClose();
    } catch (error) {
      console.error('Batch operation failed:', error);
    } finally {
      setLoading(false);
    }
  };

  const renderFormFields = () => {
    switch (operationType) {
      case 'updatePrice':
        return (
          <>
            <div className="form-field">
              <label>Base Price (ETH)</label>
              <input
                type="number"
                step="0.0001"
                placeholder="0.01"
                onChange={e => setParams({ ...params, basePrice: BigInt(Math.floor(parseFloat(e.target.value) * 1e18)) })}
              />
            </div>
            <div className="form-field">
              <label>Discount Price (ETH, optional)</label>
              <input
                type="number"
                step="0.0001"
                placeholder="0.008"
                onChange={e => setParams({ ...params, discountPrice: BigInt(Math.floor(parseFloat(e.target.value) * 1e18)) })}
              />
            </div>
          </>
        );

      case 'toggleStatus':
        return (
          <div className="form-field">
            <label>New Status</label>
            <select onChange={e => setParams({ ...params, active: e.target.value === 'active' })}>
              <option value="active">Active</option>
              <option value="inactive">Inactive</option>
            </select>
          </div>
        );

      case 'updateCategory':
        return (
          <div className="form-field">
            <label>New Category</label>
            <select onChange={e => setParams({ ...params, category: e.target.value as any })}>
              <option value="LLM">LLM</option>
              <option value="Vision">Vision</option>
              <option value="Audio">Audio</option>
              <option value="Multimodal">Multimodal</option>
            </select>
          </div>
        );

      case 'updateTags':
        return (
          <>
            <div className="form-field">
              <label>Add Tags (comma-separated)</label>
              <input
                type="text"
                placeholder="tag1, tag2, tag3"
                onChange={e => setParams({ ...params, addTags: e.target.value.split(',').map(t => t.trim()) })}
              />
            </div>
            <div className="form-field">
              <label>Remove Tags (comma-separated)</label>
              <input
                type="text"
                placeholder="tag1, tag2"
                onChange={e => setParams({ ...params, removeTags: e.target.value.split(',').map(t => t.trim()) })}
              />
            </div>
          </>
        );

      case 'delete':
        return (
          <div className="warning-message">
            <strong>Warning:</strong> This will permanently delete {selection.selectedIds.length} model(s). This action cannot be undone.
          </div>
        );

      default:
        return null;
    }
  };

  return (
    <div className="modal-overlay" onClick={onClose}>
      <div className="modal-content" onClick={e => e.stopPropagation()}>
        <header className="modal-header">
          <h2>Batch {operationType}</h2>
          <button onClick={onClose} className="close-btn">×</button>
        </header>

        <div className="modal-body">
          <p>Applying to {selection.selectedIds.length} model(s)</p>
          {renderFormFields()}
        </div>

        <footer className="modal-footer">
          <button onClick={onClose} className="cancel-btn">
            Cancel
          </button>
          <button
            onClick={handleSubmit}
            className="submit-btn"
            disabled={loading}
          >
            {loading ? 'Processing...' : 'Apply Changes'}
          </button>
        </footer>
      </div>
    </div>
  );
};
```

#### Acceptance Criteria
- [ ] Modal renders for each operation type
- [ ] Form fields appropriate for operation
- [ ] Validation works
- [ ] Submit triggers batch operation
- [ ] Loading state shown

---

### Task 3.5: Add Confirmation Dialogs and Undo Support
**Estimated Time:** 1 hour
**Assigned To:** Developer
**Priority:** P1

#### Code Structure
```typescript
// gui/citrate-core/src/hooks/useBatchUndo.ts
import { useState, useCallback, useEffect } from 'react';
import { BatchOperation } from '../types/batch';

const UNDO_TIMEOUT = 60000; // 60 seconds

interface UndoableOperation {
  operation: BatchOperation;
  undoData: any;
  timestamp: number;
}

export function useBatchUndo() {
  const [undoStack, setUndoStack] = useState<UndoableOperation[]>([]);
  const [timeRemaining, setTimeRemaining] = useState<number | null>(null);

  useEffect(() => {
    if (undoStack.length === 0) {
      setTimeRemaining(null);
      return;
    }

    const latest = undoStack[undoStack.length - 1];
    const elapsed = Date.now() - latest.timestamp;
    const remaining = Math.max(0, UNDO_TIMEOUT - elapsed);

    if (remaining === 0) {
      setUndoStack(prev => prev.slice(0, -1));
      return;
    }

    setTimeRemaining(remaining);

    const interval = setInterval(() => {
      const elapsed = Date.now() - latest.timestamp;
      const remaining = Math.max(0, UNDO_TIMEOUT - elapsed);

      if (remaining === 0) {
        setUndoStack(prev => prev.slice(0, -1));
        setTimeRemaining(null);
      } else {
        setTimeRemaining(remaining);
      }
    }, 1000);

    return () => clearInterval(interval);
  }, [undoStack]);

  const recordOperation = useCallback((operation: BatchOperation, undoData: any) => {
    setUndoStack(prev => [...prev, {
      operation,
      undoData,
      timestamp: Date.now(),
    }]);
  }, []);

  const undo = useCallback(async () => {
    if (undoStack.length === 0) return;

    const latest = undoStack[undoStack.length - 1];
    // Execute undo logic here
    await executeUndo(latest.undoData);

    setUndoStack(prev => prev.slice(0, -1));
  }, [undoStack]);

  const canUndo = undoStack.length > 0 && timeRemaining !== null && timeRemaining > 0;

  return {
    recordOperation,
    undo,
    canUndo,
    timeRemaining: timeRemaining ? Math.ceil(timeRemaining / 1000) : null,
  };
}

async function executeUndo(undoData: any): Promise<void> {
  // Implement undo logic
  console.log('Undoing operation:', undoData);
}
```

```typescript
// gui/citrate-core/src/components/batch/UndoToast.tsx
import React from 'react';

interface UndoToastProps {
  timeRemaining: number;
  onUndo: () => void;
  onDismiss: () => void;
}

export const UndoToast: React.FC<UndoToastProps> = ({
  timeRemaining,
  onUndo,
  onDismiss,
}) => {
  return (
    <div className="undo-toast">
      <span className="undo-message">
        Batch operation completed
      </span>
      <button onClick={onUndo} className="undo-btn">
        Undo ({timeRemaining}s)
      </button>
      <button onClick={onDismiss} className="dismiss-btn">
        ×
      </button>
    </div>
  );
};
```

#### Acceptance Criteria
- [ ] Confirmation dialog before destructive operations
- [ ] Undo button appears after operation
- [ ] Countdown timer shows remaining time
- [ ] Undo restores previous state
- [ ] Undo expires after 60 seconds

---

### Task 3.6: Test Batch Operations with Various Scenarios
**Estimated Time:** 0.5 hours
**Assigned To:** Developer
**Priority:** P0

#### Test Scenarios
1. Update price for 10 models
2. Toggle status for 5 models
3. Update category for 20 models
4. Partial failures (some succeed, some fail)
5. Undo operation
6. Large batch (100+ models)

#### Acceptance Criteria
- [ ] All scenarios tested
- [ ] Partial failures handled gracefully
- [ ] Progress indicators accurate
- [ ] Error messages helpful

---

## Day 4: Mobile Optimization (7 hours)

### Task 4.1: Audit Mobile Experience and Identify Pain Points
**Estimated Time:** 1 hour
**Assigned To:** Developer
**Priority:** P0

#### Audit Checklist
- [ ] Run Lighthouse audit on mobile
- [ ] Test on actual devices (iPhone, Android)
- [ ] Identify slow-loading pages
- [ ] Check touch target sizes
- [ ] Test navigation patterns
- [ ] Check image optimization
- [ ] Test form usability

#### Acceptance Criteria
- [ ] Audit report completed
- [ ] Pain points documented
- [ ] Optimization priorities identified

---

### Task 4.2: Optimize Responsive Breakpoints and Grid Layouts
**Estimated Time:** 1.5 hours
**Assigned To:** Developer
**Priority:** P0

#### Implementation
```css
/* gui/citrate-core/src/styles/responsive.css */

/* Mobile-first approach */
.model-grid {
  display: grid;
  grid-template-columns: 1fr; /* 1 column on mobile */
  gap: 1rem;
  padding: 1rem;
}

/* Tablet */
@media (min-width: 640px) {
  .model-grid {
    grid-template-columns: repeat(2, 1fr); /* 2 columns */
    gap: 1.5rem;
  }
}

/* Desktop */
@media (min-width: 1024px) {
  .model-grid {
    grid-template-columns: repeat(3, 1fr); /* 3 columns */
    gap: 2rem;
  }
}

/* Large desktop */
@media (min-width: 1536px) {
  .model-grid {
    grid-template-columns: repeat(4, 1fr); /* 4 columns */
  }
}

/* Model cards */
.model-card {
  width: 100%;
  min-height: 300px;
  display: flex;
  flex-direction: column;
}

.model-card-image {
  width: 100%;
  aspect-ratio: 16 / 9;
  object-fit: cover;
}

/* Touch-friendly buttons */
@media (max-width: 640px) {
  button, .btn {
    min-height: 44px; /* Apple guideline */
    min-width: 44px;
    padding: 0.75rem 1.5rem;
    font-size: 1rem;
  }

  input, select, textarea {
    min-height: 44px;
    font-size: 16px; /* Prevents zoom on iOS */
    padding: 0.75rem;
  }
}
```

#### Acceptance Criteria
- [ ] Responsive grid layouts implemented
- [ ] Breakpoints optimized
- [ ] Cards resize appropriately
- [ ] No horizontal scrolling
- [ ] Touch targets ≥ 44px

---

### Task 4.3: Implement Mobile-First Navigation Patterns
**Estimated Time:** 1.5 hours
**Assigned To:** Developer
**Priority:** P0

#### Code Structure
```typescript
// gui/citrate-core/src/components/mobile/MobileNavigation.tsx
import React, { useState } from 'react';
import { Home, Search, User, ShoppingCart, Menu } from 'lucide-react';
import { Link, useLocation } from 'react-router-dom';

export const MobileNavigation: React.FC = () => {
  const location = useLocation();
  const [showMenu, setShowMenu] = useState(false);

  const navItems = [
    { path: '/', icon: Home, label: 'Home' },
    { path: '/search', icon: Search, label: 'Search' },
    { path: '/cart', icon: ShoppingCart, label: 'Cart' },
    { path: '/profile', icon: User, label: 'Profile' },
  ];

  return (
    <>
      {/* Bottom Navigation Bar (Mobile Only) */}
      <nav className="mobile-bottom-nav">
        {navItems.map(item => (
          <Link
            key={item.path}
            to={item.path}
            className={`nav-item ${location.pathname === item.path ? 'active' : ''}`}
          >
            <item.icon size={24} />
            <span className="nav-label">{item.label}</span>
          </Link>
        ))}
        <button onClick={() => setShowMenu(true)} className="nav-item">
          <Menu size={24} />
          <span className="nav-label">Menu</span>
        </button>
      </nav>

      {/* Hamburger Menu */}
      {showMenu && (
        <div className="mobile-menu-overlay" onClick={() => setShowMenu(false)}>
          <div className="mobile-menu" onClick={e => e.stopPropagation()}>
            <header className="mobile-menu-header">
              <h2>Menu</h2>
              <button onClick={() => setShowMenu(false)} className="close-btn">
                ×
              </button>
            </header>
            <nav className="mobile-menu-nav">
              <Link to="/my-models" onClick={() => setShowMenu(false)}>
                My Models
              </Link>
              <Link to="/analytics" onClick={() => setShowMenu(false)}>
                Analytics
              </Link>
              <Link to="/settings" onClick={() => setShowMenu(false)}>
                Settings
              </Link>
              <Link to="/help" onClick={() => setShowMenu(false)}>
                Help
              </Link>
            </nav>
          </div>
        </div>
      )}
    </>
  );
};
```

```css
/* Mobile bottom navigation */
.mobile-bottom-nav {
  display: none;
  position: fixed;
  bottom: 0;
  left: 0;
  right: 0;
  background: white;
  border-top: 1px solid #e5e7eb;
  padding: 0.5rem;
  z-index: 1000;
  flex-direction: row;
  justify-content: space-around;
}

@media (max-width: 640px) {
  .mobile-bottom-nav {
    display: flex;
  }

  /* Hide desktop navigation */
  .desktop-nav {
    display: none;
  }
}

.nav-item {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 0.25rem;
  padding: 0.5rem;
  color: #6b7280;
  text-decoration: none;
  border: none;
  background: none;
  cursor: pointer;
}

.nav-item.active {
  color: #3b82f6;
}

.nav-label {
  font-size: 0.75rem;
}
```

#### Acceptance Criteria
- [ ] Bottom navigation on mobile
- [ ] Hamburger menu for secondary nav
- [ ] Active state indication
- [ ] Smooth transitions
- [ ] Accessible (keyboard, screen reader)

---

### Task 4.4: Add Touch Gestures (Swipe, Pinch-to-Zoom)
**Estimated Time:** 1 hour
**Assigned To:** Developer
**Priority:** P1

#### Code Structure
```typescript
// gui/citrate-core/src/hooks/useSwipeGesture.ts
import { useGesture } from '@use-gesture/react';
import { useSpring, animated } from '@react-spring/web';

export function useSwipeGesture(onSwipeLeft?: () => void, onSwipeRight?: () => void) {
  const [{ x }, api] = useSpring(() => ({ x: 0 }));

  const bind = useGesture({
    onDrag: ({ movement: [mx], direction: [xDir], cancel, active }) => {
      // Swipe threshold
      if (Math.abs(mx) > 100 && !active) {
        if (xDir < 0 && onSwipeLeft) {
          onSwipeLeft();
        } else if (xDir > 0 && onSwipeRight) {
          onSwipeRight();
        }
        cancel();
      }

      api.start({ x: active ? mx : 0, immediate: active });
    },
  });

  return { bind, style: { x } };
}
```

```typescript
// Example usage in carousel
import { useSwipeGesture } from '../../hooks/useSwipeGesture';

export const ModelCarousel: React.FC<ModelCarouselProps> = ({ models }) => {
  const [currentIndex, setCurrentIndex] = useState(0);

  const { bind, style } = useSwipeGesture(
    () => setCurrentIndex(i => Math.min(i + 1, models.length - 1)), // Swipe left
    () => setCurrentIndex(i => Math.max(i - 1, 0)) // Swipe right
  );

  return (
    <animated.div {...bind()} style={style} className="carousel">
      {/* Carousel content */}
    </animated.div>
  );
};
```

#### Acceptance Criteria
- [ ] Swipe gestures work on carousels
- [ ] Pull-to-refresh on lists
- [ ] Swipe to dismiss modals
- [ ] Smooth animations
- [ ] No conflicts with scroll

---

### Task 4.5: Optimize Images, Fonts, and Implement Lazy Loading
**Estimated Time:** 1 hour
**Assigned To:** Developer
**Priority:** P0

#### Implementation
```typescript
// gui/citrate-core/src/components/OptimizedImage.tsx
import React, { useState } from 'react';
import { useInView } from 'react-intersection-observer';

interface OptimizedImageProps {
  src: string;
  alt: string;
  width?: number;
  height?: number;
  className?: string;
}

export const OptimizedImage: React.FC<OptimizedImageProps> = ({
  src,
  alt,
  width,
  height,
  className,
}) => {
  const [loaded, setLoaded] = useState(false);
  const { ref, inView } = useInView({
    triggerOnce: true,
    rootMargin: '200px', // Load 200px before entering viewport
  });

  // Generate WebP and fallback URLs
  const webpSrc = src.replace(/\.(jpg|jpeg|png)$/, '.webp');

  // Generate responsive srcset
  const srcset = `
    ${src} 1x,
    ${src.replace(/\.(jpg|jpeg|png)$/, '@2x.$1')} 2x
  `.trim();

  return (
    <div
      ref={ref}
      className={`optimized-image-container ${className || ''} ${loaded ? 'loaded' : 'loading'}`}
      style={{ aspectRatio: width && height ? `${width}/${height}` : undefined }}
    >
      {inView && (
        <picture>
          <source type="image/webp" srcSet={webpSrc} />
          <img
            src={src}
            srcSet={srcset}
            alt={alt}
            width={width}
            height={height}
            loading="lazy"
            onLoad={() => setLoaded(true)}
          />
        </picture>
      )}
      {!loaded && <div className="image-skeleton" />}
    </div>
  );
};
```

```css
/* Skeleton loader */
.image-skeleton {
  width: 100%;
  height: 100%;
  background: linear-gradient(90deg, #f0f0f0 25%, #e0e0e0 50%, #f0f0f0 75%);
  background-size: 200% 100%;
  animation: shimmer 1.5s infinite;
}

@keyframes shimmer {
  0% {
    background-position: 200% 0;
  }
  100% {
    background-position: -200% 0;
  }
}
```

#### Font Optimization
```html
<!-- gui/citrate-core/index.html -->
<link rel="preconnect" href="https://fonts.googleapis.com">
<link rel="preconnect" href="https://fonts.gstatic.com" crossorigin>
<link href="https://fonts.googleapis.com/css2?family=Inter:wght@400;500;600;700&display=swap" rel="stylesheet">

<!-- Preload critical fonts -->
<link rel="preload" href="/fonts/inter-var.woff2" as="font" type="font/woff2" crossorigin>
```

#### Acceptance Criteria
- [ ] Images lazy load
- [ ] WebP format with fallback
- [ ] Responsive srcset
- [ ] Skeleton loaders
- [ ] Fonts optimized
- [ ] Preload critical assets

---

### Task 4.6: Mobile Performance Testing and Optimization
**Estimated Time:** 1 hour
**Assigned To:** Developer
**Priority:** P0

#### Testing Checklist
- [ ] Run Lighthouse in mobile mode
- [ ] Test on real devices (iPhone, Android)
- [ ] Test on slow 3G network
- [ ] Monitor bundle size
- [ ] Check memory usage
- [ ] Profile performance

#### Performance Targets
- Lighthouse Performance: > 90
- First Contentful Paint: < 2s
- Time to Interactive: < 3.5s
- Bundle size: < 200KB gzipped

#### Acceptance Criteria
- [ ] All performance targets met
- [ ] No performance regressions
- [ ] Tested on multiple devices
- [ ] Network throttling tested

---

## Day 5: Performance & Polish (6.5 hours)

### Task 5.1: Implement Virtualized Lists for Large Catalogs
**Estimated Time:** 1.5 hours
**Assigned To:** Developer
**Priority:** P0

#### Code Structure
```typescript
// gui/citrate-core/src/components/VirtualizedModelList.tsx
import React from 'react';
import { FixedSizeGrid } from 'react-window';
import AutoSizer from 'react-virtualized-auto-sizer';
import { ModelCard } from './ModelCard';

interface VirtualizedModelListProps {
  models: Model[];
  columnCount?: number;
}

export const VirtualizedModelList: React.FC<VirtualizedModelListProps> = ({
  models,
  columnCount = 3,
}) => {
  const rowCount = Math.ceil(models.length / columnCount);
  const columnWidth = 350; // Width of each card
  const rowHeight = 400; // Height of each card

  const Cell = ({ columnIndex, rowIndex, style }: any) => {
    const index = rowIndex * columnCount + columnIndex;
    if (index >= models.length) return null;

    const model = models[index];
    return (
      <div style={style}>
        <ModelCard model={model} />
      </div>
    );
  };

  return (
    <AutoSizer>
      {({ height, width }) => {
        // Recalculate column count based on available width
        const actualColumnCount = Math.max(1, Math.floor(width / columnWidth));
        const actualRowCount = Math.ceil(models.length / actualColumnCount);

        return (
          <FixedSizeGrid
            columnCount={actualColumnCount}
            columnWidth={columnWidth}
            height={height}
            rowCount={actualRowCount}
            rowHeight={rowHeight}
            width={width}
            overscanRowCount={2} // Render 2 extra rows for smooth scrolling
          >
            {Cell}
          </FixedSizeGrid>
        );
      }}
    </AutoSizer>
  );
};
```

#### Acceptance Criteria
- [ ] Virtual scrolling works smoothly
- [ ] Handles 1000+ models
- [ ] 60 FPS scrolling
- [ ] Memory usage < 100MB
- [ ] Responsive column count

---

### Task 5.2: Optimize React Re-renders with Memoization
**Estimated Time:** 1 hour
**Assigned To:** Developer
**Priority:** P0

#### Implementation
```typescript
// Memoize expensive components
export const ModelCard = React.memo<ModelCardProps>(
  ({ model, onCompare, isSelected }) => {
    // Component implementation
  },
  (prevProps, nextProps) => {
    // Custom comparison
    return (
      prevProps.model.modelId === nextProps.model.modelId &&
      prevProps.isSelected === nextProps.isSelected
    );
  }
);

// Use useMemo for expensive calculations
const sortedModels = useMemo(() => {
  return models.sort((a, b) => b.qualityScore - a.qualityScore);
}, [models]);

// Use useCallback for event handlers
const handleModelClick = useCallback((modelId: string) => {
  navigate(`/models/${modelId}`);
}, [navigate]);
```

#### Acceptance Criteria
- [ ] Components properly memoized
- [ ] Expensive calculations memoized
- [ ] Event handlers use useCallback
- [ ] No unnecessary re-renders
- [ ] React DevTools Profiler confirms optimization

---

### Task 5.3: Add Service Worker for Asset Caching
**Estimated Time:** 1 hour
**Assigned To:** Developer
**Priority:** P1

#### Implementation
```typescript
// gui/citrate-core/src/serviceWorker.ts
import { precacheAndRoute } from 'workbox-precaching';
import { registerRoute } from 'workbox-routing';
import { StaleWhileRevalidate, CacheFirst, NetworkFirst } from 'workbox-strategies';
import { CacheableResponsePlugin } from 'workbox-cacheable-response';
import { ExpirationPlugin } from 'workbox-expiration';

// Precache build assets
precacheAndRoute(self.__WB_MANIFEST);

// Cache API responses
registerRoute(
  ({ url }) => url.pathname.startsWith('/api/'),
  new NetworkFirst({
    cacheName: 'api-cache',
    plugins: [
      new CacheableResponsePlugin({
        statuses: [0, 200],
      }),
      new ExpirationPlugin({
        maxEntries: 50,
        maxAgeSeconds: 5 * 60, // 5 minutes
      }),
    ],
  })
);

// Cache images
registerRoute(
  ({ request }) => request.destination === 'image',
  new CacheFirst({
    cacheName: 'images-cache',
    plugins: [
      new CacheableResponsePlugin({
        statuses: [0, 200],
      }),
      new ExpirationPlugin({
        maxEntries: 100,
        maxAgeSeconds: 30 * 24 * 60 * 60, // 30 days
      }),
    ],
  })
);

// Cache static assets (JS, CSS, fonts)
registerRoute(
  ({ request }) =>
    request.destination === 'script' ||
    request.destination === 'style' ||
    request.destination === 'font',
  new StaleWhileRevalidate({
    cacheName: 'static-assets',
  })
);
```

#### Acceptance Criteria
- [ ] Service worker registered
- [ ] Assets cached appropriately
- [ ] Cache versioning works
- [ ] Offline fallback functional
- [ ] Cache invalidation on updates

---

### Task 5.4: Database Query Optimization and Indexing
**Estimated Time:** 1 hour
**Assigned To:** Developer
**Priority:** P0

#### Optimization Areas
1. Add database indexes
2. Optimize query patterns
3. Implement query result caching
4. Use pagination efficiently

#### Example Optimizations
```rust
// Add indexes to frequently queried fields
// In RocksDB, use proper key prefixes for efficient range queries

// Before: Scanning all models
let all_models = dag_store.get_all_models(); // O(n)

// After: Using indexed queries
let indexed_models = dag_store.get_models_by_category(category); // O(log n)

// Pagination optimization
pub fn get_models_paginated(
    &self,
    page: usize,
    page_size: usize,
    filters: ModelFilters,
) -> Result<Vec<Model>> {
    let start_key = format!("model:{}", page * page_size);
    let iter = self.db.iterator(IteratorMode::From(start_key.as_bytes(), Direction::Forward));

    // Apply filters and take only page_size items
    let models: Vec<Model> = iter
        .take(page_size)
        .filter_map(|(key, value)| {
            let model: Model = bincode::deserialize(&value).ok()?;
            if apply_filters(&model, &filters) {
                Some(model)
            } else {
                None
            }
        })
        .collect();

    Ok(models)
}
```

#### Acceptance Criteria
- [ ] Indexes added to critical fields
- [ ] Query performance improved
- [ ] Caching implemented
- [ ] Pagination optimized

---

### Task 5.5: Load Testing and Performance Benchmarking
**Estimated Time:** 1 hour
**Assigned To:** Developer
**Priority:** P0

#### Load Testing Script
```javascript
// load-test.js (using k6)
import http from 'k6/http';
import { check, sleep } from 'k6';

export const options = {
  stages: [
    { duration: '1m', target: 10 }, // Ramp up to 10 users
    { duration: '3m', target: 50 }, // Ramp up to 50 users
    { duration: '1m', target: 100 }, // Spike to 100 users
    { duration: '3m', target: 50 }, // Scale back down
    { duration: '1m', target: 0 }, // Ramp down
  ],
  thresholds: {
    http_req_duration: ['p(95)<2000'], // 95% of requests under 2s
    http_req_failed: ['rate<0.01'], // Less than 1% failure rate
  },
};

export default function () {
  // Test search endpoint
  const searchRes = http.get('http://localhost:8545/api/search?q=gpt');
  check(searchRes, {
    'search status is 200': (r) => r.status === 200,
    'search response time < 500ms': (r) => r.timings.duration < 500,
  });

  sleep(1);

  // Test model details
  const modelRes = http.get('http://localhost:8545/api/models/0x123...');
  check(modelRes, {
    'model status is 200': (r) => r.status === 200,
    'model response time < 1s': (r) => r.timings.duration < 1000,
  });

  sleep(1);
}
```

#### Acceptance Criteria
- [ ] Load testing completed
- [ ] Performance bottlenecks identified
- [ ] Optimization applied
- [ ] Benchmarks documented

---

### Task 5.6: Bug Fixes, Polish, and Documentation
**Estimated Time:** 1 hour
**Assigned To:** Developer
**Priority:** P0

#### Polish Checklist
- [ ] Fix any visual inconsistencies
- [ ] Add loading skeletons
- [ ] Improve error messages
- [ ] Add helpful tooltips
- [ ] Polish animations
- [ ] Accessibility improvements
- [ ] Update documentation

#### Acceptance Criteria
- [ ] All known bugs fixed
- [ ] UI polished
- [ ] Documentation updated
- [ ] README updated with new features

---

## Technical Dependencies

### NPM Packages
```bash
npm install react-window              # Virtual scrolling
npm install react-virtualized-auto-sizer  # Auto-sizing for virtualized lists
npm install @use-gesture/react        # Touch gestures
npm install @react-spring/web         # Animations
npm install jspdf jspdf-autotable     # PDF export
npm install papaparse                 # CSV export
npm install workbox-webpack-plugin    # Service worker
npm install react-intersection-observer # Lazy loading
```

### Development Dependencies
```bash
npm install --save-dev @types/papaparse
npm install --save-dev lighthouse-ci
npm install --save-dev k6  # Load testing
```

---

## Performance Targets

| Operation | Target | Measurement |
|-----------|--------|-------------|
| Model comparison (5 models) | < 1s | Load and render comparison table |
| Analytics dashboard load | < 2s | Initial render with charts |
| Batch operation (10 models) | < 3s | Complete update cycle |
| Virtual scroll (1000 models) | 60 FPS | Smooth scrolling performance |
| Mobile Lighthouse score | > 90 | Performance score |
| First Contentful Paint | < 2s | On 3G network |
| Bundle size (gzipped) | < 200KB | Initial JS bundle |
| Memory usage (large catalog) | < 100MB | With 1000+ models |

---

## Code Quality Checklist

- [ ] All TypeScript types defined
- [ ] Error handling on all async operations
- [ ] Loading states for all data fetching
- [ ] Input validation on all forms
- [ ] ARIA labels on interactive elements
- [ ] Dark mode styles applied
- [ ] Mobile responsive (tested on real devices)
- [ ] Unit tests for utilities (>85% coverage)
- [ ] Integration tests for flows
- [ ] Performance tests with benchmarks
- [ ] No console warnings/errors
- [ ] ESLint passes
- [ ] Build succeeds
- [ ] Lighthouse CI passes

---

**Document Version:** 1.0
**Last Updated:** February 25, 2026
**Status:** ✅ Ready for Implementation
