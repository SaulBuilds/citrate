/**
 * ImageModels.tsx - Image Generation and Training Interface
 *
 * Provides UI for:
 * - Browsing and managing image models (Stable Diffusion, Flux, etc.)
 * - Generating images with various parameters
 * - Training custom models (Dreambooth, LoRA)
 * - Gallery management for generated images
 */

import React, { useState, useEffect, useCallback } from 'react';
import { invoke } from '@tauri-apps/api/core';

// ============================================================================
// Types matching Rust backend
// ============================================================================

interface ImageModel {
  id: string;
  name: string;
  model_type: ImageModelType;
  architecture: ImageArchitecture;
  path: string;
  size_bytes: number;
  supported_resolutions: ImageResolution[];
  vae_path?: string;
  is_loaded: boolean;
  created_at: string;
}

type ImageModelType = 'StableDiffusion1x' | 'StableDiffusion2x' | 'SDXL' | 'SD3' | 'Flux' | 'Custom';

type ImageArchitecture = 'UNet' | 'DiT' | 'FluxTransformer' | 'Unknown';

interface ImageResolution {
  width: number;
  height: number;
  aspect_ratio: string;
}

interface ImageGenerationRequest {
  model_id: string;
  prompt: string;
  negative_prompt: string;
  width: number;
  height: number;
  steps: number;
  cfg_scale: number;
  seed?: number;
  scheduler: SchedulerType;
  batch_size: number;
}

type SchedulerType =
  | 'EulerAncestral'
  | 'Euler'
  | 'DPMPlusPlus2M'
  | 'DPMPlusPlus2MKarras'
  | 'DDIM'
  | 'LCM'
  | 'UniPC';

interface GenerationJob {
  id: string;
  model_id: string;
  prompt: string;
  status: GenerationStatus;
  progress: number;
  created_at: string;
  completed_at?: string;
  output_paths: string[];
  error?: string;
}

type GenerationStatus = 'Pending' | 'Running' | 'Completed' | 'Failed' | 'Cancelled';

interface ImageTrainingConfig {
  base_model_id: string;
  training_type: TrainingType;
  instance_prompt: string;
  class_prompt: string;
  instance_data_dir: string;
  output_dir: string;
  resolution: number;
  train_batch_size: number;
  gradient_accumulation_steps: number;
  learning_rate: number;
  lr_scheduler: string;
  lr_warmup_steps: number;
  max_train_steps: number;
  save_steps: number;
  mixed_precision: string;
  seed?: number;
  prior_preservation: boolean;
  prior_loss_weight: number;
  num_class_images: number;
}

type TrainingType = 'Dreambooth' | 'LoRA' | 'Textual Inversion';

interface ImageTrainingJob {
  id: string;
  config: ImageTrainingConfig;
  status: TrainingStatus;
  progress: number;
  current_step: number;
  total_steps: number;
  current_loss?: number;
  created_at: string;
  completed_at?: string;
  output_model_path?: string;
  error?: string;
}

type TrainingStatus = 'Pending' | 'Running' | 'Completed' | 'Failed' | 'Cancelled';

interface GeneratedImage {
  id: string;
  path: string;
  prompt: string;
  negative_prompt: string;
  model_id: string;
  seed: number;
  width: number;
  height: number;
  steps: number;
  cfg_scale: number;
  scheduler: string;
  created_at: string;
  favorite: boolean;
}

// ============================================================================
// Component
// ============================================================================

