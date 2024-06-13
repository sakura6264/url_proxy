use std::os::windows::ffi::OsStrExt;

pub fn exe_dir() -> std::path::PathBuf {
    std::env::current_exe()
        .unwrap()
        .parent()
        .unwrap()
        .to_path_buf()
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
        output_buf: *const *mut u8,
        width: *mut u64,
        height: *mut u64,
        bwidth: *mut u64,
    ) -> u32;
    fn FreeMemory(buf: *const u8);
    fn OpenFile(path: *const u16) -> u32;
    fn GetScreenSize(width: *mut u64, height: *mut u64) -> u32;
}

pub fn extract_icon(path: &str) -> Option<image::RgbaImage> {
    let mut path_wchar = std::ffi::OsStr::new(path)
        .encode_wide()
        .collect::<Vec<u16>>();
    path_wchar.push(0);
    let mut width: u64 = 0;
    let mut height: u64 = 0;
    let mut bwidth: u64 = 0;
    let buffer;
    unsafe {
        let mut buf: *mut u8 = std::ptr::null_mut();
        let ret = ExtractIconImpl(
            path_wchar.as_ptr(),
            &mut buf,
            &mut width,
            &mut height,
            &mut bwidth,
        );
        if ret != 0 {
            log::error!("Error From C:{ret}");
            return None;
        }
        let size = height * bwidth;
        buffer = std::slice::from_raw_parts(buf, size as usize).to_vec();
        FreeMemory(buf);
    }
    match bwidth / width {
        4 => {
            let img = image::RgbaImage::from_raw(width as u32, height as u32, buffer);
            match img {
                Some(img) => {
                    // swap red and blue channels
                    let img = image::RgbaImage::from_fn(img.width(), img.height(), |x, y| {
                        let pixel = img.get_pixel(x, y);
                        image::Rgba([pixel[2], pixel[1], pixel[0], pixel[3]])
                    });
                    Some(img)
                }
                None => None,
            }
        }
        3 => {
            let img = image::RgbImage::from_raw(width as u32, height as u32, buffer);
            match img {
                Some(img) => {
                    let rgba = image::ImageBuffer::from_fn(img.width(), img.height(), |x, y| {
                        let pixel = img.get_pixel(x, y);
                        // swap channels and add alpha channel
                        image::Rgba([pixel[0], pixel[1], pixel[2], 255])
                    });
                    Some(rgba)
                }
                None => None,
            }
        }
        _ => None,
    }
}

pub fn open_file(path: &str) {
    let mut path_wchar = std::ffi::OsStr::new(path)
        .encode_wide()
        .collect::<Vec<u16>>();
    path_wchar.push(0);
    let result;
    unsafe { result = OpenFile(path_wchar.as_ptr()) }
    if result != 0 {
        log::error!("Error opening file:{path}");
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
