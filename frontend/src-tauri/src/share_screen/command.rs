// src/share_screen/command.rs
use crate::share_screen::{MANAGER, WEBRTC_SERVER, dto::*};
use anyhow::Result;

#[tauri::command]
pub async fn get_monitors(fps: i32, width: i32, height: i32) -> Result<Vec<CaptureSource>, String> {
    let config = CaptureConfig {
        fps,
        width,
        height,
        ..Default::default()
    };

    MANAGER
        .read()
        .await
        .get_monitors(config)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_windows(fps: i32, width: i32, height: i32) -> Result<Vec<CaptureSource>, String> {
    let config = CaptureConfig {
        fps,
        width,
        height,
        ..Default::default()
    };

    // TODO: handle sent this data using emit, so it always update, and make sure it have watchdog
    MANAGER
        .read()
        .await
        .get_windows(config)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_monitor_by_id(
    id: String,
    fps: i32,
    width: i32,
    height: i32,
) -> Result<CaptureSource, String> {
    let config = CaptureConfig {
        fps,
        width,
        height,
        ..Default::default()
    };

    let source_type =
        CaptureSourceType::from_id(&id).ok_or_else(|| "Invalid monitor ID".to_string())?;

    match source_type {
        CaptureSourceType::Monitor(hmonitor) => MANAGER
            .read()
            .await
            .get_monitor_by_hmonitor(hmonitor, config)
            .await
            .map_err(|e| e.to_string()),
        _ => Err("ID is not a monitor".to_string()),
    }
}

#[tauri::command]
pub async fn get_window_by_id(
    id: String,
    fps: i32,
    width: i32,
    height: i32,
) -> Result<CaptureSource, String> {
    let config = CaptureConfig {
        fps,
        width,
        height,
        ..Default::default()
    };

    let source_type =
        CaptureSourceType::from_id(&id).ok_or_else(|| "Invalid window ID".to_string())?;

    match source_type {
        CaptureSourceType::Window(hwnd) => MANAGER
            .read()
            .await
            .get_window_by_hwnd(hwnd, config)
            .await
            .map_err(|e| e.to_string()),
        _ => Err("ID is not a window".to_string()),
    }
}

#[tauri::command]
pub async fn start_monitor_preview(
    hmonitor: isize,
    fps: i32,
    width: i32,
    height: i32,
) -> Result<(), String> {
    let config = CaptureConfig {
        fps,
        width,
        height,
        ..Default::default()
    };

    MANAGER
        .write()
        .await
        .start_preview(CaptureSourceType::Monitor(hmonitor), config)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn start_window_preview(
    hwnd: isize,
    fps: i32,
    width: i32,
    height: i32,
) -> Result<(), String> {
    let config = CaptureConfig {
        fps,
        width,
        height,
        ..Default::default()
    };

    MANAGER
        .write()
        .await
        .start_preview(CaptureSourceType::Window(hwnd), config)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn stop_preview(id: String) -> Result<(), String> {
    MANAGER
        .write()
        .await
        .stop_preview(&id)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_preview_offer(id: String) -> Result<PreviewOffer, String> {
    WEBRTC_SERVER
        .write()
        .await
        .get_preview_offer(&id)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn accept_preview_answer(id: String, sdp: String) -> Result<(), String> {
    WEBRTC_SERVER
        .write()
        .await
        .accept_preview_answer(&id, sdp)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn add_preview_ice_candidate(
    id: String,
    candidate: String,
    sdp_mid: Option<String>,
    sdp_mline_index: Option<u16>,
) -> Result<(), String> {
    WEBRTC_SERVER
        .write()
        .await
        .add_preview_ice_candidate(id, candidate, sdp_mid, sdp_mline_index)
        .await
        .map_err(|e| e.to_string())
}
