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

pub fn set_from_url(url: &str) -> Result<()> {
    let mut response = reqwest::blocking::get(url)
        .context("Failed to download image from URL")?;
    
    let suffix = if url.to_lowercase().ends_with(".png") { ".png" } else { ".jpg" };
    
    // Create a persistent file in the temp directory instead of using tempfile
    // This ensures Windows has time to read the file before it's deleted
    let temp_dir = std::env::temp_dir();
    let file_name = format!("nasa-wallpaper-{}{}", chrono::Utc::now().timestamp(), suffix);
    let file_path = temp_dir.join(file_name);
    
    let mut file = std::fs::File::create(&file_path)
        .context("Failed to create wallpaper file")?;

    copy(&mut response, &mut file)
        .context("Failed to write image to file")?;
    
    // Ensure the file is flushed to disk
    drop(file);

    let path_str = file_path.to_str()
        .context("Failed to get file path")?;

    wallpaper::set_from_path(path_str)
        .map_err(|e| anyhow::anyhow!("Failed to set wallpaper: {}", e))?;

    // Note: We intentionally don't delete the file here to ensure Windows
    // has time to process it. Old wallpaper files can be cleaned up manually
    // from the temp directory if needed.
    Ok(())
}
