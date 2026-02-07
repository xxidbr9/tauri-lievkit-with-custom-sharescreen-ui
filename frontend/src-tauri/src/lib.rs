// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use crate::app_window::{autostart, panic_hook};
use anyhow::{Context, Result};
use std::sync::Mutex;

mod app_window;
mod dto;
mod share_screen;
mod sharescreen;
mod tray;

struct AppState {
    selected_window_hwnd: Mutex<Option<isize>>,
    selected_monitor_hmonitor: Mutex<Option<isize>>,
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_autostart::Builder::new().build())
        .manage(AppState {
            selected_window_hwnd: Mutex::new(None),
            selected_monitor_hmonitor: Mutex::new(None),
        })
        .on_window_event(|_window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                api.prevent_close();
            }
        })
        .setup(|app| {
            let handle = app.handle();

            panic_hook::setup(handle.clone());
            app_window::setup_window::setup(&app);
            autostart::setup(&app);
            let _ = tray::setup_tray(&app);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            sharescreen::get_windows::start_share_screen,
            sharescreen::get_windows::close_share_screen,
            sharescreen::get_windows::stream_list,
            sharescreen::get_windows::close_stream_list,
            risk_command,
            panic_test,
            // SHARE SCREEN FUNCTIONALITY
            share_screen::command::get_monitors,
            share_screen::command::get_windows,
            share_screen::command::get_monitor_by_id,
            share_screen::command::get_window_by_id,
            share_screen::command::start_monitor_preview,
            share_screen::command::start_window_preview,
            share_screen::command::stop_preview,
            share_screen::command::get_preview_offer,
            share_screen::command::accept_preview_answer,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

// NOTE: testing error
fn risky_fn() -> Result<String> {
    let data = std::fs::read_to_string("data.txt").context("Failed to read data.txt")?;

    Ok(data)
}
#[tauri::command]
async fn risk_command() -> Result<String, dto::CmdError> {
    let result = risky_fn().map_err(dto::CmdError::from)?;
    Ok(result)
}

#[tauri::command]
fn panic_test() {
    let _ = std::panic::catch_unwind(|| {
        panic!("test");
    });
}
