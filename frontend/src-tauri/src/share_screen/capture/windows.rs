use crate::share_screen::dto::{
    AudioDevice, CaptureConfig, CaptureError, CaptureSourceType, Result, WindowInfo,
};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use vpx_encode::{Config as VpxConfig, Encoder, VideoCodecId};
use windows::Win32::Graphics::Direct3D::D3D_DRIVER_TYPE_HARDWARE;
use windows::Win32::Graphics::Gdi::HMONITOR;
use windows::Win32::System::WinRT::Direct3D11::{
    CreateDirect3D11DeviceFromDXGIDevice, IDirect3DDxgiInterfaceAccess,
};
use windows::Win32::System::WinRT::Graphics::Capture::IGraphicsCaptureItemInterop;
use windows::core::{Interface, Ref};
use windows::{
    Foundation::TypedEventHandler, Graphics::Capture::*, Graphics::DirectX::Direct3D11::*,
    Win32::Foundation::*, Win32::Graphics::Direct3D11::*, Win32::Graphics::Dxgi::Common::*,
    Win32::Graphics::Dxgi::*, Win32::Media::Audio::*, Win32::System::Com::*,
    Win32::UI::WindowsAndMessaging::*, core::*,
};

#[derive(Clone)]
pub struct WindowCapture;

impl WindowCapture {
    pub fn new() -> Self {
        Self
    }

    // TODO: handle hide non windows app and self
    // TODO: using watch dog to know what apps are closing
    pub fn enumerate(&self) -> Result<Vec<WindowInfo>> {
        let mut windows = Vec::new();

        unsafe {
            EnumWindows(
                Some(enum_window_callback),
                LPARAM(&mut windows as *mut Vec<WindowInfo> as isize),
            )
            .map_err(|e| CaptureError::PlatformError(e.to_string()))?;
        }

        Ok(windows)
    }

    pub fn get_info(&self, hwnd: isize) -> Result<WindowInfo> {
        unsafe {
            let hwnd = HWND(hwnd as *mut _);

            if !IsWindowVisible(hwnd).as_bool() {
                return Err(CaptureError::SourceNotFound(
                    "Window not visible".to_string(),
                ));
            }

            let mut text = [0u16; 512];
            let len = GetWindowTextW(hwnd, &mut text);
            let title = String::from_utf16_lossy(&text[..len as usize]);

            let mut rect = RECT::default();
            GetWindowRect(hwnd, &mut rect)
                .map_err(|e| CaptureError::PlatformError(e.to_string()))?;

            let icon = extract_window_icon(hwnd).ok();

            Ok(WindowInfo {
                hwnd: hwnd.0 as isize,
                title,
                width: (rect.right - rect.left),
                height: (rect.bottom - rect.top),
                icon,
                // TODO: handle this
                is_capturable: None,
            })
        }
    }

    pub async fn capture_thumbnail(&self, hwnd: isize, width: i32, height: i32) -> Result<Vec<u8>> {
        capture_single_frame_internal(CaptureSourceType::Window(hwnd), width, height).await
    }

    pub async fn start_capture(
        &self,
        hwnd: isize,
        config: CaptureConfig,
        video_tx: tokio::sync::mpsc::Sender<Vec<u8>>,
    ) -> Result<()> {
        start_capture_internal(CaptureSourceType::Window(hwnd), config, video_tx).await
    }
}

