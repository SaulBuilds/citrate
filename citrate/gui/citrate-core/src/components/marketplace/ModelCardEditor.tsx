/**
 * ModelCardEditor Component
 *
 * Comprehensive form for model owners to edit metadata.
 * Features:
 * - Rich metadata editing (description, tags, category, etc.)
 * - Example prompts/inputs management
 * - Performance benchmarks
 * - Real-time preview
 * - Validation with inline errors
 * - IPFS upload functionality
 * - Owner-only access
 */

import React, { useState } from 'react';
import {
  EnrichedModelMetadata,
  ModelCategory,
  ModelSize,
  ModelExample,
  PerformanceBenchmark,
  CATEGORY_LABELS,
  LICENSE_OPTIONS,
} from '../../utils/types/reviews';
import { validateModelMetadata, hasFieldError, getValidationMessage } from '../../utils/metadata/validation';
import { validateAndUpload, ProgressCallback, UploadProgress } from '../../utils/metadata/ipfsUploader';
import ModelCardPreview from './ModelCardPreview';

export interface ModelCardEditorProps {
  modelId: string;
  currentMetadata?: EnrichedModelMetadata;
  ownerAddress: string;
  onSave?: (metadata: EnrichedModelMetadata, cid: string) => void;
  className?: string;
}

