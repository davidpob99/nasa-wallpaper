//! Renderer di overlay testuale per le immagini APOD.
//!
//! Gestisce gli errori con `anyhow::Context`: nessun errore di filesystem o
//! di decodifica immagine viene silenziosamente ignorato.
//!
//! Font e rasterizzazione usano `swash`, progetto mantenuto basato su
//! `skrifa`. Nessuna dipendenza da `ttf-parser`, non mantenuto secondo
//! RUSTSEC-2026-0192.

use anyhow::{anyhow, bail, Context, Result};
use image::{GenericImageView, Rgba, RgbaImage};
use swash::scale::{image::Image, Render, ScaleContext, Scaler, Source};
use swash::{CacheKey, FontRef, GlyphId};
use std::path::{Path, PathBuf};
use tempfile::{Builder, NamedTempFile};

/// Dimensione massima accettata per un'immagine (32 MiB).
const MAX_IMAGE_BYTES: usize = 32 * 1024 * 1024;
/// Lunghezza massima del testo di overlay.
const MAX_TESTO: usize = 8_192;
/// Percorsi di font di sistema esaminati in ordine.
const CANDIDATI_FONT: &[&str] = &[
    "C:\\Windows\\Fonts\\segoeui.ttf",
    "C:\\Windows\\Fonts\\arial.ttf",
    "C:\\Windows\\Fonts\\calibri.ttf",
    "/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf",
    "/usr/share/fonts/TTF/DejaVuSans.ttf",
    "/usr/share/fonts/truetype/freefont/FreeSans.ttf",
    "/usr/share/fonts/truetype/liberation/LiberationSans-Regular.ttf",
    "/usr/share/fonts/noto/NotoSans-Regular.ttf",
    "/System/Library/Fonts/SFCompact.ttf",
    "/System/Library/Fonts/SFNS.ttf",
    "/System/Library/Fonts/Helvetica.ttc",
    "/Library/Fonts/Arial.ttf",
];

/// Byte del font e metadati necessari a `swash` per riutilizzare la cache.
pub struct FontData {
    bytes: Vec<u8>,
    offset: u32,
    key: CacheKey,
}

impl FontData {
    /// Restituisce il riferimento preso in prestito dal file.
    fn as_ref(&self) -> FontRef<'_> {
        FontRef {
            data: &self.bytes,
            offset: self.offset,
            key: self.key,
        }
    }
}

/// Carica e valida un font TTF da percorso esplicito.
fn carica_font(path: &Path) -> Result<FontData> {
    let bytes = std::fs::read(path)
        .with_context(|| format!("Impossibile leggere il font {}", path.display()))?;
    let (offset, key) = {
        let font = FontRef::from_index(&bytes, 0)
            .ok_or_else(|| anyhow!("Font TTF non valido: {}", path.display()))?;
        (font.offset, font.key)
    };
    Ok(FontData { bytes, offset, key })
}

/// Prova i percorsi in ordine e restituisce il primo font valido.
fn carica_da_elenco<I, P>(percorsi: I) -> Result<FontData>
where
    I: IntoIterator<Item = P>,
    P: AsRef<Path>,
{
    let mut errore: Option<anyhow::Error> = None;
    for candidato in percorsi {
        let percorso = candidato.as_ref();
        if percorso.is_file() {
            match carica_font(percorso) {
                Ok(font) => return Ok(font),
                Err(err) => errore = Some(err),
            }
        }
    }
    errore_finale(errore)
}

/// Restituisce l'errore contestualizzato quando nessun font è valido.
fn errore_finale(errore: Option<anyhow::Error>) -> Result<FontData> {
    match errore {
        Some(err) => Err(err.context("Nessun font di sistema valido trovato")),
        None => bail!("Nessun font di sistema trovato: impostare NASA_WALLPAPER_FONT"),
    }
}

