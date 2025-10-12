import React, { useState, useEffect } from 'react';
import {
  Box,
  Grid,
  Card,
  CardContent,
  Typography,
  AppBar,
  Toolbar,
  Tabs,
  Tab,
  Alert,
  CircularProgress,
} from '@mui/material';
import {
  Timeline,
  TrendingUp,
  Memory,
  NetworkCheck,
  BugReport,
} from '@mui/icons-material';
import { LineChart, Line, XAxis, YAxis, CartesianGrid, ResponsiveContainer } from 'recharts';
import Web3 from 'web3';

function TransactionMonitor() {
  const [transactions, setTransactions] = useState([]);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    // Simulate real-time transaction monitoring
    const interval = setInterval(() => {
      const newTx = {
        hash: `0x${Math.random().toString(16).slice(2)}`,
        from: `0x${Math.random().toString(16).slice(2, 42)}`,
        to: `0x${Math.random().toString(16).slice(2, 42)}`,
        value: Math.random() * 10,
        gasUsed: Math.floor(Math.random() * 100000),
        timestamp: new Date(),
        status: Math.random() > 0.1 ? 'success' : 'failed'
      };

      setTransactions(prev => [newTx, ...prev.slice(0, 9)]);
      setLoading(false);
    }, 2000);

    return () => clearInterval(interval);
  }, []);

  if (loading) {
    return <CircularProgress />;
  }

  return (
    <Card>
      <CardContent>
        <Typography variant="h6" gutterBottom>
          Recent Transactions
        </Typography>
        {transactions.map((tx, index) => (
          <Box key={index} sx={{ mb: 1, p: 1, border: 1, borderColor: 'divider', borderRadius: 1 }}>
            <Typography variant="body2" sx={{ fontFamily: 'monospace' }}>
              {tx.hash.slice(0, 20)}...
            </Typography>
            <Typography variant="caption" color={tx.status === 'success' ? 'success.main' : 'error.main'}>
              {tx.status.toUpperCase()} | Gas: {tx.gasUsed.toLocaleString()} | {tx.timestamp.toLocaleTimeString()}
            </Typography>
          </Box>
        ))}
      </CardContent>
    </Card>
  );
}

function NetworkMetrics() {
  const [metrics, setMetrics] = useState({
    blockHeight: 12345,
    tps: 127,
    peers: 15,
    mempool: 234
  });

  const [chartData, setChartData] = useState([]);

  useEffect(() => {
    const interval = setInterval(() => {
      const newData = {
        time: new Date().toLocaleTimeString(),
        tps: Math.floor(Math.random() * 200) + 50,
        blocks: metrics.blockHeight + Math.floor(Math.random() * 3),
        mempool: Math.floor(Math.random() * 500) + 100
      };

      setChartData(prev => [...prev.slice(-19), newData]);

      setMetrics({
        blockHeight: newData.blocks,
        tps: newData.tps,
        peers: 15 + Math.floor(Math.random() * 10),
        mempool: newData.mempool
      });
    }, 3000);

    return () => clearInterval(interval);
  }, [metrics.blockHeight]);

  return (
    <Grid container spacing={2}>
      <Grid item xs={12} md={6}>
        <Card>
          <CardContent>
            <Typography variant="h6" gutterBottom>
              Network Statistics
            </Typography>
            <Grid container spacing={2}>
              <Grid item xs={6}>
                <Box textAlign="center">
                  <Typography variant="h4" color="primary">
                    {metrics.blockHeight.toLocaleString()}
                  </Typography>
                  <Typography variant="caption">Block Height</Typography>
                </Box>
              </Grid>
              <Grid item xs={6}>
                <Box textAlign="center">
                  <Typography variant="h4" color="secondary">
                    {metrics.tps}
                  </Typography>
                  <Typography variant="caption">TPS</Typography>
                </Box>
              </Grid>
              <Grid item xs={6}>
                <Box textAlign="center">
                  <Typography variant="h4" color="success.main">
                    {metrics.peers}
                  </Typography>
                  <Typography variant="caption">Peers</Typography>
                </Box>
              </Grid>
              <Grid item xs={6}>
                <Box textAlign="center">
                  <Typography variant="h4" color="warning.main">
                    {metrics.mempool}
                  </Typography>
                  <Typography variant="caption">Mempool</Typography>
                </Box>
              </Grid>
            </Grid>
          </CardContent>
        </Card>
      </Grid>

      <Grid item xs={12} md={6}>
        <Card>
          <CardContent>
            <Typography variant="h6" gutterBottom>
              TPS Over Time
            </Typography>
            <ResponsiveContainer width="100%" height={200}>
              <LineChart data={chartData}>
                <CartesianGrid strokeDasharray="3 3" />
                <XAxis dataKey="time" />
                <YAxis />
                <Line type="monotone" dataKey="tps" stroke="#8884d8" strokeWidth={2} />
              </LineChart>
            </ResponsiveContainer>
          </CardContent>
        </Card>
      </Grid>
    </Grid>
  );
}