export const ModelCardEditor: React.FC<ModelCardEditorProps> = ({
  modelId,
  currentMetadata,
  ownerAddress,
  onSave,
  className = '',
}) => {
  const [metadata, setMetadata] = useState<EnrichedModelMetadata>(
    currentMetadata || {
      name: '',
      description: '',
      category: ModelCategory.OTHER,
      tags: [],
    }
  );

  const [showPreview, setShowPreview] = useState(false);
  const [errors, setErrors] = useState<{ field: string; message: string }[]>([]);
  const [warnings, setWarnings] = useState<{ field: string; message: string }[]>([]);
  const [isSaving, setIsSaving] = useState(false);
  const [uploadProgress, setUploadProgress] = useState<UploadProgress | null>(null);
  const [saveError, setSaveError] = useState<string | null>(null);
  const [saveSuccess, setSaveSuccess] = useState(false);

  // Tag input
  const [tagInput, setTagInput] = useState('');

  // Example management
  const [examples, setExamples] = useState<ModelExample[]>(metadata.examples || []);
  const [currentExample, setCurrentExample] = useState<Partial<ModelExample>>({});

  // Benchmark management
  const [benchmarks, setBenchmarks] = useState<PerformanceBenchmark[]>(metadata.benchmarks || []);
  const [currentBenchmark, setCurrentBenchmark] = useState<Partial<PerformanceBenchmark>>({});

  const handleValidate = () => {
    const result = validateModelMetadata(metadata);
    setErrors(result.errors);
    setWarnings(result.warnings);
    return result.valid;
  };

  const handleSave = async () => {
    if (!handleValidate()) {
      setSaveError('Please fix validation errors before saving');
      return;
    }

    setIsSaving(true);
    setSaveError(null);
    setSaveSuccess(false);

    const progressCallback: ProgressCallback = (progress) => {
      setUploadProgress(progress);
    };

    try {
      const result = await validateAndUpload(metadata, progressCallback);

      if (!result.success) {
        throw new Error(result.error || 'Upload failed');
      }

      setSaveSuccess(true);
      if (onSave && result.cid) {
        onSave(metadata, result.cid);
      }

      setTimeout(() => setSaveSuccess(false), 5000);
    } catch (error) {
      setSaveError(error instanceof Error ? error.message : 'Failed to save metadata');
    } finally {
      setIsSaving(false);
      setUploadProgress(null);
    }
  };

  const addTag = () => {
    if (tagInput.trim() && !metadata.tags.includes(tagInput.trim())) {
      setMetadata({ ...metadata, tags: [...metadata.tags, tagInput.trim()] });
      setTagInput('');
    }
  };

  const removeTag = (tag: string) => {
    setMetadata({ ...metadata, tags: metadata.tags.filter((t) => t !== tag) });
  };

  const addExample = () => {
    if (currentExample.title && currentExample.input && currentExample.output) {
      setExamples([...examples, currentExample as ModelExample]);
      setMetadata({ ...metadata, examples: [...examples, currentExample as ModelExample] });
      setCurrentExample({});
    }
  };

  const removeExample = (index: number) => {
    const updated = examples.filter((_, i) => i !== index);
    setExamples(updated);
    setMetadata({ ...metadata, examples: updated });
  };

  const addBenchmark = () => {
    if (currentBenchmark.metric && currentBenchmark.value !== undefined && currentBenchmark.unit) {
      setBenchmarks([...benchmarks, currentBenchmark as PerformanceBenchmark]);
      setMetadata({ ...metadata, benchmarks: [...benchmarks, currentBenchmark as PerformanceBenchmark] });
      setCurrentBenchmark({});
    }
  };

  const removeBenchmark = (index: number) => {
    const updated = benchmarks.filter((_, i) => i !== index);
    setBenchmarks(updated);
    setMetadata({ ...metadata, benchmarks: updated });
  };

  return (
    <div className={`model-card-editor ${className}`}>
      <div className="editor-header">
        <h2>Edit Model Metadata</h2>
        <div className="header-actions">
          <button
            className="preview-toggle"
            onClick={() => setShowPreview(!showPreview)}
          >
            {showPreview ? 'Hide Preview' : 'Show Preview'}
          </button>
          <button
            className="validate-btn"
            onClick={handleValidate}
          >
            Validate
          </button>
          <button
            className="save-btn"
            onClick={handleSave}
            disabled={isSaving}
          >
            {isSaving ? 'Saving...' : 'Save to IPFS'}
          </button>
        </div>
      </div>

      {uploadProgress && (
        <div className="upload-progress">
          <div className="progress-bar">
            <div className="progress-fill" style={{ width: `${uploadProgress.progress}%` }} />
          </div>
          <span className="progress-message">{uploadProgress.message}</span>
        </div>
      )}

      {saveSuccess && (
        <div className="success-message">
          Metadata saved successfully to IPFS!
        </div>
      )}

      {saveError && (
        <div className="error-message">{saveError}</div>
      )}

      <div className="editor-layout">
        <div className="editor-form">
          {/* Basic Information */}
          <section className="form-section">
            <h3>Basic Information</h3>

            <div className="form-group">
              <label htmlFor="name">Model Name *</label>
              <input
                id="name"
                type="text"
                value={metadata.name}
                onChange={(e) => setMetadata({ ...metadata, name: e.target.value })}
                className={hasFieldError('name', errors) ? 'error' : ''}
              />
              {getValidationMessage('name', errors) && (
                <span className="field-error">{getValidationMessage('name', errors)}</span>
              )}
            </div>

            <div className="form-group">
              <label htmlFor="description">Description *</label>
              <textarea
                id="description"
                value={metadata.description}
                onChange={(e) => setMetadata({ ...metadata, description: e.target.value })}
                rows={6}
                className={hasFieldError('description', errors) ? 'error' : ''}
              />
              {getValidationMessage('description', errors) && (
                <span className="field-error">{getValidationMessage('description', errors)}</span>
              )}
            </div>

            <div className="form-row">
              <div className="form-group">
                <label htmlFor="category">Category *</label>
                <select
                  id="category"
                  value={metadata.category}
                  onChange={(e) => setMetadata({ ...metadata, category: parseInt(e.target.value) as ModelCategory })}
                >
                  {Object.entries(CATEGORY_LABELS).map(([value, label]) => (
                    <option key={value} value={value}>{label}</option>
                  ))}
                </select>
              </div>

              <div className="form-group">
                <label htmlFor="version">Version</label>
                <input
                  id="version"
                  type="text"
                  value={metadata.version || ''}
                  onChange={(e) => setMetadata({ ...metadata, version: e.target.value })}
                  placeholder="e.g., 1.0.0"
                />
              </div>
            </div>

            <div className="form-group">
              <label htmlFor="tags">Tags</label>
              <div className="tag-input">
                <input
                  id="tags"
                  type="text"
                  value={tagInput}
                  onChange={(e) => setTagInput(e.target.value)}
                  onKeyPress={(e) => e.key === 'Enter' && (e.preventDefault(), addTag())}
                  placeholder="Add a tag and press Enter"
                />
                <button type="button" onClick={addTag}>Add</button>
              </div>
              <div className="tags-list">
                {metadata.tags.map((tag, idx) => (
                  <span key={idx} className="tag">
                    {tag}
                    <button onClick={() => removeTag(tag)}>×</button>
                  </span>
                ))}
              </div>
            </div>
          </section>

          {/* Technical Details */}
          <section className="form-section">
            <h3>Technical Details</h3>

            <div className="form-row">
              <div className="form-group">
                <label htmlFor="framework">Framework</label>
                <input
                  id="framework"
                  type="text"
                  value={metadata.framework || ''}
                  onChange={(e) => setMetadata({ ...metadata, framework: e.target.value })}
                  placeholder="e.g., PyTorch, TensorFlow"
                />
              </div>

              <div className="form-group">
                <label htmlFor="architecture">Architecture</label>
                <input
                  id="architecture"
                  type="text"
                  value={metadata.architecture || ''}
                  onChange={(e) => setMetadata({ ...metadata, architecture: e.target.value })}
                  placeholder="e.g., Transformer, CNN"
                />
              </div>
            </div>

            <div className="form-group">
              <label htmlFor="license">License</label>
              <select
                id="license"
                value={metadata.license || ''}
                onChange={(e) => setMetadata({ ...metadata, license: e.target.value })}
              >
                <option value="">Select a license</option>
                {LICENSE_OPTIONS.map((license) => (
                  <option key={license.value} value={license.value}>{license.label}</option>
                ))}
              </select>
            </div>

            <div className="form-group">
              <label htmlFor="trainingDataset">Training Dataset</label>
              <input
                id="trainingDataset"
                type="text"
                value={metadata.trainingDataset || ''}
                onChange={(e) => setMetadata({ ...metadata, trainingDataset: e.target.value })}
              />
            </div>
          </section>

          {/* Examples */}
          <section className="form-section">
            <h3>Examples</h3>
            <div className="examples-list">
              {examples.map((example, idx) => (
                <div key={idx} className="example-item">
                  <strong>{example.title}</strong>
                  <button onClick={() => removeExample(idx)}>Remove</button>
                </div>
              ))}
            </div>

            <div className="example-form">
              <input
                type="text"
                placeholder="Example title"
                value={currentExample.title || ''}
                onChange={(e) => setCurrentExample({ ...currentExample, title: e.target.value })}
              />
              <textarea
                placeholder="Input"
                value={currentExample.input || ''}
                onChange={(e) => setCurrentExample({ ...currentExample, input: e.target.value })}
                rows={2}
              />
              <textarea
                placeholder="Output"
                value={currentExample.output || ''}
                onChange={(e) => setCurrentExample({ ...currentExample, output: e.target.value })}
                rows={2}
              />
              <button type="button" onClick={addExample}>Add Example</button>
            </div>
          </section>

          {/* Benchmarks */}
          <section className="form-section">
            <h3>Performance Benchmarks</h3>
            <div className="benchmarks-list">
              {benchmarks.map((benchmark, idx) => (
                <div key={idx} className="benchmark-item">
                  <strong>{benchmark.metric}:</strong> {benchmark.value} {benchmark.unit}
                  <button onClick={() => removeBenchmark(idx)}>Remove</button>
                </div>
              ))}
            </div>

            <div className="benchmark-form">
              <input
                type="text"
                placeholder="Metric name"
                value={currentBenchmark.metric || ''}
                onChange={(e) => setCurrentBenchmark({ ...currentBenchmark, metric: e.target.value })}
              />
              <input
                type="number"
                placeholder="Value"
                value={currentBenchmark.value || ''}
                onChange={(e) => setCurrentBenchmark({ ...currentBenchmark, value: parseFloat(e.target.value) })}
              />
              <input
                type="text"
                placeholder="Unit"
                value={currentBenchmark.unit || ''}
                onChange={(e) => setCurrentBenchmark({ ...currentBenchmark, unit: e.target.value })}
              />
              <button type="button" onClick={addBenchmark}>Add Benchmark</button>
            </div>
          </section>

          {warnings.length > 0 && (
            <div className="warnings">
              <h4>Warnings:</h4>
              <ul>
                {warnings.map((warning, idx) => (
                  <li key={idx}>{warning.field}: {warning.message}</li>
                ))}
              </ul>
            </div>
          )}
        </div>

        {showPreview && (
          <div className="preview-panel">
            <ModelCardPreview metadata={metadata} />
          </div>
        )}
      </div>

      <style jsx>{`
        .model-card-editor {
          background: white;
          border: 1px solid #e5e7eb;
          border-radius: 12px;
          padding: 24px;
        }

        .editor-header {
          display: flex;
          justify-content: space-between;
          align-items: center;
          margin-bottom: 24px;
          padding-bottom: 16px;
          border-bottom: 2px solid #f3f4f6;
        }

        .editor-header h2 {
          margin: 0;
          font-size: 24px;
          font-weight: 700;
          color: #111827;
        }

        .header-actions {
          display: flex;
          gap: 12px;
        }

        .header-actions button {
          padding: 8px 16px;
          border: none;
          border-radius: 8px;
          font-weight: 600;
          cursor: pointer;
          transition: all 0.2s ease;
        }

        .preview-toggle {
          background: #f3f4f6;
          color: #374151;
        }

        .preview-toggle:hover {
          background: #e5e7eb;
        }

        .validate-btn {
          background: #eff6ff;
          color: #1e40af;
        }

        .validate-btn:hover {
          background: #dbeafe;
        }

        .save-btn {
          background: #3b82f6;
          color: white;
        }

        .save-btn:hover:not(:disabled) {
          background: #2563eb;
        }

        .save-btn:disabled {
          background: #d1d5db;
          cursor: not-allowed;
        }

        .upload-progress {
          margin-bottom: 20px;
        }

        .progress-bar {
          height: 8px;
          background: #e5e7eb;
          border-radius: 4px;
          overflow: hidden;
          margin-bottom: 8px;
        }

        .progress-fill {
          height: 100%;
          background: #3b82f6;
          transition: width 0.3s ease;
        }

        .progress-message {
          font-size: 14px;
          color: #6b7280;
        }

        .success-message,
        .error-message {
          padding: 12px 16px;
          border-radius: 8px;
          margin-bottom: 20px;
          font-size: 14px;
          font-weight: 500;
        }

        .success-message {
          background: #d1fae5;
          color: #065f46;
        }

        .error-message {
          background: #fee2e2;
          color: #991b1b;
        }

        .editor-layout {
          display: grid;
          grid-template-columns: 1fr;
          gap: 24px;
        }

        .editor-layout.preview-open {
          grid-template-columns: 1fr 1fr;
        }

        .form-section {
          margin-bottom: 32px;
        }

        .form-section h3 {
          margin: 0 0 16px 0;
          font-size: 18px;
          font-weight: 700;
          color: #111827;
        }

        .form-group {
          margin-bottom: 20px;
        }

        .form-group label {
          display: block;
          margin-bottom: 6px;
          font-weight: 600;
          color: #374151;
          font-size: 14px;
        }

        .form-group input,
        .form-group textarea,
        .form-group select {
          width: 100%;
          padding: 10px 12px;
          border: 1px solid #e5e7eb;
          border-radius: 8px;
          font-family: inherit;
          font-size: 14px;
        }

        .form-group input.error,
        .form-group textarea.error {
          border-color: #ef4444;
        }

        .field-error {
          display: block;
          margin-top: 6px;
          color: #ef4444;
          font-size: 13px;
        }

        .form-row {
          display: grid;
          grid-template-columns: 1fr 1fr;
          gap: 16px;
        }

        .tag-input {
          display: flex;
          gap: 8px;
        }

        .tag-input button {
          padding: 10px 16px;
          background: #3b82f6;
          color: white;
          border: none;
          border-radius: 8px;
          cursor: pointer;
          font-weight: 600;
        }

        .tags-list {
          display: flex;
          flex-wrap: wrap;
          gap: 8px;
          margin-top: 12px;
        }

        .tag {
          background: #f3f4f6;
          color: #374151;
          padding: 6px 12px;
          border-radius: 6px;
          display: flex;
          align-items: center;
          gap: 8px;
        }

        .tag button {
          background: none;
          border: none;
          color: #6b7280;
          cursor: pointer;
          font-size: 18px;
          line-height: 1;
        }

        .examples-list,
        .benchmarks-list {
          margin-bottom: 16px;
        }

        .example-item,
        .benchmark-item {
          display: flex;
          justify-content: space-between;
          align-items: center;
          padding: 12px;
          background: #f9fafb;
          border-radius: 8px;
          margin-bottom: 8px;
        }

        .example-item button,
        .benchmark-item button {
          background: #fee2e2;
          color: #991b1b;
          border: none;
          padding: 4px 12px;
          border-radius: 6px;
          cursor: pointer;
          font-size: 13px;
        }

        .example-form,
        .benchmark-form {
          display: flex;
          flex-direction: column;
          gap: 8px;
        }

        .example-form button,
        .benchmark-form button {
          background: #3b82f6;
          color: white;
          border: none;
          padding: 10px;
          border-radius: 8px;
          cursor: pointer;
          font-weight: 600;
        }

        .warnings {
          background: #fffbeb;
          border: 1px solid #fde68a;
          border-radius: 8px;
          padding: 16px;
          margin-top: 20px;
        }

        .warnings h4 {
          margin: 0 0 12px 0;
          color: #92400e;
        }

        .warnings ul {
          margin: 0;
          padding-left: 20px;
          color: #78350f;
        }

        @media (max-width: 768px) {
          .editor-header {
            flex-direction: column;
            align-items: flex-start;
            gap: 12px;
          }

          .form-row {
            grid-template-columns: 1fr;
          }

          .editor-layout.preview-open {
            grid-template-columns: 1fr;
          }
        }
      `}</style>
    </div>
  );
};

export default ModelCardEditor;
