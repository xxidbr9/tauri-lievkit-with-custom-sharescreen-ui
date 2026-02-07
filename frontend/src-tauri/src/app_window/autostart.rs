use tauri_plugin_autostart::MacosLauncher;
use tauri_plugin_autostart::ManagerExt;

pub fn setup(app: &tauri::App) {
    #[cfg(desktop)]
    {
        let _ = app.handle().plugin(tauri_plugin_autostart::init(
            MacosLauncher::LaunchAgent,
            Some(vec!["--flag1", "--flag2"]),
        ));

        // Get the autostart manager
        let autostart_manager = app.autolaunch();
        // Enable autostart
        let _ = autostart_manager.enable();
        // Check enable state
        println!(
            "registered for autostart? {}",
            autostart_manager.is_enabled().unwrap()
        );
        // // Disable autostart
        // let _ = autostart_manager.disable();
    }
}
