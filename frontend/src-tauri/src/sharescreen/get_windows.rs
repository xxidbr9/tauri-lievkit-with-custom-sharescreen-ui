use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    thread,
    time::Duration,
};

use crate::sharescreen::{
    capturer::{capture_app_window, capture_monitor_display, get_window_icon},
    draw_overlay,
    dto::{CaptureSource, DisplayInfo, MonitorInfo, MonitorRect, SourcesUpdate},
};
use dashmap::DashMap;
use rayon::prelude::*;
use tauri::{AppHandle, Emitter, Window};
use windows::Win32::{
    Foundation::{CloseHandle, HANDLE, HWND, LPARAM, RECT, WAIT_OBJECT_0},
    Graphics::Gdi::{
        EnumDisplayMonitors, GetMonitorInfoW, HDC, HMONITOR, MONITORINFO, MONITORINFOEXW,
    },
    System::Threading::{OpenProcess, WaitForSingleObject, PROCESS_QUERY_LIMITED_INFORMATION},
    UI::WindowsAndMessaging::*,
};
use windows_core::BOOL;

// Global mutable to store the current Tauri window handle
static mut MAIN_HWND: HWND = HWND(std::ptr::null_mut());
static mut SHARE_SCREEN_POPUP_HWND: HWND = HWND(std::ptr::null_mut());

// TODO: make this on AppState level
lazy_static::lazy_static! {
    static ref STREAM_REGISTRY: Arc<DashMap<String, Arc<std::sync::atomic::AtomicBool>>> =
        Arc::new(DashMap::new());
}

const STREAM_ID: &str = "capture_stream";

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
                && hwnd != SHARE_SCREEN_POPUP_HWND
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

fn get_monitors_info() -> Vec<MonitorInfo> {
    let mut monitors = Vec::new();

    unsafe extern "system" fn callback(
        hmonitor: HMONITOR,
        _: HDC,
        _: *mut RECT,
        lparam: LPARAM,
    ) -> BOOL {
        let monitors = &mut *(lparam.0 as *mut Vec<MonitorInfo>);

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
            monitors.push(MonitorInfo {
                hmonitor,
                device_name,
                rect: MonitorRect {
                    left: info.monitorInfo.rcMonitor.left,
                    top: info.monitorInfo.rcMonitor.top,
                    right: info.monitorInfo.rcMonitor.right,
                    bottom: info.monitorInfo.rcMonitor.bottom,
                },
            });
        }

        BOOL(1)
    }

    unsafe {
        let _ = EnumDisplayMonitors(
            None,
            None,
            Some(callback),
            LPARAM(&mut monitors as *mut _ as isize),
        );
    }

    monitors
}

#[tauri::command]
pub fn stream_list(window: Window, app: AppHandle, fps: Option<u64>) {
    let tauri_hwnd = window.hwnd().expect("Failed to get HWND");

    unsafe {
        MAIN_HWND = tauri_hwnd;
    }

    let fps = fps.unwrap_or(180);
    let interval_duration = Duration::from_millis(1000 / fps);
    let stream_id = STREAM_ID.to_string();

    // Create stop flag
    let should_stop = Arc::new(std::sync::atomic::AtomicBool::new(false));
    STREAM_REGISTRY.insert(stream_id.clone(), should_stop.clone());

    tauri::async_runtime::spawn(async move {
        let mut ticker = tokio::time::interval(interval_duration);

        loop {
            // Wait for next tick
            ticker.tick().await;

            // Check stop flag
            if should_stop.load(Ordering::Relaxed) {
                break;
            }

            // Capture windows
            let mut windows: Vec<DisplayInfo> = Vec::new();
            unsafe {
                let _ = EnumWindows(
                    Some(enum_windows_callback),
                    LPARAM(&mut windows as *mut _ as isize),
                );
            }

            // NOTE: this is for window
            // Extract to Send-able data
            let window_data: Vec<_> = windows
                .iter()
                .filter(|w| w.is_capturable)
                .map(|w| (w.hwnd.0 as isize, w.handle, w.title.clone(), w.rect))
                .collect();

            let window_sources: Vec<CaptureSource> = window_data
                .par_iter()
                .map(|(hwnd_ptr, handle, title, rect)| {
                    let hwnd = HWND(*hwnd_ptr as *mut _);
                    CaptureSource {
                        id: handle.to_string(),
                        title: title.clone(),
                        // thumbnail: capture_app_window(hwnd, 320, 180).unwrap_or_default(),
                        thumbnail: "".to_string(),
                        // TODO: handle cache icon
                        icon: get_window_icon(hwnd),
                        // icon: None,
                        source_type: "window".to_string(),
                        width: rect.right - rect.left,
                        height: rect.bottom - rect.top,
                    }
                })
                .collect();

            let monitors = get_monitors_info();
            // NOTE: this is for monitor
            let monitor_data: Vec<_> = monitors
                .iter()
                .map(|m| (m.hmonitor.0 as isize, m.device_name.clone(), m.rect.clone()))
                .collect();

            let monitor_sources: Vec<CaptureSource> = monitor_data
                .par_iter()
                .map(|(hmon_ptr, device_name, rect)| {
                    let hmonitor = HMONITOR(*hmon_ptr as *mut _);
                    CaptureSource {
                        id: format!("monitor_{}", device_name),
                        title: device_name.clone(),
                        thumbnail: capture_monitor_display(hmonitor, 320, 180).unwrap_or_default(),
                        // thumbnail: "".to_string(),
                        icon: None,
                        source_type: "monitor".to_string(),
                        width: rect.right - rect.left,
                        height: rect.bottom - rect.top,
                    }
                })
                .collect();

            let sources: Vec<CaptureSource> = [window_sources, monitor_sources].concat();

            let update = SourcesUpdate {
                sources,
                fps: fps as i32,
            };

            let _ = app.emit("share-screen-list", &update);
        }

        // Cleanup
        STREAM_REGISTRY.remove(&stream_id);
    });
}

