/**
 * ModelCardPreview Component
 *
 * Visual preview of enhanced model card with all enriched metadata.
 * Displays formatted, read-only view of model information.
 */

import React from 'react';
import {
  EnrichedModelMetadata,
  CATEGORY_LABELS,
  MODEL_SIZE_INFO,
  LICENSE_OPTIONS,
} from '../../utils/types/reviews';

export interface ModelCardPreviewProps {
  metadata: EnrichedModelMetadata;
  className?: string;
}

export const ModelCardPreview: React.FC<ModelCardPreviewProps> = ({
  metadata,
  className = '',
}) => {
  const categoryLabel = CATEGORY_LABELS[metadata.category] || 'Other';
  const sizeInfo = metadata.modelSize ? MODEL_SIZE_INFO[metadata.modelSize] : null;
  const license = LICENSE_OPTIONS.find((l) => l.value === metadata.license);

  const formatBytes = (bytes?: number) => {
    if (!bytes) return 'N/A';
    const GB = 1024 * 1024 * 1024;
    const MB = 1024 * 1024;
    if (bytes >= GB) return `${(bytes / GB).toFixed(2)} GB`;
    if (bytes >= MB) return `${(bytes / MB).toFixed(2)} MB`;
    return `${(bytes / 1024).toFixed(2)} KB`;
  };

  return (
    <div className={`model-card-preview ${className}`}>
      <div className="preview-header">
        <h2>{metadata.name}</h2>
        {metadata.version && <span className="version">v{metadata.version}</span>}
      </div>

      <div className="metadata-section">
        <h3>Overview</h3>
        <p className="description">{metadata.description}</p>
      </div>

      <div className="metadata-grid">
        <div className="metadata-item">
          <label>Category</label>
          <span className="badge category-badge">{categoryLabel}</span>
        </div>

        {metadata.framework && (
          <div className="metadata-item">
            <label>Framework</label>
            <span className="value">{metadata.framework}</span>
          </div>
        )}

        {sizeInfo && (
          <div className="metadata-item">
            <label>Model Size</label>
            <span className="badge size-badge">{sizeInfo.label} ({sizeInfo.range})</span>
          </div>
        )}

        {metadata.sizeBytes && (
          <div className="metadata-item">
            <label>File Size</label>
            <span className="value">{formatBytes(metadata.sizeBytes)}</span>
          </div>
        )}

        {license && (
          <div className="metadata-item">
            <label>License</label>
            <span className="value">{license.label}</span>
          </div>
        )}

        {metadata.architecture && (
          <div className="metadata-item">
            <label>Architecture</label>
            <span className="value">{metadata.architecture}</span>
          </div>
        )}
      </div>

      {metadata.tags && metadata.tags.length > 0 && (
        <div className="metadata-section">
          <h3>Tags</h3>
          <div className="tags">
            {metadata.tags.map((tag, idx) => (
              <span key={idx} className="tag">{tag}</span>
            ))}
          </div>
        </div>
      )}

      {metadata.examples && metadata.examples.length > 0 && (
        <div className="metadata-section">
          <h3>Examples</h3>
          {metadata.examples.map((example, idx) => (
            <div key={idx} className="example">
              <h4>{example.title}</h4>
              {example.description && <p className="example-desc">{example.description}</p>}
              <div className="io-block">
                <strong>Input:</strong>
                <pre>{example.input}</pre>
              </div>
              <div className="io-block">
                <strong>Output:</strong>
                <pre>{example.output}</pre>
              </div>
            </div>
          ))}
        </div>
      )}

      {metadata.supportedFormats && (
        <div className="metadata-section">
          <h3>Supported Formats</h3>
          <div className="formats">
            <div>
              <strong>Input:</strong> {metadata.supportedFormats.input.join(', ')}
            </div>
            <div>
              <strong>Output:</strong> {metadata.supportedFormats.output.join(', ')}
            </div>
          </div>
        </div>
      )}

      {metadata.benchmarks && metadata.benchmarks.length > 0 && (
        <div className="metadata-section">
          <h3>Performance Benchmarks</h3>
          <table className="benchmarks-table">
            <thead>
              <tr>
                <th>Metric</th>
                <th>Value</th>
                <th>Dataset</th>
              </tr>
            </thead>
            <tbody>
              {metadata.benchmarks.map((benchmark, idx) => (
                <tr key={idx}>
                  <td>{benchmark.metric}</td>
                  <td>{benchmark.value} {benchmark.unit}</td>
                  <td>{benchmark.dataset || 'N/A'}</td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      )}

      {(metadata.trainingDataset || metadata.pretrainedOn || metadata.finetuneMethod) && (
        <div className="metadata-section">
          <h3>Training Information</h3>
          {metadata.trainingDataset && (
            <div className="info-item">
              <strong>Training Dataset:</strong> {metadata.trainingDataset}
            </div>
          )}
          {metadata.pretrainedOn && (
            <div className="info-item">
              <strong>Pretrained On:</strong> {metadata.pretrainedOn}
            </div>
          )}
          {metadata.finetuneMethod && (
            <div className="info-item">
              <strong>Finetune Method:</strong> {metadata.finetuneMethod}
            </div>
          )}
        </div>
      )}

      <style jsx>{`
        .model-card-preview {
          background: white;
          border: 1px solid #e5e7eb;
          border-radius: 12px;
          padding: 24px;
          max-width: 800px;
        }

        .preview-header {
          display: flex;
          align-items: center;
          gap: 12px;
          margin-bottom: 24px;
          padding-bottom: 16px;
          border-bottom: 2px solid #f3f4f6;
        }

        .preview-header h2 {
          margin: 0;
          font-size: 28px;
          font-weight: 700;
          color: #111827;
        }

        .version {
          background: #eff6ff;
          color: #1e40af;
          padding: 4px 10px;
          border-radius: 6px;
          font-size: 14px;
          font-weight: 600;
        }

        .metadata-section {
          margin-bottom: 24px;
        }

        .metadata-section h3 {
          margin: 0 0 12px 0;
          font-size: 18px;
          font-weight: 700;
          color: #111827;
        }

        .description {
          color: #374151;
          line-height: 1.6;
          margin: 0;
        }

        .metadata-grid {
          display: grid;
          grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
          gap: 16px;
          margin-bottom: 24px;
        }

        .metadata-item {
          display: flex;
          flex-direction: column;
          gap: 6px;
        }

        .metadata-item label {
          font-size: 12px;
          font-weight: 600;
          color: #6b7280;
          text-transform: uppercase;
          letter-spacing: 0.5px;
        }

        .value {
          color: #111827;
          font-weight: 500;
        }

        .badge {
          display: inline-block;
          padding: 6px 12px;
          border-radius: 6px;
          font-size: 14px;
          font-weight: 600;
        }

        .category-badge {
          background: #eff6ff;
          color: #1e40af;
        }

        .size-badge {
          background: #f0fdf4;
          color: #166534;
        }

        .tags {
          display: flex;
          flex-wrap: wrap;
          gap: 8px;
        }

        .tag {
          background: #f3f4f6;
          color: #374151;
          padding: 6px 12px;
          border-radius: 6px;
          font-size: 13px;
          font-weight: 500;
        }

        .example {
          background: #f9fafb;
          border: 1px solid #e5e7eb;
          border-radius: 8px;
          padding: 16px;
          margin-bottom: 16px;
        }

        .example h4 {
          margin: 0 0 8px 0;
          font-size: 16px;
          font-weight: 600;
          color: #111827;
        }

        .example-desc {
          color: #6b7280;
          font-size: 14px;
          margin: 0 0 12px 0;
        }

        .io-block {
          margin-bottom: 12px;
        }

        .io-block:last-child {
          margin-bottom: 0;
        }

        .io-block strong {
          display: block;
          margin-bottom: 6px;
          color: #374151;
          font-size: 13px;
        }

        .io-block pre {
          background: white;
          border: 1px solid #e5e7eb;
          border-radius: 6px;
          padding: 12px;
          margin: 0;
          color: #111827;
          font-size: 13px;
          overflow-x: auto;
          white-space: pre-wrap;
          word-wrap: break-word;
        }

        .formats {
          display: flex;
          flex-direction: column;
          gap: 8px;
          color: #374151;
          font-size: 14px;
        }

        .benchmarks-table {
          width: 100%;
          border-collapse: collapse;
        }

        .benchmarks-table th,
        .benchmarks-table td {
          padding: 12px;
          text-align: left;
          border-bottom: 1px solid #e5e7eb;
        }

        .benchmarks-table th {
          background: #f9fafb;
          font-weight: 600;
          color: #374151;
          font-size: 13px;
          text-transform: uppercase;
        }

        .benchmarks-table td {
          color: #111827;
          font-size: 14px;
        }

        .info-item {
          margin-bottom: 8px;
          color: #374151;
          font-size: 14px;
          line-height: 1.6;
        }

        .info-item strong {
          color: #111827;
        }

        @media (max-width: 640px) {
          .preview-header {
            flex-direction: column;
            align-items: flex-start;
          }

          .metadata-grid {
            grid-template-columns: 1fr;
          }
        }
      `}</style>
    </div>
  );
};

export default ModelCardPreview;
