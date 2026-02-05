// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/

#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

mod app_window;
mod sharescreen;
// #[cfg(target_os = "windows")]
// use tauri::utils::config::WindowEffectsConfig;
// use tauri::Manager;
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .on_window_event(|_window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                api.prevent_close();
            }
        })
        .setup(|app| {
            app_window::setup_window::setup(&app);

            Ok(())
        })
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            greet,
            sharescreen::get_windows::get_list
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
