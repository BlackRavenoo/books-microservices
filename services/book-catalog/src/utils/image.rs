use image::{imageops::FilterType, GenericImageView, ImageFormat};
use std::io::Cursor;

pub fn process_image(input: &[u8], width: u32) -> Result<Vec<u8>, String> {
    let img = image::load_from_memory(input).map_err(|e| e.to_string())?;
    let (orig_w, orig_h) = img.dimensions();
    let height = (orig_h as f32 * (width as f32 / orig_w as f32)) as u32;
    let resized = img.resize_exact(width, height, FilterType::Lanczos3);

    let mut buf = Vec::new();
    resized.write_to(&mut Cursor::new(&mut buf), ImageFormat::Jpeg).map_err(|e| e.to_string())?;
    Ok(buf)
}