function ErrorAnalyzer() {
  const [errors, setErrors] = useState([
    {
      type: 'Gas Estimation Failed',
      count: 23,
      lastSeen: '2 minutes ago',
      severity: 'high'
    },
    {
      type: 'Invalid Signature',
      count: 8,
      lastSeen: '5 minutes ago',
      severity: 'medium'
    },
    {
      type: 'Nonce Too Low',
      count: 45,
      lastSeen: '1 minute ago',
      severity: 'low'
    }
  ]);

  return (
    <Card>
      <CardContent>
        <Typography variant="h6" gutterBottom>
          Error Analysis
        </Typography>
        {errors.map((error, index) => (
          <Alert
            key={index}
            severity={error.severity === 'high' ? 'error' : error.severity === 'medium' ? 'warning' : 'info'}
            sx={{ mb: 1 }}
          >
            <Typography variant="body2">
              <strong>{error.type}</strong> - {error.count} occurrences
            </Typography>
            <Typography variant="caption">
              Last seen: {error.lastSeen}
            </Typography>
          </Alert>
        ))}
      </CardContent>
    </Card>
  );
}

function App() {
  const [activeTab, setActiveTab] = useState(0);

  return (
    <Box sx={{ bgcolor: '#0f172a', minHeight: '100vh', color: 'white' }}>
      <AppBar position="static" sx={{ bgcolor: '#1e293b' }}>
        <Toolbar>
          <BugReport sx={{ mr: 2 }} />
          <Typography variant="h6" component="div" sx={{ flexGrow: 1 }}>
            Lattice Debug Dashboard
          </Typography>
          <Typography variant="body2" sx={{ color: 'success.main' }}>
            ● Connected
          </Typography>
        </Toolbar>
      </AppBar>

      <Box sx={{ p: 3 }}>
        <Tabs
          value={activeTab}
          onChange={(e, newValue) => setActiveTab(newValue)}
          sx={{ mb: 3 }}
        >
          <Tab icon={<NetworkCheck />} label="Network" />
          <Tab icon={<Timeline />} label="Transactions" />
          <Tab icon={<BugReport />} label="Errors" />
          <Tab icon={<Memory />} label="Performance" />
        </Tabs>

        {activeTab === 0 && <NetworkMetrics />}

        {activeTab === 1 && (
          <Grid container spacing={3}>
            <Grid item xs={12}>
              <TransactionMonitor />
            </Grid>
          </Grid>
        )}

        {activeTab === 2 && (
          <Grid container spacing={3}>
            <Grid item xs={12} md={8}>
              <ErrorAnalyzer />
            </Grid>
          </Grid>
        )}

        {activeTab === 3 && (
          <Grid container spacing={3}>
            <Grid item xs={12}>
              <Card>
                <CardContent>
                  <Typography variant="h6">Performance Metrics</Typography>
                  <Typography>Block processing time, gas usage optimization, and node performance analytics.</Typography>
                </CardContent>
              </Card>
            </Grid>
          </Grid>
        )}
      </Box>
    </Box>
  );
}

export default App;