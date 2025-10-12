import React, { useState, useEffect } from 'react';
import { Routes, Route, useNavigate, useLocation } from 'react-router-dom';
import {
  Box,
  AppBar,
  Toolbar,
  Typography,
  Drawer,
  List,
  ListItem,
  ListItemIcon,
  ListItemText,
  ListItemButton,
  Divider,
  IconButton,
  Badge,
  Tooltip,
  Alert,
  Snackbar,
} from '@mui/material';
import {
  Code as CodeIcon,
  Storage as StorageIcon,
  Timeline as TimelineIcon,
  Settings as SettingsIcon,
  PlayArrow as PlayIcon,
  Stop as StopIcon,
  Dashboard as DashboardIcon,
  AccountTree as TreeIcon,
  BugReport as BugIcon,
  CloudUpload as DeployIcon,
  Description as DocsIcon,
  Notifications as NotificationIcon,
} from '@mui/icons-material';

import Dashboard from './components/Dashboard';
import CodeEditor from './components/CodeEditor';
import ModelManager from './components/ModelManager';
import NetworkViewer from './components/NetworkViewer';
import Debugger from './components/Debugger';
import Deployment from './components/Deployment';
import Settings from './components/Settings';
import { LatticeService } from './services/LatticeService';

const DRAWER_WIDTH = 240;

const navigationItems = [
  { path: '/', icon: <DashboardIcon />, label: 'Dashboard', component: Dashboard },
  { path: '/editor', icon: <CodeIcon />, label: 'Code Editor', component: CodeEditor },
  { path: '/models', icon: <StorageIcon />, label: 'Model Manager', component: ModelManager },
  { path: '/network', icon: <TreeIcon />, label: 'Network', component: NetworkViewer },
  { path: '/debug', icon: <BugIcon />, label: 'Debugger', component: Debugger },
  { path: '/deploy', icon: <DeployIcon />, label: 'Deployment', component: Deployment },
  { path: '/settings', icon: <SettingsIcon />, label: 'Settings', component: Settings },
];

function App() {
  const navigate = useNavigate();
  const location = useLocation();
  const [latticeService, setLatticeService] = useState(null);
  const [connectionStatus, setConnectionStatus] = useState('disconnected');
  const [notifications, setNotifications] = useState([]);
  const [isRunning, setIsRunning] = useState(false);

  useEffect(() => {
    // Initialize Lattice service
    const service = new LatticeService();
    setLatticeService(service);

    // Check connection status
    const checkConnection = async () => {
      try {
        const status = await service.getNetworkStatus();
        setConnectionStatus(status ? 'connected' : 'disconnected');
      } catch (error) {
        setConnectionStatus('error');
        addNotification('Failed to connect to Lattice node', 'error');
      }
    };

    checkConnection();
    const interval = setInterval(checkConnection, 10000); // Check every 10 seconds

    return () => clearInterval(interval);
  }, []);

  const addNotification = (message, severity = 'info') => {
    const id = Date.now();
    setNotifications(prev => [...prev, { id, message, severity }]);
    setTimeout(() => {
      setNotifications(prev => prev.filter(notif => notif.id !== id));
    }, 5000);
  };

  const handleNavigation = (path) => {
    navigate(path);
  };

  const getConnectionColor = () => {
    switch (connectionStatus) {
      case 'connected': return 'success.main';
      case 'disconnected': return 'warning.main';
      case 'error': return 'error.main';
      default: return 'grey.500';
    }
  };

  const getConnectionText = () => {
    switch (connectionStatus) {
      case 'connected': return 'Connected to Lattice Node';
      case 'disconnected': return 'Disconnected from Lattice Node';
      case 'error': return 'Connection Error';
      default: return 'Unknown Status';
    }
  };

  return (
    <Box sx={{ display: 'flex', height: '100vh' }}>
      {/* App Bar */}
      <AppBar
        position="fixed"
        sx={{
          width: `calc(100% - ${DRAWER_WIDTH}px)`,
          ml: `${DRAWER_WIDTH}px`,
          zIndex: (theme) => theme.zIndex.drawer + 1,
        }}
      >
        <Toolbar>
          <Typography variant="h6" noWrap component="div" sx={{ flexGrow: 1 }}>
            Lattice Studio
          </Typography>

          <Tooltip title={getConnectionText()}>
            <Box
              sx={{
                width: 12,
                height: 12,
                borderRadius: '50%',
                backgroundColor: getConnectionColor(),
                mr: 2,
                animation: connectionStatus === 'connected' ? 'pulse 2s infinite' : 'none',
                '@keyframes pulse': {
                  '0%': { opacity: 1 },
                  '50%': { opacity: 0.5 },
                  '100%': { opacity: 1 },
                },
              }}
            />
          </Tooltip>

          <Tooltip title="Notifications">
            <IconButton color="inherit">
              <Badge badgeContent={notifications.length} color="error">
                <NotificationIcon />
              </Badge>
            </IconButton>
          </Tooltip>

          <Tooltip title={isRunning ? "Stop Development Server" : "Start Development Server"}>
            <IconButton
              color="inherit"
              onClick={() => setIsRunning(!isRunning)}
              sx={{ ml: 1 }}
            >
              {isRunning ? <StopIcon /> : <PlayIcon />}
            </IconButton>
          </Tooltip>
        </Toolbar>
      </AppBar>

      {/* Navigation Drawer */}
      <Drawer
        sx={{
          width: DRAWER_WIDTH,
          flexShrink: 0,
          '& .MuiDrawer-paper': {
            width: DRAWER_WIDTH,
            boxSizing: 'border-box',
          },
        }}
        variant="permanent"
        anchor="left"
      >
        <Toolbar>
          <Typography variant="h6" sx={{ fontWeight: 'bold', color: 'primary.main' }}>
            ⟡ Lattice
          </Typography>
        </Toolbar>
        <Divider />
        <List>
          {navigationItems.map((item) => (
            <ListItem key={item.path} disablePadding>
              <ListItemButton
                selected={location.pathname === item.path}
                onClick={() => handleNavigation(item.path)}
                sx={{
                  '&.Mui-selected': {
                    backgroundColor: 'primary.main',
                    color: 'white',
                    '&:hover': {
                      backgroundColor: 'primary.dark',
                    },
                  },
                }}
              >
                <ListItemIcon sx={{ color: 'inherit' }}>
                  {item.icon}
                </ListItemIcon>
                <ListItemText primary={item.label} />
              </ListItemButton>
            </ListItem>
          ))}
        </List>
      </Drawer>

      {/* Main Content */}
      <Box
        component="main"
        sx={{
          flexGrow: 1,
          bgcolor: 'background.default',
          p: 0,
          height: '100vh',
          overflow: 'hidden',
        }}
      >
        <Toolbar />
        <Routes>
          {navigationItems.map((item) => (
            <Route
              key={item.path}
              path={item.path}
              element={
                <item.component
                  latticeService={latticeService}
                  connectionStatus={connectionStatus}
                  addNotification={addNotification}
                  isRunning={isRunning}
                />
              }
            />
          ))}
        </Routes>
      </Box>

      {/* Notifications */}
      {notifications.map((notification) => (
        <Snackbar
          key={notification.id}
          open={true}
          autoHideDuration={5000}
          anchorOrigin={{ vertical: 'bottom', horizontal: 'right' }}
        >
          <Alert severity={notification.severity} variant="filled">
            {notification.message}
          </Alert>
        </Snackbar>
      ))}
    </Box>
  );
}

export default App;