const ImageModels: React.FC = () => {
  // State
  const [activeTab, setActiveTab] = useState<'models' | 'generate' | 'training' | 'gallery'>('generate');
  const [models, setModels] = useState<ImageModel[]>([]);
  const [generationJobs, setGenerationJobs] = useState<GenerationJob[]>([]);
  const [trainingJobs, setTrainingJobs] = useState<ImageTrainingJob[]>([]);
  const [gallery, setGallery] = useState<GeneratedImage[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  // Generation form state
  const [genForm, setGenForm] = useState<ImageGenerationRequest>({
    model_id: '',
    prompt: '',
    negative_prompt: 'blurry, low quality, distorted, deformed',
    width: 512,
    height: 512,
    steps: 30,
    cfg_scale: 7.5,
    scheduler: 'EulerAncestral',
    batch_size: 1,
  });

  // Training form state
  const [trainForm, setTrainForm] = useState<Partial<ImageTrainingConfig>>({
    training_type: 'LoRA',
    instance_prompt: '',
    class_prompt: '',
    resolution: 512,
    train_batch_size: 1,
    gradient_accumulation_steps: 1,
    learning_rate: 1e-4,
    lr_scheduler: 'constant',
    lr_warmup_steps: 0,
    max_train_steps: 1000,
    save_steps: 500,
    mixed_precision: 'fp16',
    prior_preservation: false,
    prior_loss_weight: 1.0,
    num_class_images: 200,
  });

  // Selected items
  const [selectedModel, setSelectedModel] = useState<string | null>(null);
  const [selectedImage, setSelectedImage] = useState<GeneratedImage | null>(null);

  // ============================================================================
  // Data Loading
  // ============================================================================

  const loadModels = useCallback(async () => {
    try {
      const result = await invoke<ImageModel[]>('image_list_models');
      setModels(result);
      if (result.length > 0 && !genForm.model_id) {
        setGenForm(prev => ({ ...prev, model_id: result[0].id }));
      }
    } catch (err) {
      console.error('Failed to load models:', err);
    }
  }, [genForm.model_id]);

  const loadGenerationJobs = useCallback(async () => {
    try {
      const result = await invoke<GenerationJob[]>('image_list_generation_jobs');
      setGenerationJobs(result);
    } catch (err) {
      console.error('Failed to load generation jobs:', err);
    }
  }, []);

  const loadTrainingJobs = useCallback(async () => {
    try {
      const result = await invoke<ImageTrainingJob[]>('image_list_training_jobs');
      setTrainingJobs(result);
    } catch (err) {
      console.error('Failed to load training jobs:', err);
    }
  }, []);

  const loadGallery = useCallback(async () => {
    try {
      const result = await invoke<GeneratedImage[]>('image_get_gallery');
      setGallery(result);
    } catch (err) {
      console.error('Failed to load gallery:', err);
    }
  }, []);

  useEffect(() => {
    loadModels();
    loadGenerationJobs();
    loadTrainingJobs();
    loadGallery();

    // Poll for updates every 5 seconds
    const interval = setInterval(() => {
      loadGenerationJobs();
      loadTrainingJobs();
    }, 5000);

    return () => clearInterval(interval);
  }, [loadModels, loadGenerationJobs, loadTrainingJobs, loadGallery]);

  // ============================================================================
  // Actions
  // ============================================================================

  const handleGenerate = async () => {
    if (!genForm.model_id || !genForm.prompt) {
      setError('Please select a model and enter a prompt');
      return;
    }

    setLoading(true);
    setError(null);

    try {
      await invoke<GenerationJob>('image_create_generation_job', {
        request: genForm,
      });
      await loadGenerationJobs();
    } catch (err) {
      setError(`Generation failed: ${err}`);
    } finally {
      setLoading(false);
    }
  };

  const handleCancelGeneration = async (jobId: string) => {
    try {
      await invoke('image_cancel_generation_job', { jobId });
      await loadGenerationJobs();
    } catch (err) {
      setError(`Cancel failed: ${err}`);
    }
  };

  const handleStartTraining = async () => {
    if (!trainForm.base_model_id || !trainForm.instance_prompt || !trainForm.instance_data_dir) {
      setError('Please fill in required training fields');
      return;
    }

    setLoading(true);
    setError(null);

    try {
      await invoke<ImageTrainingJob>('image_start_training', {
        config: trainForm as ImageTrainingConfig,
      });
      await loadTrainingJobs();
    } catch (err) {
      setError(`Training failed to start: ${err}`);
    } finally {
      setLoading(false);
    }
  };

  const handleCancelTraining = async (jobId: string) => {
    try {
      await invoke('image_cancel_training_job', { jobId });
      await loadTrainingJobs();
    } catch (err) {
      setError(`Cancel failed: ${err}`);
    }
  };

  const handleDeleteImage = async (imageId: string) => {
    try {
      await invoke('image_delete_from_gallery', { imageId });
      await loadGallery();
      if (selectedImage?.id === imageId) {
        setSelectedImage(null);
      }
    } catch (err) {
      setError(`Delete failed: ${err}`);
    }
  };

  // ============================================================================
  // Render Helpers
  // ============================================================================

  const formatBytes = (bytes: number): string => {
    if (bytes < 1024) return `${bytes} B`;
    if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
    if (bytes < 1024 * 1024 * 1024) return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
    return `${(bytes / (1024 * 1024 * 1024)).toFixed(2)} GB`;
  };

  const getStatusColor = (status: string): string => {
    switch (status) {
      case 'Completed':
        return 'text-green-400';
      case 'Running':
        return 'text-blue-400';
      case 'Failed':
        return 'text-red-400';
      case 'Cancelled':
        return 'text-gray-400';
      default:
        return 'text-yellow-400';
    }
  };

  const getModelTypeLabel = (type: ImageModelType): string => {
    switch (type) {
      case 'StableDiffusion1x':
        return 'SD 1.x';
      case 'StableDiffusion2x':
        return 'SD 2.x';
      case 'SDXL':
        return 'SDXL';
      case 'SD3':
        return 'SD3';
      case 'Flux':
        return 'Flux';
      default:
        return 'Custom';
    }
  };

  // ============================================================================
  // Render
  // ============================================================================

  return (
    <div className="h-full flex flex-col bg-gray-900 text-white">
      {/* Header */}
      <div className="border-b border-gray-700 p-4">
        <h1 className="text-xl font-bold mb-4">Image Models</h1>

        {/* Tabs */}
        <div className="flex space-x-4">
          {(['generate', 'models', 'training', 'gallery'] as const).map(tab => (
            <button
              key={tab}
              onClick={() => setActiveTab(tab)}
              className={`px-4 py-2 rounded-lg transition-colors ${
                activeTab === tab
                  ? 'bg-purple-600 text-white'
                  : 'bg-gray-800 text-gray-400 hover:bg-gray-700'
              }`}
            >
              {tab.charAt(0).toUpperCase() + tab.slice(1)}
            </button>
          ))}
        </div>
      </div>

      {/* Error Banner */}
      {error && (
        <div className="bg-red-900/50 border border-red-500 text-red-200 px-4 py-2 m-4 rounded">
          {error}
          <button onClick={() => setError(null)} className="float-right text-red-400 hover:text-red-200">
            x
          </button>
        </div>
      )}

      {/* Content */}
      <div className="flex-1 overflow-auto p-4">
        {/* Generate Tab */}
        {activeTab === 'generate' && (
          <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
            {/* Generation Form */}
            <div className="bg-gray-800 rounded-lg p-6">
              <h2 className="text-lg font-semibold mb-4">Generate Image</h2>

              <div className="space-y-4">
                {/* Model Selection */}
                <div>
                  <label className="block text-sm text-gray-400 mb-1">Model</label>
                  <select
                    value={genForm.model_id}
                    onChange={e => setGenForm(prev => ({ ...prev, model_id: e.target.value }))}
                    className="w-full bg-gray-700 rounded px-3 py-2 text-white"
                  >
                    <option value="">Select a model...</option>
                    {models.map(model => (
                      <option key={model.id} value={model.id}>
                        {model.name} ({getModelTypeLabel(model.model_type)})
                      </option>
                    ))}
                  </select>
                </div>

                {/* Prompt */}
                <div>
                  <label className="block text-sm text-gray-400 mb-1">Prompt</label>
                  <textarea
                    value={genForm.prompt}
                    onChange={e => setGenForm(prev => ({ ...prev, prompt: e.target.value }))}
                    placeholder="A beautiful sunset over mountains, photorealistic, 8k..."
                    rows={3}
                    className="w-full bg-gray-700 rounded px-3 py-2 text-white resize-none"
                  />
                </div>

                {/* Negative Prompt */}
                <div>
                  <label className="block text-sm text-gray-400 mb-1">Negative Prompt</label>
                  <textarea
                    value={genForm.negative_prompt}
                    onChange={e => setGenForm(prev => ({ ...prev, negative_prompt: e.target.value }))}
                    rows={2}
                    className="w-full bg-gray-700 rounded px-3 py-2 text-white resize-none"
                  />
                </div>

                {/* Resolution */}
                <div className="grid grid-cols-2 gap-4">
                  <div>
                    <label className="block text-sm text-gray-400 mb-1">Width</label>
                    <select
                      value={genForm.width}
                      onChange={e => setGenForm(prev => ({ ...prev, width: parseInt(e.target.value) }))}
                      className="w-full bg-gray-700 rounded px-3 py-2"
                    >
                      {[256, 512, 768, 1024, 1280, 1536, 2048].map(w => (
                        <option key={w} value={w}>
                          {w}px
                        </option>
                      ))}
                    </select>
                  </div>
                  <div>
                    <label className="block text-sm text-gray-400 mb-1">Height</label>
                    <select
                      value={genForm.height}
                      onChange={e => setGenForm(prev => ({ ...prev, height: parseInt(e.target.value) }))}
                      className="w-full bg-gray-700 rounded px-3 py-2"
                    >
                      {[256, 512, 768, 1024, 1280, 1536, 2048].map(h => (
                        <option key={h} value={h}>
                          {h}px
                        </option>
                      ))}
                    </select>
                  </div>
                </div>

                {/* Steps and CFG */}
                <div className="grid grid-cols-2 gap-4">
                  <div>
                    <label className="block text-sm text-gray-400 mb-1">Steps: {genForm.steps}</label>
                    <input
                      type="range"
                      min={1}
                      max={150}
                      value={genForm.steps}
                      onChange={e => setGenForm(prev => ({ ...prev, steps: parseInt(e.target.value) }))}
                      className="w-full"
                    />
                  </div>
                  <div>
                    <label className="block text-sm text-gray-400 mb-1">CFG Scale: {genForm.cfg_scale}</label>
                    <input
                      type="range"
                      min={1}
                      max={20}
                      step={0.5}
                      value={genForm.cfg_scale}
                      onChange={e => setGenForm(prev => ({ ...prev, cfg_scale: parseFloat(e.target.value) }))}
                      className="w-full"
                    />
                  </div>
                </div>

                {/* Scheduler */}
                <div>
                  <label className="block text-sm text-gray-400 mb-1">Scheduler</label>
                  <select
                    value={genForm.scheduler}
                    onChange={e => setGenForm(prev => ({ ...prev, scheduler: e.target.value as SchedulerType }))}
                    className="w-full bg-gray-700 rounded px-3 py-2"
                  >
                    <option value="EulerAncestral">Euler Ancestral</option>
                    <option value="Euler">Euler</option>
                    <option value="DPMPlusPlus2M">DPM++ 2M</option>
                    <option value="DPMPlusPlus2MKarras">DPM++ 2M Karras</option>
                    <option value="DDIM">DDIM</option>
                    <option value="LCM">LCM</option>
                    <option value="UniPC">UniPC</option>
                  </select>
                </div>

                {/* Seed */}
                <div>
                  <label className="block text-sm text-gray-400 mb-1">Seed (empty for random)</label>
                  <input
                    type="number"
                    value={genForm.seed ?? ''}
                    onChange={e =>
                      setGenForm(prev => ({
                        ...prev,
                        seed: e.target.value ? parseInt(e.target.value) : undefined,
                      }))
                    }
                    placeholder="Random"
                    className="w-full bg-gray-700 rounded px-3 py-2"
                  />
                </div>

                {/* Batch Size */}
                <div>
                  <label className="block text-sm text-gray-400 mb-1">Batch Size</label>
                  <select
                    value={genForm.batch_size}
                    onChange={e => setGenForm(prev => ({ ...prev, batch_size: parseInt(e.target.value) }))}
                    className="w-full bg-gray-700 rounded px-3 py-2"
                  >
                    {[1, 2, 4, 8].map(b => (
                      <option key={b} value={b}>
                        {b} image{b > 1 ? 's' : ''}
                      </option>
                    ))}
                  </select>
                </div>

                {/* Generate Button */}
                <button
                  onClick={handleGenerate}
                  disabled={loading || !genForm.model_id || !genForm.prompt}
                  className={`w-full py-3 rounded-lg font-semibold transition-colors ${
                    loading || !genForm.model_id || !genForm.prompt
                      ? 'bg-gray-600 text-gray-400 cursor-not-allowed'
                      : 'bg-purple-600 hover:bg-purple-700 text-white'
                  }`}
                >
                  {loading ? 'Generating...' : 'Generate'}
                </button>
              </div>
            </div>

            {/* Generation Queue */}
            <div className="bg-gray-800 rounded-lg p-6">
              <h2 className="text-lg font-semibold mb-4">Generation Queue</h2>

              {generationJobs.length === 0 ? (
                <p className="text-gray-500 text-center py-8">No generation jobs yet</p>
              ) : (
                <div className="space-y-3">
                  {generationJobs.map(job => (
                    <div key={job.id} className="bg-gray-700 rounded-lg p-4">
                      <div className="flex justify-between items-start mb-2">
                        <div className="flex-1 mr-4">
                          <p className="text-sm text-gray-300 truncate">{job.prompt}</p>
                          <p className={`text-xs ${getStatusColor(job.status)}`}>{job.status}</p>
                        </div>
                        {(job.status === 'Pending' || job.status === 'Running') && (
                          <button
                            onClick={() => handleCancelGeneration(job.id)}
                            className="text-red-400 hover:text-red-300 text-sm"
                          >
                            Cancel
                          </button>
                        )}
                      </div>

                      {job.status === 'Running' && (
                        <div className="w-full bg-gray-600 rounded-full h-2">
                          <div
                            className="bg-purple-500 h-2 rounded-full transition-all"
                            style={{ width: `${job.progress}%` }}
                          />
                        </div>
                      )}

                      {job.status === 'Completed' && job.output_paths.length > 0 && (
                        <div className="mt-2 grid grid-cols-4 gap-2">
                          {job.output_paths.slice(0, 4).map((path, i) => (
                            <div
                              key={i}
                              className="aspect-square bg-gray-600 rounded overflow-hidden"
                            >
                              {/* Placeholder - in real app would show image */}
                              <div className="w-full h-full flex items-center justify-center text-gray-400 text-xs">
                                Image {i + 1}
                              </div>
                            </div>
                          ))}
                        </div>
                      )}

                      {job.error && <p className="text-red-400 text-xs mt-2">{job.error}</p>}
                    </div>
                  ))}
                </div>
              )}
            </div>
          </div>
        )}

        {/* Models Tab */}
        {activeTab === 'models' && (
          <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
            {models.length === 0 ? (
              <div className="col-span-full text-center py-12">
                <p className="text-gray-500 mb-4">No image models found</p>
                <p className="text-gray-600 text-sm">
                  Download models from Hugging Face or add custom models to get started
                </p>
              </div>
            ) : (
              models.map(model => (
                <div
                  key={model.id}
                  onClick={() => setSelectedModel(selectedModel === model.id ? null : model.id)}
                  className={`bg-gray-800 rounded-lg p-4 cursor-pointer transition-all ${
                    selectedModel === model.id ? 'ring-2 ring-purple-500' : 'hover:bg-gray-750'
                  }`}
                >
                  <div className="flex justify-between items-start mb-3">
                    <h3 className="font-semibold">{model.name}</h3>
                    <span
                      className={`px-2 py-1 rounded text-xs ${
                        model.is_loaded ? 'bg-green-900 text-green-400' : 'bg-gray-700 text-gray-400'
                      }`}
                    >
                      {model.is_loaded ? 'Loaded' : 'Ready'}
                    </span>
                  </div>

                  <div className="space-y-2 text-sm">
                    <div className="flex justify-between text-gray-400">
                      <span>Type</span>
                      <span className="text-white">{getModelTypeLabel(model.model_type)}</span>
                    </div>
                    <div className="flex justify-between text-gray-400">
                      <span>Architecture</span>
                      <span className="text-white">{model.architecture}</span>
                    </div>
                    <div className="flex justify-between text-gray-400">
                      <span>Size</span>
                      <span className="text-white">{formatBytes(model.size_bytes)}</span>
                    </div>
                    <div className="flex justify-between text-gray-400">
                      <span>Resolutions</span>
                      <span className="text-white">
                        {model.supported_resolutions.length > 0
                          ? `${model.supported_resolutions[0].width}x${model.supported_resolutions[0].height}`
                          : 'Any'}
                      </span>
                    </div>
                  </div>

                  {selectedModel === model.id && (
                    <div className="mt-4 pt-4 border-t border-gray-700 space-y-2">
                      <button
                        onClick={() => setGenForm(prev => ({ ...prev, model_id: model.id }))}
                        className="w-full py-2 bg-purple-600 hover:bg-purple-700 rounded text-sm"
                      >
                        Use for Generation
                      </button>
                      <button
                        onClick={() => {
                          setTrainForm(prev => ({ ...prev, base_model_id: model.id }));
                          setActiveTab('training');
                        }}
                        className="w-full py-2 bg-gray-700 hover:bg-gray-600 rounded text-sm"
                      >
                        Fine-tune Model
                      </button>
                    </div>
                  )}
                </div>
              ))
            )}
          </div>
        )}

        {/* Training Tab */}
        {activeTab === 'training' && (
          <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
            {/* Training Form */}
            <div className="bg-gray-800 rounded-lg p-6">
              <h2 className="text-lg font-semibold mb-4">Train Custom Model</h2>

              <div className="space-y-4">
                {/* Base Model */}
                <div>
                  <label className="block text-sm text-gray-400 mb-1">Base Model</label>
                  <select
                    value={trainForm.base_model_id ?? ''}
                    onChange={e => setTrainForm(prev => ({ ...prev, base_model_id: e.target.value }))}
                    className="w-full bg-gray-700 rounded px-3 py-2"
                  >
                    <option value="">Select base model...</option>
                    {models.map(model => (
                      <option key={model.id} value={model.id}>
                        {model.name} ({getModelTypeLabel(model.model_type)})
                      </option>
                    ))}
                  </select>
                </div>

                {/* Training Type */}
                <div>
                  <label className="block text-sm text-gray-400 mb-1">Training Type</label>
                  <select
                    value={trainForm.training_type}
                    onChange={e =>
                      setTrainForm(prev => ({ ...prev, training_type: e.target.value as TrainingType }))
                    }
                    className="w-full bg-gray-700 rounded px-3 py-2"
                  >
                    <option value="LoRA">LoRA (Low-Rank Adaptation)</option>
                    <option value="Dreambooth">Dreambooth (Full Fine-tune)</option>
                    <option value="Textual Inversion">Textual Inversion</option>
                  </select>
                </div>

                {/* Instance Prompt */}
                <div>
                  <label className="block text-sm text-gray-400 mb-1">Instance Prompt</label>
                  <input
                    type="text"
                    value={trainForm.instance_prompt ?? ''}
                    onChange={e => setTrainForm(prev => ({ ...prev, instance_prompt: e.target.value }))}
                    placeholder="a photo of sks dog"
                    className="w-full bg-gray-700 rounded px-3 py-2"
                  />
                  <p className="text-xs text-gray-500 mt-1">
                    Use a unique token (e.g., "sks") to represent your subject
                  </p>
                </div>

                {/* Class Prompt */}
                <div>
                  <label className="block text-sm text-gray-400 mb-1">Class Prompt</label>
                  <input
                    type="text"
                    value={trainForm.class_prompt ?? ''}
                    onChange={e => setTrainForm(prev => ({ ...prev, class_prompt: e.target.value }))}
                    placeholder="a photo of dog"
                    className="w-full bg-gray-700 rounded px-3 py-2"
                  />
                </div>

                {/* Instance Data Directory */}
                <div>
                  <label className="block text-sm text-gray-400 mb-1">Training Images Directory</label>
                  <input
                    type="text"
                    value={trainForm.instance_data_dir ?? ''}
                    onChange={e => setTrainForm(prev => ({ ...prev, instance_data_dir: e.target.value }))}
                    placeholder="/path/to/training/images"
                    className="w-full bg-gray-700 rounded px-3 py-2"
                  />
                </div>

                {/* Resolution and Steps */}
                <div className="grid grid-cols-2 gap-4">
                  <div>
                    <label className="block text-sm text-gray-400 mb-1">Resolution</label>
                    <select
                      value={trainForm.resolution}
                      onChange={e => setTrainForm(prev => ({ ...prev, resolution: parseInt(e.target.value) }))}
                      className="w-full bg-gray-700 rounded px-3 py-2"
                    >
                      <option value={512}>512x512</option>
                      <option value={768}>768x768</option>
                      <option value={1024}>1024x1024</option>
                    </select>
                  </div>
                  <div>
                    <label className="block text-sm text-gray-400 mb-1">Max Steps</label>
                    <input
                      type="number"
                      value={trainForm.max_train_steps}
                      onChange={e =>
                        setTrainForm(prev => ({ ...prev, max_train_steps: parseInt(e.target.value) }))
                      }
                      min={100}
                      max={10000}
                      className="w-full bg-gray-700 rounded px-3 py-2"
                    />
                  </div>
                </div>

                {/* Learning Rate */}
                <div>
                  <label className="block text-sm text-gray-400 mb-1">Learning Rate</label>
                  <select
                    value={trainForm.learning_rate}
                    onChange={e =>
                      setTrainForm(prev => ({ ...prev, learning_rate: parseFloat(e.target.value) }))
                    }
                    className="w-full bg-gray-700 rounded px-3 py-2"
                  >
                    <option value={1e-3}>1e-3 (Fast, may overfit)</option>
                    <option value={5e-4}>5e-4</option>
                    <option value={1e-4}>1e-4 (Recommended)</option>
                    <option value={5e-5}>5e-5</option>
                    <option value={1e-5}>1e-5 (Slow, stable)</option>
                  </select>
                </div>

                {/* Prior Preservation */}
                <div className="flex items-center space-x-3">
                  <input
                    type="checkbox"
                    id="prior-preservation"
                    checked={trainForm.prior_preservation}
                    onChange={e => setTrainForm(prev => ({ ...prev, prior_preservation: e.target.checked }))}
                    className="w-4 h-4"
                  />
                  <label htmlFor="prior-preservation" className="text-sm text-gray-400">
                    Enable prior preservation (helps prevent overfitting)
                  </label>
                </div>

                {/* Start Training Button */}
                <button
                  onClick={handleStartTraining}
                  disabled={loading || !trainForm.base_model_id || !trainForm.instance_prompt}
                  className={`w-full py-3 rounded-lg font-semibold transition-colors ${
                    loading || !trainForm.base_model_id || !trainForm.instance_prompt
                      ? 'bg-gray-600 text-gray-400 cursor-not-allowed'
                      : 'bg-green-600 hover:bg-green-700 text-white'
                  }`}
                >
                  {loading ? 'Starting...' : 'Start Training'}
                </button>
              </div>
            </div>

            {/* Training Jobs */}
            <div className="bg-gray-800 rounded-lg p-6">
              <h2 className="text-lg font-semibold mb-4">Training Jobs</h2>

              {trainingJobs.length === 0 ? (
                <p className="text-gray-500 text-center py-8">No training jobs yet</p>
              ) : (
                <div className="space-y-3">
                  {trainingJobs.map(job => (
                    <div key={job.id} className="bg-gray-700 rounded-lg p-4">
                      <div className="flex justify-between items-start mb-2">
                        <div>
                          <p className="text-sm font-medium">{job.config.training_type} Training</p>
                          <p className="text-xs text-gray-400">{job.config.instance_prompt}</p>
                          <p className={`text-xs ${getStatusColor(job.status)}`}>{job.status}</p>
                        </div>
                        {(job.status === 'Pending' || job.status === 'Running') && (
                          <button
                            onClick={() => handleCancelTraining(job.id)}
                            className="text-red-400 hover:text-red-300 text-sm"
                          >
                            Cancel
                          </button>
                        )}
                      </div>

                      {job.status === 'Running' && (
                        <>
                          <div className="w-full bg-gray-600 rounded-full h-2 mb-2">
                            <div
                              className="bg-green-500 h-2 rounded-full transition-all"
                              style={{ width: `${job.progress}%` }}
                            />
                          </div>
                          <div className="flex justify-between text-xs text-gray-400">
                            <span>
                              Step {job.current_step} / {job.total_steps}
                            </span>
                            {job.current_loss && <span>Loss: {job.current_loss.toFixed(4)}</span>}
                          </div>
                        </>
                      )}

                      {job.status === 'Completed' && job.output_model_path && (
                        <p className="text-green-400 text-xs mt-2">
                          Output: {job.output_model_path.split('/').pop()}
                        </p>
                      )}

                      {job.error && <p className="text-red-400 text-xs mt-2">{job.error}</p>}
                    </div>
                  ))}
                </div>
              )}
            </div>
          </div>
        )}

        {/* Gallery Tab */}
        {activeTab === 'gallery' && (
          <div className="grid grid-cols-2 md:grid-cols-3 lg:grid-cols-4 xl:grid-cols-6 gap-4">
            {gallery.length === 0 ? (
              <div className="col-span-full text-center py-12">
                <p className="text-gray-500 mb-4">Your gallery is empty</p>
                <p className="text-gray-600 text-sm">Generated images will appear here</p>
              </div>
            ) : (
              gallery.map(image => (
                <div
                  key={image.id}
                  onClick={() => setSelectedImage(selectedImage?.id === image.id ? null : image)}
                  className={`relative aspect-square bg-gray-800 rounded-lg overflow-hidden cursor-pointer transition-all ${
                    selectedImage?.id === image.id ? 'ring-2 ring-purple-500' : 'hover:ring-1 hover:ring-gray-600'
                  }`}
                >
                  {/* Placeholder - would show actual image in production */}
                  <div className="w-full h-full flex items-center justify-center bg-gradient-to-br from-purple-900/30 to-blue-900/30">
                    <span className="text-gray-500 text-xs text-center px-2">{image.prompt.slice(0, 30)}...</span>
                  </div>

                  {image.favorite && (
                    <div className="absolute top-2 right-2 text-yellow-400">
                      <svg className="w-4 h-4" fill="currentColor" viewBox="0 0 20 20">
                        <path d="M9.049 2.927c.3-.921 1.603-.921 1.902 0l1.07 3.292a1 1 0 00.95.69h3.462c.969 0 1.371 1.24.588 1.81l-2.8 2.034a1 1 0 00-.364 1.118l1.07 3.292c.3.921-.755 1.688-1.54 1.118l-2.8-2.034a1 1 0 00-1.175 0l-2.8 2.034c-.784.57-1.838-.197-1.539-1.118l1.07-3.292a1 1 0 00-.364-1.118L2.98 8.72c-.783-.57-.38-1.81.588-1.81h3.461a1 1 0 00.951-.69l1.07-3.292z" />
                      </svg>
                    </div>
                  )}
                </div>
              ))
            )}
          </div>
        )}

        {/* Image Detail Modal */}
        {selectedImage && activeTab === 'gallery' && (
          <div className="fixed inset-0 bg-black/80 flex items-center justify-center z-50 p-8">
            <div className="bg-gray-800 rounded-lg max-w-4xl w-full max-h-full overflow-auto">
              <div className="p-6">
                <div className="flex justify-between items-start mb-4">
                  <h3 className="text-lg font-semibold">Image Details</h3>
                  <button
                    onClick={() => setSelectedImage(null)}
                    className="text-gray-400 hover:text-white"
                  >
                    <svg className="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                      <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M6 18L18 6M6 6l12 12" />
                    </svg>
                  </button>
                </div>

                <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
                  {/* Image Preview */}
                  <div className="aspect-square bg-gray-700 rounded-lg flex items-center justify-center">
                    <span className="text-gray-500">Image Preview</span>
                  </div>

                  {/* Metadata */}
                  <div className="space-y-4">
                    <div>
                      <label className="block text-sm text-gray-400 mb-1">Prompt</label>
                      <p className="text-white bg-gray-700 rounded p-3 text-sm">{selectedImage.prompt}</p>
                    </div>

                    {selectedImage.negative_prompt && (
                      <div>
                        <label className="block text-sm text-gray-400 mb-1">Negative Prompt</label>
                        <p className="text-white bg-gray-700 rounded p-3 text-sm">
                          {selectedImage.negative_prompt}
                        </p>
                      </div>
                    )}

                    <div className="grid grid-cols-2 gap-4 text-sm">
                      <div>
                        <span className="text-gray-400">Size</span>
                        <p className="text-white">
                          {selectedImage.width}x{selectedImage.height}
                        </p>
                      </div>
                      <div>
                        <span className="text-gray-400">Steps</span>
                        <p className="text-white">{selectedImage.steps}</p>
                      </div>
                      <div>
                        <span className="text-gray-400">CFG Scale</span>
                        <p className="text-white">{selectedImage.cfg_scale}</p>
                      </div>
                      <div>
                        <span className="text-gray-400">Seed</span>
                        <p className="text-white font-mono text-xs">{selectedImage.seed}</p>
                      </div>
                      <div>
                        <span className="text-gray-400">Scheduler</span>
                        <p className="text-white">{selectedImage.scheduler}</p>
                      </div>
                      <div>
                        <span className="text-gray-400">Created</span>
                        <p className="text-white">{new Date(selectedImage.created_at).toLocaleDateString()}</p>
                      </div>
                    </div>

                    <div className="flex space-x-3 pt-4">
                      <button
                        onClick={() => {
                          setGenForm(prev => ({
                            ...prev,
                            prompt: selectedImage.prompt,
                            negative_prompt: selectedImage.negative_prompt,
                            width: selectedImage.width,
                            height: selectedImage.height,
                            steps: selectedImage.steps,
                            cfg_scale: selectedImage.cfg_scale,
                            seed: selectedImage.seed,
                          }));
                          setActiveTab('generate');
                          setSelectedImage(null);
                        }}
                        className="flex-1 py-2 bg-purple-600 hover:bg-purple-700 rounded text-sm"
                      >
                        Regenerate
                      </button>
                      <button
                        onClick={() => handleDeleteImage(selectedImage.id)}
                        className="px-4 py-2 bg-red-600 hover:bg-red-700 rounded text-sm"
                      >
                        Delete
                      </button>
                    </div>
                  </div>
                </div>
              </div>
            </div>
          </div>
        )}
      </div>
    </div>
  );
};

export default ImageModels;
