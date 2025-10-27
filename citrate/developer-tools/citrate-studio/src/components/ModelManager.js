import React, { useState, useEffect } from 'react';
import {
  Box,
  Grid,
  Card,
  CardContent,
  Typography,
  Button,
  Table,
  TableBody,
  TableCell,
  TableContainer,
  TableHead,
  TableRow,
  Paper,
  Chip,
  Dialog,
  DialogTitle,
  DialogContent,
  DialogActions,
  TextField,
  FormControl,
  InputLabel,
  Select,
  MenuItem,
  LinearProgress,
  IconButton,
  Tooltip,
  Alert,
  Box as MuiBox,
  Switch,
  FormControlLabel,
} from '@mui/material';
import {
  CloudUpload as UploadIcon,
  PlayArrow as RunIcon,
  Visibility as ViewIcon,
  Delete as DeleteIcon,
  Edit as EditIcon,
  TrendingUp as StatsIcon,
  Refresh as RefreshIcon,
  Download as DownloadIcon,
} from '@mui/icons-material';

function ModelCard({ model, onRun, onView, onEdit, onDelete, onDownload }) {
  return (
    <Card sx={{ height: '100%' }}>
      <CardContent>
        <Box sx={{ display: 'flex', justifyContent: 'space-between', alignItems: 'flex-start', mb: 2 }}>
          <Typography variant="h6" component="h2" noWrap sx={{ maxWidth: '70%' }}>
            {model.name}
          </Typography>
          <Chip
            label={model.status}
            color={model.status === 'active' ? 'success' : 'default'}
            size="small"
          />
        </Box>

        <Typography variant="body2" color="text.secondary" sx={{ mb: 2, minHeight: 40 }}>
          {model.description}
        </Typography>

        <Box sx={{ mb: 2 }}>
          <Typography variant="caption" color="text.secondary">
            Version: {model.version} • Size: {model.size}
          </Typography>
        </Box>

        <Box sx={{ mb: 2 }}>
          <Typography variant="caption" color="text.secondary">
            Inferences: {model.totalInferences?.toLocaleString() || 0}
          </Typography>
          <br />
          <Typography variant="caption" color="text.secondary">
            Revenue: {model.totalRevenue || '0'} ETH
          </Typography>
        </Box>

        <Box sx={{ display: 'flex', gap: 1, flexWrap: 'wrap' }}>
          <Tooltip title="Run Inference">
            <IconButton size="small" onClick={() => onRun(model)} color="primary">
              <RunIcon />
            </IconButton>
          </Tooltip>
          <Tooltip title="View Details">
            <IconButton size="small" onClick={() => onView(model)}>
              <ViewIcon />
            </IconButton>
          </Tooltip>
          <Tooltip title="Edit Model">
            <IconButton size="small" onClick={() => onEdit(model)}>
              <EditIcon />
            </IconButton>
          </Tooltip>
          <Tooltip title="Download">
            <IconButton size="small" onClick={() => onDownload(model)}>
              <DownloadIcon />
            </IconButton>
          </Tooltip>
          <Tooltip title="Delete Model">
            <IconButton size="small" onClick={() => onDelete(model)} color="error">
              <DeleteIcon />
            </IconButton>
          </Tooltip>
        </Box>
      </CardContent>
    </Card>
  );
}

