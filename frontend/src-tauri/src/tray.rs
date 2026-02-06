use tauri::image::Image;
use tauri::menu::{Menu, MenuItem};
use tauri::tray::TrayIconBuilder;

#[cfg(target_os = "windows")]
pub fn setup_tray(app: &tauri::App) -> tauri::Result<()> {
    // Create menu
    let quit = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&quit])?;

    // Load icon - use proper path to your icon file
    let icon = Image::from_path("icons/32x32.png")?;

    TrayIconBuilder::new()
        .icon(icon)
        .menu(&menu)
        .on_menu_event(|app, event| match event.id.as_ref() {
            "quit" => app.exit(0),
            _ => {}
        })
        .build(app)?;

    Ok(())
}
