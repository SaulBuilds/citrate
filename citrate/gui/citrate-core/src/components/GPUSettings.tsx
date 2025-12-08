import { useState, useEffect, useCallback } from 'react';
import { invoke } from '@tauri-apps/api/core';

// ============================================================================
// Types
// ============================================================================

interface GPUDevice {
  id: string;
  name: string;
  vendor: 'Apple' | 'NVIDIA' | 'AMD' | 'Intel' | 'Unknown';
  total_memory: number;
  available_memory: number;
  compute_capability: string;
  in_use: boolean;
  backend: 'Metal' | 'CUDA' | 'ROCm' | 'Vulkan' | 'OpenCL' | 'CPU';
  temperature: number | null;
  power_usage: number | null;
  utilization: number;
}

type ComputeJobType = 'Inference' | 'Training' | 'LoRAFineTune' | 'Embedding' | 'ImageGeneration';

interface GPUAllocationSettings {
  enabled: boolean;
  allocation_percentage: number;
  max_memory_allocation: number;
  min_payment_threshold: number;
  max_concurrent_jobs: number;
  allowed_job_types: ComputeJobType[];
  schedule: [number, number] | null;
}

interface GPUStats {
  jobs_completed: number;
  jobs_failed: number;
  tokens_earned: number;
  total_compute_time: number;
  avg_job_duration: number;
  queue_depth: number;
  current_memory_usage: number;
  session_start: number;
}

interface ProviderStatus {
  is_registered: boolean;
  address: string | null;
  stake: number;
  reputation: number;
  last_heartbeat: number;
  active_jobs: string[];
}

interface ComputeJobStatus {
  Queued?: null;
  Running?: { started_at: number; progress: number };
  Completed?: { started_at: number; completed_at: number; result_hash: string };
  Failed?: { error: string; failed_at: number };
  Cancelled?: null;
}

interface ComputeJob {
  id: string;
  job_type: ComputeJobType;
  model_id: string;
  input_hash: string;
  requester: string;
  max_payment: number;
  status: ComputeJobStatus;
  created_at: number;
  memory_required: number;
  estimated_time: number;
  priority: number;
}

// ============================================================================
// Helper Functions
// ============================================================================

function formatBytes(bytes: number): string {
  if (bytes === 0) return '0 B';
  const k = 1024;
  const sizes = ['B', 'KB', 'MB', 'GB', 'TB'];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  return `${parseFloat((bytes / Math.pow(k, i)).toFixed(1))} ${sizes[i]}`;
}

function formatDuration(seconds: number): string {
  if (seconds < 60) return `${seconds}s`;
  if (seconds < 3600) return `${Math.floor(seconds / 60)}m ${seconds % 60}s`;
  const hours = Math.floor(seconds / 3600);
  const mins = Math.floor((seconds % 3600) / 60);
  return `${hours}h ${mins}m`;
}

function getJobStatusLabel(status: ComputeJobStatus): string {
  if ('Queued' in status) return 'Queued';
  if ('Running' in status) return `Running (${Math.round((status.Running?.progress || 0) * 100)}%)`;
  if ('Completed' in status) return 'Completed';
  if ('Failed' in status) return 'Failed';
  if ('Cancelled' in status) return 'Cancelled';
  return 'Unknown';
}

function getJobStatusColor(status: ComputeJobStatus): string {
  if ('Queued' in status) return 'text-yellow-400';
  if ('Running' in status) return 'text-blue-400';
  if ('Completed' in status) return 'text-green-400';
  if ('Failed' in status) return 'text-red-400';
  if ('Cancelled' in status) return 'text-gray-400';
  return 'text-gray-400';
}

const JOB_TYPE_LABELS: Record<ComputeJobType, string> = {
  Inference: 'Inference',
  Training: 'Training',
  LoRAFineTune: 'LoRA Fine-tune',
  Embedding: 'Embeddings',
  ImageGeneration: 'Image Generation',
};

// ============================================================================
// Component
// ============================================================================

