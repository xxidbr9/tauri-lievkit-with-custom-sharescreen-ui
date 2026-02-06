// #[tauri::command]
// pub fn stream_list(window: Window, app: AppHandle, fps: Option<u64>) {
//     let tauri_hwnd = window.hwnd().expect("Failed to get HWND");

//     // TODO: using dashmap
//     unsafe {
//         MAIN_HWND = tauri_hwnd;
//     }

//     let fps = fps.unwrap_or(24);
//     let interval = Duration::from_millis(1000 / fps);
//     let stream_id = STREAM_ID.to_string();

//     // Create stop flag
//     let should_stop = Arc::new(std::sync::atomic::AtomicBool::new(false));
//     STREAM_REGISTRY.insert(stream_id.clone(), should_stop.clone());

//     std::thread::spawn(move || {
//         while !should_stop.load(std::sync::atomic::Ordering::Relaxed) {
//             let mut windows: Vec<DisplayInfo> = Vec::new();

//             unsafe {
//                 let _ = EnumWindows(
//                     Some(enum_windows_callback),
//                     LPARAM(&mut windows as *mut _ as isize),
//                 );
//             }
//             let start = std::time::Instant::now();

//             let sources: Vec<CaptureSource> = windows
//                 .iter()
//                 .filter(|w| w.is_capturable)
//                 .map(|w| CaptureSource {
//                     id: w.handle.to_string(),
//                     title: w.title.clone(),
//                     thumbnail: capture_window(w.hwnd, 320, 180).unwrap_or_default(),
//                     icon: get_window_icon(w.hwnd),
//                     source_type: "window".to_string(),
//                     width: w.rect.right - w.rect.left,
//                     height: w.rect.bottom - w.rect.top,
//                 })
//                 .collect();

//             let elapsed = start.elapsed().as_millis();
//             let update = SourcesUpdate {
//                 sources,
//                 fps: elapsed as i32,
//             };

//             let _ = app.emit("share-screen-list", &update);

//             // DEBUG FPS

//             // let _ = app.emit("debug-stream-fps", &elapsed);

//             std::thread::sleep(interval);
//         }

//         // Cleanup
//         STREAM_REGISTRY.remove(&stream_id);
//     });
// }

// #[tauri::command]
// pub fn close_stream_list() {
//     let stream_id = STREAM_ID.to_string();

//     if let Some(entry) = STREAM_REGISTRY.get(&stream_id) {
//         entry.store(true, std::sync::atomic::Ordering::Relaxed);
//     }
// }
