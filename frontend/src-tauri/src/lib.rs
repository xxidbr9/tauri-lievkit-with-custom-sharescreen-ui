// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/

#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

mod app_window;
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
            app_window::setup_window::setup(&app);
            let _ = tray::setup_tray(&app);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // sharescreen::get_windows::get_list
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
