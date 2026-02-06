use windows::Win32::Foundation::{HWND, RECT};

#[derive(Clone)]
pub struct MonitorRect {
    pub left: i32,
    pub top: i32,
    pub right: i32,
    pub bottom: i32,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct CaptureSource {
    pub id: String,
    pub title: String,
    pub thumbnail: String,
    pub icon: Option<String>,
    pub source_type: String,
    pub width: i32,
    pub height: i32,
}

#[derive(Debug)]
pub struct DisplayInfo {
    pub hwnd: HWND,
    pub handle: isize,
    pub title: String,
    pub class_name: String,
    pub rect: RECT,
    pub is_capturable: bool,
}

#[derive(Clone, serde::Serialize)]
pub struct SourcesUpdate {
    pub sources: Vec<CaptureSource>,
}
