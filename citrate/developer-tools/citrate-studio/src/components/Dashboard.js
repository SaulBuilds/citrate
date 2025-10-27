import React, { useState, useEffect } from 'react';
import {
  Box,
  Grid,
  Card,
  CardContent,
  Typography,
  LinearProgress,
  Chip,
  Table,
  TableBody,
  TableCell,
  TableContainer,
  TableHead,
  TableRow,
  Paper,
  IconButton,
  Tooltip,
  Button,
  Divider,
} from '@mui/material';
import {
  TrendingUp as TrendingUpIcon,
  Storage as StorageIcon,
  Speed as SpeedIcon,
  People as PeopleIcon,
  Refresh as RefreshIcon,
  Launch as LaunchIcon,
  Timeline as TimelineIcon,
} from '@mui/icons-material';
import { Line } from 'react-chartjs-2';
import {
  Chart as ChartJS,
  CategoryScale,
  LinearScale,
  PointElement,
  LineElement,
  Title,
  Tooltip as ChartTooltip,
  Legend,
} from 'chart.js';

ChartJS.register(
  CategoryScale,
  LinearScale,
  PointElement,
  LineElement,
  Title,
  ChartTooltip,
  Legend
);

function MetricCard({ title, value, subtitle, icon, color = 'primary', progress }) {
  return (
    <Card sx={{ height: '100%', display: 'flex', flexDirection: 'column' }}>
      <CardContent sx={{ flexGrow: 1 }}>
        <Box sx={{ display: 'flex', alignItems: 'center', mb: 2 }}>
          <Box sx={{ color: `${color}.main`, mr: 1 }}>{icon}</Box>
          <Typography variant="h6" component="div">
            {title}
          </Typography>
        </Box>
        <Typography variant="h4" component="div" sx={{ mb: 1, fontWeight: 'bold' }}>
          {value}
        </Typography>
        {subtitle && (
          <Typography variant="body2" color="text.secondary">
            {subtitle}
          </Typography>
        )}
        {progress !== undefined && (
          <Box sx={{ mt: 2 }}>
            <LinearProgress
              variant="determinate"
              value={progress}
              sx={{ height: 8, borderRadius: 4 }}
            />
            <Typography variant="caption" color="text.secondary" sx={{ mt: 1 }}>
              {progress}% capacity
            </Typography>
          </Box>
        )}
      </CardContent>
    </Card>
  );
}

