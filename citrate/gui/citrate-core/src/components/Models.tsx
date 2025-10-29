import React, { useState, useEffect } from 'react';
import { modelService } from '../services/tauri';
import { ModelInfo, ModelDeployment, InferenceRequest } from '../types';
import {
  Brain,
  Upload,
  Play,
  Download,
  Activity,
  Clock,
  Zap
} from 'lucide-react';
import { SkeletonCard } from './Skeleton';

export const Models: React.FC = () => {
  const [models, setModels] = useState<ModelInfo[]>([]);
  const [selectedModel, setSelectedModel] = useState<ModelInfo | null>(null);
  const [showDeployModal, setShowDeployModal] = useState(false);
  const [showInferenceModal, setShowInferenceModal] = useState(false);
  const [loading, setLoading] = useState(false);

  useEffect(() => {
    loadModels();
  }, []);

  const loadModels = async () => {
    try {
      setLoading(true);
      const modelList = await modelService.list();
      setModels(modelList);
    } catch (err) {
      console.error('Failed to load models:', err);
    } finally {
      setLoading(false);
    }
  };

  const getStatusColor = (status: string) => {
    switch (status) {
      case 'Active': return 'text-green';
      case 'Training': return 'text-blue';
      case 'Updating': return 'text-yellow';
      case 'Deprecated': return 'text-gray';
      default: return 'text-gray';
    }
  };

  const formatTimestamp = (timestamp: number) => {
    return new Date(timestamp * 1000).toLocaleDateString();
  };

  return (
    <div className="models">
      <div className="models-header">
        <h2>AI Model Management</h2>
        <button 
          className="btn btn-primary"
          onClick={() => setShowDeployModal(true)}
        >
          <Upload size={16} />
          Deploy Model
        </button>
      </div>

      <div className="models-grid">
        {loading ? (
          <>
            <SkeletonCard height="240px" />
            <SkeletonCard height="240px" />
            <SkeletonCard height="240px" />
            <SkeletonCard height="240px" />
          </>
        ) : (
          <>
            {models.map(model => (
              <div
                key={model.id}
                className="model-card"
                onClick={() => setSelectedModel(model)}
              >
                <div className="model-header">
                  <Brain size={24} className="text-purple" />
                  <div className="model-status">
                    <span className={`status-badge ${getStatusColor(model.status)}`}>
                      {model.status}
                    </span>
                  </div>
                </div>

                <h3>{model.name}</h3>
                <p className="model-architecture">{model.architecture}</p>

                <div className="model-stats">
                  <div className="stat">
                    <Activity size={14} />
                    <span>{model.totalInferences} inferences</span>
                  </div>
                  <div className="stat">
                    <Clock size={14} />
                    <span>v{model.version}</span>
                  </div>
                </div>

                <div className="model-footer">
                  <span className="deploy-date">
                    Deployed: {formatTimestamp(model.deploymentTime)}
                  </span>
                  <button
                    className="btn-sm btn-primary"
                    onClick={(e) => {
                      e.stopPropagation();
                      setSelectedModel(model);
                      setShowInferenceModal(true);
                    }}
                  >
                    <Zap size={14} />
                    Run
                  </button>
                </div>
              </div>
            ))}

            {models.length === 0 && (
              <div className="empty-state">
                <Brain size={48} className="text-gray" />
                <p>No models deployed</p>
                <p className="text-muted">Deploy your first AI model to get started</p>
              </div>
            )}
          </>
        )}
      </div>

      {selectedModel && !showInferenceModal && !showDeployModal && (
        <div className="model-details">
          <h3>Model Details</h3>
          <button className="close-btn" onClick={() => setSelectedModel(null)}>×</button>
          
          <div className="detail-section">
            <h4>General Information</h4>
            <div className="detail-row">
              <span className="label">Model ID:</span>
              <span className="value mono">{selectedModel.id}</span>
            </div>
            <div className="detail-row">
              <span className="label">Name:</span>
              <span className="value">{selectedModel.name}</span>
            </div>
            <div className="detail-row">
              <span className="label">Architecture:</span>
              <span className="value">{selectedModel.architecture}</span>
            </div>
            <div className="detail-row">
              <span className="label">Version:</span>
              <span className="value">{selectedModel.version}</span>
            </div>
          </div>

          <div className="detail-section">
            <h4>Storage</h4>
            <div className="detail-row">
              <span className="label">Weights CID:</span>
              <span className="value mono">{selectedModel.weightsCid.slice(0, 20)}...</span>
            </div>
            <div className="detail-row">
              <span className="label">Owner:</span>
              <span className="value mono">{selectedModel.owner.slice(0, 12)}...</span>
            </div>
          </div>

          <div className="detail-section">
            <h4>Statistics</h4>
            <div className="detail-row">
              <span className="label">Total Inferences:</span>
              <span className="value">{selectedModel.totalInferences}</span>
            </div>
            <div className="detail-row">
              <span className="label">Last Updated:</span>
              <span className="value">{formatTimestamp(selectedModel.lastUpdated)}</span>
            </div>
          </div>

          <div className="model-actions">
            <button className="btn btn-secondary">
              <Download size={16} />
              Download Weights
            </button>
            <button 
              className="btn btn-primary"
              onClick={() => setShowInferenceModal(true)}
            >
              <Play size={16} />
              Run Inference
            </button>
          </div>
        </div>
      )}

      {showDeployModal && (
        <DeployModelModal
          onClose={() => setShowDeployModal(false)}
          onDeployed={() => {
            loadModels();
            setShowDeployModal(false);
          }}
        />
      )}

      {showInferenceModal && selectedModel && (
        <InferenceModal
          model={selectedModel}
          onClose={() => setShowInferenceModal(false)}
        />
      )}

      <style jsx>{`
        .models {
          padding: 2rem;
        }

        .models-header {
          display: flex;
          justify-content: space-between;
          align-items: center;
          margin-bottom: 2rem;
        }

        .models-header h2 {
          margin: 0;
          font-size: 1.5rem;
          font-weight: 600;
        }

        .btn {
          display: flex;
          align-items: center;
          gap: 0.5rem;
          padding: 0.75rem 1.5rem;
          border: none;
          border-radius: 0.5rem;
          font-size: 1rem;
          font-weight: 500;
          cursor: pointer;
          transition: all 0.2s;
        }

        .btn-primary {
          background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
          color: white;
        }

        .btn-secondary {
          background: #f3f4f6;
          color: #374151;
        }

        .btn-sm {
          padding: 0.5rem 1rem;
          font-size: 0.875rem;
        }

        .models-grid {
          display: grid;
          grid-template-columns: repeat(auto-fill, minmax(340px, 1fr));
          gap: 1.5rem;
        }

        .model-card {
          background: white;
          border-radius: 1rem;
          padding: 1.5rem;
          box-shadow: 0 4px 6px rgba(0, 0, 0, 0.1);
          cursor: pointer;
          transition: all 0.2s;
        }

        .model-card:hover {
          transform: translateY(-4px);
          box-shadow: 0 8px 16px rgba(0, 0, 0, 0.15);
        }

        .model-header {
          display: flex;
          justify-content: space-between;
          align-items: flex-start;
          margin-bottom: 1rem;
        }

        .model-card h3 {
          margin: 0 0 0.5rem 0;
          font-size: 1.25rem;
          font-weight: 600;
        }

        .model-architecture {
          color: #6b7280;
          margin: 0 0 1rem 0;
        }

        .status-badge {
          padding: 0.25rem 0.75rem;
          border-radius: 1rem;
          font-size: 0.75rem;
          font-weight: 500;
        }

        .model-stats {
          display: flex;
          gap: 1.5rem;
          margin-bottom: 1rem;
        }

        .stat {
          display: flex;
          align-items: center;
          gap: 0.5rem;
          color: #6b7280;
          font-size: 0.875rem;
        }

        .model-footer {
          display: flex;
          justify-content: space-between;
          align-items: center;
          padding-top: 1rem;
          border-top: 1px solid #f3f4f6;
        }

        .deploy-date {
          color: #9ca3af;
          font-size: 0.875rem;
        }

        .model-details {
          position: fixed;
          right: 0;
          top: 0;
          bottom: 0;
          width: 400px;
          background: white;
          box-shadow: -4px 0 16px rgba(0, 0, 0, 0.1);
          padding: 2rem;
          overflow-y: auto;
          z-index: 100;
        }

        .model-details h3 {
          margin: 0 0 1.5rem 0;
        }

        .close-btn {
          position: absolute;
          top: 1rem;
          right: 1rem;
          background: none;
          border: none;
          font-size: 1.5rem;
          cursor: pointer;
          color: #6b7280;
        }

        .detail-section {
          margin-bottom: 2rem;
        }

        .detail-section h4 {
          margin: 0 0 1rem 0;
          color: #374151;
          font-size: 0.875rem;
          text-transform: uppercase;
          letter-spacing: 0.05em;
        }

        .detail-row {
          display: flex;
          justify-content: space-between;
          padding: 0.5rem 0;
          border-bottom: 1px solid #f3f4f6;
        }

        .detail-row:last-child {
          border-bottom: none;
        }

        .label {
          color: #6b7280;
          font-size: 0.875rem;
        }

        .value {
          font-weight: 500;
          text-align: right;
        }

        .mono {
          font-family: monospace;
          font-size: 0.875rem;
        }

        .model-actions {
          display: flex;
          gap: 1rem;
          margin-top: 2rem;
        }

        .empty-state {
          grid-column: 1 / -1;
          text-align: center;
          padding: 3rem;
        }

        .empty-state p {
          margin: 0.5rem 0;
        }

        .text-gray { color: #6b7280; }
        .text-green { color: #10b981; }
        .text-blue { color: #3b82f6; }
        .text-yellow { color: #f59e0b; }
        .text-purple { color: #8b5cf6; }
        .text-muted { color: #9ca3af; }
      `}</style>
    </div>
  );
};

