use tauri::Window;
use windows::Win32::{
    Foundation::{HWND, LPARAM, RECT},
    Graphics::Gdi::{
        EnumDisplayMonitors, GetMonitorInfoW, HDC, HMONITOR, MONITORINFO, MONITORINFOEXW,
    },
    UI::WindowsAndMessaging::*,
};
use windows_core::BOOL;

use crate::sharescreen::{draw_overlay, dto::MonitorRect};
// Global mutable to store the current Tauri window handle
static mut MAIN_HWND: HWND = HWND(std::ptr::null_mut());

#[derive(Debug)]
struct DisplayInfo {
    hwnd: HWND,
    handle: isize,
    title: String,
    class_name: String,
    rect: RECT,
    is_capturable: bool,
}

unsafe extern "system" fn enum_windows_callback(hwnd: HWND, lparam: LPARAM) -> BOOL {
    unsafe {
        let windows = &mut *(lparam.0 as *mut Vec<DisplayInfo>);

        if IsWindowVisible(hwnd).as_bool() {
            let mut title = [0u16; 512];
            let len = GetWindowTextW(hwnd, &mut title);

            let mut class_name = [0u16; 256];
            GetClassNameW(hwnd, &mut class_name);

            let mut rect = RECT::default();
            let _ = GetWindowRect(hwnd, &mut rect);

            let title_str = String::from_utf16_lossy(&title[..len as usize]);
            let class_str = String::from_utf16_lossy(
                &class_name[..class_name.iter().position(|&c| c == 0).unwrap_or(0)],
            );

            let ex_style = GetWindowLongW(hwnd, GWL_EXSTYLE) as u32;

            let is_cloaked = {
                let mut cloaked: u32 = 0;
                let _ = windows::Win32::Graphics::Dwm::DwmGetWindowAttribute(
                    hwnd,
                    windows::Win32::Graphics::Dwm::DWMWA_CLOAKED,
                    &mut cloaked as *mut _ as *mut _,
                    std::mem::size_of::<u32>() as u32,
                );
                cloaked != 0
            };

            // Proper off-screen check: intersects any monitor?
            let intersects_any_monitor = get_monitor_rects().iter().any(|m| {
                rect.left < m.right
                    && rect.right > m.left
                    && rect.top < m.bottom
                    && rect.bottom > m.top
            });

            let is_capturable = !title_str.is_empty()
                && rect.right - rect.left > 0
                && rect.bottom - rect.top > 0
                && !is_cloaked
                && (ex_style & WS_EX_TOOLWINDOW.0) == 0
                && class_str != "Windows.UI.Core.CoreWindow"
                && class_str != "ApplicationFrameWindow"
                && !class_str.starts_with("RainmeterMeterWindow")
                && class_str != "CEF-OSC-WIDGET"
                && title_str != "Program Manager"
                && hwnd != MAIN_HWND
                && intersects_any_monitor;

            windows.push(DisplayInfo {
                hwnd,
                handle: hwnd.0 as isize,
                title: title_str,
                class_name: class_str,
                rect,
                is_capturable,
            });
        }
    }

    BOOL(1)
}

fn get_monitor_rects() -> Vec<MonitorRect> {
    let mut rects = Vec::new();

    unsafe extern "system" fn callback(
        hmonitor: HMONITOR,
        _: HDC,
        _: *mut RECT,
        lparam: LPARAM,
    ) -> BOOL {
        let rects = &mut *(lparam.0 as *mut Vec<MonitorRect>);
        let mut info = MONITORINFO {
            cbSize: std::mem::size_of::<MONITORINFO>() as u32,
            ..Default::default()
        };
        if GetMonitorInfoW(hmonitor, &mut info).as_bool() {
            rects.push(MonitorRect {
                left: info.rcMonitor.left,
                top: info.rcMonitor.top,
                right: info.rcMonitor.right,
                bottom: info.rcMonitor.bottom,
            });
        }
        BOOL(1)
    }

    unsafe {
        let _ = EnumDisplayMonitors(
            None,
            None,
            Some(callback),
            LPARAM(&mut rects as *mut _ as isize),
        );
    }

    rects
}

