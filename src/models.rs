use colored::*;
use serde::Deserialize;
use std::fmt;

#[derive(Deserialize, Debug)]
pub struct Apod {
    #[serde(default)]
    pub copyright: String,
    pub date: String,
    pub explanation: String,
    #[serde(default)]
    pub hdurl: String,
    pub media_type: String,
    pub title: String,
    pub url: String,
}

impl fmt::Display for Apod {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Title: {}\nDate: {}\nExplanation: {}\nCopyright: {}",
            self.title.bold().italic(),
            self.date.italic(),
            self.explanation,
            self.copyright
        )
    }
}

#[derive(Deserialize, Debug)]
pub struct NasaImage {
    pub nasa_id: String,
    pub title: String,
    pub center: String,
    pub description: String,
    pub date: String,
    pub url: String,
}

impl fmt::Display for NasaImage {
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