// Modal Components
const DeployModelModal: React.FC<{
  onClose: () => void;
  onDeployed: () => void;
}> = ({ onClose, onDeployed }) => {
  const [deployment, setDeployment] = useState<Partial<ModelDeployment>>({
    modelId: '',
    name: '',
    description: '',
    architecture: 'transformer',
    version: '1.0.0',
    weightsCid: '',
    metadata: {},
    owner: ''
  });
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const handleDeploy = async () => {
    setLoading(true);
    setError(null);
    
    try {
      await modelService.deploy(deployment as ModelDeployment);
      onDeployed();
    } catch (err: any) {
      setError(err.toString());
    } finally {
      setLoading(false);
    }
  };

  return (
    <div className="modal-overlay" onClick={onClose}>
      <div className="modal" onClick={e => e.stopPropagation()}>
        <h3>Deploy AI Model</h3>
        
        {error && <div className="error-message">{error}</div>}
        
        <div className="form-group">
          <label>Model Name</label>
          <input
            type="text"
            value={deployment.name}
            onChange={e => setDeployment({...deployment, name: e.target.value})}
            placeholder="GPT-2 Fine-tuned"
          />
        </div>

        <div className="form-group">
          <label>Architecture</label>
          <select 
            value={deployment.architecture}
            onChange={e => setDeployment({...deployment, architecture: e.target.value})}
          >
            <option value="transformer">Transformer</option>
            <option value="cnn">CNN</option>
            <option value="rnn">RNN</option>
            <option value="gan">GAN</option>
            <option value="custom">Custom</option>
          </select>
        </div>

        <div className="form-group">
          <label>Weights CID (IPFS)</label>
          <input
            type="text"
            value={deployment.weightsCid}
            onChange={e => setDeployment({...deployment, weightsCid: e.target.value})}
            placeholder="QmXoypizjW3WknFiJnKLwHCnL72vedxjQkDDP1mXWo6uco"
          />
        </div>

        <div className="form-group">
          <label>Description</label>
          <textarea
            value={deployment.description}
            onChange={e => setDeployment({...deployment, description: e.target.value})}
            placeholder="Model description..."
            rows={3}
          />
        </div>

        <div className="modal-actions">
          <button className="btn btn-secondary" onClick={onClose}>
            Cancel
          </button>
          <button 
            className="btn btn-primary" 
            onClick={handleDeploy}
            disabled={loading || !deployment.name || !deployment.weightsCid}
          >
            {loading ? 'Deploying...' : 'Deploy Model'}
          </button>
        </div>
      </div>

      <style jsx>{`
        .modal-overlay {
          position: fixed;
          top: 0;
          left: 0;
          right: 0;
          bottom: 0;
          background: rgba(0, 0, 0, 0.5);
          display: flex;
          align-items: center;
          justify-content: center;
          z-index: 1000;
        }

        .modal {
          background: white;
          border-radius: 1rem;
          padding: 2rem;
          width: 90%;
          max-width: 500px;
        }

        .modal h3 {
          margin: 0 0 1.5rem 0;
        }

        .form-group {
          margin-bottom: 1.5rem;
        }

        .form-group label {
          display: block;
          margin-bottom: 0.5rem;
          font-weight: 500;
        }

        .form-group input,
        .form-group select,
        .form-group textarea {
          width: 100%;
          padding: 0.75rem;
          border: 1px solid #e5e7eb;
          border-radius: 0.5rem;
          font-size: 1rem;
        }

        .error-message {
          background: #fee;
          color: #c00;
          padding: 0.75rem;
          border-radius: 0.5rem;
          margin-bottom: 1rem;
        }

        .modal-actions {
          display: flex;
          gap: 1rem;
          justify-content: flex-end;
        }

        .btn {
          padding: 0.75rem 1.5rem;
          border: none;
          border-radius: 0.5rem;
          font-size: 1rem;
          font-weight: 500;
          cursor: pointer;
        }

        .btn-primary {
          background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
          color: white;
        }

        .btn-secondary {
          background: #f3f4f6;
          color: #374151;
        }

        .btn:disabled {
          opacity: 0.5;
          cursor: not-allowed;
        }
      `}</style>
    </div>
  );
};

