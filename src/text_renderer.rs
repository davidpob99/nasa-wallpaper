//! Renderer di overlay testuale per le immagini APOD.
//!
//! Gestisce gli errori con `anyhow::Context`: nessun errore di filesystem o
//! di decodifica immagine viene silenziosamente ignorato.
//!
//! Il parsing del font usa `skrifa` e la rasterizzazione usa
//! `ab_glyph_rasterizer`: entrambi i progetti sono mantenuti e non dipendono
//! da `ttf-parser`, dichiarato non mantenuto in RUSTSEC-2026-0192.

use ab_glyph_rasterizer::{point, Rasterizer};
use anyhow::{anyhow, bail, Context, Result};
use image::{GenericImageView, Rgba, RgbaImage};
use skrifa::{DrawMemory, Font, PathSink, ScalerFont};
use std::path::{Path, PathBuf};
use tempfile::{Builder, NamedTempFile};

/// Dimensione massima accettata per un'immagine (32 MiB).
const MAX_IMAGE_BYTES: usize = 32 * 1024 * 1024;
/// Lunghezza massima del testo di overlay.
const MAX_TESTO: usize = 8_192;

/// Carica un font TTF da percorso esplicito, con errori contestualizzati.
/// `skrifa::Font` possiede i byte del file, quindi non ci sono prestiti da
/// gestire durante il disegno.
fn carica_font(path: &Path) -> Result<Font> {
    let bytes = std::fs::read(path)
        .with_context(|| format!("Impossibile leggere il font {}", path.display()))?;
    Font::new(bytes, 0).ok_or_else(|| anyhow!("Font TTF non valido: {}", path.display()))
}

/// Trova un font di sistema utilizzabile.
///
/// La variabile `NASA_WALLPAPER_FONT`, se impostata, ha priorità: un suo
/// valore non valido è un errore esplicito e non viene ignorato.
pub fn find_system_font() -> Result<Font> {
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

/// Verifica i limiti di dimensione dei dati in ingresso.
fn valida_overlay(image_bytes: &[u8], title: &str, explanation: &str) -> Result<()> {
    if image_bytes.is_empty() || image_bytes.len() > MAX_IMAGE_BYTES {
        bail!("Dimensione immagine non valida");
    }
    if title.chars().count() > MAX_TESTO || explanation.chars().count() > MAX_TESTO {
        bail!("Testo di overlay troppo lungo");
    }
    Ok(())
}

/// Scurisce la fascia inferiore dell'immagine e ne restituisce l'inizio.
fn scurisci_fascia(immagine: &mut RgbaImage, height: u32) -> u32 {
    let box_height = (f64::from(height) * 0.28) as u32;
    let box_y = height - box_height;
    for y in box_y..height {
        for x in 0..immagine.width() {
            let pixel = immagine.get_pixel_mut(x, y);
            let p = pixel.0;
            pixel.0 = [
                (f32::from(p[0]) * 0.3) as u8,
                (f32::from(p[1]) * 0.3) as u8,
                (f32::from(p[2]) * 0.3) as u8,
                255,
            ];
        }
    }
    box_y
}

/// Trasforma i comandi del contorno del glyph in segmenti rasterizzabili.
///
/// `skrifa` restituisce coordinate in pixel per em con origine alla baseline
/// e asse y crescente verso l'alto; il rasterizzatore usa l'asse y crescente
/// verso il basso, quindi la y viene rispecchiata.
struct RasterSink<'a> {
    rasterizer: &'a mut Rasterizer,
    current: (f32, f32),
    start: Option<(f32, f32)>,
    offset_x: f32,
    baseline_y: f32,
}

