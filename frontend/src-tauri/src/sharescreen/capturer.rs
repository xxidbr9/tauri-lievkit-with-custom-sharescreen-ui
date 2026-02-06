use anyhow::Result;
use base64::{engine::general_purpose, Engine as _};
use fast_image_resize::images::Image;
use fast_image_resize::{IntoImageView, PixelType, Resizer};
use image::{codecs::jpeg::JpegEncoder, ImageBuffer, RgbaImage};
use windows::Win32::{
    Foundation::{HWND, LPARAM, RECT, WPARAM},
    Graphics::Gdi::*,
    UI::WindowsAndMessaging::*,
};
use windows_core::BOOL;

#[link(name = "user32")]
extern "system" {
    fn PrintWindow(hwnd: HWND, hdcBlt: HDC, nFlags: u32) -> BOOL;
}

pub fn capture_app_window(hwnd: HWND, max_width: i32, max_height: i32) -> Option<String> {
    unsafe {
        let hdc_screen = GetDC(None);
        let hdc_mem = CreateCompatibleDC(Some(hdc_screen));

        let mut rect = RECT::default();
        if GetWindowRect(hwnd, &mut rect).is_err() {
            return None;
        }

        let orig_width = rect.right - rect.left;
        let orig_height = rect.bottom - rect.top;
        if orig_width <= 0 || orig_height <= 0 {
            return None;
        }

        // Aspect-ratio scaling
        let aspect = orig_width as f32 / orig_height as f32;
        let (thumb_w, thumb_h) = if orig_width > orig_height {
            let w = max_width.min(orig_width);
            let h = (w as f32 / aspect).round() as i32;
            (w, h)
        } else {
            let h = max_height.min(orig_height);
            let w = (h as f32 * aspect).round() as i32;
            (w, h)
        };
        if thumb_w <= 0 || thumb_h <= 0 {
            return None;
        }

        let hbitmap = CreateCompatibleBitmap(hdc_screen, orig_width, orig_height);
        let old_bitmap = SelectObject(hdc_mem, hbitmap.into());

        // Use PrintWindow instead of BitBlt
        // TODO: this slowing the process
        // let ok = PrintWindow(hwnd, hdc_mem, PW_RENDERFULLCONTENT).as_bool();
        // if !ok {
        //     SelectObject(hdc_mem, old_bitmap);
        //     let _ = DeleteObject(hbitmap.into());
        //     let _ = DeleteDC(hdc_mem);
        //     let _ = ReleaseDC(None, hdc_screen);
        //     return None;
        // }

        // TODO: this is also slowing the process
        // let base64_data =
        //     bitmap_to_base64_png_resized(hbitmap, orig_width, orig_height, thumb_w, thumb_h);

        let base64_data = Some("".to_string());
        // let base64_data = bitmap_to_base64_png(hbitmap, orig_width, orig_height);

        SelectObject(hdc_mem, old_bitmap);
        let _ = DeleteObject(hbitmap.into());
        let _ = DeleteDC(hdc_mem);
        let _ = ReleaseDC(None, hdc_screen);

        base64_data
    }
}

pub fn capture_monitor_display(
    hmonitor: HMONITOR,
    max_width: i32,
    max_height: i32,
) -> Option<String> {
    unsafe {
        let mut info = MONITORINFO {
            cbSize: std::mem::size_of::<MONITORINFO>() as u32,
            ..Default::default()
        };

        if !GetMonitorInfoW(hmonitor, &mut info).as_bool() {
            return None;
        }
        let hdc_screen = GetDC(None);
        let hdc_mem = CreateCompatibleDC(Some(hdc_screen));

        let rect = info.rcMonitor;
        let orig_width = rect.right - rect.left;
        let orig_height = rect.bottom - rect.top;
        if orig_width <= 0 || orig_height <= 0 {
            return None;
        }

        // Aspect-ratio scaling
        let aspect = orig_width as f32 / orig_height as f32;
        let (thumb_w, thumb_h) = if orig_width > orig_height {
            let w = max_width.min(orig_width);
            let h = (w as f32 / aspect).round() as i32;
            (w, h)
        } else {
            let h = max_height.min(orig_height);
            let w = (h as f32 * aspect).round() as i32;
            (w, h)
        };
        if thumb_w <= 0 || thumb_h <= 0 {
            return None;
        }

        let hbitmap = CreateCompatibleBitmap(hdc_screen, orig_width, orig_height);
        let old_bitmap = SelectObject(hdc_mem, hbitmap.into());

        // TODO: this slowing the process
        // let ok = BitBlt(
        //     hdc_mem,
        //     0,
        //     0,
        //     orig_width,
        //     orig_height,
        //     Some(hdc_screen),
        //     rect.left,
        //     rect.top,
        //     SRCCOPY | CAPTUREBLT,
        // )
        // .is_ok();

        // if !ok {
        //     SelectObject(hdc_mem, old_bitmap);
        //     let _ = DeleteObject(hbitmap.into());
        //     let _ = DeleteDC(hdc_mem);
        //     let _ = ReleaseDC(None, hdc_screen);
        //     return None;
        // }

        // TODO: this is also slowing the process
        // let base64_data =
        //     bitmap_to_base64_png_resized(hbitmap, orig_width, orig_height, thumb_w, thumb_h);
        let base64_data = Some("".to_string());
        // let base64_data = bitmap_to_base64_png(hbitmap, orig_width, orig_height);

        SelectObject(hdc_mem, old_bitmap);
        let _ = DeleteObject(hbitmap.into());
        let _ = DeleteDC(hdc_mem);
        let _ = ReleaseDC(None, hdc_screen);

        base64_data
    }
}

