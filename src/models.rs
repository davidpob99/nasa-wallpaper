use crate::sources::apod::{html_to_text, low_res_url};
use colored::*;
use std::fmt;

#[derive(Debug, Deserialize)]
pub struct Apod {
    pub date: String,
    pub title: String,
    pub media_type: String,
    /// May contain HTML markup.
    #[serde(default)]
    pub explanation: String,
    /// May contain HTML markup.
    #[serde(default)]
    pub copyright: Option<String>,
    /// URL of the APOD post page (not an image).
    pub url: String,
    /// Full-size image URL, when available.
    #[serde(default)]
    pub hdurl: Option<String>,
}

impl Apod {
    /// Returns the image URL to use as wallpaper, if the APOD has one.
    ///
    /// When `low` is `true`, a reduced-size version is requested (see [`low_res_url`]).
    pub fn image_url(&self, low: bool) -> Option<String> {
        let hdurl = self.hdurl.as_deref().filter(|u| !u.is_empty())?;
        if low {
            Some(low_res_url(hdurl))
        } else {
            Some(hdurl.to_owned())
        }
    }
}

impl fmt::Display for Apod {
    /// Formats an [`Apod`] instance for terminal output.
    ///
    /// This is the text shown by `println!("{apod}")`.
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Title: {}\nDate: {}\nExplanation: {}\nCopyright: {}\nLink: {}",
            html_to_text(&self.title).bold().italic(),
            self.date.italic(),
            html_to_text(&self.explanation),
            html_to_text(self.copyright.as_deref().unwrap_or("")),
            self.url
        )
    }
}

#[derive(Deserialize)]
pub struct NasaImage {
    pub nasa_id: String,
    pub title: String,
    pub center: String,
    pub description: String,
    pub date: String,
    pub url: String,
}

impl fmt::Display for NasaImage {
    /// Formats a [`NasaImage`] instance for terminal output.
    ///
    /// This is the text shown by `println!("{nasa_image}")`.
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Title: {}\nDate: {}\nExplanation: {}\nCenter: {}\nNASA id: {}",
            self.title.bold().italic(),
            self.date.italic(),
            self.description,
            self.center,
            self.nasa_id
        )
    }
}