impl PathSink for RasterSink<'_> {
    fn move_to(&mut self, x: f32, y: f32) {
        self.current = (x, y);
        self.start = Some((x, y));
    }

    fn line_to(&mut self, x: f32, y: f32) {
        let from = point(
            self.offset_x + self.current.0,
            self.baseline_y - self.current.1,
        );
        let to = point(self.offset_x + x, self.baseline_y - y);
        self.rasterizer.draw_line(from, to);
        self.current = (x, y);
    }

    fn quad_to(&mut self, cx: f32, cy: f32, x: f32, y: f32) {
        let from = point(
            self.offset_x + self.current.0,
            self.baseline_y - self.current.1,
        );
        let control = point(self.offset_x + cx, self.baseline_y - cy);
        let to = point(self.offset_x + x, self.baseline_y - y);
        self.rasterizer.draw_quad(from, control, to);
        self.current = (x, y);
    }

    fn curve_to(&mut self, cx0: f32, cy0: f32, cx1: f32, cy1: f32, x: f32, y: f32) {
        let from = point(
            self.offset_x + self.current.0,
            self.baseline_y - self.current.1,
        );
        let control0 = point(self.offset_x + cx0, self.baseline_y - cy0);
        let control1 = point(self.offset_x + cx1, self.baseline_y - cy1);
        let to = point(self.offset_x + x, self.baseline_y - y);
        self.rasterizer.draw_cubic(from, control0, control1, to);
        self.current = (x, y);
    }

    fn close(&mut self) {
        if let Some(start) = self.start {
            let from = point(
                self.offset_x + self.current.0,
                self.baseline_y - self.current.1,
            );
            let to = point(self.offset_x + start.0, self.baseline_y - start.1);
            self.rasterizer.draw_line(from, to);
            self.current = start;
            self.start = None;
        }
    }
}

/// Disegna testo bianco in posizione (x0, y0), gestendo le interruzioni di riga.
fn disegna_testo(
    immagine: &mut RgbaImage,
    font: &Font,
    x0: i32,
    y0: i32,
    size: f32,
    testo: &str,
) -> Result<()> {
    let glyph_ids: Vec<_> = {
        let charmap = font.charmap();
        testo.chars()
            .map(|c| charmap.map_unicode(c).unwrap_or_default())
            .collect()
    };
    let scaler = ScalerFont::unhinted(font.clone(), Some(size));

    // Un glyph può arrivare a ~3 em di avanzamento; margini generosi evitano
    // di troncare contorni con ascensori particolarmente alti.
    let larghezza = (size * 3.0).ceil() as usize + 8;
    let altezza = (size * 2.0).ceil() as usize + 8;
    let offset_x = 3.0_f32;
    let baseline_offset = size * 1.25;
    let interlinea = (size * 1.25).round() as i32;

    let mut pen_x = 0.0_f32;
    let mut pen_y = 0.0_f32;

    for (ch, glyph_id) in testo.chars().zip(glyph_ids) {
        if ch == '\n' {
            pen_x = 0.0;
            pen_y += interlinea as f32;
            continue;
        }
        let Some(outline) = scaler.outline(glyph_id) else {
            continue;
        };
        let mut raster = Rasterizer::new(larghezza, altezza);
        let advance = {
            let mut sink = RasterSink {
                rasterizer: &mut raster,
                current: (0.0, 0.0),
                start: None,
                offset_x,
                baseline_y: baseline_offset,
            };
            let metriche = outline
                .draw(DrawMemory::Scratch, &mut sink)
                .context("Disegno del contorno del glyph non riuscito")?;
            metriche.h_advance.unwrap_or(size * 0.6)
        };
        let base_x = x0 as f32 + pen_x;
        let base_y = y0 as f32 + pen_y + baseline_offset;

        raster.for_each_pixel_2d(|px, py, alpha| {
            let copertura = alpha.clamp(0.0, 1.0);
            if copertura <= 0.0 {
                return;
            }
            let dest_x = (base_x + px as f32 - offset_x).round() as i32;
            let dest_y = (base_y + py as f32 - baseline_offset).round() as i32;
            if dest_x < 0 || dest_y < 0 {
                return;
            }
            if dest_x >= immagine.width() as i32 || dest_y >= immagine.height() as i32 {
                return;
            }
            let x = dest_x as u32;
            let y = dest_y as u32;
            let sfondo = immagine.get_pixel(x, y).0;
            let mescola = |canale: u8| -> u8 {
                (f32::from(canale) * (1.0 - copertura) + 255.0 * copertura).round() as u8
            };
            immagine.put_pixel(
                x,
                y,
                Rgba([
                    mescola(sfondo[0]),
                    mescola(sfondo[1]),
                    mescola(sfondo[2]),
                    255,
                ]),
            );
        });
        pen_x += advance;
    }
    Ok(())
}

/// Font e dimensione usati per il testo dell'overlay.
struct StileTesto<'a> {
    font: &'a Font,
    font_size: f32,
}

/// Disegna il titolo nella fascia scurita.
fn disegna_titolo(
    immagine: &mut RgbaImage,
    box_y: u32,
    stile: &StileTesto<'_>,
    title: &str,
) -> Result<()> {
    disegna_testo(
        immagine,
        stile.font,
        30,
        box_y as i32 + 20,
        stile.font_size,
        title,
    )
}

