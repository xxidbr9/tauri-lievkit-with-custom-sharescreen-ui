// src/share_screen/manager.rs
use crate::share_screen::{
    capture::CaptureDevice,
    dto::{
        AudioDevice, CaptureConfig, CaptureError, CaptureSource, CaptureSourceType, CaptureStream,
        Result,
    },
};
use base64::{Engine as _, engine::general_purpose};

pub struct CaptureManager {
    active_streams: DashMap<String, CaptureStream>,
    capture_device: CaptureDevice,
}

use dashmap::DashMap;
use tauri::async_runtime;

impl CaptureManager {
    pub fn new() -> Self {
        Self {
            active_streams: DashMap::new(),
            capture_device: CaptureDevice::new(),
        }
    }

    #[allow(dead_code)]
    pub async fn test_function(&self) {
        println!("Test function called");
    }

    pub async fn get_monitors(&self, config: CaptureConfig) -> Result<Vec<CaptureSource>> {
        let monitors = self.capture_device.enumerate_monitors()?;

        let mut sources = Vec::new();

        for monitor in monitors {
            // let thumbnail = self
            //     .capture_single_frame_monitor(monitor.hmonitor, config.width, config.height)
            //     .await?;

            let thumbnail = "".to_string();

            sources.push(CaptureSource {
                id: CaptureSourceType::Monitor(monitor.hmonitor).to_id(),
                title: monitor.name,
                thumbnail: general_purpose::STANDARD.encode(&thumbnail),
                icon: None,
                source_type: "monitor".to_string(),
                width: monitor.width,
                height: monitor.height,
            });
        }

        Ok(sources)
    }

    pub async fn get_windows(&self, config: CaptureConfig) -> Result<Vec<CaptureSource>> {
        let windows = self.capture_device.enumerate_windows()?;

        let mut sources = Vec::new();

        for window in windows {
            // let thumbnail = self
            //     .capture_single_frame_window(window.hwnd, config.width, config.height)
            //     .await?;

            let thumbnail = "".to_string();

            let icon = window
                .icon
                .map(|data| general_purpose::STANDARD.encode(&data));

            sources.push(CaptureSource {
                id: CaptureSourceType::Window(window.hwnd).to_id(),
                title: window.title,
                thumbnail: general_purpose::STANDARD.encode(&thumbnail),
                icon,
                source_type: "window".to_string(),
                width: window.width,
                height: window.height,
            });
        }

        Ok(sources)
    }

    pub async fn get_monitor_by_hmonitor(
        &self,
        hmonitor: isize,
        config: CaptureConfig,
    ) -> Result<CaptureSource> {
        let monitor = self.capture_device.get_monitor_info(hmonitor)?;

        // let thumbnail = self
        //     .capture_single_frame_monitor(hmonitor, config.width, config.height)
        //     .await?;

        let thumbnail = "".to_string();

        Ok(CaptureSource {
            id: CaptureSourceType::Monitor(hmonitor).to_id(),
            title: monitor.name,
            thumbnail: general_purpose::STANDARD.encode(&thumbnail),
            icon: None,
            source_type: "monitor".to_string(),
            width: monitor.width,
            height: monitor.height,
        })
    }

    pub async fn get_window_by_hwnd(
        &self,
        hwnd: isize,
        config: CaptureConfig,
    ) -> Result<CaptureSource> {
        let window = self.capture_device.get_window_info(hwnd)?;

        // let thumbnail = self
        //     .capture_single_frame_window(hwnd, config.width, config.height)
        //     .await?;

        let thumbnail = "".to_string();

        let icon = window
            .icon
            .map(|data| general_purpose::STANDARD.encode(&data));

        Ok(CaptureSource {
            id: CaptureSourceType::Window(hwnd).to_id(),
            title: window.title,
            thumbnail: general_purpose::STANDARD.encode(&thumbnail),
            icon,
            source_type: "window".to_string(),
            width: window.width,
            height: window.height,
        })
    }

    pub async fn start_preview(
        &mut self,
        source_type: CaptureSourceType,
        config: CaptureConfig,
    ) -> Result<()> {
        let id = source_type.to_id();

        if self.active_streams.contains_key(&id) {
            return Err(CaptureError::CaptureAlreadyActive(id));
        }

        let (video_tx, video_rx) = tokio::sync::mpsc::channel(100);

        let video_tx_for_task = video_tx.clone();
        // Register with WebRTC server
        crate::share_screen::WEBRTC_SERVER
            .write()
            .await
            .create_preview_track(&id, video_rx)
            .await
            .map_err(|e| CaptureError::WebRTCError(e.to_string()))?;

        // Start capture
        let capture_device = self.capture_device.clone();
        let source_type_clone = source_type.clone();
        let config_clone = config.clone();

        // Spawn blocking task to preserve !Send handle
        let handle: tauri::async_runtime::JoinHandle<()> =
            async_runtime::spawn_blocking(move || {
                futures::executor::block_on(async move {
                    let result = match source_type_clone {
                        CaptureSourceType::Monitor(hmonitor) => {
                            capture_device
                                .start_monitor_capture(hmonitor, config_clone, video_tx_for_task)
                                .await
                        }
                        CaptureSourceType::Window(hwnd) => {
                            capture_device
                                .start_window_capture(hwnd, config_clone, video_tx_for_task)
                                .await
                        }
                    };

                    if let Err(e) = result {
                        eprintln!("Capture error: {:?}", e);
                    }
                })
            });

        self.active_streams.insert(
            id.clone(),
            CaptureStream {
                id,
                source_type,
                config,
                video_tx,
                capture_handle: Some(handle),
            },
        );

        Ok(())
    }

    pub async fn stop_preview(&mut self, id: &str) -> Result<()> {
        if let Some((_, stream)) = self.active_streams.remove(id) {
            if let Some(handle) = stream.capture_handle {
                handle.abort();
            }

            crate::share_screen::WEBRTC_SERVER
                .write()
                .await
                .close_preview(id)
                .await;
        }

        Ok(())
    }

    pub async fn get_audio_devices(&self) -> Result<Vec<AudioDevice>> {
        self.capture_device.enumerate_audio_devices()
    }

    async fn capture_single_frame_monitor(
        &self,
        hmonitor: isize,
        width: i32,
        height: i32,
    ) -> Result<Vec<u8>> {
        self.capture_device
            .capture_monitor_thumbnail(hmonitor, width, height)
            .await
    }

    async fn capture_single_frame_window(
        &self,
        hwnd: isize,
        width: i32,
        height: i32,
    ) -> Result<Vec<u8>> {
        self.capture_device
            .capture_window_thumbnail(hwnd, width, height)
            .await
    }
}
