use windows::Win32::Foundation::RECT;
use windows::Win32::UI::WindowsAndMessaging::{
    SystemParametersInfoW, SPI_GETWORKAREA, SYSTEM_PARAMETERS_INFO_UPDATE_FLAGS,
};

use tauri::Manager as _;

#[cfg(target_os = "windows")]
pub fn setup(app: &tauri::App) {
    let app_window = app.get_webview_window("main").unwrap();

    #[cfg(target_os = "windows")]
    {
        let mut rect = RECT::default();
        unsafe {
            #[warn(unused)]
            let _ = SystemParametersInfoW(
                SPI_GETWORKAREA,
                0,
                Some(&mut rect as *mut _ as *mut _),
                SYSTEM_PARAMETERS_INFO_UPDATE_FLAGS(0),
            );
        }

        let margin = 20;
        let work_height = (rect.bottom - rect.top) as u32;
        let work_width = (rect.right - rect.left) as u32;

        // let current_size = app_window.inner_size().unwrap();
        // NOTE: windows are not count the screen properly if you include decorations
        let height = work_height - margin * 2;

        // TODO: this is my phone
        let width = ((height as f64) * (9.0 / 19.5)).round() as u32;
        // let width = 400 as u32;
        let x = (work_width - (width + margin)) as i32;
        let y = rect.top + margin as i32;

        app_window
            .set_size(tauri::Size::Physical(tauri::PhysicalSize { width, height }))
            .unwrap();

        app_window
            .set_position(tauri::Position::Physical(tauri::PhysicalPosition { x, y }))
            .unwrap();

        // app_window.set_always_on_top(true).unwrap();

        // TODO: share the screen to server
    }
}
