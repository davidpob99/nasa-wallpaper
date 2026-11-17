use crate::models::Apod;
use chrono::prelude::*;
use chrono_tz::Tz;
use chrono_tz::US::Eastern;
use std::error::Error;

pub const LICENSE_TEXT: &str = r#"
   Copyright 2019-2026 David Población Criado

   Licensed under the Apache License, Version 2.0 (the 'License');
   you may not use this file except in compliance with the License.
   You may obtain a copy of the License at

       https://www.apache.org/licenses/LICENSE-2.0

   Unless required by applicable law or agreed to in writing, software
   distributed under the License is distributed on an "AS IS" BASIS,
   WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
"#;

pub type WallpaperResult<T> = Result<T, Box<dyn Error>>;

pub fn print_license() {
    println!("{}", LICENSE_TEXT);
}

pub fn get_today_est() -> (i32, u32, u32) {
    let est_now: DateTime<Tz> = Utc::now().with_timezone(&Eastern);
    (est_now.year(), est_now.month(), est_now.day())
}

/// Sets the system wallpaper to the APOD image.
///
/// # Arguments
/// - `apod`: APOD metadata previously fetched from the API.
/// - `low`: When `true`, use a reduced-size image (see [`Apod::image_url`]).
///
/// # Errors
/// Returns an error if the APOD has no image URL, or if the underlying
/// wallpaper backend fails to download or set the image.
pub fn set_wallpaper(apod: &Apod, low: bool) -> WallpaperResult<()> {
    let image_url = apod
        .image_url(low)
        .ok_or("No image URL found for this APOD")?;
    wallpaper::set_from_url(&image_url)?;
    Ok(())
}
