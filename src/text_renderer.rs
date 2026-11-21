//! Renderer di overlay testuale per le immagini APOD.
//!
//! Gestisce gli errori con `anyhow::Context`: nessun errore di filesystem o
//! di decodifica immagine viene silenziosamente ignorato.

use anyhow::{anyhow, bail, Context, Result};
use image::{GenericImageView, Rgba};
use imageproc::drawing::draw_text_mut;
use rusttype::{Font, Scale};
use std::path::{Path, PathBuf};

/// Dimensione massima accettata per un'immagine (32 MiB).
const MAX_IMAGE_BYTES: usize = 32 * 1024 * 1024;
/// Lunghezza massima del testo di overlay.
const MAX_TESTO: usize = 8_192;

/// Carica un font TTF da percorso esplicito, con errori contestualizzati.
fn carica_font(path: &Path) -> Result<Font<'static>> {
    let data = std::fs::read(path)
        .with_context(|| format!("Impossibile leggere il font {}", path.display()))?;
    let font = Font::try_from_vec(data)
        .ok_or_else(|| anyhow!("Font TTF non valido: {}", path.display()))?;
    Ok(font)
}

/// Trova un font di sistema utilizzabile.
///
/// La variabile `NASA_WALLPAPER_FONT`, se impostata, ha priorità: un suo
/// valore non valido è un errore esplicito e non viene ignorato.
pub fn find_system_font() -> Result<Font<'static>> {
    if let Some(font_path) = std::env::var_os("NASA_WALLPAPER_FONT") {
        let percorso = PathBuf::from(font_path);
        return carica_font(&percorso)
            .context("NASA_WALLPAPER_FONT non punta a un font TTF valido");
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

    let mut ultimo_errore: Option<anyhow::Error> = None;
    for path in &candidate_paths {
        let percorso = Path::new(path);
        if !percorso.is_file() {
            continue;
        }
        match carica_font(percorso) {
            Ok(font) => return Ok(font),
            Err(err) => ultimo_errore = Some(err),
        }
    }

    #[cfg(target_os = "windows")]
    if let Ok(windir) = std::env::var("WINDIR") {
        let percorso = Path::new(&windir).join("Fonts").join("arial.ttf");
        if percorso.is_file() {
            match carica_font(&percorso) {
                Ok(font) => return Ok(font),
                Err(err) => ultimo_errore = Some(err),
            }
        }
    }

    match ultimo_errore {
        Some(err) => Err(err).context("Nessun font di sistema valido trovato"),
        None => bail!(
            "Nessun font di sistema trovato: impostare NASA_WALLPAPER_FONT a un file TTF valido"
        ),
    }
}

/// Disegna un overlay scuro con titolo e spiegazione sull'immagine data.
///
/// Restituisce il percorso del file temporaneo generato. I dati in ingresso
/// sono limitati in dimensione per evitare consumi non controllati.
pub fn draw_overlay(image_bytes: &[u8], title: &str, explanation: &str) -> Result<PathBuf> {
    if image_bytes.is_empty() || image_bytes.len() > MAX_IMAGE_BYTES {
        bail!("Dimensione immagine non valida");
    }
    if title.chars().count() > MAX_TESTO || explanation.chars().count() > MAX_TESTO {
        bail!("Testo di overlay troppo lungo");
    }

    let font = find_system_font().context("Caricamento font per overlay non riuscito")?;
    let image = image::load_from_memory(image_bytes).context("Decodifica immagine non riuscita")?;

    let (width, height) = image.dimensions();
    if width == 0 || height == 0 {
        bail!("Immagine con dimensioni nulle");
    }

    let scale_factor = (f64::from(width) / 1920.0).max(1.0) as f32;
    let font_size = 22.0 * scale_factor;
    let scale = Scale {
        x: font_size,
        y: font_size,
    };

    let box_height = (f64::from(height) * 0.28) as u32;
    let box_y = height - box_height;
    let mut rgba_image = image.to_rgba8();

    for y in box_y..height {
        for x in 0..width {
            let pixel = rgba_image.get_pixel_mut(x, y);
            let p = pixel.0;
            pixel.0 = [
                (f32::from(p[0]) * 0.3) as u8,
                (f32::from(p[1]) * 0.3) as u8,
                (f32::from(p[2]) * 0.3) as u8,
                255,
            ];
        }
    }

    let chars_per_line = (f64::from(width) / f64::from(font_size * 0.52)).max(20.0) as usize;
    let wrapped_explanation = textwrap::fill(explanation, chars_per_line);
    let text_color = Rgba([255, 255, 255, 255]);

    draw_text_mut(
        &mut rgba_image,
        text_color,
        30,
        box_y as i32 + 20,
        scale,
        &font,
        title,
    );
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

    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .context("Orologio di sistema non valido")?
        .as_millis();
    let percorso = std::env::temp_dir().join(format!("nasa-wallpaper-overlay-{timestamp}.jpg"));
    rgba_image.save(&percorso).with_context(|| {
        format!(
            "Salvataggio immagine in {} non riuscito",
            percorso.display()
        )
    })?;
    Ok(percorso)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn find_system_font_or_skip() {
        match find_system_font() {
            Ok(_) => println!("System font successfully found."),
            Err(e) => println!("System font not found (can happen in headless CI): {e}"),
        }
    }

    #[test]
    fn input_non_validi_vengono_rifiutati() {
        assert!(draw_overlay(&[], "Titolo", "Testo").is_err());
        let titolo_lungo = "x".repeat(MAX_TESTO + 1);
        assert!(draw_overlay(&[0u8; 4], &titolo_lungo, "Testo").is_err());
    }
}