unsafe extern "system" fn enum_window_callback(hwnd: HWND, lparam: LPARAM) -> BOOL {
    unsafe {
        let windows = &mut *(lparam.0 as *mut Vec<WindowInfo>);

        if !IsWindowVisible(hwnd).as_bool() {
            return true.into();
        }

        let mut text = [0u16; 512];
        let len = GetWindowTextW(hwnd, &mut text);

        if len == 0 {
            return true.into();
        }

        let title = String::from_utf16_lossy(&text[..len as usize]);

        if title.is_empty() {
            return true.into();
        }

        let mut class_name = [0u16; 256];
        let _ = GetClassNameW(hwnd, &mut class_name);

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

        let mut rect = RECT::default();
        let _ = GetWindowRect(hwnd, &mut rect);

        let width = rect.right - rect.left;
        let height = rect.bottom - rect.top;

        if width < 100 || height < 100 {
            return true.into();
        }

        let intersects_any_monitor = super::monitor::get_monitor_rects().iter().any(|m| {
            rect.left < m.right && rect.right > m.left && rect.top < m.bottom && rect.bottom > m.top
        });

        let is_capturable = !title.is_empty()
            && rect.right - rect.left > 0
            && rect.bottom - rect.top > 0
            && !is_cloaked
            && (ex_style & WS_EX_TOOLWINDOW.0) == 0
            && class_str != "Windows.UI.Core.CoreWindow"
            && class_str != "ApplicationFrameWindow"
            && !class_str.starts_with("RainmeterMeterWindow")
            && class_str != "CEF-OSC-WIDGET"
            && title != "Program Manager"
            && intersects_any_monitor;

        // TODO: make sure not showing own windows and sharing popup windows
        // NOTE: require using global state
        // && hwnd != MAIN_HWND
        // && hwnd != SHARE_SCREEN_POPUP_HWND

        let icon = extract_window_icon(hwnd).ok();

        if is_capturable {
            windows.push(WindowInfo {
                hwnd: hwnd.0 as isize,
                title,
                width,
                height,
                icon,
                is_capturable: Some(is_capturable),
            });
        }

        true.into()
    }
}

// TODO: Implement icon extraction
unsafe fn extract_window_icon(hwnd: HWND) -> Result<Vec<u8>> {
    // Get window icon and convert to PNG bytes
    Ok(vec![])
}

pub fn enumerate_audio_devices() -> Result<Vec<AudioDevice>> {
    unsafe {
        // CoInitializeEx(None, COINIT_MULTITHREADED)
        //     .map_err(|e| CaptureError::PlatformError(e.to_string()))?;
        let ok = CoInitializeEx(None, COINIT_MULTITHREADED).is_ok();
        if !ok {
            return Err(CaptureError::PlatformError(
                "CoInitializeEx failed".to_string(),
            ));
        }

        let enumerator: IMMDeviceEnumerator =
            CoCreateInstance(&MMDeviceEnumerator, None, CLSCTX_ALL)
                .map_err(|e| CaptureError::PlatformError(e.to_string()))?;

        let collection = enumerator
            .EnumAudioEndpoints(eRender, DEVICE_STATE_ACTIVE)
            .map_err(|e| CaptureError::PlatformError(e.to_string()))?;

        let count = collection
            .GetCount()
            .map_err(|e| CaptureError::PlatformError(e.to_string()))?;

        let default_device = enumerator.GetDefaultAudioEndpoint(eRender, eConsole).ok();

        let mut devices = Vec::new();

        for i in 0..count {
            if let Ok(device) = collection.Item(i) {
                if let Ok(id) = device.GetId() {
                    let id_str = id
                        .to_string()
                        .map_err(|e| CaptureError::PlatformError(format!("{:?}", e)))?;

                    let is_default = if let Some(ref def) = default_device {
                        if let Ok(def_id) = def.GetId() {
                            def_id.to_string().ok() == Some(id_str.clone())
                        } else {
                            false
                        }
                    } else {
                        false
                    };

                    devices.push(AudioDevice {
                        id: id_str,
                        name: format!("Audio Device {}", i),
                        is_default,
                    });
                }
            }
        }

        Ok(devices)
    }
}

