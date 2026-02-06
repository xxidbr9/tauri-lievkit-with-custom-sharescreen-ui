// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/

#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use crate::app_window::panic_hook;

mod app_window;
mod dto;
mod sharescreen;
mod tray;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_autostart::Builder::new().build())
        .on_window_event(|_window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                api.prevent_close();
            }
        })
        .setup(|app| {
            let handle = app.handle();

            panic_hook::setup(handle.clone());
            app_window::setup_window::setup(&app);
            let _ = tray::setup_tray(&app);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // sharescreen::get_windows::get_list
            risk_command,
            panic_test
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

// NOTE: testing error
use anyhow::{Context, Result};

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
