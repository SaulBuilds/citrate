//! Window Management Module
//!
//! Sprint 5: Multi-Window & Terminal Integration
//!
//! This module provides multi-window management for the Citrate GUI,
//! enabling terminal, preview, and editor windows.

pub mod manager;

pub use manager::WindowManager;

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tauri::{AppHandle, Emitter, Manager, WebviewUrl, WebviewWindowBuilder};
use tokio::sync::RwLock;

/// Window type enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum WindowType {
    Main,
    Terminal,
    Preview,
    Editor,
}

impl WindowType {
    /// Get the URL path for this window type
    pub fn url_path(&self) -> &'static str {
        match self {
            WindowType::Main => "/",
            WindowType::Terminal => "/terminal",
            WindowType::Preview => "/preview",
            WindowType::Editor => "/editor",
        }
    }

    /// Get default size for this window type
    pub fn default_size(&self) -> (f64, f64) {
        match self {
            WindowType::Main => (1200.0, 800.0),
            WindowType::Terminal => (800.0, 500.0),
            WindowType::Preview => (1024.0, 768.0),
            WindowType::Editor => (1000.0, 700.0),
        }
    }

    /// Get default title for this window type
    pub fn default_title(&self) -> &'static str {
        match self {
            WindowType::Main => "Citrate",
            WindowType::Terminal => "Terminal",
            WindowType::Preview => "App Preview",
            WindowType::Editor => "Code Editor",
        }
    }
}

impl std::str::FromStr for WindowType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "main" => Ok(WindowType::Main),
            "terminal" => Ok(WindowType::Terminal),
            "preview" => Ok(WindowType::Preview),
            "editor" => Ok(WindowType::Editor),
            _ => Err(format!("Unknown window type: {}", s)),
        }
    }
}

/// Window state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowState {
    pub id: String,
    pub window_type: WindowType,
    pub title: String,
    pub is_open: bool,
    pub is_focused: bool,
    pub position: Option<(f64, f64)>,
    pub size: Option<(f64, f64)>,
    pub data: Option<serde_json::Value>,
    pub created_at: u64,
}

/// Window event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowEvent {
    pub event_type: String,
    pub window_id: String,
    pub timestamp: u64,
    pub data: Option<serde_json::Value>,
}

impl WindowEvent {
    pub fn new(event_type: &str, window_id: &str) -> Self {
        Self {
            event_type: event_type.to_string(),
            window_id: window_id.to_string(),
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis() as u64,
            data: None,
        }
    }

    pub fn with_data(mut self, data: serde_json::Value) -> Self {
        self.data = Some(data);
        self
    }
}

/// Create a new window
pub fn create_window(
    app: &AppHandle,
    window_id: &str,
    window_type: WindowType,
    title: &str,
    width: f64,
    height: f64,
    x: Option<f64>,
    y: Option<f64>,
) -> Result<(), String> {
    let url = WebviewUrl::App(window_type.url_path().into());

    let mut builder = WebviewWindowBuilder::new(app, window_id, url)
        .title(title)
        .inner_size(width, height)
        .resizable(true)
        .visible(true);

    // Set position if provided
    if let (Some(x), Some(y)) = (x, y) {
        builder = builder.position(x, y);
    } else {
        builder = builder.center();
    }

    // Additional settings based on window type
    match window_type {
        WindowType::Terminal => {
            builder = builder.decorations(true);
        }
        WindowType::Preview => {
            builder = builder.decorations(true);
        }
        WindowType::Editor => {
            builder = builder.decorations(true);
        }
        WindowType::Main => {
            // Main window settings are in tauri.conf.json
        }
    }

    builder.build().map_err(|e| e.to_string())?;

    Ok(())
}

/// Close a window
pub fn close_window(app: &AppHandle, window_id: &str) -> Result<(), String> {
    if let Some(window) = app.get_webview_window(window_id) {
        window.close().map_err(|e| e.to_string())?;
    }
    Ok(())
}

/// Focus a window
pub fn focus_window(app: &AppHandle, window_id: &str) -> Result<(), String> {
    if let Some(window) = app.get_webview_window(window_id) {
        window.set_focus().map_err(|e| e.to_string())?;
    } else {
        return Err(format!("Window not found: {}", window_id));
    }
    Ok(())
}

/// Send a message to a window
pub fn send_to_window(
    app: &AppHandle,
    window_id: &str,
    event: &str,
    payload: serde_json::Value,
) -> Result<(), String> {
    if let Some(window) = app.get_webview_window(window_id) {
        window.emit(event, payload).map_err(|e| e.to_string())?;
    } else {
        return Err(format!("Window not found: {}", window_id));
    }
    Ok(())
}

/// Broadcast a message to all windows
pub fn broadcast_to_all(
    app: &AppHandle,
    event: &str,
    payload: serde_json::Value,
) -> Result<(), String> {
    app.emit(event, payload).map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_window_type_from_str() {
        assert_eq!("terminal".parse::<WindowType>().unwrap(), WindowType::Terminal);
        assert_eq!("preview".parse::<WindowType>().unwrap(), WindowType::Preview);
        assert_eq!("editor".parse::<WindowType>().unwrap(), WindowType::Editor);
        assert_eq!("main".parse::<WindowType>().unwrap(), WindowType::Main);
    }

    #[test]
    fn test_window_type_default_size() {
        assert_eq!(WindowType::Terminal.default_size(), (800.0, 500.0));
        assert_eq!(WindowType::Preview.default_size(), (1024.0, 768.0));
    }

    #[test]
    fn test_window_event() {
        let event = WindowEvent::new("window:opened", "terminal_123");
        assert_eq!(event.event_type, "window:opened");
        assert_eq!(event.window_id, "terminal_123");
    }
}