#[tauri::command]
pub fn close_stream_list() {
    let stream_id = STREAM_ID.to_string();

    if let Some(entry) = STREAM_REGISTRY.get(&stream_id) {
        entry.store(true, Ordering::Relaxed);
    }
}

// ============== Share Screen Popup Window
unsafe fn is_process_alive(hwnd: HWND) -> bool {
    let mut pid: u32 = 0;
    GetWindowThreadProcessId(hwnd, Some(&mut pid));
    if pid == 0 {
        return false;
    }

    let handle: HANDLE = match OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid) {
        Ok(h) => h,
        Err(_) => return false,
    };

    let status = WaitForSingleObject(handle, 0);
    let _ = CloseHandle(handle);

    status != WAIT_OBJECT_0 // WAIT_OBJECT_0 means process exited
}

pub unsafe fn watchdog_loop(hwnd: HWND, stop: Arc<AtomicBool>) {
    loop {
        if stop.load(Ordering::Relaxed) {
            break;
        }

        let alive_window: BOOL = IsWindow(Some(hwnd));
        if !alive_window.as_bool() {
            break;
        }

        // optional visibility check
        if !IsWindowVisible(hwnd).as_bool() {
            break;
        }

        // optional process check (stronger)
        if !is_process_alive(hwnd) {
            break;
        }

        thread::sleep(Duration::from_millis(200));
    }

    // signal tracker to stop
    stop.store(true, Ordering::Relaxed);
}
// TODO: make this on AppState level
lazy_static::lazy_static! {
    static ref SHARE_SCREEN_STREAM_REGISTRY: Arc<DashMap<String, Arc<std::sync::atomic::AtomicBool>>> =
        Arc::new(DashMap::new());
}
const SHARE_SCREEN_STREAM_ID: &str = "capture_stream";
#[tauri::command]
pub fn start_share_screen(window: Window) {
    let tauri_hwnd = window.hwnd().expect("Failed to get HWND");

    // TODO: this is use on share screen popup window, need to change it later
    unsafe {
        MAIN_HWND = tauri_hwnd;
    }

    // Create stop flag
    let stream_id = SHARE_SCREEN_STREAM_ID.to_string();
    let should_stop_share_screen = Arc::new(std::sync::atomic::AtomicBool::new(false));
    SHARE_SCREEN_STREAM_REGISTRY.insert(stream_id.clone(), should_stop_share_screen.clone());

    // TODO: get rect size
    println!("=== MONITORS ===");
    let monitors = get_monitors();
    for (idx, (name, handle)) in monitors.iter().enumerate() {
        println!("Monitor {}: {} (Handle: {:?})", idx + 1, name, handle);
    }

    // TODO: draw full screen
    // NOTE: this track run not on main thread
    let stop_flag_window = should_stop_share_screen.clone();
    std::thread::spawn(|| unsafe {
        let monitors = get_monitors();
        if let Some((_, hmonitor)) = monitors.get(0) {
            if !hmonitor.is_invalid() {
                draw_overlay::track_monitor(*hmonitor, stop_flag_window);
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

    // for (idx, win) in windows.iter().enumerate() {
    //     println!(
    //         "Window {}: \"{}\" | Class: {} | Handle: {} | Capturable: {} | Rect: ({},{}) {}x{}",
    //         idx + 1,
    //         win.title,
    //         win.class_name,
    //         win.handle,
    //         win.is_capturable,
    //         win.rect.left,
    //         win.rect.top,
    //         win.rect.right - win.rect.left,
    //         win.rect.bottom - win.rect.top
    //     );
    // }

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
    let stop_flag_window = should_stop_share_screen.clone();
    std::thread::spawn(move || unsafe {
        if let Some(handle) = hwnd_to_track {
            let hwnd = HWND(handle as *mut _);
            let stop = stop_flag_window.clone();
            std::thread::spawn(move || {
                // TODO: add watch dog for make sure if the window is still valid / active
                watchdog_loop(HWND(handle as *mut _), stop);
            });
            draw_overlay::track_window(hwnd, stop_flag_window);
        }
    });

    // unsafe {
    //     draw_overlay::track_window(capturable[0].hwnd);
    // }
}

#[tauri::command]
pub fn close_share_screen() {
    let stream_id = SHARE_SCREEN_STREAM_ID.to_string();

    if let Some(entry) = SHARE_SCREEN_STREAM_REGISTRY.get(&stream_id) {
        entry.store(true, Ordering::Relaxed);
    }
}
