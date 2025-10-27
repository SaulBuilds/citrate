import React, { useState } from 'react';
import {
  Box,
  Typography,
  TextField,
  Button,
  Card,
  CardContent,
  Paper,
  Grid,
} from '@mui/material';
import { Search as SearchIcon } from '@mui/icons-material';

function Debugger({ latticeService, addNotification }) {
  const [txHash, setTxHash] = useState('');
  const [debugResult, setDebugResult] = useState(null);
  const [loading, setLoading] = useState(false);

  const handleDebug = async () => {
    if (!txHash) return;

    setLoading(true);
    try {
      const [transaction, receipt, trace] = await Promise.all([
        latticeService.getTransaction(txHash),
        latticeService.getTransactionReceipt(txHash),
        latticeService.getTransactionTrace(txHash),
      ]);

      setDebugResult({ transaction, receipt, trace });
      addNotification('Transaction debug completed', 'success');
    } catch (error) {
      addNotification('Failed to debug transaction: ' + error.message, 'error');
      // Mock result for demo
      setDebugResult({
        transaction: { hash: txHash, from: '0x123...', to: '0x456...', value: '1000000000000000000' },
        receipt: { status: '0x1', gasUsed: '21000' },
        trace: { calls: [] }
      });
    } finally {
      setLoading(false);
    }
  };

  return (
    <Box sx={{ p: 3 }}>
      <Typography variant="h4" sx={{ mb: 3 }}>Transaction Debugger</Typography>

      <Card sx={{ mb: 3 }}>
        <CardContent>
          <Box sx={{ display: 'flex', gap: 2, alignItems: 'center' }}>
            <TextField
              label="Transaction Hash"
              value={txHash}
              onChange={(e) => setTxHash(e.target.value)}
              placeholder="0x..."
              fullWidth
            />
            <Button
              variant="contained"
              startIcon={<SearchIcon />}
              onClick={handleDebug}
              disabled={loading || !txHash}
            >
              Debug
            </Button>
          </Box>
        </CardContent>
      </Card>

      {debugResult && (
        <Grid container spacing={3}>
          <Grid item xs={12} md={6}>
            <Card>
              <CardContent>
                <Typography variant="h6" sx={{ mb: 2 }}>Transaction Details</Typography>
                <Paper sx={{ p: 2, backgroundColor: 'background.default' }}>
                  <pre style={{ margin: 0, fontSize: '14px' }}>
                    {JSON.stringify(debugResult.transaction, null, 2)}
                  </pre>
                </Paper>
              </CardContent>
            </Card>
          </Grid>
          <Grid item xs={12} md={6}>
            <Card>
              <CardContent>
                <Typography variant="h6" sx={{ mb: 2 }}>Receipt</Typography>
                <Paper sx={{ p: 2, backgroundColor: 'background.default' }}>
                  <pre style={{ margin: 0, fontSize: '14px' }}>
                    {JSON.stringify(debugResult.receipt, null, 2)}
                  </pre>
                </Paper>
              </CardContent>
            </Card>
          </Grid>
        </Grid>
      )}
    </Box>
  );
}

export default Debugger;