fn get_monitors() -> Vec<(String, HMONITOR)> {
    let mut monitors = Vec::new();

    unsafe {
        let _ = EnumDisplayMonitors(
            None,
            None,
            Some(monitor_enum_proc),
            LPARAM(&mut monitors as *mut _ as isize),
        );
    }

    monitors
}

unsafe extern "system" fn monitor_enum_proc(
    hmonitor: HMONITOR,
    _: HDC,
    _: *mut RECT,
    lparam: LPARAM,
) -> BOOL {
    unsafe {
        let monitors = &mut *(lparam.0 as *mut Vec<(String, HMONITOR)>);

        let mut info = MONITORINFOEXW {
            monitorInfo: MONITORINFO {
                cbSize: std::mem::size_of::<MONITORINFOEXW>() as u32,
                ..Default::default()
            },
            szDevice: [0; 32],
        };

        if GetMonitorInfoW(
            hmonitor,
            &mut info.monitorInfo as *mut _ as *mut MONITORINFO,
        )
        .as_bool()
        {
            let device_name = String::from_utf16_lossy(
                &info.szDevice[..info.szDevice.iter().position(|&c| c == 0).unwrap_or(0)],
            );
            monitors.push((device_name, hmonitor));
        }
    }

    BOOL(1)
}

#[tauri::command]
pub fn get_list(window: Window) {
    let tauri_hwnd = window.hwnd().expect("Failed to get HWND");

    unsafe {
        MAIN_HWND = tauri_hwnd;
    }

    // TODO: get rect size
    println!("=== MONITORS ===");
    let monitors = get_monitors();
    for (idx, (name, handle)) in monitors.iter().enumerate() {
        println!("Monitor {}: {} (Handle: {:?})", idx + 1, name, handle);
    }

    // TODO: draw full screen
    // NOTE: this track run not on main thread
    std::thread::spawn(|| unsafe {
        let monitors = get_monitors();
        if let Some((_, hmonitor)) = monitors.get(0) {
            if !hmonitor.is_invalid() {
                draw_overlay::track_monitor(*hmonitor);
            } else {
                eprintln!("Invalid monitor handle, skipping overlay.");
            }
        } else {
            eprintln!("No monitors found, skipping overlay.");
        }
    });

    println!("\n=== WINDOWS ===");
    let mut windows: Vec<DisplayInfo> = Vec::new();

    unsafe {
        let _ = EnumWindows(
            Some(enum_windows_callback),
            LPARAM(&mut windows as *mut _ as isize),
        );
    }

    for (idx, win) in windows.iter().enumerate() {
        println!(
            "Window {}: \"{}\" | Class: {} | Handle: {} | Capturable: {} | Rect: ({},{}) {}x{}",
            idx + 1,
            win.title,
            win.class_name,
            win.handle,
            win.is_capturable,
            win.rect.left,
            win.rect.top,
            win.rect.right - win.rect.left,
            win.rect.bottom - win.rect.top
        );
    }

    println!("\n=== CAPTURABLE WINDOWS ONLY ===");
    let capturable: Vec<_> = windows.iter().filter(|w| w.is_capturable).collect();
    for (idx, win) in capturable.iter().enumerate() {
        println!(
            "Window {}: \"{}\" | Class: {} | Handle: {} | Rect: ({},{}) {}x{}",
            idx + 1,
            win.title,
            win.class_name,
            win.handle,
            win.rect.left,
            win.rect.top,
            win.rect.right - win.rect.left,
            win.rect.bottom - win.rect.top
        );
    }
    let capturable_handles: Vec<isize> = windows
        .iter()
        .filter(|w| w.is_capturable)
        .map(|w| w.hwnd.0 as isize)
        .collect();
    let hwnd_to_track = capturable_handles.get(0).copied();
    std::thread::spawn(move || unsafe {
        if let Some(handle) = hwnd_to_track {
            let hwnd = HWND(handle as *mut _);
            draw_overlay::track_window(hwnd);
        }
    });

    // unsafe {
    //     draw_overlay::track_window(capturable[0].hwnd);
    // }
}