const InferenceModal: React.FC<{
  model: ModelInfo;
  onClose: () => void;
}> = ({ model, onClose }) => {
  const [input, setInput] = useState('');
  const [result, setResult] = useState<any>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const handleInference = async () => {
    setLoading(true);
    setError(null);
    setResult(null);
    
    try {
      const request: InferenceRequest = {
        modelId: model.id,
        input: JSON.parse(input),
        parameters: {}
      };
      
      const res = await modelService.runInference(request);
      setResult(res);
    } catch (err: any) {
      setError(err.toString());
    } finally {
      setLoading(false);
    }
  };

  return (
    <div className="modal-overlay" onClick={onClose}>
      <div className="modal large" onClick={e => e.stopPropagation()}>
        <h3>Run Inference - {model.name}</h3>
        
        {error && <div className="error-message">{error}</div>}
        
        <div className="form-group">
          <label>Input (JSON)</label>
          <textarea
            value={input}
            onChange={e => setInput(e.target.value)}
            placeholder='{"text": "Hello, world!"}'
            rows={5}
          />
        </div>

        {result && (
          <div className="result-section">
            <h4>Result</h4>
            <pre>{JSON.stringify(result, null, 2)}</pre>
          </div>
        )}

        <div className="modal-actions">
          <button className="btn btn-secondary" onClick={onClose}>
            Close
          </button>
          <button 
            className="btn btn-primary" 
            onClick={handleInference}
            disabled={loading || !input}
          >
            {loading ? 'Running...' : 'Run Inference'}
          </button>
        </div>
      </div>

      <style jsx>{`
        .modal-overlay {
          position: fixed;
          top: 0;
          left: 0;
          right: 0;
          bottom: 0;
          background: rgba(0, 0, 0, 0.5);
          display: flex;
          align-items: center;
          justify-content: center;
          z-index: 1000;
        }

        .modal {
          background: white;
          border-radius: 1rem;
          padding: 2rem;
          width: 90%;
          max-width: 600px;
        }

        .modal.large {
          max-width: 700px;
        }

        .modal h3 {
          margin: 0 0 1.5rem 0;
        }

        .form-group {
          margin-bottom: 1.5rem;
        }

        .form-group label {
          display: block;
          margin-bottom: 0.5rem;
          font-weight: 500;
        }

        .form-group textarea {
          width: 100%;
          padding: 0.75rem;
          border: 1px solid #e5e7eb;
          border-radius: 0.5rem;
          font-size: 1rem;
          font-family: monospace;
        }

        .result-section {
          background: #f9fafb;
          border-radius: 0.5rem;
          padding: 1rem;
          margin-bottom: 1.5rem;
        }

        .result-section h4 {
          margin: 0 0 0.5rem 0;
        }

        .result-section pre {
          margin: 0;
          font-size: 0.875rem;
          overflow-x: auto;
        }

        .error-message {
          background: #fee;
          color: #c00;
          padding: 0.75rem;
          border-radius: 0.5rem;
          margin-bottom: 1rem;
        }

        .modal-actions {
          display: flex;
          gap: 1rem;
          justify-content: flex-end;
        }

        .btn {
          padding: 0.75rem 1.5rem;
          border: none;
          border-radius: 0.5rem;
          font-size: 1rem;
          font-weight: 500;
          cursor: pointer;
        }

        .btn-primary {
          background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
          color: white;
        }

        .btn-secondary {
          background: #f3f4f6;
          color: #374151;
        }

        .btn:disabled {
          opacity: 0.5;
          cursor: not-allowed;
        }
      `}</style>
    </div>
  );
};