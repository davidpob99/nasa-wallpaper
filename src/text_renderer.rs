use anyhow::{Context, Result};
use image::{GenericImageView, Rgba};
use imageproc::drawing::draw_text_mut;
use rusttype::{Font, Scale};
use std::path::PathBuf;

pub fn find_system_font() -> Result<Font<'static>> {
    if let Ok(font_path) = std::env::var("NASA_WALLPAPER_FONT") {
        if let Ok(data) = std::fs::read(&font_path) {
            if let Some(font) = Font::try_from_vec(data) {
                return Ok(font);
            }
        }
    }

    let candidate_paths = [
        // Windows
        "C:\\Windows\\Fonts\\segoeui.ttf",
        "C:\\Windows\\Fonts\\arial.ttf",
        "C:\\Windows\\Fonts\\calibri.ttf",
        // Linux (Debian, Ubuntu, Fedora, Arch)
        "/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf",
        "/usr/share/fonts/TTF/DejaVuSans.ttf",
        "/usr/share/fonts/truetype/freefont/FreeSans.ttf",
        "/usr/share/fonts/truetype/liberation/LiberationSans-Regular.ttf",
        "/usr/share/fonts/noto/NotoSans-Regular.ttf",
        // macOS
        "/System/Library/Fonts/SFCompact.ttf",
        "/System/Library/Fonts/SFNS.ttf",
        "/System/Library/Fonts/Helvetica.ttc",
        "/Library/Fonts/Arial.ttf",
    ];

    for path in &candidate_paths {
        if let Ok(data) = std::fs::read(path) {
            if let Some(font) = Font::try_from_vec(data) {
                return Ok(font);
            }
        }
    }

    #[cfg(target_os = "windows")]
    if let Ok(windir) = std::env::var("WINDIR") {
        let p = std::path::Path::new(&windir).join("Fonts").join("arial.ttf");
        if let Ok(data) = std::fs::read(p) {
            if let Some(font) = Font::try_from_vec(data) {
                return Ok(font);
            }
        }
    }

    anyhow::bail!("Could not find a system font. Please set the NASA_WALLPAPER_FONT environment variable to a valid TTF font path.");
}

pub fn draw_overlay(image_bytes: &[u8], title: &str, explanation: &str) -> Result<PathBuf> {
    let font = find_system_font().context("Failed to load system font for explanation overlay")?;

    let image = image::load_from_memory(image_bytes)
        .context("Failed to load image from memory")?;

    let (width, height) = image.dimensions();

    let scale_factor = width as f32 / 1920.0;
    let font_size = 22.0 * scale_factor.max(1.0);
    let scale = Scale {
        x: font_size,
        y: font_size,
    };

    let box_height = (height as f32 * 0.28) as u32;
    let box_y = height - box_height;

    let mut rgba_image = image.to_rgba8();

    // Dark semi-transparent box background (dim pixels)
    for y in box_y..height {
        for x in 0..width {
            let pixel = rgba_image.get_pixel_mut(x, y);
            let p = pixel.0;
            pixel.0 = [
                (p[0] as f32 * 0.3) as u8,
                (p[1] as f32 * 0.3) as u8,
                (p[2] as f32 * 0.3) as u8,
                255,
            ];
        }
    }

    let chars_per_line = (width as f32 / (font_size * 0.52)) as usize;
    let wrapped_explanation = textwrap::fill(explanation, chars_per_line.max(20));

    let text_color = Rgba([255, 255, 255, 255]);

    // Draw Title
    draw_text_mut(
        &mut rgba_image,
        text_color,
        30,
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
        30,
        explanation_y,
        Scale {
            x: font_size * 0.85,
            y: font_size * 0.85,
        },
        &font,
        &wrapped_explanation,
    );

    let temp_dir = std::env::temp_dir();
    let file_name = format!("nasa-wallpaper-overlay-{}.jpg", chrono::Utc::now().timestamp_millis());
    let file_path = temp_dir.join(file_name);

    rgba_image.save(&file_path).context("Failed to save overlay wallpaper image")?;

    Ok(file_path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_system_font_or_skip() {
        match find_system_font() {
            Ok(_) => println!("System font successfully found."),
            Err(e) => println!("System font not found (can happen in headless CI): {}", e),
        }
    }
}
