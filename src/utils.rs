use chrono::prelude::*;
use chrono_tz::Tz;
use chrono_tz::US::Eastern;
use anyhow::{Context, Result};
use std::io::copy;
use tempfile::Builder;

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

pub fn set_from_url(url: &str) -> Result<()> {
    let mut response = reqwest::blocking::get(url)
        .context("Failed to download image from URL")?;
    
    let suffix = if url.to_lowercase().ends_with(".png") { ".png" } else { ".jpg" };
    
    let mut tmp_file = Builder::new()
        .prefix("nasa-wallpaper")
        .suffix(suffix)
        .tempfile()
        .context("Failed to create temporary file")?;

    copy(&mut response, &mut tmp_file)
        .context("Failed to write image to temporary file")?;

    let path = tmp_file.path().to_str()
        .context("Failed to get temporary file path")?;

    wallpaper::set_from_path(path)
        .map_err(|e| anyhow::anyhow!("Failed to set wallpaper: {}", e))?;

    // On Windows, the wallpaper setter might need the file to persist for a bit 
    // or it might copy it. wallpaper crate seems to handle it, but we keep 
    // the tmp_file alive until the end of this function.
    Ok(())
}
