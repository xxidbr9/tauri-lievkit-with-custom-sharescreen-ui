use base64::{engine::general_purpose, Engine as _};

use windows::Win32::{
    Foundation::{HWND, LPARAM, RECT, WPARAM},
    Graphics::{Dwm::*, Gdi::*},
    UI::WindowsAndMessaging::*,
};

// NOTE: this is too slow idk haha
pub fn capture_window_dwm(hwnd: HWND, max_width: i32, max_height: i32) -> Option<String> {
    unsafe {
        let hdc_screen = GetDC(None);
        let hdc_mem = CreateCompatibleDC(Some(hdc_screen));

        let mut rect = RECT::default();
        DwmGetWindowAttribute(
            hwnd,
            DWMWA_EXTENDED_FRAME_BOUNDS,
            &mut rect as *mut _ as *mut _,
            std::mem::size_of::<RECT>() as u32,
        )
        .ok()?;

        let orig_width = rect.right - rect.left;
        let orig_height = rect.bottom - rect.top;

        if orig_width <= 0 || orig_height <= 0 {
            return None;
        }

        // Compute scaled size preserving aspect ratio
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

        // Capture full-size bitmap
        let hbitmap = CreateCompatibleBitmap(hdc_screen, orig_width, orig_height);
        let old_bitmap = SelectObject(hdc_mem, hbitmap.into());

        let _ = BitBlt(
            hdc_mem,
            0,
            0,
            orig_width,
            orig_height,
            Some(hdc_screen),
            rect.left,
            rect.top,
            SRCCOPY | CAPTUREBLT,
        );

        let base64_data =
            bitmap_to_base64_png_resized(hbitmap, orig_width, orig_height, thumb_w, thumb_h);

        SelectObject(hdc_mem, old_bitmap);
        let _ = DeleteObject(hbitmap.into());
        let _ = DeleteDC(hdc_mem);
        let _ = ReleaseDC(None, hdc_screen);

        base64_data
    }
}

// TODO: Implement capture_monitor_thumbnail function
pub fn capture_monitor_thumbnail(
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

        let rect = info.rcMonitor;
        let width = rect.right - rect.left;
        let height = rect.bottom - rect.top;

        let aspect = width as f32 / height as f32;
        let (thumb_w, thumb_h) = if width > height {
            (max_width, (max_width as f32 / aspect) as i32)
        } else {
            ((max_height as f32 * aspect) as i32, max_height)
        };

        let hdc_screen = GetDC(None);
        let hdc_mem = CreateCompatibleDC(Some(hdc_screen));
        let hbitmap = CreateCompatibleBitmap(hdc_screen, thumb_w, thumb_h);
        let old_bitmap = SelectObject(hdc_mem, hbitmap.into());

        SetStretchBltMode(hdc_mem, STRETCH_HALFTONE);
        let _ = StretchBlt(
            hdc_mem,
            0,
            0,
            thumb_w,
            thumb_h,
            Some(hdc_screen),
            rect.left,
            rect.top,
            width,
            height,
            SRCCOPY,
        );

        let base64_data = bitmap_to_base64_png(hbitmap, thumb_w, thumb_h);

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
                biHeight: -orig_height, // top-down
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

        // Convert BGRA to RGBA
        for chunk in pixels.chunks_exact_mut(4) {
            chunk.swap(0, 2);
        }

        // Resize pixel buffer manually
        let mut resized_pixels = vec![0u8; (width * height * 4) as usize];
        for y in 0..height {
            for x in 0..width {
                let src_x = x * orig_width / width;
                let src_y = y * orig_height / height;
                let src_idx = ((src_y * orig_width + src_x) * 4) as usize;
                let dst_idx = ((y * width + x) * 4) as usize;
                resized_pixels[dst_idx..dst_idx + 4].copy_from_slice(&pixels[src_idx..src_idx + 4]);
            }
        }

        // Encode PNG
        let mut png_data = Vec::new();
        {
            let mut encoder = png::Encoder::new(&mut png_data, width as u32, height as u32);
            encoder.set_color(png::ColorType::Rgba);
            encoder.set_depth(png::BitDepth::Eight);

            let mut writer = encoder.write_header().ok()?;
            writer.write_image_data(&resized_pixels).ok()?;
        }

        Some(format!(
            "data:image/png;base64,{}",
            general_purpose::STANDARD.encode(&png_data)
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