// TODO: this is need to broadcast to WebRTC
pub async fn capture_single_frame_internal(
    source_type: CaptureSourceType,
    width: i32,
    height: i32,
) -> Result<Vec<u8>> {
    use windows::Win32::System::WinRT::*;
    if let CaptureSourceType::Window(hwnd) = source_type {
        unsafe {
            let hwnd = HWND(hwnd as *mut _);
            // This prevents the yellow border from appearing
            let _ = SetWindowDisplayAffinity(hwnd, WDA_EXCLUDEFROMCAPTURE);
            // Wait a bit for the change to take effect
            tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
        }
    }
    unsafe {
        RoInitialize(RO_INIT_MULTITHREADED)
            .map_err(|e| CaptureError::PlatformError(e.to_string()))?;

        let (device, context) = create_d3d11_device()?;
        let d3d_device = create_winrt_device(&device)?;

        let item = match source_type {
            CaptureSourceType::Window(hwnd) => create_capture_item_window(hwnd)?,
            CaptureSourceType::Monitor(hmonitor) => create_capture_item_monitor(hmonitor)?,
        };

        let size = item
            .Size()
            .map_err(|e| CaptureError::PlatformError(e.to_string()))?;

        let frame_pool = Direct3D11CaptureFramePool::CreateFreeThreaded(
            &d3d_device,
            windows::Graphics::DirectX::DirectXPixelFormat::B8G8R8A8UIntNormalized,
            1,
            size,
        )
        .map_err(|e| CaptureError::PlatformError(e.to_string()))?;

        let session = frame_pool
            .CreateCaptureSession(&item)
            .map_err(|e| CaptureError::PlatformError(e.to_string()))?;

        session
            .SetIsBorderRequired(false)
            .map_err(|e| CaptureError::PlatformError(e.to_string()))?;

        session
            .StartCapture()
            .map_err(|e| CaptureError::PlatformError(e.to_string()))?;

        // Wait for frame
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        let frame = frame_pool
            .TryGetNextFrame()
            .map_err(|e| CaptureError::PlatformError(e.to_string()))?;

        let surface = frame
            .Surface()
            .map_err(|e| CaptureError::PlatformError(e.to_string()))?;

        let access: IDirect3DDxgiInterfaceAccess = surface
            .cast()
            .map_err(|e| CaptureError::PlatformError(e.to_string()))?;

        let texture: ID3D11Texture2D = access
            .GetInterface()
            .map_err(|e| CaptureError::PlatformError(e.to_string()))?;

        let resized = resize_texture_gpu(&device, &context, &texture, width as u32, height as u32)?;
        let bytes = texture_to_bytes(&context, &resized)?;

        session
            .Close()
            .map_err(|e| CaptureError::PlatformError(e.to_string()))?;

        Ok(bytes)
    }
}

