use image::io::Reader as ImageReader;
use std::ptr::null_mut;
use winapi::{
    shared::minwindef::BYTE,
    shared::ntdef::VOID,
    shared::windef::HBITMAP,
    um::wingdi::{CreateCompatibleDC, CreateDIBSection, BITMAPINFO},
    um::winuser::GetDC,
};

pub fn load_bitmap_from_file(
    path: &str,
    width: Option<u32>,
    height: Option<u32>,
) -> Result<image::DynamicImage, image::ImageError> {
    let mut image_reader = ImageReader::open(path)?.decode()?;
    let (resized_width, resized_height) = (width.unwrap_or(16), height.unwrap_or(16));
    image_reader = image_reader.resize(
        resized_width,
        resized_height,
        image::imageops::FilterType::Nearest,
    );
    Ok(image_reader)
}

pub fn convert_to_hbitmap(img: image::DynamicImage) -> Result<HBITMAP, String> {
    // Convert the image to a monochrome 1-bit per pixel format
    let img = img.to_rgba8();
    let (width, height) = img.dimensions();

    // Get the device context for the screen
    let hdc_screen = unsafe { GetDC(null_mut()) };
    if hdc_screen.is_null() {
        return Err("Failed to get device context.".to_string());
    }

    // Create a compatible memory device context
    let hdc_memory = unsafe { CreateCompatibleDC(hdc_screen) };
    if hdc_memory.is_null() {
        return Err("Failed to create memory device context.".to_string());
    }

    // Create a compatible bitmap
    let mut bmi: BITMAPINFO = unsafe { std::mem::zeroed() };
    bmi.bmiHeader.biSize = std::mem::size_of::<BITMAPINFO>() as u32;
    bmi.bmiHeader.biWidth = width as i32;
    bmi.bmiHeader.biHeight = -(height as i32); // Top-down
    bmi.bmiHeader.biPlanes = 1;
    bmi.bmiHeader.biBitCount = 32; // 32 bits per pixel for RGBA
    bmi.bmiHeader.biCompression = winapi::um::wingdi::BI_RGB;

    let mut bits: *mut VOID = std::ptr::null_mut();
    let hbitmap = unsafe {
        CreateDIBSection(
            hdc_screen,
            &bmi,
            winapi::um::wingdi::DIB_RGB_COLORS,
            &mut bits,
            std::ptr::null_mut(),
            0,
        )
    };
    if hbitmap.is_null() {
        return Err("Failed to create DIB section.".to_string());
    }

    // Copy image pixels to the bitmap
    for y in 0..height {
        for x in 0..width {
            let pixel = img.get_pixel(x, y);
            let offset = (y * width + x) * 4;
            let dst = unsafe { bits.offset(offset as isize) as *mut BYTE };
            let alpha = pixel[3] as f32 / 255.0;
            unsafe {
                *dst.offset(0) = (pixel[2] as f32 * alpha) as BYTE; // Blue channel
                *dst.offset(1) = (pixel[1] as f32 * alpha) as BYTE; // Green channel
                *dst.offset(2) = (pixel[0] as f32 * alpha) as BYTE; // Red channel
                *dst.offset(3) = pixel[3]; // Alpha channel
            }
        }
    }

    Ok(hbitmap)
}
