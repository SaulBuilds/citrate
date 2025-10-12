import React, { useState } from 'react';
import {
  Box,
  Typography,
  Card,
  CardContent,
  TextField,
  Switch,
  FormControlLabel,
  Button,
  Divider,
  Grid,
} from '@mui/material';
import { Save as SaveIcon } from '@mui/icons-material';

function Settings({ latticeService, addNotification }) {
  const [settings, setSettings] = useState({
    rpcUrl: 'http://localhost:8545',
    autoRefresh: true,
    refreshInterval: 30,
    debugMode: false,
    notifications: true,
  });

  const handleSave = () => {
    // Save settings to localStorage
    localStorage.setItem('lattice-studio-settings', JSON.stringify(settings));
    addNotification('Settings saved successfully', 'success');
  };

  return (
    <Box sx={{ p: 3 }}>
      <Typography variant="h4" sx={{ mb: 3 }}>Settings</Typography>

      <Grid container spacing={3}>
        <Grid item xs={12} md={6}>
          <Card>
            <CardContent>
              <Typography variant="h6" sx={{ mb: 2 }}>Network Settings</Typography>
              <TextField
                label="RPC URL"
                value={settings.rpcUrl}
                onChange={(e) => setSettings(prev => ({ ...prev, rpcUrl: e.target.value }))}
                fullWidth
                sx={{ mb: 2 }}
              />
              <FormControlLabel
                control={
                  <Switch
                    checked={settings.autoRefresh}
                    onChange={(e) => setSettings(prev => ({ ...prev, autoRefresh: e.target.checked }))}
                  />
                }
                label="Auto Refresh"
              />
              <TextField
                label="Refresh Interval (seconds)"
                type="number"
                value={settings.refreshInterval}
                onChange={(e) => setSettings(prev => ({ ...prev, refreshInterval: parseInt(e.target.value) }))}
                fullWidth
                sx={{ mt: 2 }}
                disabled={!settings.autoRefresh}
              />
            </CardContent>
          </Card>
        </Grid>

        <Grid item xs={12} md={6}>
          <Card>
            <CardContent>
              <Typography variant="h6" sx={{ mb: 2 }}>Application Settings</Typography>
              <FormControlLabel
                control={
                  <Switch
                    checked={settings.debugMode}
                    onChange={(e) => setSettings(prev => ({ ...prev, debugMode: e.target.checked }))}
                  />
                }
                label="Debug Mode"
              />
              <br />
              <FormControlLabel
                control={
                  <Switch
                    checked={settings.notifications}
                    onChange={(e) => setSettings(prev => ({ ...prev, notifications: e.target.checked }))}
                  />
                }
                label="Show Notifications"
              />
            </CardContent>
          </Card>
        </Grid>
      </Grid>

      <Box sx={{ mt: 3 }}>
        <Button
          variant="contained"
          startIcon={<SaveIcon />}
          onClick={handleSave}
        >
          Save Settings
        </Button>
      </Box>
    </Box>
  );
}

export default Settings;