// TODO: handle panic unwind here
pub async fn start_capture_internal(
    source_type: CaptureSourceType,
    config: CaptureConfig,
    video_tx: tokio::sync::mpsc::Sender<Vec<u8>>,
) -> Result<()> {
    use windows::Win32::System::WinRT::*;

    unsafe {
        RoInitialize(RO_INIT_MULTITHREADED)
            .map_err(|e| CaptureError::PlatformError(e.to_string()))?;

        let (device, context) = create_d3d11_device()?;
        let d3d_device = create_winrt_device(&device)?;

        let item = match source_type {
            CaptureSourceType::Window(hwnd) => create_capture_item_window(hwnd)?,
            CaptureSourceType::Monitor(hmonitor) => create_capture_item_monitor(hmonitor)?,
        };

        let size = item
            .Size()
            .map_err(|e| CaptureError::PlatformError(e.to_string()))?;

        let frame_pool = Direct3D11CaptureFramePool::CreateFreeThreaded(
            &d3d_device,
            windows::Graphics::DirectX::DirectXPixelFormat::B8G8R8A8UIntNormalized,
            2,
            size,
        )
        .map_err(|e| CaptureError::PlatformError(e.to_string()))?;

        let session = frame_pool
            .CreateCaptureSession(&item)
            .map_err(|e| CaptureError::PlatformError(e.to_string()))?;

        session
            .SetIsCursorCaptureEnabled(true)
            .map_err(|e| CaptureError::PlatformError(e.to_string()))?;

        session
            .SetIsBorderRequired(config.withborder.unwrap())
            .map_err(|e| CaptureError::PlatformError(e.to_string()))?;

        // Create channel for raw frames
        let (frame_tx, frame_rx) = std::sync::mpsc::sync_channel::<(Vec<u8>, u64)>(10);

        let target_frame_time = Duration::from_secs_f64(1.0 / config.fps as f64);
        let last_frame_time = Arc::new(std::sync::Mutex::new(Instant::now()));
        let frame_counter = Arc::new(std::sync::atomic::AtomicU64::new(0));

        let device_clone = device.clone();
        let context_clone = context.clone();
        let last_frame_time_clone = last_frame_time.clone();
        let frame_counter_clone = frame_counter.clone();
        let config_clone = config.clone();

        // Frame Arrived Handler
        frame_pool
            .FrameArrived(&TypedEventHandler::new(
                move |pool_ref: Ref<Direct3D11CaptureFramePool>, _| {
                    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                        // {
                        //     let mut last_time = last_frame_time_clone.lock().unwrap();
                        //     let now = Instant::now();
                        //     if now.duration_since(*last_time) < target_frame_time {
                        //         return Ok(());
                        //     }
                        //     *last_time = now;
                        // }

                        let frame_num =
                            frame_counter_clone.fetch_add(1, std::sync::atomic::Ordering::Relaxed);

                        let pool: &Direct3D11CaptureFramePool = match pool_ref.as_ref() {
                            Some(p) => p,
                            None => return Ok(()),
                        };

                        let frame = match pool.TryGetNextFrame() {
                            Ok(f) => f,
                            Err(_) => return Ok(()),
                        };

                        let surface = match frame.Surface() {
                            Ok(s) => s,
                            Err(_) => return Ok(()),
                        };

                        let access = match surface.cast::<IDirect3DDxgiInterfaceAccess>() {
                            Ok(a) => a,
                            Err(_) => return Ok(()),
                        };

                        let texture = match access.GetInterface::<ID3D11Texture2D>() {
                            Ok(t) => t,
                            Err(_) => return Ok(()),
                        };

                        let resized = match resize_texture_gpu(
                            &device_clone,
                            &context_clone,
                            &texture,
                            config_clone.width as u32,
                            config_clone.height as u32,
                        ) {
                            Ok(r) => r,
                            Err(e) => {
                                if frame_num % 30 == 0 {
                                    eprintln!("[Capture] Resize error: {:?}", e);
                                }
                                return Ok(());
                            }
                        };

                        let bgra_bytes = match texture_to_bytes(&context_clone, &resized) {
                            Ok(b) => b,
                            Err(e) => {
                                if frame_num % 30 == 0 {
                                    eprintln!("[Capture] Texture read error: {:?}", e);
                                }
                                return Ok(());
                            }
                        };

                        // DEBUG: Log captured frame
                        // if frame_num % 30 == 0 {
                        //     println!(
                        //         "[Capture] Captured frame {}, size: {} bytes",
                        //         frame_num,
                        //         bgra_bytes.len()
                        //     );
                        // }

                        // Send raw frame to encoder task
                        let _ = frame_tx.try_send((bgra_bytes, frame_num));
                        // Ok(_) => {
                        //     if frame_num % 30 == 0 {
                        //         println!("[Capture] ✓ Frame {} sent to encoder", frame_num);
                        //     }
                        // }
                        // Err(e) => {
                        //     if frame_num % 100 == 0 {
                        //         eprintln!("[Capture] Frame send error: {:?}", e);
                        //     }
                        // }

                        Ok(())
                    }));

                    match result {
                        Ok(r) => r,
                        Err(e) => {
                            eprintln!("[Capture] Handler panic: {:?}", e);
                            Ok(())
                        }
                    }
                },
            ))
            .map_err(|e| CaptureError::PlatformError(e.to_string()))?;

        // Encoder Task
        let encoder_config = VpxConfig {
            width: config.width as u32,
            height: config.height as u32,
            timebase: [1, config.fps as i32],
            bitrate: 1000,
            codec: VideoCodecId::VP8,
        };

        let width = config.width as usize;
        let height = config.height as usize;

        tokio::task::spawn_blocking(move || {
            // println!("[Encode] Encoder task starting...");

            let mut encoder = match Encoder::new(encoder_config) {
                Ok(e) => {
                    // println!("[Encode] ✓ VP8 encoder created successfully");
                    e
                }
                Err(e) => {
                    eprintln!("[Encode] ✗ Failed to create VP8 encoder: {:?}", e);
                    return;
                }
            };

            // println!("[Encode] Waiting for frames...");

            // Block and process frames
            loop {
                match frame_rx.recv() {
                    Ok((bgra_bytes, frame_num)) => {
                        // if frame_num % 30 == 0 {
                        //     println!(
                        //         "[Encode] Received frame {} for encoding, size: {} bytes",
                        //         frame_num,
                        //         bgra_bytes.len()
                        //     );
                        // }

                        // Convert BGRA to I420
                        let i420_data = bgra_to_i420(&bgra_bytes, width, height);

                        // if frame_num % 30 == 0 {
                        //     println!(
                        //         "[Encode] Converted to I420, size: {} bytes",
                        //         i420_data.len()
                        //     );
                        // }

                        // Encode
                        match encoder.encode(frame_num as i64, &i420_data) {
                            Ok(packets) => {
                                // if frame_num % 30 == 0 {
                                //     println!(
                                //         "[Encode] Encoded frame {}, got packets",
                                //         frame_num,
                                //         // packets.len()
                                //     );
                                // }

                                for packet in packets {
                                    // if frame_num % 30 == 0 {
                                    //     println!(
                                    //         "[Encode] Packet: {} bytes (keyframe: {})",
                                    //         packet.data.len(),
                                    //         packet.key
                                    //     );
                                    // }

                                    let data = packet.data.to_vec();

                                    // Send to WebRTC
                                    match video_tx.blocking_send(data) {
                                        Ok(_) => {
                                            if frame_num % 30 == 0 {
                                                println!("[Encode] ✓ Sent VP8 packet to WebRTC");
                                            }
                                        }
                                        Err(e) => {
                                            eprintln!("[Encode] ✗ Failed to send to WebRTC: {}", e);
                                        }
                                    }
                                }
                            }
                            Err(e) => {
                                if frame_num % 30 == 0 {
                                    eprintln!("[Encode] VP8 encode error: {:?}", e);
                                }
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!("[Encode] Channel closed: {:?}", e);
                        break;
                    }
                }
            }

            println!("[Encode] Encoder task ended");
        });

        session
            .StartCapture()
            .map_err(|e| CaptureError::PlatformError(e.to_string()))?;

        loop {
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        }
    }
}