pub fn bitmap_to_base64_png(hbitmap: HBITMAP, width: i32, height: i32) -> Option<String> {
    unsafe {
        let mut bmp_info = BITMAPINFO {
            bmiHeader: BITMAPINFOHEADER {
                biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
                biWidth: width,
                biHeight: -height, // Top-down
                biPlanes: 1,
                biBitCount: 32,
                biCompression: BI_RGB.0 as u32,
                ..Default::default()
            },
            ..Default::default()
        };

        let hdc = CreateCompatibleDC(None);
        let mut pixels: Vec<u8> = vec![0; (width * height * 4) as usize];

        GetDIBits(
            hdc,
            hbitmap,
            0,
            height as u32,
            Some(pixels.as_mut_ptr() as *mut _),
            &mut bmp_info,
            DIB_RGB_COLORS,
        );

        let _ = DeleteDC(hdc);

        // Convert BGRA to RGBA
        for chunk in pixels.chunks_exact_mut(4) {
            chunk.swap(0, 2);
        }

        // Encode to PNG
        let mut png_data = Vec::new();
        {
            let mut encoder = png::Encoder::new(&mut png_data, width as u32, height as u32);
            encoder.set_color(png::ColorType::Rgba);
            encoder.set_depth(png::BitDepth::Eight);

            let mut writer = encoder.write_header().ok()?;
            writer.write_image_data(&pixels).ok()?;
        }

        Some(format!(
            "data:image/png;base64,{}",
            general_purpose::STANDARD.encode(&png_data)
        ))
    }
}

/// Modified bitmap_to_base64_png to include resizing
pub fn bitmap_to_base64_png_resized(
    hbitmap: HBITMAP,
    orig_width: i32,
    orig_height: i32,
    width: i32,
    height: i32,
) -> Option<String> {
    unsafe {
        let mut bmp_info = BITMAPINFO {
            bmiHeader: BITMAPINFOHEADER {
                biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
                biWidth: orig_width,
                biHeight: -orig_height,
                biPlanes: 1,
                biBitCount: 32,
                biCompression: BI_RGB.0 as u32,
                ..Default::default()
            },
            ..Default::default()
        };

        let hdc = CreateCompatibleDC(None);
        let mut pixels: Vec<u8> = vec![0; (orig_width * orig_height * 4) as usize];

        GetDIBits(
            hdc,
            hbitmap,
            0,
            orig_height as u32,
            Some(pixels.as_mut_ptr() as *mut _),
            &mut bmp_info,
            DIB_RGB_COLORS,
        );

        let _ = DeleteDC(hdc);

        // Convert BGRA to RGBA for fast_image_resize
        for chunk in pixels.chunks_exact_mut(4) {
            chunk.swap(0, 2);
        }

        // Use fast_image_resize for SIMD-accelerated scaling
        let src_image = Image::from_vec_u8(
            orig_width as u32,
            orig_height as u32,
            pixels,
            PixelType::U8x4,
        )
        .ok()?;

        let mut dst_image = Image::new(width as u32, height as u32, PixelType::U8x3);

        let mut resizer = Resizer::new();
        resizer.resize(&src_image, &mut dst_image, None).ok()?;

        let rgb_pixels = dst_image.buffer();

        // Encode to JPEG
        let mut jpeg_data = Vec::new();
        let mut encoder = JpegEncoder::new_with_quality(&mut jpeg_data, 80);
        encoder
            .encode(
                rgb_pixels,
                width as u32,
                height as u32,
                image::ExtendedColorType::Rgb8,
            )
            .ok()?;

        Some(format!(
            "data:image/jpeg;base64,{}",
            general_purpose::STANDARD.encode(&jpeg_data)
        ))
    }
}

// NOTE: this is used for capturing window icon
pub fn get_window_icon(hwnd: HWND) -> Option<String> {
    unsafe {
        let hicon = SendMessageW(hwnd, WM_GETICON, Some(WPARAM(2)), Some(LPARAM(0))); // ICON_SMALL2
        if hicon.0 == 0 {
            let hicon = SendMessageW(hwnd, WM_GETICON, Some(WPARAM(0)), Some(LPARAM(0))); // ICON_SMALL
            if hicon.0 == 0 {
                return None;
            }
        }

        let hicon = HICON(hicon.0 as *mut _);
        icon_to_base64_png(hicon, 32, 32)
    }
}

pub fn icon_to_base64_png(hicon: HICON, width: i32, height: i32) -> Option<String> {
    unsafe {
        let hdc_screen = GetDC(None);
        let hdc_mem = CreateCompatibleDC(Some(hdc_screen));
        let hbitmap = CreateCompatibleBitmap(hdc_screen, width, height);
        let old_bitmap = SelectObject(hdc_mem, hbitmap.into());

        let _ = DrawIconEx(hdc_mem, 0, 0, hicon, width, height, 0, None, DI_NORMAL);

        let base64_data = bitmap_to_base64_png(hbitmap, width, height);

        SelectObject(hdc_mem, old_bitmap);
        let _ = DeleteObject(hbitmap.into());
        let _ = DeleteDC(hdc_mem);
        let _ = ReleaseDC(None, hdc_screen);

        base64_data
    }
}
