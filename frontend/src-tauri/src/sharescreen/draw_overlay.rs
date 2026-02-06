use std::{panic::AssertUnwindSafe, ptr, thread, time::Duration};
use windows::Win32::{
    Foundation::{COLORREF, HWND, LPARAM, LRESULT, RECT, WPARAM},
    Graphics::Gdi::{
        CreatePen, CreateSolidBrush, DeleteObject, GetDC, GetMonitorInfoW, InvalidateRect,
        Rectangle, ReleaseDC, SelectObject, UpdateWindow, HMONITOR, MONITORINFO, PS_SOLID,
    },
    System::LibraryLoader::GetModuleHandleW,
    UI::WindowsAndMessaging::*,
};
use windows_core::PCWSTR;

use crate::sharescreen::dto::MonitorRect;

pub unsafe fn draw_border(hwnd: HWND, width: i32, height: i32, color: COLORREF, thickness: i32) {
    let hdc = GetDC(Some(hwnd));

    let pen = CreatePen(PS_SOLID, thickness, color);
    let brush = CreateSolidBrush(COLORREF(0));

    // Convert HPEN/HBRUSH to HGDIOBJ
    let old_pen = SelectObject(hdc, pen.into());
    let old_brush = SelectObject(hdc, brush.into());

    let _ = Rectangle(hdc, 0, 0, width, height);

    // Restore old objects
    SelectObject(hdc, old_pen);
    SelectObject(hdc, old_brush);

    // Delete created objects
    let _ = DeleteObject(pen.into());
    let _ = DeleteObject(brush.into());

    let _ = ReleaseDC(Some(hwnd), hdc);
}

fn to_pcwstr(s: &str) -> PCWSTR {
    let mut v: Vec<u16> = s.encode_utf16().collect();
    v.push(0);
    PCWSTR(v.as_ptr())
}

unsafe extern "system" fn window_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    DefWindowProcW(hwnd, msg, wparam, lparam)
}

pub unsafe fn create_overlay(left: i32, top: i32, width: i32, height: i32) -> HWND {
    let class_name = "OverlayWindow";
    let hinstance = GetModuleHandleW(None).unwrap();

    let wnd_class = WNDCLASSW {
        lpfnWndProc: Some(window_proc),
        hInstance: hinstance.into(),
        lpszClassName: to_pcwstr(class_name),
        ..Default::default()
    };

    let _ = RegisterClassW(&wnd_class);

    let hwnd = CreateWindowExW(
        WS_EX_LAYERED | WS_EX_TOPMOST | WS_EX_TOOLWINDOW | WS_EX_TRANSPARENT,
        to_pcwstr(class_name),
        to_pcwstr("Overlay"),
        WS_POPUP | WS_VISIBLE,
        left,
        top,
        width,
        height,
        None,
        None,
        Some(hinstance.into()),
        Some(ptr::null_mut()),
    )
    .expect("Failed to create overlay window");

    let _ = SetLayeredWindowAttributes(hwnd, COLORREF(0), 255, LWA_COLORKEY);
    let _ = ShowWindow(hwnd, SW_SHOW);
    let _ = UpdateWindow(hwnd);

    hwnd
}

unsafe fn run_tracking_loop(hwnd_target: HWND) {
    let mut rect = RECT::default();

    if GetWindowRect(hwnd_target, &mut rect).is_err() {
        eprintln!("GetWindowRect failed at start");
        return;
    }

    let hwnd_overlay = create_overlay(
        rect.left,
        rect.top,
        rect.right - rect.left,
        rect.bottom - rect.top,
    );

    loop {
        let step = std::panic::catch_unwind(AssertUnwindSafe(|| {
            if GetWindowRect(hwnd_target, &mut rect).is_ok() {
                let width = rect.right - rect.left;
                let height = rect.bottom - rect.top;

                if width <= 0 || height <= 0 {
                    return;
                }

                let _ = SetWindowPos(
                    hwnd_overlay,
                    Some(HWND_TOPMOST),
                    rect.left,
                    rect.top,
                    width,
                    height,
                    SWP_NOACTIVATE | SWP_SHOWWINDOW,
                );

                let _ = InvalidateRect(Some(hwnd_overlay), None, true);

                draw_border(hwnd_overlay, width, height, COLORREF(0x81B910), 4);
            }
        }));

        if step.is_err() {
            eprintln!("track_window loop panic recovered");
        }

        thread::sleep(Duration::from_millis(50));
    }
}

// TODO: error handling
pub unsafe fn track_window(hwnd_target: HWND) {
    let _ = std::panic::catch_unwind(AssertUnwindSafe(|| {
        run_tracking_loop(hwnd_target);
    }));
}

fn get_monitor_rect(hmonitor: HMONITOR) -> Option<MonitorRect> {
    let mut info = MONITORINFO {
        cbSize: std::mem::size_of::<MONITORINFO>() as u32,
        ..Default::default()
    };

    unsafe {
        if GetMonitorInfoW(hmonitor, &mut info).as_bool() {
            Some(MonitorRect {
                left: info.rcMonitor.left,
                top: info.rcMonitor.top,
                right: info.rcMonitor.right,
                bottom: info.rcMonitor.bottom,
            })
        } else {
            None
        }
    }
}

/// Track a monitor using its HMONITOR
pub unsafe fn track_monitor(hmonitor: HMONITOR) {
    // Get monitor rectangle
    let rect = get_monitor_rect(hmonitor).expect("Failed to get monitor rect");

    // Create overlay covering the monitor
    let hwnd_overlay = create_overlay(
        rect.left,
        rect.top,
        rect.right - rect.left,
        rect.bottom - rect.top,
    );

    let _ = std::panic::catch_unwind(|| loop {
        // Optional: refresh monitor rect in case of resolution changes
        if let Some(rect) = get_monitor_rect(hmonitor) {
            let width = rect.right - rect.left;
            let height = rect.bottom - rect.top;

            let _ = SetWindowPos(
                hwnd_overlay,
                Some(HWND_TOPMOST),
                rect.left,
                rect.top,
                width,
                height,
                SWP_NOACTIVATE | SWP_SHOWWINDOW,
            );

            let _ = InvalidateRect(Some(hwnd_overlay), None, true);

            draw_border(hwnd_overlay, width, height, COLORREF(0x81B910), 4);
        }

        thread::sleep(Duration::from_millis(60));
    });
}

// TODO: add impl for Overlay, so it easier to handle new and close it

// // TODO: close the overlay window
// pub unsafe fn close_window_overlay(hwnd: HWND) {
//     DestroyWindow(hwnd);
// }

// pub unsafe fn close_monitor_overlay(hwnd: HWND) {
//     DestroyWindow(hwnd);
// }
