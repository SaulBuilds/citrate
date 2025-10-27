import React, { useState, useEffect } from 'react';
import { Box, Typography, Button, Card, CardContent, Grid } from '@mui/material';
import { Refresh as RefreshIcon } from '@mui/icons-material';

function NetworkViewer({ latticeService, addNotification }) {
  const [topology, setTopology] = useState({ nodes: [], edges: [] });
  const [loading, setLoading] = useState(false);

  const loadTopology = async () => {
    if (!latticeService) return;

    setLoading(true);
    try {
      const data = await latticeService.getNetworkTopology();
      setTopology(data);
    } catch (error) {
      console.error('Failed to load topology:', error);
      setTopology({
        nodes: [
          { id: 'node1', address: '192.168.1.100', type: 'validator' },
          { id: 'node2', address: '192.168.1.101', type: 'peer' },
          { id: 'node3', address: '192.168.1.102', type: 'peer' },
        ],
        edges: [
          { from: 'node1', to: 'node2' },
          { from: 'node1', to: 'node3' },
        ]
      });
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    loadTopology();
  }, [latticeService]);

  return (
    <Box sx={{ p: 3 }}>
      <Box sx={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', mb: 3 }}>
        <Typography variant="h4">Network Topology</Typography>
        <Button startIcon={<RefreshIcon />} onClick={loadTopology} disabled={loading}>
          Refresh
        </Button>
      </Box>

      <Grid container spacing={3}>
        <Grid item xs={12}>
          <Card>
            <CardContent>
              <Typography variant="h6">Network Overview</Typography>
              <Typography>Nodes: {topology.nodes.length}</Typography>
              <Typography>Connections: {topology.edges.length}</Typography>
            </CardContent>
          </Card>
        </Grid>
      </Grid>
    </Box>
  );
}

export default NetworkViewer;