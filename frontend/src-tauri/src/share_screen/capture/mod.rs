// src/share_screen/capture/mod.rs
#[cfg(windows)]
pub mod windows;

#[cfg(windows)]
pub mod monitor;

use crate::share_screen::dto::*;

#[cfg(windows)]
pub use self::windows::WindowCapture;

#[cfg(windows)]
pub use self::monitor::MonitorCapture;

#[derive(Clone)]
pub struct CaptureDevice {
    #[cfg(windows)]
    window_capture: WindowCapture,
    #[cfg(windows)]
    monitor_capture: MonitorCapture,
}

unsafe impl Send for CaptureDevice {}

impl CaptureDevice {
    pub fn new() -> Self {
        Self {
            #[cfg(windows)]
            window_capture: WindowCapture::new(),
            #[cfg(windows)]
            monitor_capture: MonitorCapture::new(),
        }
    }

    #[cfg(windows)]
    pub fn enumerate_monitors(&self) -> Result<Vec<MonitorInfo>> {
        self.monitor_capture.enumerate()
    }

    #[cfg(not(windows))]
    pub fn enumerate_monitors(&self) -> Result<Vec<MonitorInfo>> {
        Ok(vec![])
    }

    #[cfg(windows)]
    pub fn enumerate_windows(&self) -> Result<Vec<WindowInfo>> {
        self.window_capture.enumerate()
    }

    #[cfg(not(windows))]
    pub fn enumerate_windows(&self) -> Result<Vec<WindowInfo>> {
        Ok(vec![])
    }

    #[cfg(windows)]
    pub fn get_monitor_info(&self, hmonitor: isize) -> Result<MonitorInfo> {
        self.monitor_capture.get_info(hmonitor)
    }

    #[cfg(not(windows))]
    pub fn get_monitor_info(&self, _hmonitor: isize) -> Result<MonitorInfo> {
        Err(CaptureError::PlatformError("Not supported".to_string()))
    }

    #[cfg(windows)]
    pub fn get_window_info(&self, hwnd: isize) -> Result<WindowInfo> {
        self.window_capture.get_info(hwnd)
    }

    #[cfg(not(windows))]
    pub fn get_window_info(&self, _hwnd: isize) -> Result<WindowInfo> {
        Err(CaptureError::PlatformError("Not supported".to_string()))
    }

    #[cfg(windows)]
    pub async fn capture_monitor_thumbnail(
        &self,
        hmonitor: isize,
        width: i32,
        height: i32,
    ) -> Result<Vec<u8>> {
        self.monitor_capture
            .capture_thumbnail(hmonitor, width, height)
            .await
    }

    #[cfg(not(windows))]
    pub async fn capture_monitor_thumbnail(
        &self,
        _hmonitor: isize,
        _width: i32,
        _height: i32,
    ) -> Result<Vec<u8>> {
        Err(CaptureError::PlatformError("Not supported".to_string()))
    }

    #[cfg(windows)]
    pub async fn capture_window_thumbnail(
        &self,
        hwnd: isize,
        width: i32,
        height: i32,
    ) -> Result<Vec<u8>> {
        self.window_capture
            .capture_thumbnail(hwnd, width, height)
            .await
    }

    #[cfg(not(windows))]
    pub async fn capture_window_thumbnail(
        &self,
        _hwnd: isize,
        _width: i32,
        _height: i32,
    ) -> Result<Vec<u8>> {
        Err(CaptureError::PlatformError("Not supported".to_string()))
    }

    #[cfg(windows)]
    pub async fn start_monitor_capture(
        &self,
        hmonitor: isize,
        config: CaptureConfig,
        video_tx: tokio::sync::mpsc::Sender<Vec<u8>>,
    ) -> Result<()> {
        self.monitor_capture
            .start_capture(hmonitor, config, video_tx)
            .await
    }

    #[cfg(not(windows))]
    pub async fn start_monitor_capture(
        &self,
        _hmonitor: isize,
        _config: CaptureConfig,
        _video_tx: tokio::sync::mpsc::Sender<Vec<u8>>,
    ) -> Result<()> {
        Err(CaptureError::PlatformError("Not supported".to_string()))
    }

    #[cfg(windows)]
    pub async fn start_window_capture(
        &self,
        hwnd: isize,
        config: CaptureConfig,
        video_tx: tokio::sync::mpsc::Sender<Vec<u8>>,
    ) -> Result<()> {
        self.window_capture
            .start_capture(hwnd, config, video_tx)
            .await
    }

    #[cfg(not(windows))]
    pub async fn start_window_capture(
        &self,
        _hwnd: isize,
        _config: CaptureConfig,
        _video_tx: tokio::sync::mpsc::Sender<Vec<u8>>,
    ) -> Result<()> {
        Err(CaptureError::PlatformError("Not supported".to_string()))
    }

    #[cfg(windows)]
    pub fn enumerate_audio_devices(&self) -> Result<Vec<AudioDevice>> {
        self::windows::enumerate_audio_devices()
    }

    #[cfg(not(windows))]
    pub fn enumerate_audio_devices(&self) -> Result<Vec<AudioDevice>> {
        Ok(vec![])
    }
}
