import React, { useState } from 'react';
import {
  Box,
  Typography,
  Stepper,
  Step,
  StepLabel,
  Card,
  CardContent,
  Button,
  TextField,
  FormControl,
  InputLabel,
  Select,
  MenuItem,
  LinearProgress,
  Alert,
} from '@mui/material';
import { CloudUpload as DeployIcon } from '@mui/icons-material';

const steps = ['Configure', 'Deploy', 'Verify'];

function Deployment({ latticeService, addNotification }) {
  const [activeStep, setActiveStep] = useState(0);
  const [config, setConfig] = useState({
    environment: 'testnet',
    gasLimit: '1000000',
    gasPrice: '20',
  });
  const [deploying, setDeploying] = useState(false);
  const [deployResult, setDeployResult] = useState(null);

  const handleDeploy = async () => {
    setDeploying(true);
    try {
      // Simulate deployment
      await new Promise(resolve => setTimeout(resolve, 3000));
      setDeployResult({
        txHash: '0x123...abc',
        contractAddress: '0x456...def',
        gasUsed: '654321'
      });
      setActiveStep(2);
      addNotification('Deployment successful!', 'success');
    } catch (error) {
      addNotification('Deployment failed: ' + error.message, 'error');
    } finally {
      setDeploying(false);
    }
  };

  return (
    <Box sx={{ p: 3 }}>
      <Typography variant="h4" sx={{ mb: 3 }}>Deployment Center</Typography>

      <Stepper activeStep={activeStep} sx={{ mb: 4 }}>
        {steps.map((label) => (
          <Step key={label}>
            <StepLabel>{label}</StepLabel>
          </Step>
        ))}
      </Stepper>

      <Card>
        <CardContent>
          {activeStep === 0 && (
            <Box>
              <Typography variant="h6" sx={{ mb: 2 }}>Configure Deployment</Typography>
              <FormControl fullWidth sx={{ mb: 2 }}>
                <InputLabel>Environment</InputLabel>
                <Select
                  value={config.environment}
                  onChange={(e) => setConfig(prev => ({ ...prev, environment: e.target.value }))}
                >
                  <MenuItem value="testnet">Testnet</MenuItem>
                  <MenuItem value="mainnet">Mainnet</MenuItem>
                </Select>
              </FormControl>
              <TextField
                label="Gas Limit"
                value={config.gasLimit}
                onChange={(e) => setConfig(prev => ({ ...prev, gasLimit: e.target.value }))}
                fullWidth
                sx={{ mb: 2 }}
              />
              <TextField
                label="Gas Price (Gwei)"
                value={config.gasPrice}
                onChange={(e) => setConfig(prev => ({ ...prev, gasPrice: e.target.value }))}
                fullWidth
                sx={{ mb: 2 }}
              />
              <Button
                variant="contained"
                onClick={() => setActiveStep(1)}
                startIcon={<DeployIcon />}
              >
                Start Deployment
              </Button>
            </Box>
          )}

          {activeStep === 1 && (
            <Box>
              <Typography variant="h6" sx={{ mb: 2 }}>Deploying...</Typography>
              {deploying && <LinearProgress sx={{ mb: 2 }} />}
              <Button
                variant="contained"
                onClick={handleDeploy}
                disabled={deploying}
                fullWidth
              >
                {deploying ? 'Deploying...' : 'Deploy Now'}
              </Button>
            </Box>
          )}

          {activeStep === 2 && deployResult && (
            <Box>
              <Typography variant="h6" sx={{ mb: 2 }}>Deployment Complete</Typography>
              <Alert severity="success" sx={{ mb: 2 }}>
                Your model has been successfully deployed!
              </Alert>
              <Typography>Transaction Hash: {deployResult.txHash}</Typography>
              <Typography>Contract Address: {deployResult.contractAddress}</Typography>
              <Typography>Gas Used: {deployResult.gasUsed}</Typography>
            </Box>
          )}
        </CardContent>
      </Card>
    </Box>
  );
}

export default Deployment;