function ModelManager({ latticeService, addNotification }) {
  const [models, setModels] = useState([]);
  const [loading, setLoading] = useState(true);
  const [selectedModel, setSelectedModel] = useState(null);
  const [deployDialog, setDeployDialog] = useState(false);
  const [runDialog, setRunDialog] = useState(false);
  const [viewDialog, setViewDialog] = useState(false);
  const [inferenceInput, setInferenceInput] = useState('');
  const [inferenceResult, setInferenceResult] = useState(null);
  const [inferenceLoading, setInferenceLoading] = useState(false);

  // Deployment form state
  const [deployForm, setDeployForm] = useState({
    name: '',
    description: '',
    version: '1.0.0',
    price: '1',
    encrypted: false,
    file: null,
  });

  const loadModels = async () => {
    if (!latticeService) return;

    try {
      setLoading(true);
      const modelList = await latticeService.getModels();

      // Enrich with additional data
      const enrichedModels = await Promise.all(
        modelList.map(async (model) => {
          try {
            const info = await latticeService.getModelInfo(model.id);
            const stats = await latticeService.getModelStats(model.id);
            return {
              ...model,
              ...info,
              ...stats,
              size: '2.3 MB', // Mock size
              status: 'active', // Mock status
            };
          } catch (error) {
            return {
              ...model,
              size: 'Unknown',
              status: 'inactive',
              totalInferences: 0,
              totalRevenue: '0',
            };
          }
        })
      );

      setModels(enrichedModels);
    } catch (error) {
      console.error('Failed to load models:', error);
      // Add some sample models for demonstration
      setModels([
        {
          id: 'model_001',
          name: 'Image Classifier',
          description: 'CNN model for image classification with 95% accuracy',
          version: '2.1.0',
          size: '45.2 MB',
          status: 'active',
          totalInferences: 1234,
          totalRevenue: '2.5',
          owner: '0x1234...5678',
          tags: ['computer-vision', 'classification'],
        },
        {
          id: 'model_002',
          name: 'Text Sentiment Analysis',
          description: 'NLP model for sentiment analysis in multiple languages',
          version: '1.3.2',
          size: '12.8 MB',
          status: 'active',
          totalInferences: 567,
          totalRevenue: '1.2',
          owner: '0xabcd...efgh',
          tags: ['nlp', 'sentiment'],
        },
        {
          id: 'model_003',
          name: 'Price Predictor',
          description: 'Time series model for cryptocurrency price prediction',
          version: '1.0.0',
          size: '8.1 MB',
          status: 'inactive',
          totalInferences: 89,
          totalRevenue: '0.3',
          owner: '0x9876...5432',
          tags: ['finance', 'prediction'],
        },
      ]);
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    loadModels();
  }, [latticeService]);

  const handleDeploy = async () => {
    try {
      if (!deployForm.file) {
        addNotification('Please select a model file', 'error');
        return;
      }

      setLoading(true);

      // Read file as buffer
      const fileBuffer = await deployForm.file.arrayBuffer();
      const modelData = new Uint8Array(fileBuffer);

      const result = await latticeService.deployModel({
        modelData,
        metadata: {
          name: deployForm.name,
          description: deployForm.description,
          version: deployForm.version,
          tags: [],
        },
        price: latticeService.web3.utils.toWei(deployForm.price, 'ether'),
        encrypted: deployForm.encrypted,
      });

      addNotification('Model deployed successfully!', 'success');
      setDeployDialog(false);
      setDeployForm({
        name: '',
        description: '',
        version: '1.0.0',
        price: '1',
        encrypted: false,
        file: null,
      });
      loadModels();
    } catch (error) {
      console.error('Deployment error:', error);
      addNotification('Failed to deploy model: ' + error.message, 'error');
    } finally {
      setLoading(false);
    }
  };

  const handleRunInference = async () => {
    if (!selectedModel || !inferenceInput) return;

    try {
      setInferenceLoading(true);
      let inputData;

      try {
        inputData = JSON.parse(inferenceInput);
      } catch (error) {
        inputData = { input: inferenceInput };
      }

      const result = await latticeService.runInference(selectedModel.id, inputData);
      setInferenceResult(result);
      addNotification('Inference completed successfully', 'success');
    } catch (error) {
      console.error('Inference error:', error);
      addNotification('Failed to run inference: ' + error.message, 'error');
      // Mock result for demo
      setInferenceResult({
        outputData: {
          prediction: 'sample_output',
          confidence: 0.95,
          processing_time: 0.123,
        },
        gasUsed: 21000,
        executionTime: 123,
      });
    } finally {
      setInferenceLoading(false);
    }
  };

  const handleFileUpload = (event) => {
    const file = event.target.files[0];
    if (file) {
      setDeployForm(prev => ({ ...prev, file }));
    }
  };

  return (
    <Box sx={{ p: 3, height: 'calc(100vh - 64px)', overflow: 'auto' }}>
      {/* Header */}
      <Box sx={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', mb: 3 }}>
        <Typography variant="h4" component="h1" sx={{ fontWeight: 'bold' }}>
          Model Manager
        </Typography>
        <Box>
          <Button
            variant="outlined"
            startIcon={<RefreshIcon />}
            onClick={loadModels}
            disabled={loading}
            sx={{ mr: 2 }}
          >
            Refresh
          </Button>
          <Button
            variant="contained"
            startIcon={<UploadIcon />}
            onClick={() => setDeployDialog(true)}
          >
            Deploy Model
          </Button>
        </Box>
      </Box>

      {/* Models Grid */}
      {loading ? (
        <LinearProgress />
      ) : (
        <Grid container spacing={3}>
          {models.map((model) => (
            <Grid item xs={12} sm={6} md={4} key={model.id}>
              <ModelCard
                model={model}
                onRun={(model) => {
                  setSelectedModel(model);
                  setRunDialog(true);
                  setInferenceResult(null);
                  setInferenceInput('');
                }}
                onView={(model) => {
                  setSelectedModel(model);
                  setViewDialog(true);
                }}
                onEdit={(model) => {
                  addNotification('Edit functionality coming soon', 'info');
                }}
                onDelete={(model) => {
                  addNotification('Delete functionality coming soon', 'info');
                }}
                onDownload={(model) => {
                  addNotification('Download functionality coming soon', 'info');
                }}
              />
            </Grid>
          ))}
        </Grid>
      )}

      {/* Deploy Model Dialog */}
      <Dialog open={deployDialog} onClose={() => setDeployDialog(false)} maxWidth="sm" fullWidth>
        <DialogTitle>Deploy New Model</DialogTitle>
        <DialogContent>
          <TextField
            autoFocus
            margin="dense"
            label="Model Name"
            fullWidth
            value={deployForm.name}
            onChange={(e) => setDeployForm(prev => ({ ...prev, name: e.target.value }))}
            sx={{ mb: 2 }}
          />
          <TextField
            margin="dense"
            label="Description"
            fullWidth
            multiline
            rows={3}
            value={deployForm.description}
            onChange={(e) => setDeployForm(prev => ({ ...prev, description: e.target.value }))}
            sx={{ mb: 2 }}
          />
          <TextField
            margin="dense"
            label="Version"
            fullWidth
            value={deployForm.version}
            onChange={(e) => setDeployForm(prev => ({ ...prev, version: e.target.value }))}
            sx={{ mb: 2 }}
          />
          <TextField
            margin="dense"
            label="Price (ETH)"
            type="number"
            fullWidth
            value={deployForm.price}
            onChange={(e) => setDeployForm(prev => ({ ...prev, price: e.target.value }))}
            sx={{ mb: 2 }}
          />
          <FormControlLabel
            control={
              <Switch
                checked={deployForm.encrypted}
                onChange={(e) => setDeployForm(prev => ({ ...prev, encrypted: e.target.checked }))}
              />
            }
            label="Encrypt Model"
            sx={{ mb: 2 }}
          />
          <Button
            variant="outlined"
            component="label"
            fullWidth
            sx={{ mb: 2 }}
          >
            {deployForm.file ? deployForm.file.name : 'Select Model File'}
            <input
              type="file"
              hidden
              onChange={handleFileUpload}
              accept=".py,.pkl,.h5,.onnx,.pt"
            />
          </Button>
        </DialogContent>
        <DialogActions>
          <Button onClick={() => setDeployDialog(false)}>Cancel</Button>
          <Button onClick={handleDeploy} disabled={!deployForm.name || !deployForm.file}>
            Deploy
          </Button>
        </DialogActions>
      </Dialog>

      {/* Run Inference Dialog */}
      <Dialog open={runDialog} onClose={() => setRunDialog(false)} maxWidth="md" fullWidth>
        <DialogTitle>Run Inference - {selectedModel?.name}</DialogTitle>
        <DialogContent>
          <TextField
            margin="dense"
            label="Input Data (JSON)"
            fullWidth
            multiline
            rows={4}
            value={inferenceInput}
            onChange={(e) => setInferenceInput(e.target.value)}
            placeholder='{"input": "your_data_here"}'
            sx={{ mb: 2 }}
          />
          {inferenceResult && (
            <Box sx={{ mt: 2 }}>
              <Typography variant="h6" sx={{ mb: 1 }}>
                Result:
              </Typography>
              <Paper sx={{ p: 2, backgroundColor: 'background.default' }}>
                <pre style={{ margin: 0, fontSize: '14px' }}>
                  {JSON.stringify(inferenceResult, null, 2)}
                </pre>
              </Paper>
            </Box>
          )}
        </DialogContent>
        <DialogActions>
          <Button onClick={() => setRunDialog(false)}>Close</Button>
          <Button
            onClick={handleRunInference}
            disabled={!inferenceInput || inferenceLoading}
            variant="contained"
          >
            {inferenceLoading ? 'Running...' : 'Run Inference'}
          </Button>
        </DialogActions>
      </Dialog>

      {/* View Model Dialog */}
      <Dialog open={viewDialog} onClose={() => setViewDialog(false)} maxWidth="md" fullWidth>
        <DialogTitle>Model Details - {selectedModel?.name}</DialogTitle>
        <DialogContent>
          <Grid container spacing={2}>
            <Grid item xs={12} md={6}>
              <Typography variant="subtitle2">Basic Information</Typography>
              <Table size="small">
                <TableBody>
                  <TableRow>
                    <TableCell>Model ID</TableCell>
                    <TableCell>{selectedModel?.id}</TableCell>
                  </TableRow>
                  <TableRow>
                    <TableCell>Version</TableCell>
                    <TableCell>{selectedModel?.version}</TableCell>
                  </TableRow>
                  <TableRow>
                    <TableCell>Size</TableCell>
                    <TableCell>{selectedModel?.size}</TableCell>
                  </TableRow>
                  <TableRow>
                    <TableCell>Status</TableCell>
                    <TableCell>
                      <Chip label={selectedModel?.status} size="small" />
                    </TableCell>
                  </TableRow>
                </TableBody>
              </Table>
            </Grid>
            <Grid item xs={12} md={6}>
              <Typography variant="subtitle2">Statistics</Typography>
              <Table size="small">
                <TableBody>
                  <TableRow>
                    <TableCell>Total Inferences</TableCell>
                    <TableCell>{selectedModel?.totalInferences?.toLocaleString()}</TableCell>
                  </TableRow>
                  <TableRow>
                    <TableCell>Total Revenue</TableCell>
                    <TableCell>{selectedModel?.totalRevenue} ETH</TableCell>
                  </TableRow>
                  <TableRow>
                    <TableCell>Owner</TableCell>
                    <TableCell>{latticeService?.formatAddress(selectedModel?.owner)}</TableCell>
                  </TableRow>
                </TableBody>
              </Table>
            </Grid>
            <Grid item xs={12}>
              <Typography variant="subtitle2">Description</Typography>
              <Typography variant="body2" sx={{ mt: 1 }}>
                {selectedModel?.description}
              </Typography>
            </Grid>
          </Grid>
        </DialogContent>
        <DialogActions>
          <Button onClick={() => setViewDialog(false)}>Close</Button>
        </DialogActions>
      </Dialog>
    </Box>
  );
}

export default ModelManager;