/// Adatta la spiegazione alla larghezza dell'immagine.
fn spiegazione_a_capo(immagine: &RgbaImage, stile: &StileTesto<'_>, explanation: &str) -> String {
    let chars_per_line =
        (f64::from(immagine.width()) / f64::from(stile.font_size * 0.52)).max(20.0) as usize;
    textwrap::fill(explanation, chars_per_line)
}

/// Disegna la spiegazione, a capo automatico, sotto il titolo.
fn disegna_spiegazione(
    img: &mut RgbaImage,
    y0: u32,
    st: &StileTesto<'_>,
    text: &str,
) -> Result<()> {
    let wrapped = spiegazione_a_capo(img, st, text);
    let y = y0 as i32 + 20 + (st.font_size * 1.5) as i32;
    let scala = st.font_size * 0.85;
    disegna_testo(img, st.font, 30, y, scala, &wrapped)
}

/// Verifica che l'immagine abbia dimensioni non nulle.
fn dimensioni_valide((width, height): (u32, u32)) -> Result<(u32, u32)> {
    if width == 0 || height == 0 {
        bail!("Immagine con dimensioni nulle");
    }
    Ok((width, height))
}

/// Salva l'immagine in un file temporaneo e ne restituisce il percorso.
fn salva_overlay(immagine: &RgbaImage) -> Result<PathBuf> {
    let mut overlay = nuovo_file_overlay()?;
    {
        let file = overlay.as_file_mut();
        immagine
            .write_to(file, image::ImageFormat::Jpeg)
            .context("Salvataggio immagine non riuscito")?;
    }
    let (_file, percorso) = overlay
        .keep()
        .context("Persistenza del file temporaneo non riuscita")?;
    Ok(percorso)
}

/// Disegna un overlay scuro con titolo e spiegazione sull'immagine data.
///
/// Restituisce il percorso del file temporaneo generato. I dati in ingresso
/// sono limitati in dimensione per evitare consumi non controllati.
pub fn draw_overlay(image_bytes: &[u8], title: &str, explanation: &str) -> Result<PathBuf> {
    valida_overlay(image_bytes, title, explanation)?;

    let font = find_system_font().context("Caricamento font per overlay non riuscito")?;
    let image = image::load_from_memory(image_bytes).context("Decodifica immagine non riuscita")?;
    let (width, height) = dimensioni_valide(image.dimensions())?;
    let scale_factor = (f64::from(width) / 1920.0).max(1.0) as f32;
    let stile = StileTesto {
        font: &font,
        font_size: 22.0 * scale_factor,
    };
    let mut rgba_image = image.to_rgba8();
    let box_y = scurisci_fascia(&mut rgba_image, height);
    disegna_titolo(&mut rgba_image, box_y, &stile, title)?;
    disegna_spiegazione(&mut rgba_image, box_y, &stile, explanation)?;
    salva_overlay(&rgba_image)
}

/// Crea il file di overlay con nome casuale e apertura esclusiva.
///
/// `tempfile` genera un nome imprevedibile e crea il file in modalità
/// `create_new` (O_EXCL), evitando il nome prevedibile basato su timestamp
/// e le race condition con symlink in directory condivise.
fn nuovo_file_overlay() -> Result<NamedTempFile> {
    Builder::new()
        .prefix("nasa-wallpaper-overlay-")
        .suffix(".jpg")
        .tempfile()
        .context("Creazione del file temporaneo non riuscita")
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
    fn font_non_valido_viene_rifiutato() {
        assert!(Font::new(&[0u8; 32][..], 0).is_none());
    }

    #[test]
    fn file_overlay_has_unique_unpredictable_name() {
        let primo = nuovo_file_overlay().expect("primo file temporaneo");
        let secondo = nuovo_file_overlay().expect("secondo file temporaneo");
        assert_ne!(primo.path(), secondo.path());
        let nome = primo
            .path()
            .file_name()
            .and_then(|n| n.to_str())
            .expect("nome file valido");
        assert!(nome.starts_with("nasa-wallpaper-overlay-"));
        assert!(nome.ends_with(".jpg"));
    }

    #[test]
    fn input_non_validi_vengono_rifiutati() {
        assert!(draw_overlay(&[], "Titolo", "Testo").is_err());
        let titolo_lungo = "x".repeat(MAX_TESTO + 1);
        assert!(draw_overlay(&[0u8; 4], &titolo_lungo, "Testo").is_err());
    }
}
