use std::panic;
use tauri::{AppHandle, Emitter};

pub fn setup(app_handle: AppHandle) {
    let handle = app_handle.clone(); // owned, 'static

    panic::set_hook(Box::new(move |info| {
        let msg = if let Some(s) = info.payload().downcast_ref::<&str>() {
            s.to_string()
        } else if let Some(s) = info.payload().downcast_ref::<String>() {
            s.clone()
        } else {
            "Rust panic occurred".to_string()
        };

        let location = info
            .location()
            .map(|l| format!("{}:{}", l.file(), l.line()))
            .unwrap_or_else(|| "unknown".into());

        let full = format!("{} @ {}", msg, location);

        let _ = handle.emit("rust-panic", full);
    }));
}