/// Trova un font di sistema; `NASA_WALLPAPER_FONT` ha priorità ed è obbligatorio se invalido.
pub fn find_system_font() -> Result<FontData> {
    if let Some(font_path) = std::env::var_os("NASA_WALLPAPER_FONT") {
        let percorso = PathBuf::from(font_path);
        return carica_font(&percorso)
            .context("NASA_WALLPAPER_FONT non punta a un font TTF valido");
    }
    let mut percorsi: Vec<PathBuf> = CANDIDATI_FONT.iter().map(PathBuf::from).collect();
    #[cfg(target_os = "windows")]
    if let Ok(windir) = std::env::var("WINDIR") {
        percorsi.push(Path::new(&windir).join("Fonts").join("arial.ttf"));
    }
    carica_da_elenco(percorsi)
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

/// Mappa i caratteri del testo agli identificatori di glyph del font.
fn mappa_glyph(font: FontRef<'_>, testo: &str) -> Vec<GlyphId> {
    let charmap = font.charmap();
    testo.chars().map(|c| charmap.map(c)).collect()
}

/// Produce la maschera di un glyph alla dimensione richiesta.
fn render_glyph(scaler: &mut Scaler<'_>, id: GlyphId) -> Option<Image> {
    Render::new(&[Source::Outline]).render(scaler, id)
}

/// Restituisce l'avanzamento orizzontale del glyph in pixel.
fn avanzamento(font: FontRef<'_>, id: GlyphId, size: f32) -> f32 {
    let unita = f32::from(font.metrics(&[]).units_per_em);
    let avanzamento = font.glyph_metrics(&[]).advance_width(id) * size / unita;
    if avanzamento > 0.0 { avanzamento } else { size * 0.6 }
}

/// Fonde un pixel bianco sull'immagine in base alla copertura del glyph.
fn mescola(immagine: &mut RgbaImage, x: i32, y: i32, alpha: u8) {
    if x < 0 || y < 0 || x >= immagine.width() as i32 || y >= immagine.height() as i32 {
        return;
    }
    let (x, y) = (x as u32, y as u32);
    let copertura = f32::from(alpha) / 255.0;
    let sfondo = immagine.get_pixel(x, y).0;
    let canale = |v: u8| (f32::from(v) * (1.0 - copertura) + 255.0 * copertura).round() as u8;
    immagine.put_pixel(
        x,
        y,
        Rgba([
            canale(sfondo[0]),
            canale(sfondo[1]),
            canale(sfondo[2]),
            255,
        ]),
    );
}

/// Compone la maschera del glyph sull'immagine.
fn componi(immagine: &mut RgbaImage, immagine_glyph: &Image, base: (i32, i32)) {
    let larghezza = immagine_glyph.placement.width as usize;
    let altezza = immagine_glyph.placement.height as usize;
    let inizio_x = base.0 + immagine_glyph.placement.left;
    let inizio_y = base.1 + immagine_glyph.placement.top;
    for riga in 0..altezza {
        for colonna in 0..larghezza {
            let alpha = immagine_glyph.data[riga * larghezza + colonna];
            if alpha > 0 {
                mescola(
                    immagine,
                    inizio_x + colonna as i32,
                    inizio_y + riga as i32,
                    alpha,
                );
            }
        }
    }
}

/// Posizione della penna durante il disegno di più righe.
struct Penna {
    x: f32,
    y: i32,
    interlinea: i32,
}

impl Penna {
    fn nuovo(size: f32) -> Self {
        Self {
            x: 0.0,
            y: 0,
            interlinea: (size * 1.25).round() as i32,
        }
    }

    fn a_capo(&mut self) {
        self.x = 0.0;
        self.y += self.interlinea;
    }
}

/// Parametri di una richiesta di disegno testuale.
struct RichiestaTesto<'a> {
    immagine: &'a mut RgbaImage,
    font: &'a FontData,
    x0: i32,
    y0: i32,
    size: f32,
    testo: &'a str,
}