unsafe fn create_d3d11_device() -> Result<(ID3D11Device, ID3D11DeviceContext)> {
    unsafe {
        let mut device: Option<ID3D11Device> = None;
        let mut context: Option<ID3D11DeviceContext> = None;

        D3D11CreateDevice(
            None,
            D3D_DRIVER_TYPE_HARDWARE,
            HMODULE::default(),
            D3D11_CREATE_DEVICE_BGRA_SUPPORT,
            None,
            D3D11_SDK_VERSION,
            Some(&mut device),
            None,
            Some(&mut context),
        )
        .map_err(|e| CaptureError::PlatformError(e.to_string()))?;

        Ok((device.unwrap(), context.unwrap()))
    }
}

unsafe fn create_winrt_device(device: &ID3D11Device) -> Result<IDirect3DDevice> {
    unsafe {
        let dxgi_device: IDXGIDevice = device
            .cast()
            .map_err(|e| CaptureError::PlatformError(e.to_string()))?;

        let inspectable = CreateDirect3D11DeviceFromDXGIDevice(&dxgi_device)
            .map_err(|e| CaptureError::PlatformError(e.to_string()))?;

        let d3d_device: IDirect3DDevice = inspectable
            .cast()
            .map_err(|e| CaptureError::PlatformError(e.to_string()))?;

        Ok(d3d_device)
    }
}

unsafe fn create_capture_item_window(hwnd: isize) -> Result<GraphicsCaptureItem> {
    unsafe {
        let interop = windows::core::factory::<GraphicsCaptureItem, IGraphicsCaptureItemInterop>()
            .map_err(|e| CaptureError::PlatformError(e.to_string()))?;

        interop
            .CreateForWindow(HWND(hwnd as *mut _))
            .map_err(|e| CaptureError::PlatformError(e.to_string()))
    }
}

unsafe fn create_capture_item_monitor(hmonitor: isize) -> Result<GraphicsCaptureItem> {
    unsafe {
        let interop = windows::core::factory::<GraphicsCaptureItem, IGraphicsCaptureItemInterop>()
            .map_err(|e| CaptureError::PlatformError(e.to_string()))?;

        interop
            .CreateForMonitor(HMONITOR(hmonitor as *mut _))
            .map_err(|e| CaptureError::PlatformError(e.to_string()))
    }
}

