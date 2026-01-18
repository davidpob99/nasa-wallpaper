use image::{DynamicImage, Rgba, RgbaImage, GenericImageView};
use imageproc::drawing::draw_text_mut;
use rusttype::{Font, Scale};
use std::path::PathBuf;
use anyhow::{Context, Result};

pub fn draw_overlay(image_bytes: &[u8], title: &str, explanation: &str) -> Result<PathBuf> {
    let font_data = include_bytes!("font.ttf");
    let font = Font::try_from_bytes(font_data as &[u8]).context("Error constructing Font")?;

    let mut image = image::load_from_memory(image_bytes)
        .context("Failed to load image from memory")?;

    let (width, height) = image.dimensions();
    
    // Scale for font size based on image width
    // Target roughly 40 chars per line or readable size
    let scale_factor = width as f32 / 1920.0; 
    let font_size = 24.0 * scale_factor.max(1.0);
    // rusttype scale
    let scale = Scale { x: font_size, y: font_size };

    // Filter characters that are not in the font or handle newlines? 
    // basic simple drawing test first
    
    // We want to draw a dark box at the bottom
    let box_height = (height as f32 * 0.3) as u32; // Bottom 30%
    let box_y = height - box_height;
    
    // Draw semi-transparent box (manual pixel manipulation for now or use imageproc rect)
    // imageproc::drawing::draw_filled_rect_mut does not support alpha blending nicely on DynamicImage easily without converting
    // simpler to iterate pixels
    
    // Draw semi-transparent box
    let mut rgba_image = image.to_rgba8();
    
    for y in box_y..height {
        for x in 0..width {
            let pixel = rgba_image.get_pixel_mut(x, y);
            // Darken background: result = src * (1-alpha) + dst * alpha
            // Simple approach: 0.7 opacity black
            let p = pixel.0;
            pixel.0 = [
                (p[0] as f32 * 0.3) as u8,
                (p[1] as f32 * 0.3) as u8,
                (p[2] as f32 * 0.3) as u8,
                255
            ];
        }
    }

    // Wrap text
    // Simple wrapping logic: need a different approach or verify textwrap works with rusttype widths?
    // For now stick to simple char counting or use textwrap with a best guess.
    let chars_per_line = (width as f32 / (font_size * 0.5)) as usize; 
    let wrapped_explanation = textwrap::fill(explanation, chars_per_line);
    
    let text_color = Rgba([255, 255, 255, 255]);
    
    // Draw Title
    draw_text_mut(
        &mut rgba_image,
        text_color,
        20,
        box_y as i32 + 20,
        scale,
        &font,
        title,
    );
    
    // Draw Explanation
    let explanation_y = box_y as i32 + 20 + (font_size * 1.5) as i32;
    draw_text_mut(
        &mut rgba_image,
        text_color,
        20,
        explanation_y,
        scale, // slightly smaller?
        &font,
        &wrapped_explanation,
    );

    // Save to temp file
    let temp_dir = std::env::temp_dir();
    let file_name = format!("nasa-wallpaper-overlay-{}.jpg", chrono::Utc::now().timestamp());
    let file_path = temp_dir.join(file_name);
    
    rgba_image.save(&file_path).context("Failed to save modified image")?;
    
    Ok(file_path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_draw_overlay() {
        // Create a blank 100x100 image
        let mut image = image::RgbaImage::new(100, 100);
        // Fill with black
        for pixel in image.pixels_mut() {
            *pixel = image::Rgba([0, 0, 0, 255]);
        }
        let mut bytes = Vec::new();
        image.write_to(&mut std::io::Cursor::new(&mut bytes), image::ImageOutputFormat::Png).unwrap();
        
        let result = draw_overlay(&bytes, "Test Title", "Test Explanation");
        assert!(result.is_ok());
        let path = result.unwrap();
        assert!(path.exists());
        // Clean up
        let _ = std::fs::remove_file(path);
    }
}