/// Disegna testo bianco gestendo le interruzioni di riga.
fn disegna_testo(richiesta: RichiestaTesto<'_>) -> Result<()> {
    let font = richiesta.font.as_ref();
    let glyphs = mappa_glyph(font, richiesta.testo);
    let mut contesto = ScaleContext::new();
    let mut scaler = contesto.builder(font).size(richiesta.size).build();
    let mut penna = Penna::nuovo(richiesta.size);
    for (ch, id) in richiesta.testo.chars().zip(glyphs) {
        if ch == '\n' {
            penna.a_capo();
            continue;
        }
        if let Some(immagine_glyph) = render_glyph(&mut scaler, id) {
            let base = (richiesta.x0 + penna.x as i32, richiesta.y0 + penna.y);
            componi(&mut *richiesta.immagine, &immagine_glyph, base);
        }
        penna.x += avanzamento(font, id, richiesta.size);
    }
    Ok(())
}

/// Font e dimensione usati per il testo dell'overlay.
struct StileTesto<'a> {
    font: &'a FontData,
    font_size: f32,
}

/// Disegna il titolo nella fascia scurita.
fn disegna_titolo(
    immagine: &mut RgbaImage,
    box_y: u32,
    stile: &StileTesto<'_>,
    title: &str,
) -> Result<()> {
    disegna_testo(RichiestaTesto {
        immagine,
        font: stile.font,
        x0: 30,
        y0: box_y as i32 + 20,
        size: stile.font_size,
        testo: title,
    })
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
    disegna_testo(RichiestaTesto {
        immagine: img,
        font: st.font,
        x0: 30,
        y0: y,
        size: st.font_size * 0.85,
        testo: &wrapped,
    })
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
pub fn draw_overlay(image_bytes: &[u8], title: &str, explanation: &str) -> Result<PathBuf> {
    valida_overlay(image_bytes, title, explanation)?;
    let font = find_system_font().context("Caricamento font per overlay non riuscito")?;
    let image = image::load_from_memory(image_bytes).context("Decodifica immagine non riuscita")?;
    let (width, height) = dimensioni_valide(image.dimensions())?;
    let scala = (f64::from(width) / 1920.0).max(1.0) as f32;
    let stile = StileTesto {
        font: &font,
        font_size: 22.0 * scala,
    };
    let mut rgba = image.to_rgba8();
    let fascia = scurisci_fascia(&mut rgba, height);
    disegna_titolo(&mut rgba, fascia, &stile, title)?;
    disegna_spiegazione(&mut rgba, fascia, &stile, explanation)?;
    salva_overlay(&rgba)
}

/// Crea il file di overlay con nome casuale e apertura esclusiva.
fn nuovo_file_overlay() -> Result<NamedTempFile> {
    Builder::new()
        .prefix("nasa-wallpaper-overlay-")
        .suffix(".jpg")
        .tempfile()
        .context("Creazione del file temporaneo non riuscita")
}

/// Conta i pixel sufficientemente chiari in un'immagine.
#[cfg(test)]
fn pixel_chiari(immagine: &RgbaImage) -> usize {
    immagine
        .pixels()
        .filter(|p| p.0[0] > 128 && p.0[1] > 128 && p.0[2] > 128)
        .count()
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
        assert!(FontRef::from_index(&[0u8; 32], 0).is_none());
    }

    #[test]
    fn disegna_testo_produce_pixel_visibili() {
        let Ok(font) = find_system_font() else {
            println!("Nessun font di sistema: test di rendering non applicabile");
            return;
        };
        let mut immagine = RgbaImage::from_pixel(640, 360, Rgba([0, 0, 0, 255]));
        disegna_testo(RichiestaTesto {
            immagine: &mut immagine,
            font: &font,
            x0: 20,
            y0: 100,
            size: 20.0,
            testo: "NASA",
        })
        .expect("disegno del testo");
        assert!(pixel_chiari(&immagine) > 0, "nessun pixel disegnato");
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