// TODO: handle resize texture gpu
unsafe fn resize_texture_gpu(
    device: &ID3D11Device,
    context: &ID3D11DeviceContext,
    src: &ID3D11Texture2D,
    width: u32,
    height: u32,
) -> Result<ID3D11Texture2D> {
    unsafe {
        let desc = D3D11_TEXTURE2D_DESC {
            Width: width,
            Height: height,
            MipLevels: 1,
            ArraySize: 1,
            Format: DXGI_FORMAT_B8G8R8A8_UNORM,
            SampleDesc: DXGI_SAMPLE_DESC {
                Count: 1,
                Quality: 0,
            },
            Usage: D3D11_USAGE_DEFAULT,
            BindFlags: D3D11_BIND_RENDER_TARGET.0 as u32,
            CPUAccessFlags: D3D11_CPU_ACCESS_FLAG(0).0 as u32,
            MiscFlags: D3D11_RESOURCE_MISC_FLAG(0).0 as u32,
        };

        let mut dst: Option<ID3D11Texture2D> = None;
        device
            .CreateTexture2D(&desc, None, Some(&mut dst))
            .map_err(|e| CaptureError::PlatformError(e.to_string()))?;

        let dst = dst.unwrap();

        // Use Video Processor for GPU resize
        // let mut video_device: Option<ID3D11VideoDevice> = None;
        // device
        //     .QueryInterface(&mut video_device)
        //     .map_err(|e| CaptureError::PlatformError(e.to_string()))?;

        let video_device: ID3D11VideoDevice = device
            .cast()
            .map_err(|e| CaptureError::PlatformError(e.to_string()))?;

        // TODO: Implementation video resize processor...

        Ok(dst)
    }
}

unsafe fn texture_to_bytes(
    context: &ID3D11DeviceContext,
    texture: &ID3D11Texture2D,
) -> Result<Vec<u8>> {
    unsafe {
        let mut desc = D3D11_TEXTURE2D_DESC::default();
        texture.GetDesc(&mut desc);

        desc.Usage = D3D11_USAGE_STAGING;
        desc.BindFlags = D3D11_BIND_FLAG(0).0 as u32;
        desc.CPUAccessFlags = D3D11_CPU_ACCESS_READ.0 as u32;
        desc.MiscFlags = D3D11_RESOURCE_MISC_FLAG(0).0 as u32;

        let device: ID3D11Device = context
            .GetDevice()
            .map_err(|e| CaptureError::PlatformError(e.to_string()))?;

        let mut staging: Option<ID3D11Texture2D> = None;
        device
            .CreateTexture2D(&desc, None, Some(&mut staging))
            .map_err(|e| CaptureError::PlatformError(e.to_string()))?;

        let staging = staging.unwrap();

        context.CopyResource(&staging, texture);

        let mut mapped = D3D11_MAPPED_SUBRESOURCE::default();
        context
            .Map(&staging, 0, D3D11_MAP_READ, 0, Some(&mut mapped))
            .map_err(|e| CaptureError::PlatformError(e.to_string()))?;

        let size = (desc.Height * mapped.RowPitch) as usize;
        let data = std::slice::from_raw_parts(mapped.pData as *const u8, size);
        let result = data.to_vec();

        context.Unmap(&staging, 0);

        Ok(result)
    }
}

// Convert BGRA to I420 (YUV420p)
fn bgra_to_i420(bgra: &[u8], width: usize, height: usize) -> Vec<u8> {
    let y_size = width * height;
    let u_size = y_size / 4;
    let v_size = y_size / 4;

    let mut i420 = vec![0u8; y_size + u_size + v_size];

    // Safe non-overlapping mutable slices
    let (y_plane, uv) = i420.split_at_mut(y_size);
    let (u_plane, v_plane) = uv.split_at_mut(u_size);

    for y in 0..height {
        for x in 0..width {
            let bgra_idx = (y * width + x) * 4;
            let b = bgra[bgra_idx] as f32;
            let g = bgra[bgra_idx + 1] as f32;
            let r = bgra[bgra_idx + 2] as f32;

            let y_val = (0.299 * r + 0.587 * g + 0.114 * b) as u8;
            y_plane[y * width + x] = y_val;

            if y % 2 == 0 && x % 2 == 0 {
                let uv_idx = (y / 2) * (width / 2) + (x / 2);

                let u_val = (-0.147 * r - 0.289 * g + 0.436 * b + 128.0) as u8;
                u_plane[uv_idx] = u_val;

                let v_val = (0.615 * r - 0.515 * g - 0.100 * b + 128.0) as u8;
                v_plane[uv_idx] = v_val;
            }
        }
    }

    i420
}
