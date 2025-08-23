use std::os::windows::ffi::OsStrExt;
use std::path::PathBuf;

pub fn exe_dir() -> PathBuf {
    match std::env::current_exe() {
        Ok(path) => match path.parent() {
            Some(parent) => parent.to_path_buf(),
            None => {
                log::error!("Failed to get parent directory of exe");
                fallback_dir()
            }
        },
        Err(e) => {
            log::error!("Failed to get current exe path: {e}");
            fallback_dir()
        }
    }
}

/// Fallback directory when exe_dir fails
fn fallback_dir() -> PathBuf {
    log::warn!("Falling back to current directory");
    std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
}

pub fn path_to(relative_path: &str) -> std::path::PathBuf {
    exe_dir().join(relative_path)
}

pub fn settings_path() -> std::path::PathBuf {
    path_to("settings.json")
}

pub fn cache_path() -> std::path::PathBuf {
    path_to("cache")
}

pub fn log_path() -> std::path::PathBuf {
    path_to("output.log")
}

pub fn get_unix_msec() -> usize {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis() as usize
}

extern "C" {
    fn ExtractIconImpl(
        path: *const u16,
        output_buf: *mut *mut u8,
        width: *mut u64,
        height: *mut u64,
        bwidth: *mut u64,
    ) -> u32;
    fn FreeMemory(buf: *mut u8);
    fn OpenFile(path: *const u16) -> u32;
    fn GetScreenSize(width: *mut u64, height: *mut u64) -> u32;
}

pub fn extract_icon(path: &str) -> Option<image::RgbaImage> {
    // Convert path to wide string for Windows API
    let mut path_wchar = std::ffi::OsStr::new(path)
        .encode_wide()
        .collect::<Vec<u16>>();
    path_wchar.push(0); // Null terminator

    let mut width: u64 = 0;
    let mut height: u64 = 0;
    let mut bwidth: u64 = 0;

    // Extract icon using FFI
    let buffer = extract_icon_ffi(&path_wchar, &mut width, &mut height, &mut bwidth)?;

    // Process the image data based on bytes per pixel
    match bwidth / width {
        4 => process_rgba_image(width, height, buffer),
        3 => process_rgb_image(width, height, buffer),
        _ => {
            log::error!("Unsupported bytes per pixel: {}", bwidth / width);
            None
        }
    }
}

/// Extract icon data using FFI
fn extract_icon_ffi(
    path_wchar: &[u16],
    width: &mut u64,
    height: &mut u64,
    bwidth: &mut u64,
) -> Option<Vec<u8>> {
    unsafe {
        let mut buf: *mut u8 = std::ptr::null_mut();

        let ret = ExtractIconImpl(path_wchar.as_ptr(), &mut buf, width, height, bwidth);

        if ret != 0 {
            log::error!("Error extracting icon: {ret}");
            return None;
        }

        if buf.is_null() {
            log::error!("Received null buffer from ExtractIconImpl");
            return None;
        }

        let size = *height * *bwidth;
        if size == 0 {
            log::error!(
                "Invalid image dimensions: {}x{} with {} bytes width",
                width,
                height,
                bwidth
            );
            FreeMemory(buf);
            return None;
        }

        // Copy data to avoid memory issues
        let buffer = std::slice::from_raw_parts(buf, size as usize).to_vec();

        // Free the memory allocated by C
        FreeMemory(buf);

        Some(buffer)
    }
}

/// Process RGBA image data
fn process_rgba_image(width: u64, height: u64, buffer: Vec<u8>) -> Option<image::RgbaImage> {
    let img = image::RgbaImage::from_raw(width as u32, height as u32, buffer)?;

    // Swap red and blue channels (BGR to RGB)
    let img = image::RgbaImage::from_fn(img.width(), img.height(), |x, y| {
        let pixel = img.get_pixel(x, y);
        image::Rgba([pixel[2], pixel[1], pixel[0], pixel[3]])
    });

    Some(img)
}

/// Process RGB image data
fn process_rgb_image(width: u64, height: u64, buffer: Vec<u8>) -> Option<image::RgbaImage> {
    let img = image::RgbImage::from_raw(width as u32, height as u32, buffer)?;

    // Convert RGB to RGBA
    let rgba = image::ImageBuffer::from_fn(img.width(), img.height(), |x, y| {
        let pixel = img.get_pixel(x, y);
        image::Rgba([pixel[0], pixel[1], pixel[2], 255])
    });

    Some(rgba)
}

pub fn open_file(path: &str) -> bool {
    let mut path_wchar = std::ffi::OsStr::new(path)
        .encode_wide()
        .collect::<Vec<u16>>();
    path_wchar.push(0); // Null terminator

    let result = unsafe { OpenFile(path_wchar.as_ptr()) };

    if result != 0 {
        log::error!("Error opening file: {path} (error code: {result})");
        false
    } else {
        true
    }
}

pub fn get_screen_size() -> (u64, u64) {
    let mut width: u64 = 0;
    let mut height: u64 = 0;
    unsafe {
        GetScreenSize(&mut width, &mut height);
    }
    (width, height)
}

mod tests {

    #[test]
    fn test_extract_icon() {
        let icon = super::extract_icon("C:\\Windows\\explorer.exe");
        if let Some(ico) = icon {
            ico.save("test.png").unwrap();
        } else {
            panic!();
        }
    }
}
