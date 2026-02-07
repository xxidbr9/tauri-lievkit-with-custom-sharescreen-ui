use crate::share_screen::dto::{
    CaptureConfig, CaptureError, CaptureSourceType, MonitorInfo, MonitorRect, Result,
};
use windows::Win32::{
    Foundation::*, Graphics::Gdi::*, UI::WindowsAndMessaging::MONITORINFOF_PRIMARY,
};
use windows_core::BOOL;

#[derive(Clone)]
pub struct MonitorCapture;

impl MonitorCapture {
    pub fn new() -> Self {
        Self
    }

    pub fn enumerate(&self) -> Result<Vec<MonitorInfo>> {
        let mut monitors = Vec::new();

        unsafe {
            let _ = EnumDisplayMonitors(
                None,
                None,
                Some(enum_monitor_callback),
                LPARAM(&mut monitors as *mut Vec<MonitorInfo> as isize),
            );
        }

        Ok(monitors)
    }

    pub fn get_info(&self, hmonitor: isize) -> Result<MonitorInfo> {
        unsafe {
            let hmonitor = HMONITOR(hmonitor as *mut _);

            let mut info = MONITORINFOEXW {
                monitorInfo: MONITORINFO {
                    cbSize: std::mem::size_of::<MONITORINFOEXW>() as u32,
                    ..Default::default()
                },
                ..Default::default()
            };

            let ok = GetMonitorInfoW(
                hmonitor,
                &mut info.monitorInfo as *mut _ as *mut MONITORINFO,
            )
            .as_bool();

            if !ok {
                return Err(CaptureError::PlatformError(
                    "Failed to get monitor info".to_string(),
                ));
            }

            let name = String::from_utf16_lossy(&info.szDevice);
            let rect = info.monitorInfo.rcMonitor;

            Ok(MonitorInfo {
                hmonitor: hmonitor.0 as isize,
                name,
                width: rect.right - rect.left,
                height: rect.bottom - rect.top,
                x: rect.left,
                y: rect.top,
                is_primary: (info.monitorInfo.dwFlags & MONITORINFOF_PRIMARY) != 0,
            })
        }
    }

    pub async fn capture_thumbnail(
        &self,
        hmonitor: isize,
        width: i32,
        height: i32,
    ) -> Result<Vec<u8>> {
        super::windows::capture_single_frame_internal(
            CaptureSourceType::Monitor(hmonitor),
            width,
            height,
        )
        .await
    }

    pub async fn start_capture(
        &self,
        hmonitor: isize,
        config: CaptureConfig,
        video_tx: tokio::sync::mpsc::Sender<Vec<u8>>,
    ) -> Result<()> {
        super::windows::start_capture_internal(
            CaptureSourceType::Monitor(hmonitor),
            config,
            video_tx,
        )
        .await
    }
}

unsafe extern "system" fn enum_monitor_callback(
    hmonitor: HMONITOR,
    _: HDC,
    _: *mut RECT,
    lparam: LPARAM,
) -> BOOL {
    unsafe {
        let monitors = &mut *(lparam.0 as *mut Vec<MonitorInfo>);

        let mut info = MONITORINFOEXW {
            monitorInfo: MONITORINFO {
                cbSize: std::mem::size_of::<MONITORINFOEXW>() as u32,
                ..Default::default()
            },
            ..Default::default()
        };

        let ok = GetMonitorInfoW(hmonitor, &mut info.monitorInfo).as_bool();

        if ok {
            let name = String::from_utf16_lossy(
                &info.szDevice[..info.szDevice.iter().position(|&c| c == 0).unwrap_or(0)],
            );

            let rect = info.monitorInfo.rcMonitor;

            monitors.push(MonitorInfo {
                hmonitor: hmonitor.0 as isize,
                name,
                width: rect.right - rect.left,
                height: rect.bottom - rect.top,
                x: rect.left,
                y: rect.top,
                is_primary: (info.monitorInfo.dwFlags & MONITORINFOF_PRIMARY) != 0,
            });
        }

        true.into()
    }
}

pub unsafe fn get_monitor_rects() -> Vec<MonitorRect> {
    let mut rects = Vec::new();

    unsafe extern "system" fn callback(
        hmonitor: HMONITOR,
        _: HDC,
        _: *mut RECT,
        lparam: LPARAM,
    ) -> BOOL {
        unsafe {
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
