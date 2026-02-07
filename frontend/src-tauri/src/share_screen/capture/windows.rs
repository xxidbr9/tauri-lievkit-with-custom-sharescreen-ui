use crate::share_screen::dto::{
    AudioDevice, CaptureConfig, CaptureError, CaptureSourceType, MonitorRect, Result, WindowInfo,
};
use std::sync::Arc;
use windows::Win32::Graphics::Direct3D::D3D_DRIVER_TYPE_HARDWARE;
use windows::Win32::Graphics::Gdi::{
    EnumDisplayMonitors, GetMonitorInfoW, HDC, HMONITOR, MONITORINFO,
};
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
unsafe impl Send for WindowCapture {}
unsafe impl Sync for WindowCapture {}

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

        let frame_interval = std::time::Duration::from_millis(1000 / config.fps as u64);
        let last_frame = Arc::new(std::sync::Mutex::new(std::time::Instant::now()));

        let device_clone = device.clone();
        let context_clone = context.clone();

        frame_pool
            .FrameArrived(&TypedEventHandler::new(
                move |pool_ref: Ref<Direct3D11CaptureFramePool>, _| {
                    let mut last = last_frame.lock().unwrap();
                    if last.elapsed() < frame_interval {
                        return Ok(());
                    }
                    *last = std::time::Instant::now();
                    // TODO: handle this if error
                    let pool: &Direct3D11CaptureFramePool = pool_ref.as_ref().unwrap();
                    if let Ok(frame) = pool.TryGetNextFrame() {
                        if let Ok(surface) = frame.Surface() {
                            if let Ok(access) = surface.cast::<IDirect3DDxgiInterfaceAccess>() {
                                if let Ok(texture) = access.GetInterface::<ID3D11Texture2D>() {
                                    if let Ok(resized) = resize_texture_gpu(
                                        &device_clone,
                                        &context_clone,
                                        &texture,
                                        config.width as u32,
                                        config.height as u32,
                                    ) {
                                        if let Ok(bytes) =
                                            texture_to_bytes(&context_clone, &resized)
                                        {
                                            let tx = video_tx.clone();
                                            tokio::spawn(async move {
                                                let _ = tx.send(bytes).await;
                                            });
                                        }
                                    }
                                }
                            }
                        }
                    }
                    Ok(())
                },
            ))
            .map_err(|e| CaptureError::PlatformError(e.to_string()))?;

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

        // CreateDirect3D11DeviceFromDXGIDevice(&dxgi_device)
        //     .map_err(|e| CaptureError::PlatformError(e.to_string()))
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