export function GPUSettings() {
  // State
  const [devices, setDevices] = useState<GPUDevice[]>([]);
  const [settings, setSettings] = useState<GPUAllocationSettings | null>(null);
  const [stats, setStats] = useState<GPUStats | null>(null);
  const [providerStatus, setProviderStatus] = useState<ProviderStatus | null>(null);
  const [jobs, setJobs] = useState<ComputeJob[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [saving, setSaving] = useState(false);
  const [activeTab, setActiveTab] = useState<'devices' | 'settings' | 'jobs' | 'stats'>('devices');

  // Edit state for settings
  const [editSettings, setEditSettings] = useState<GPUAllocationSettings | null>(null);

  // Fetch all data
  const fetchData = useCallback(async () => {
    try {
      const [devs, sett, st, prov, allJobs] = await Promise.all([
        invoke<GPUDevice[]>('gpu_get_devices'),
        invoke<GPUAllocationSettings>('gpu_get_settings'),
        invoke<GPUStats>('gpu_get_stats'),
        invoke<ProviderStatus>('gpu_get_provider_status'),
        invoke<ComputeJob[]>('gpu_get_all_jobs'),
      ]);
      setDevices(devs);
      setSettings(sett);
      setEditSettings(sett);
      setStats(st);
      setProviderStatus(prov);
      setJobs(allJobs);
      setError(null);
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    fetchData();
    // Refresh every 5 seconds
    const interval = setInterval(fetchData, 5000);
    return () => clearInterval(interval);
  }, [fetchData]);

  // Refresh devices
  const handleRefreshDevices = async () => {
    try {
      const devs = await invoke<GPUDevice[]>('gpu_refresh_devices');
      setDevices(devs);
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
    }
  };

  // Save settings
  const handleSaveSettings = async () => {
    if (!editSettings) return;
    setSaving(true);
    try {
      await invoke('gpu_update_settings', { settings: editSettings });
      setSettings(editSettings);
      setError(null);
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
    } finally {
      setSaving(false);
    }
  };

  // Cancel job
  const handleCancelJob = async (jobId: string) => {
    try {
      await invoke('gpu_cancel_job', { jobId });
      await fetchData();
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
    }
  };

  // Toggle job type in settings
  const toggleJobType = (jobType: ComputeJobType) => {
    if (!editSettings) return;
    const current = editSettings.allowed_job_types;
    const newTypes = current.includes(jobType)
      ? current.filter((t) => t !== jobType)
      : [...current, jobType];
    setEditSettings({ ...editSettings, allowed_job_types: newTypes });
  };

  if (loading) {
    return (
      <div className="flex items-center justify-center h-64">
        <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-blue-500"></div>
        <span className="ml-3 text-gray-400">Loading GPU information...</span>
      </div>
    );
  }

  return (
    <div className="p-4 bg-gray-900 text-white min-h-screen">
      <div className="max-w-6xl mx-auto">
        {/* Header */}
        <div className="flex items-center justify-between mb-6">
          <h1 className="text-2xl font-bold">GPU Compute</h1>
          <div className="flex items-center space-x-2">
            {settings?.enabled ? (
              <span className="px-2 py-1 bg-green-500/20 text-green-400 rounded text-sm">
                Enabled
              </span>
            ) : (
              <span className="px-2 py-1 bg-gray-500/20 text-gray-400 rounded text-sm">
                Disabled
              </span>
            )}
          </div>
        </div>

        {/* Error Banner */}
        {error && (
          <div className="mb-4 p-3 bg-red-500/20 border border-red-500 rounded text-red-400">
            {error}
            <button
              onClick={() => setError(null)}
              className="ml-4 text-red-300 hover:text-red-100"
            >
              Dismiss
            </button>
          </div>
        )}

        {/* Tab Navigation */}
        <div className="flex border-b border-gray-700 mb-6">
          {(['devices', 'settings', 'jobs', 'stats'] as const).map((tab) => (
            <button
              key={tab}
              onClick={() => setActiveTab(tab)}
              className={`px-4 py-2 font-medium capitalize ${
                activeTab === tab
                  ? 'text-blue-400 border-b-2 border-blue-400'
                  : 'text-gray-400 hover:text-gray-200'
              }`}
            >
              {tab}
            </button>
          ))}
        </div>

        {/* Devices Tab */}
        {activeTab === 'devices' && (
          <div>
            <div className="flex justify-between items-center mb-4">
              <h2 className="text-lg font-semibold">Detected Devices</h2>
              <button
                onClick={handleRefreshDevices}
                className="px-3 py-1 bg-gray-700 hover:bg-gray-600 rounded text-sm"
              >
                Refresh
              </button>
            </div>
            <div className="grid gap-4">
              {devices.map((device) => (
                <div
                  key={device.id}
                  className="p-4 bg-gray-800 rounded-lg border border-gray-700"
                >
                  <div className="flex justify-between items-start mb-3">
                    <div>
                      <h3 className="font-semibold text-lg">{device.name}</h3>
                      <p className="text-sm text-gray-400">
                        {device.vendor} | {device.backend} | {device.compute_capability}
                      </p>
                    </div>
                    <span
                      className={`px-2 py-1 rounded text-xs ${
                        device.in_use
                          ? 'bg-yellow-500/20 text-yellow-400'
                          : 'bg-green-500/20 text-green-400'
                      }`}
                    >
                      {device.in_use ? 'In Use' : 'Available'}
                    </span>
                  </div>
                  <div className="grid grid-cols-2 md:grid-cols-4 gap-4 text-sm">
                    <div>
                      <span className="text-gray-500">Total Memory</span>
                      <p className="font-medium">{formatBytes(device.total_memory)}</p>
                    </div>
                    <div>
                      <span className="text-gray-500">Available</span>
                      <p className="font-medium">{formatBytes(device.available_memory)}</p>
                    </div>
                    <div>
                      <span className="text-gray-500">Utilization</span>
                      <p className="font-medium">{device.utilization}%</p>
                    </div>
                    <div>
                      <span className="text-gray-500">Temperature</span>
                      <p className="font-medium">
                        {device.temperature !== null ? `${device.temperature}°C` : 'N/A'}
                      </p>
                    </div>
                  </div>
                  {/* Memory bar */}
                  <div className="mt-3">
                    <div className="h-2 bg-gray-700 rounded-full overflow-hidden">
                      <div
                        className="h-full bg-blue-500 rounded-full"
                        style={{
                          width: `${((device.total_memory - device.available_memory) / device.total_memory) * 100}%`,
                        }}
                      ></div>
                    </div>
                  </div>
                </div>
              ))}
              {devices.length === 0 && (
                <p className="text-gray-500 text-center py-8">No GPU devices detected</p>
              )}
            </div>
          </div>
        )}

        {/* Settings Tab */}
        {activeTab === 'settings' && editSettings && (
          <div className="space-y-6">
            {/* Enable/Disable */}
            <div className="p-4 bg-gray-800 rounded-lg">
              <div className="flex items-center justify-between">
                <div>
                  <h3 className="font-semibold">GPU Compute Provider</h3>
                  <p className="text-sm text-gray-400">
                    Enable to contribute GPU resources to the network and earn tokens
                  </p>
                </div>
                <label className="relative inline-flex items-center cursor-pointer">
                  <input
                    type="checkbox"
                    checked={editSettings.enabled}
                    onChange={(e) =>
                      setEditSettings({ ...editSettings, enabled: e.target.checked })
                    }
                    className="sr-only peer"
                  />
                  <div className="w-11 h-6 bg-gray-600 peer-focus:ring-2 peer-focus:ring-blue-500 rounded-full peer peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:rounded-full after:h-5 after:w-5 after:transition-all peer-checked:bg-blue-500"></div>
                </label>
              </div>
            </div>

            {/* Allocation Percentage */}
            <div className="p-4 bg-gray-800 rounded-lg">
              <h3 className="font-semibold mb-3">Resource Allocation</h3>
              <div className="space-y-4">
                <div>
                  <label className="text-sm text-gray-400">
                    GPU Allocation: {editSettings.allocation_percentage}%
                  </label>
                  <input
                    type="range"
                    min="10"
                    max="100"
                    step="5"
                    value={editSettings.allocation_percentage}
                    onChange={(e) =>
                      setEditSettings({
                        ...editSettings,
                        allocation_percentage: parseInt(e.target.value),
                      })
                    }
                    className="w-full h-2 bg-gray-700 rounded-lg appearance-none cursor-pointer"
                  />
                  <div className="flex justify-between text-xs text-gray-500">
                    <span>10%</span>
                    <span>50%</span>
                    <span>100%</span>
                  </div>
                </div>

                <div>
                  <label className="text-sm text-gray-400">Max Concurrent Jobs</label>
                  <select
                    value={editSettings.max_concurrent_jobs}
                    onChange={(e) =>
                      setEditSettings({
                        ...editSettings,
                        max_concurrent_jobs: parseInt(e.target.value),
                      })
                    }
                    className="w-full mt-1 p-2 bg-gray-700 border border-gray-600 rounded"
                  >
                    {[1, 2, 3, 4, 5].map((n) => (
                      <option key={n} value={n}>
                        {n} job{n > 1 ? 's' : ''}
                      </option>
                    ))}
                  </select>
                </div>
              </div>
            </div>

            {/* Allowed Job Types */}
            <div className="p-4 bg-gray-800 rounded-lg">
              <h3 className="font-semibold mb-3">Allowed Job Types</h3>
              <div className="grid grid-cols-2 md:grid-cols-3 gap-2">
                {(Object.keys(JOB_TYPE_LABELS) as ComputeJobType[]).map((jobType) => (
                  <label
                    key={jobType}
                    className="flex items-center space-x-2 p-2 bg-gray-700 rounded cursor-pointer hover:bg-gray-600"
                  >
                    <input
                      type="checkbox"
                      checked={editSettings.allowed_job_types.includes(jobType)}
                      onChange={() => toggleJobType(jobType)}
                      className="rounded border-gray-500 text-blue-500 focus:ring-blue-500"
                    />
                    <span className="text-sm">{JOB_TYPE_LABELS[jobType]}</span>
                  </label>
                ))}
              </div>
            </div>

            {/* Payment Threshold */}
            <div className="p-4 bg-gray-800 rounded-lg">
              <h3 className="font-semibold mb-3">Payment Settings</h3>
              <div>
                <label className="text-sm text-gray-400">
                  Minimum Payment Threshold (tokens)
                </label>
                <input
                  type="number"
                  min="0"
                  value={editSettings.min_payment_threshold}
                  onChange={(e) =>
                    setEditSettings({
                      ...editSettings,
                      min_payment_threshold: parseInt(e.target.value) || 0,
                    })
                  }
                  className="w-full mt-1 p-2 bg-gray-700 border border-gray-600 rounded"
                />
                <p className="text-xs text-gray-500 mt-1">
                  Only accept jobs that pay at least this amount. Set to 0 to accept all jobs.
                </p>
              </div>
            </div>

            {/* Save Button */}
            <div className="flex justify-end">
              <button
                onClick={handleSaveSettings}
                disabled={saving}
                className={`px-6 py-2 rounded font-medium ${
                  saving
                    ? 'bg-gray-600 cursor-not-allowed'
                    : 'bg-blue-500 hover:bg-blue-600'
                }`}
              >
                {saving ? 'Saving...' : 'Save Settings'}
              </button>
            </div>
          </div>
        )}

        {/* Jobs Tab */}
        {activeTab === 'jobs' && (
          <div>
            <h2 className="text-lg font-semibold mb-4">
              Compute Jobs ({jobs.length} total)
            </h2>
            {jobs.length > 0 ? (
              <div className="space-y-3">
                {jobs.map((job) => (
                  <div
                    key={job.id}
                    className="p-4 bg-gray-800 rounded-lg border border-gray-700"
                  >
                    <div className="flex justify-between items-start mb-2">
                      <div>
                        <span className="font-mono text-sm text-gray-400">
                          {job.id.substring(0, 8)}...
                        </span>
                        <h3 className="font-medium">{JOB_TYPE_LABELS[job.job_type]}</h3>
                      </div>
                      <span className={`font-medium ${getJobStatusColor(job.status)}`}>
                        {getJobStatusLabel(job.status)}
                      </span>
                    </div>
                    <div className="grid grid-cols-2 md:grid-cols-4 gap-4 text-sm text-gray-400">
                      <div>
                        <span className="text-gray-500">Model</span>
                        <p>{job.model_id}</p>
                      </div>
                      <div>
                        <span className="text-gray-500">Memory</span>
                        <p>{formatBytes(job.memory_required)}</p>
                      </div>
                      <div>
                        <span className="text-gray-500">Est. Time</span>
                        <p>{formatDuration(job.estimated_time)}</p>
                      </div>
                      <div>
                        <span className="text-gray-500">Payment</span>
                        <p>{job.max_payment} tokens</p>
                      </div>
                    </div>
                    {('Running' in job.status || 'Queued' in job.status) && (
                      <div className="mt-3 flex justify-end">
                        <button
                          onClick={() => handleCancelJob(job.id)}
                          className="px-3 py-1 bg-red-500/20 text-red-400 hover:bg-red-500/30 rounded text-sm"
                        >
                          Cancel
                        </button>
                      </div>
                    )}
                  </div>
                ))}
              </div>
            ) : (
              <p className="text-gray-500 text-center py-8">No compute jobs in queue</p>
            )}
          </div>
        )}

        {/* Stats Tab */}
        {activeTab === 'stats' && stats && providerStatus && (
          <div className="space-y-6">
            {/* Provider Status */}
            <div className="p-4 bg-gray-800 rounded-lg">
              <h3 className="font-semibold mb-3">Provider Status</h3>
              <div className="grid grid-cols-2 md:grid-cols-4 gap-4">
                <div>
                  <span className="text-gray-500 text-sm">Status</span>
                  <p
                    className={`font-medium ${
                      providerStatus.is_registered ? 'text-green-400' : 'text-gray-400'
                    }`}
                  >
                    {providerStatus.is_registered ? 'Registered' : 'Not Registered'}
                  </p>
                </div>
                <div>
                  <span className="text-gray-500 text-sm">Reputation</span>
                  <p className="font-medium">{providerStatus.reputation}/100</p>
                </div>
                <div>
                  <span className="text-gray-500 text-sm">Stake</span>
                  <p className="font-medium">{providerStatus.stake} tokens</p>
                </div>
                <div>
                  <span className="text-gray-500 text-sm">Active Jobs</span>
                  <p className="font-medium">{providerStatus.active_jobs.length}</p>
                </div>
              </div>
            </div>

            {/* Session Stats */}
            <div className="p-4 bg-gray-800 rounded-lg">
              <h3 className="font-semibold mb-3">Session Statistics</h3>
              <div className="grid grid-cols-2 md:grid-cols-4 gap-4">
                <div>
                  <span className="text-gray-500 text-sm">Jobs Completed</span>
                  <p className="text-2xl font-bold text-green-400">{stats.jobs_completed}</p>
                </div>
                <div>
                  <span className="text-gray-500 text-sm">Jobs Failed</span>
                  <p className="text-2xl font-bold text-red-400">{stats.jobs_failed}</p>
                </div>
                <div>
                  <span className="text-gray-500 text-sm">Tokens Earned</span>
                  <p className="text-2xl font-bold text-blue-400">{stats.tokens_earned}</p>
                </div>
                <div>
                  <span className="text-gray-500 text-sm">Queue Depth</span>
                  <p className="text-2xl font-bold">{stats.queue_depth}</p>
                </div>
              </div>
            </div>

            {/* Performance Stats */}
            <div className="p-4 bg-gray-800 rounded-lg">
              <h3 className="font-semibold mb-3">Performance</h3>
              <div className="grid grid-cols-2 md:grid-cols-3 gap-4">
                <div>
                  <span className="text-gray-500 text-sm">Total Compute Time</span>
                  <p className="font-medium">{formatDuration(stats.total_compute_time)}</p>
                </div>
                <div>
                  <span className="text-gray-500 text-sm">Avg Job Duration</span>
                  <p className="font-medium">{formatDuration(Math.round(stats.avg_job_duration))}</p>
                </div>
                <div>
                  <span className="text-gray-500 text-sm">Memory Usage</span>
                  <p className="font-medium">{formatBytes(stats.current_memory_usage)}</p>
                </div>
              </div>
            </div>
          </div>
        )}
      </div>
    </div>
  );
}

export default GPUSettings;
