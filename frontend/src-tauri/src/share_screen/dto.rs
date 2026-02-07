use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CaptureSource {
    pub id: String,
    pub title: String,
    pub thumbnail: String,    // base64 encoded image
    pub icon: Option<String>, // base64 encoded icon
    pub source_type: String,  // "monitor" or "window"
    pub width: i32,
    pub height: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PreviewOffer {
    pub id: String,
    pub sdp: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CaptureConfig {
    pub fps: i32,
    pub width: i32,
    pub height: i32,
}

#[derive(Debug, Clone)]
pub struct MonitorInfo {
    pub hmonitor: isize,
    pub name: String,
    pub width: i32,
    pub height: i32,
    pub x: i32,
    pub y: i32,
    pub is_primary: bool,
}

#[derive(Debug, Clone)]
pub struct MonitorRect {
    pub left: i32,
    pub top: i32,
    pub right: i32,
    pub bottom: i32,
}

#[derive(Debug, Clone)]
pub struct WindowInfo {
    pub hwnd: isize,
    pub title: String,
    pub width: i32,
    pub height: i32,
    pub icon: Option<Vec<u8>>,
    pub is_capturable: Option<bool>,
}

#[derive(Debug, Clone)]
pub struct AudioDevice {
    pub id: String,
    pub name: String,
    pub is_default: bool,
}

#[derive(Debug)]
pub struct CaptureStream {
    pub id: String,
    pub source_type: CaptureSourceType,
    pub config: CaptureConfig,
    pub video_tx: tokio::sync::mpsc::Sender<Vec<u8>>,
    pub capture_handle: Option<tauri::async_runtime::JoinHandle<()>>,
}
unsafe impl Send for CaptureStream {}

#[derive(Debug, Clone, PartialEq)]
pub enum CaptureSourceType {
    Monitor(isize), // hmonitor
    Window(isize),  // hwnd
}

impl CaptureSourceType {
    pub fn to_id(&self) -> String {
        match self {
            CaptureSourceType::Monitor(hmonitor) => format!("monitor_{}", hmonitor),
            CaptureSourceType::Window(hwnd) => format!("window_{}", hwnd),
        }
    }

    pub fn from_id(id: &str) -> Option<Self> {
        if let Some(hmonitor) = id.strip_prefix("monitor_") {
            hmonitor.parse().ok().map(CaptureSourceType::Monitor)
        } else if let Some(hwnd) = id.strip_prefix("window_") {
            hwnd.parse().ok().map(CaptureSourceType::Window)
        } else {
            None
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum CaptureError {
    #[error("Source not found: {0}")]
    SourceNotFound(String),

    #[error("Capture already active: {0}")]
    CaptureAlreadyActive(String),

    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),

    #[error("Platform error: {0}")]
    PlatformError(String),

    #[error("WebRTC error: {0}")]
    WebRTCError(String),
}

pub type Result<T> = anyhow::Result<T, CaptureError>;
