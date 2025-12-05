//! Window Manager
//!
//! Centralized management of all application windows.

use super::{WindowEvent, WindowState, WindowType};
use std::collections::HashMap;
use std::sync::Arc;
use tauri::{AppHandle, Manager};
use tokio::sync::RwLock;

/// Window Manager
///
/// Tracks all open windows and provides methods for window operations.
pub struct WindowManager {
    /// Map of window ID to window state
    windows: Arc<RwLock<HashMap<String, WindowState>>>,
    /// App handle for window operations
    app_handle: Option<AppHandle>,
}

impl WindowManager {
    /// Create a new window manager
    pub fn new() -> Self {
        Self {
            windows: Arc::new(RwLock::new(HashMap::new())),
            app_handle: None,
        }
    }

    /// Set the app handle (called during app setup)
    pub fn set_app_handle(&mut self, app: AppHandle) {
        self.app_handle = Some(app);
    }

    /// Get a reference to the app handle
    pub fn app_handle(&self) -> Option<&AppHandle> {
        self.app_handle.as_ref()
    }

    /// Register a window
    pub async fn register_window(&self, state: WindowState) {
        let mut windows = self.windows.write().await;
        windows.insert(state.id.clone(), state);
    }

    /// Unregister a window
    pub async fn unregister_window(&self, window_id: &str) -> Option<WindowState> {
        let mut windows = self.windows.write().await;
        windows.remove(window_id)
    }

    /// Get window state
    pub async fn get_window(&self, window_id: &str) -> Option<WindowState> {
        let windows = self.windows.read().await;
        windows.get(window_id).cloned()
    }

    /// Get all windows
    pub async fn get_all_windows(&self) -> Vec<WindowState> {
        let windows = self.windows.read().await;
        windows.values().cloned().collect()
    }

    /// Get windows by type
    pub async fn get_windows_by_type(&self, window_type: WindowType) -> Vec<WindowState> {
        let windows = self.windows.read().await;
        windows
            .values()
            .filter(|w| w.window_type == window_type && w.is_open)
            .cloned()
            .collect()
    }

    /// Update window state
    pub async fn update_window<F>(&self, window_id: &str, updater: F) -> Option<WindowState>
    where
        F: FnOnce(&mut WindowState),
    {
        let mut windows = self.windows.write().await;
        if let Some(state) = windows.get_mut(window_id) {
            updater(state);
            Some(state.clone())
        } else {
            None
        }
    }

    /// Set window focus
    pub async fn set_focus(&self, window_id: &str) {
        let mut windows = self.windows.write().await;

        // Blur all windows
        for state in windows.values_mut() {
            state.is_focused = false;
        }

        // Focus the target window
        if let Some(state) = windows.get_mut(window_id) {
            state.is_focused = true;
        }
    }

    /// Check if any window of type is open
    pub async fn has_window_type(&self, window_type: WindowType) -> bool {
        let windows = self.windows.read().await;
        windows
            .values()
            .any(|w| w.window_type == window_type && w.is_open)
    }

    /// Get count of open windows
    pub async fn window_count(&self) -> usize {
        let windows = self.windows.read().await;
        windows.values().filter(|w| w.is_open).count()
    }

    /// Create a window
    pub async fn create_window(
        &self,
        window_id: &str,
        window_type: WindowType,
        title: &str,
        width: f64,
        height: f64,
        x: Option<f64>,
        y: Option<f64>,
        data: Option<serde_json::Value>,
    ) -> Result<WindowState, String> {
        let app = self
            .app_handle
            .as_ref()
            .ok_or_else(|| "App handle not set".to_string())?;

        // Create the actual window
        super::create_window(app, window_id, window_type, title, width, height, x, y)?;

        // Create state
        let state = WindowState {
            id: window_id.to_string(),
            window_type,
            title: title.to_string(),
            is_open: true,
            is_focused: true,
            position: x.zip(y),
            size: Some((width, height)),
            data,
            created_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis() as u64,
        };

        // Register
        self.register_window(state.clone()).await;

        // Emit event
        let event = WindowEvent::new("window:opened", window_id);
        super::broadcast_to_all(app, "window-opened", serde_json::to_value(&event).unwrap())?;

        Ok(state)
    }

    /// Close a window
    pub async fn close_window(&self, window_id: &str) -> Result<(), String> {
        let app = self
            .app_handle
            .as_ref()
            .ok_or_else(|| "App handle not set".to_string())?;

        // Close the actual window
        super::close_window(app, window_id)?;

        // Unregister
        self.unregister_window(window_id).await;

        // Emit event
        let event = WindowEvent::new("window:closed", window_id);
        super::broadcast_to_all(app, "window-closed", serde_json::to_value(&event).unwrap())?;

        Ok(())
    }

    /// Focus a window
    pub async fn focus_window(&self, window_id: &str) -> Result<(), String> {
        let app = self
            .app_handle
            .as_ref()
            .ok_or_else(|| "App handle not set".to_string())?;

        // Focus the actual window
        super::focus_window(app, window_id)?;

        // Update state
        self.set_focus(window_id).await;

        // Emit event
        let event = WindowEvent::new("window:focused", window_id);
        super::broadcast_to_all(app, "window-focused", serde_json::to_value(&event).unwrap())?;

        Ok(())
    }

    /// Send message to a window
    pub async fn send_to_window(
        &self,
        window_id: &str,
        event: &str,
        payload: serde_json::Value,
    ) -> Result<(), String> {
        let app = self
            .app_handle
            .as_ref()
            .ok_or_else(|| "App handle not set".to_string())?;

        super::send_to_window(app, window_id, event, payload)
    }

    /// Broadcast to all windows
    pub async fn broadcast(&self, event: &str, payload: serde_json::Value) -> Result<(), String> {
        let app = self
            .app_handle
            .as_ref()
            .ok_or_else(|| "App handle not set".to_string())?;

        super::broadcast_to_all(app, event, payload)
    }
}

impl Default for WindowManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_window_manager_register() {
        let manager = WindowManager::new();

        let state = WindowState {
            id: "test_1".to_string(),
            window_type: WindowType::Terminal,
            title: "Test Terminal".to_string(),
            is_open: true,
            is_focused: true,
            position: None,
            size: Some((800.0, 600.0)),
            data: None,
            created_at: 0,
        };

        manager.register_window(state.clone()).await;

        let retrieved = manager.get_window("test_1").await;
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().title, "Test Terminal");
    }

    #[tokio::test]
    async fn test_window_manager_by_type() {
        let manager = WindowManager::new();

        let terminal = WindowState {
            id: "term_1".to_string(),
            window_type: WindowType::Terminal,
            title: "Terminal 1".to_string(),
            is_open: true,
            is_focused: false,
            position: None,
            size: None,
            data: None,
            created_at: 0,
        };

        let editor = WindowState {
            id: "edit_1".to_string(),
            window_type: WindowType::Editor,
            title: "Editor 1".to_string(),
            is_open: true,
            is_focused: true,
            position: None,
            size: None,
            data: None,
            created_at: 0,
        };

        manager.register_window(terminal).await;
        manager.register_window(editor).await;

        let terminals = manager.get_windows_by_type(WindowType::Terminal).await;
        assert_eq!(terminals.len(), 1);

        let editors = manager.get_windows_by_type(WindowType::Editor).await;
        assert_eq!(editors.len(), 1);

        assert!(manager.has_window_type(WindowType::Terminal).await);
        assert!(manager.has_window_type(WindowType::Editor).await);
        assert!(!manager.has_window_type(WindowType::Preview).await);
    }
}