function Dashboard({ latticeService, connectionStatus, addNotification }) {
  const [networkStats, setNetworkStats] = useState(null);
  const [recentBlocks, setRecentBlocks] = useState([]);
  const [models, setModels] = useState([]);
  const [blockchainData, setBlockchainData] = useState({
    labels: [],
    datasets: [{
      label: 'Block Height',
      data: [],
      borderColor: 'rgb(99, 102, 241)',
      backgroundColor: 'rgba(99, 102, 241, 0.1)',
      tension: 0.1,
    }]
  });
  const [loading, setLoading] = useState(true);

  const loadDashboardData = async () => {
    if (!latticeService || connectionStatus !== 'connected') return;

    try {
      setLoading(true);

      // Load network statistics
      const chainInfo = await latticeService.getChainInfo();
      setNetworkStats(chainInfo);

      // Load recent blocks
      const blocks = [];
      for (let i = 0; i < 5; i++) {
        try {
          const block = await latticeService.getBlock(chainInfo.latestBlock - i);
          if (block) blocks.push(block);
        } catch (error) {
          console.warn(`Failed to load block ${chainInfo.latestBlock - i}`);
        }
      }
      setRecentBlocks(blocks);

      // Update blockchain chart data
      const newLabels = blocks.map(block => new Date(parseInt(block.timestamp) * 1000).toLocaleTimeString()).reverse();
      const newData = blocks.map(block => parseInt(block.number, 16)).reverse();

      setBlockchainData({
        labels: newLabels,
        datasets: [{
          label: 'Block Height',
          data: newData,
          borderColor: 'rgb(99, 102, 241)',
          backgroundColor: 'rgba(99, 102, 241, 0.1)',
          tension: 0.1,
        }]
      });

      // Load models
      try {
        const modelList = await latticeService.getModels();
        setModels(modelList || []);
      } catch (error) {
        console.warn('Failed to load models:', error);
        setModels([]);
      }

      addNotification('Dashboard data updated successfully', 'success');
    } catch (error) {
      console.error('Failed to load dashboard data:', error);
      addNotification('Failed to load dashboard data', 'error');
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    loadDashboardData();
    const interval = setInterval(loadDashboardData, 30000); // Update every 30 seconds
    return () => clearInterval(interval);
  }, [latticeService, connectionStatus]);

  const chartOptions = {
    responsive: true,
    maintainAspectRatio: false,
    plugins: {
      legend: {
        position: 'top',
        labels: {
          color: '#e2e8f0',
        },
      },
      title: {
        display: true,
        text: 'Recent Block Heights',
        color: '#e2e8f0',
      },
    },
    scales: {
      x: {
        ticks: { color: '#94a3b8' },
        grid: { color: '#334155' },
      },
      y: {
        ticks: { color: '#94a3b8' },
        grid: { color: '#334155' },
      },
    },
  };

  if (connectionStatus !== 'connected') {
    return (
      <Box sx={{ p: 3, display: 'flex', flexDirection: 'column', alignItems: 'center', justifyContent: 'center', height: '60vh' }}>
        <Typography variant="h5" color="text.secondary" sx={{ mb: 2 }}>
          Not Connected to Citrate Node
        </Typography>
        <Typography variant="body1" color="text.secondary" sx={{ mb: 3 }}>
          Please ensure your Citrate node is running on {latticeService?.rpcUrl || 'localhost:8545'}
        </Typography>
        <Button variant="contained" onClick={loadDashboardData} startIcon={<RefreshIcon />}>
          Retry Connection
        </Button>
      </Box>
    );
  }

  return (
    <Box sx={{ p: 3, height: 'calc(100vh - 64px)', overflow: 'auto' }}>
      {/* Header */}
      <Box sx={{ display: 'flex', justifyContent: 'between', alignItems: 'center', mb: 3 }}>
        <Typography variant="h4" component="h1" sx={{ fontWeight: 'bold' }}>
          Dashboard
        </Typography>
        <Button
          variant="outlined"
          startIcon={<RefreshIcon />}
          onClick={loadDashboardData}
          disabled={loading}
        >
          Refresh
        </Button>
      </Box>

      {/* Network Metrics */}
      <Grid container spacing={3} sx={{ mb: 3 }}>
        <Grid item xs={12} sm={6} md={3}>
          <MetricCard
            title="Latest Block"
            value={networkStats?.latestBlock?.toLocaleString() || '0'}
            subtitle="Block height"
            icon={<StorageIcon />}
            color="primary"
          />
        </Grid>
        <Grid item xs={12} sm={6} md={3}>
          <MetricCard
            title="Gas Price"
            value={networkStats ? `${(networkStats.gasPrice / 1e9).toFixed(2)} Gwei` : '0 Gwei'}
            subtitle="Current gas price"
            icon={<SpeedIcon />}
            color="secondary"
          />
        </Grid>
        <Grid item xs={12} sm={6} md={3}>
          <MetricCard
            title="Peers"
            value={networkStats?.peerCount || '0'}
            subtitle="Connected peers"
            icon={<PeopleIcon />}
            color="success"
          />
        </Grid>
        <Grid item xs={12} sm={6} md={3}>
          <MetricCard
            title="Models"
            value={models.length}
            subtitle="Deployed models"
            icon={<TimelineIcon />}
            color="warning"
          />
        </Grid>
      </Grid>

      {/* Charts and Recent Activity */}
      <Grid container spacing={3}>
        {/* Blockchain Chart */}
        <Grid item xs={12} md={8}>
          <Card sx={{ height: 400 }}>
            <CardContent>
              <Typography variant="h6" sx={{ mb: 2 }}>
                Blockchain Activity
              </Typography>
              <Box sx={{ height: 300 }}>
                <Line data={blockchainData} options={chartOptions} />
              </Box>
            </CardContent>
          </Card>
        </Grid>

        {/* Quick Actions */}
        <Grid item xs={12} md={4}>
          <Card sx={{ height: 400 }}>
            <CardContent>
              <Typography variant="h6" sx={{ mb: 2 }}>
                Quick Actions
              </Typography>
              <Box sx={{ display: 'flex', flexDirection: 'column', gap: 2 }}>
                <Button
                  variant="contained"
                  startIcon={<LaunchIcon />}
                  fullWidth
                  onClick={() => window.open('/editor', '_self')}
                >
                  Open Code Editor
                </Button>
                <Button
                  variant="outlined"
                  startIcon={<StorageIcon />}
                  fullWidth
                  onClick={() => window.open('/models', '_self')}
                >
                  Manage Models
                </Button>
                <Button
                  variant="outlined"
                  startIcon={<TrendingUpIcon />}
                  fullWidth
                  onClick={() => window.open('/network', '_self')}
                >
                  Network Viewer
                </Button>
                <Divider />
                <Typography variant="subtitle2" color="text.secondary">
                  Network Status
                </Typography>
                <Box sx={{ display: 'flex', flexWrap: 'wrap', gap: 1 }}>
                  <Chip
                    label={`Chain ID: ${networkStats?.chainId || 'Unknown'}`}
                    size="small"
                    color="primary"
                  />
                  <Chip
                    label={`${networkStats?.peerCount || 0} Peers`}
                    size="small"
                    color="success"
                  />
                </Box>
              </Box>
            </CardContent>
          </Card>
        </Grid>

        {/* Recent Blocks */}
        <Grid item xs={12}>
          <Card>
            <CardContent>
              <Typography variant="h6" sx={{ mb: 2 }}>
                Recent Blocks
              </Typography>
              <TableContainer component={Paper} sx={{ backgroundColor: 'background.paper' }}>
                <Table>
                  <TableHead>
                    <TableRow>
                      <TableCell>Block</TableCell>
                      <TableCell>Hash</TableCell>
                      <TableCell>Timestamp</TableCell>
                      <TableCell>Transactions</TableCell>
                      <TableCell>Gas Used</TableCell>
                      <TableCell align="center">Actions</TableCell>
                    </TableRow>
                  </TableHead>
                  <TableBody>
                    {recentBlocks.map((block) => (
                      <TableRow key={block.number}>
                        <TableCell>
                          <Typography variant="body2" sx={{ fontFamily: 'monospace' }}>
                            {parseInt(block.number, 16).toLocaleString()}
                          </Typography>
                        </TableCell>
                        <TableCell>
                          <Tooltip title={block.hash}>
                            <Typography variant="body2" sx={{ fontFamily: 'monospace' }}>
                              {latticeService.formatHash(block.hash)}
                            </Typography>
                          </Tooltip>
                        </TableCell>
                        <TableCell>
                          {new Date(parseInt(block.timestamp) * 1000).toLocaleString()}
                        </TableCell>
                        <TableCell>
                          {block.transactions?.length || 0}
                        </TableCell>
                        <TableCell>
                          {parseInt(block.gasUsed || '0', 16).toLocaleString()}
                        </TableCell>
                        <TableCell align="center">
                          <Tooltip title="View Block Details">
                            <IconButton size="small">
                              <LaunchIcon fontSize="small" />
                            </IconButton>
                          </Tooltip>
                        </TableCell>
                      </TableRow>
                    ))}
                  </TableBody>
                </Table>
              </TableContainer>
            </CardContent>
          </Card>
        </Grid>
      </Grid>
    </Box>
  );
}

export default Dashboard;