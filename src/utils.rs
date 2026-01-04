use chrono::prelude::*;
use chrono_tz::Tz;
use chrono_tz::US::Eastern;
use anyhow::{Context, Result};
use std::io::copy;

pub const LICENSE_TEXT: &str = r#"
   Copyright 2019 David Población Criado

   Licensed under the Apache License, Version 2.0 (the 'License');
   you may not use this file except in compliance with the License.
   You may obtain a copy of the License at

       https://www.apache.org/licenses/LICENSE-2.0

   Unless required by applicable law or agreed to in writing, software
   distributed under the License is distributed on an "AS IS" BASIS,
   WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
   See the License for the specific language governing permissions and
   limitations under the License.
"#;

pub fn get_today_est() -> (i32, u32, u32) {
    let est_now: DateTime<Tz> = Utc::now().with_timezone(&Eastern);
    (est_now.year(), est_now.month(), est_now.day())
}

pub fn print_license() {
    println!("{}", LICENSE_TEXT);
}

pub fn set_from_url(url: &str, overlay_text: Option<(&str, &str)>) -> Result<()> {
    let mut response = reqwest::blocking::get(url)
        .context("Failed to download image from URL")?;

    let suffix = if url.to_lowercase().ends_with(".png") { ".png" } else { ".jpg" };
    
    let temp_dir = std::env::temp_dir();
    let file_path = if let Some((title, explanation)) = overlay_text {
        // Download to memory
        let mut image_bytes = Vec::new();
        copy(&mut response, &mut image_bytes)?;
        
        // Render overlay
        crate::text_renderer::draw_overlay(&image_bytes, title, explanation)?
    } else {
        // Direct download to file
        let file_name = format!("nasa-wallpaper-{}{}", chrono::Utc::now().timestamp(), suffix);
        let path = temp_dir.join(file_name);
        let mut file = std::fs::File::create(&path)
            .context("Failed to create wallpaper file")?;
        copy(&mut response, &mut file)
            .context("Failed to write image to file")?;
        path
    };

    let path_str = file_path.to_str()
        .context("Failed to get file path")?;

    if let Err(e) = wallpaper::set_from_path(path_str) {
        let err_string = e.to_string();
        if err_string.contains("No such file or directory") || err_string.contains("os error 2") {
             return Err(anyhow::anyhow!("Unable to set wallpaper. This is likely due to missing dependencies for your desktop environment. Please ensure you have the correct backend installed (e.g., 'feh', 'gsettings', 'qdbus'). See README for details.\nOriginal error: {}", e));
        }
        return Err(anyhow::anyhow!("Failed to set wallpaper: {}", e));
    }

    // Note: We intentionally don't delete the file here to ensure Windows
    // has time to process it. Old wallpaper files can be cleaned up manually
    // from the temp directory if needed.
    Ok(())
}
