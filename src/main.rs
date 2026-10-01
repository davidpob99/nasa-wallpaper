/*
   Copyright 2019-2026 David Población Criado

   Licensed under the Apache License, Version 2.0 (the "License");
   you may not use this file except in compliance with the License.
   You may obtain a copy of the License at

       https://www.apache.org/licenses/LICENSE-2.0

   Unless required by applicable law or agreed to in writing, software
   distributed under the License is distributed on an "AS IS" BASIS,
   WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
   See the License for the specific language governing permissions and
   limitations under the License.
*/

extern crate chrono;
extern crate chrono_tz;
extern crate clap;
extern crate colored;
extern crate json;
extern crate rand;
extern crate reqwest;
extern crate wallpaper;

#[macro_use]
extern crate serde_derive;

use chrono::prelude::*;
use chrono_tz::Tz;
use chrono_tz::US::Eastern;
use clap::{Arg, Command};
use colored::*;
use rand::Rng;
use std::error::Error;
use std::ffi::{OsStr, OsString};
use std::fmt;
use std::process;

const LICENSE_TEXT: &str = r#"
   Copyright 2019-2026 David Población Criado

   Licensed under the Apache License, Version 2.0 (the 'License');
   you may not use this file except in compliance with the License.
   You may obtain a copy of the License at

       https://www.apache.org/licenses/LICENSE-2.0

   Unless required by applicable law or agreed to in writing, software
   distributed under the License is distributed on an "AS IS" BASIS,
   WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
"#;

const VERSION: &str = "2.1.3";
const MSG_DONE: &str = "Done";
const MSG_CHANGING: &str = "Changing wallpaper...";
const URL_UNSPLASH: &str = "https://source.unsplash.com/user/nasa";
/// APOD Basic JSON endpoint
const APOD_API_URL: &str = "https://science.nasa.gov/wp-json/wp/v2/apod-basic";
/// Width (in pixels) requested from the image CDN when `--low` is used.
const LOW_RES_WIDTH: u32 = 1024;

type WallpaperResult<T> = Result<T, Box<dyn Error>>;

#[derive(Debug, Deserialize)]
struct Apod {
    date: String,
    title: String,
    media_type: String,
    /// May contain HTML markup.
    #[serde(default)]
    explanation: String,
    /// May contain HTML markup.
    #[serde(default)]
    copyright: Option<String>,
    /// URL of the APOD post page (not an image).
    url: String,
    /// Full-size image URL, when available.
    #[serde(default)]
    hdurl: Option<String>,
}

impl Apod {
    /// Returns the image URL to use as wallpaper, if the APOD has one.
    ///
    /// When `low` is `true`, a reduced-size version is requested (see [`low_res_url`]).
    fn image_url(&self, low: bool) -> Option<String> {
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
struct NasaImage {
    nasa_id: String,
    title: String,
    center: String,
    description: String,
    date: String,
    url: String,
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

/// Parses a user-supplied APOD date in `YYYY-MM-DD` format.
///
/// Month and day may be written without zero padding (e.g. `2026-9-3`).
fn parse_apod_date(date: &str) -> Result<NaiveDate, chrono::ParseError> {
    NaiveDate::parse_from_str(date.trim(), "%Y-%m-%d")
}

/// Returns the legacy APOD date code (`YYMMDD`) used by the APOD API routes.
fn apod_date_code(date: NaiveDate) -> String {
    date.format("%y%m%d").to_string()
}

/// Converts an HTML fragment from the APOD API into plain terminal text.
///
/// Removes a leading `Explanation:` label, turns `<br>` into line breaks,
/// strips every other tag and decodes common HTML entities.
fn html_to_text(html: &str) -> String {
    let mut text = String::with_capacity(html.len());
    let mut rest = html;
    while let Some(start) = rest.find('<') {
        text.push_str(&rest[..start]);
        match rest[start..].find('>') {
            Some(end) => {
                let tag = rest[start + 1..start + end].trim().to_ascii_lowercase();
                let name = tag.trim_start_matches('/').split([' ', '/']).next();
                if name == Some("br") {
                    text.push('\n');
                }
                rest = &rest[start + end + 1..];
            }
            None => {
                text.push_str(&rest[start..]);
                rest = "";
            }
        }
    }
    text.push_str(rest);

    let text = decode_html_entities(&text);
    let text = text.trim();
    text.strip_prefix("Explanation:")
        .unwrap_or(text)
        .trim()
        .to_owned()
}

/// Decodes named entities commonly found in APOD texts and numeric entities.
fn decode_html_entities(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut rest = text;
    while let Some(start) = rest.find('&') {
        out.push_str(&rest[..start]);
        let after = &rest[start + 1..];
        let decoded = after.find(';').filter(|&end| end <= 10).and_then(|end| {
            let entity = &after[..end];
            let c = match entity {
                "amp" => Some('&'),
                "lt" => Some('<'),
                "gt" => Some('>'),
                "quot" => Some('"'),
                "apos" => Some('\''),
                "nbsp" => Some(' '),
                _ => entity
                    .strip_prefix("#x")
                    .or_else(|| entity.strip_prefix("#X"))
                    .and_then(|hex| u32::from_str_radix(hex, 16).ok())
                    .or_else(|| entity.strip_prefix('#').and_then(|dec| dec.parse().ok()))
                    .and_then(char::from_u32),
            };
            c.map(|c| (c, end))
        });
        match decoded {
            Some((c, end)) => {
                out.push(c);
                rest = &after[end + 1..];
            }
            None => {
                out.push('&');
                rest = after;
            }
        }
    }
    out.push_str(rest);
    out
}

/// Returns a reduced-size version of an APOD image URL.
///
/// Only URLs served by NASA's resizing CDN (`/dynamicimage/`) can be resized;
/// any other URL is returned unchanged.
fn low_res_url(url: &str) -> String {
    if !url.contains("/dynamicimage/") {
        return url.to_owned();
    }
    let base = url.split(['?', '#']).next().unwrap_or(url);
    format!("{base}?w={LOW_RES_WIDTH}&fit=clip")
}

/// Fetches NASA's Astronomy Picture of the Day (APOD) metadata.
///
/// # Arguments
/// - `base_url`: URL of the APOD API (normally [`APOD_API_URL`]).
/// - `date`: Day of the APOD to fetch.
///
/// # Returns
/// An [`Apod`] struct populated from the API response.
///
/// # Errors
/// Returns an error if the request fails, the API answers with a non-2xx
/// status (the API's error message is included when available) or the
/// response body cannot be deserialized.
fn get_apod(base_url: &str, date: NaiveDate) -> WallpaperResult<Apod> {
    let request_url = format!(
        "{}/{}",
        base_url.trim_end_matches('/'),
        apod_date_code(date)
    );

    let response = reqwest::blocking::get(&request_url)?;
    let status = response.status();
    if !status.is_success() {
        let body = response.text().unwrap_or_default();
        // Error bodies look like `{"code": "apod_basic_not_found", "message": "APOD not found."}`.
        let message = match json::parse(&body).ok().and_then(|err| {
            err["message"]
                .as_str()
                .map(|m| format!("{} [{}]", m, err["code"]))
        }) {
            Some(detail) => format!("APOD API error ({}): {}", status.as_u16(), detail),
            None => format!("APOD API error ({})", status),
        };
        return Err(message.into());
    }

    Ok(response.json::<Apod>()?)
}

/// Fetches a random image from the NASA Image and Video Library.
///
/// This performs a search against `https://images-api.nasa.gov/search` and then
/// chooses a random result (potentially from a random page).
///
/// # Arguments
/// - `q`: Free text search terms.
/// - `center`: NASA center which published the media.
/// - `location`: Terms to search for in “Location” fields.
/// - `nasa_id`: The media asset’s NASA ID.
/// - `photographer`: The primary photographer’s name.
/// - `title`: Terms to search for in “Title” fields.
/// - `year_start`: Start year for results (`YYYY`).
/// - `year_end`: End year for results (`YYYY`).
///
/// # Returns
/// A [`NasaImage`] struct containing the selected item's metadata and a direct URL.
///
/// # Panics
/// This function uses `expect`/`unwrap` internally and may panic on network,
/// parsing, or response-shape errors.
///
/// # Exits
/// If the search yields zero results, this prints a message and terminates the
/// process with a non-zero exit code.
#[allow(clippy::too_many_arguments)]
fn get_nasa_image(
    q: &str,
    center: &str,
    location: &str,
    nasa_id: &str,
    photographer: &str,
    title: &str,
    year_start: &str,
    year_end: &str,
) -> NasaImage {
    let mut request_url = format!(
        "https://images-api.nasa.gov/search?media_type=image&q={q}&center={center}&location={location}&nasa_id={nasa_id}&photographer={photographer}&title={title}&year_start={year_start}&year_end={year_end}",
        q = q,
        center = center,
        location = location,
        nasa_id = nasa_id,
        photographer = photographer,
        title = title,
        year_start = year_start,
        year_end = year_end
    );

    let response_text = reqwest::blocking::get(&request_url)
        .expect("Failed to fetch NASA image")
        .text()
        .unwrap();
    let mut response_json = json::parse(&response_text).unwrap();

    let num_hits = response_json["collection"]["metadata"]["total_hits"]
        .as_usize()
        .unwrap_or(0);

    print!("Number of results: {}, {}", num_hits, request_url);
    if num_hits == 0 {
        println!("Couldn't find the file you're looking for. Try another tag.");
        process::exit(0x0100);
    }

    let pages = if (num_hits / 100) - 1 <= 100 {
        (num_hits / 100) - 1
    } else {
        100
    };
    let mut rng = rand::rng();
    let index_page = rng.random_range(0..=pages);
    request_url.push_str(&format!("&page={}", index_page));

    let response_text = reqwest::blocking::get(&request_url)
        .expect("Failed to fetch NASA image page")
        .text()
        .unwrap();
    response_json = json::parse(&response_text).unwrap();

    let index = if num_hits < 7 {
        num_hits - 1
    } else {
        rng.random_range(0..100)
    };
    let items = &response_json["collection"]["items"];
    let item = &items[index];
    let data = &item["data"][0];
    let url_collection = item["href"].as_str().unwrap();

    let response_collection = json::parse(
        &reqwest::blocking::get(url_collection)
            .unwrap()
            .text()
            .unwrap(),
    )
    .unwrap();

    let mut date = data["date_created"].as_str().unwrap().to_owned();
    date.truncate(10);

    NasaImage {
        nasa_id: data["nasa_id"].as_str().unwrap().to_owned(),
        title: data["title"].as_str().unwrap().to_owned(),
        center: data["center"].as_str().unwrap().to_owned(),
        description: data["description"].as_str().unwrap().to_owned(),
        date,
        url: response_collection[0].as_str().unwrap().to_owned(),
    }
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
fn set_wallpaper(apod: &Apod, low: bool) -> WallpaperResult<()> {
    let url = apod
        .image_url(low)
        .ok_or("The APOD for this date has no image URL")?;
    wallpaper::set_from_url(&url)?;
    Ok(())
}

/// Prints the program license text to stdout.
fn print_license() {
    println!("{}", LICENSE_TEXT);
}

/// Returns today's date in US Eastern time (EST/EDT).
///
/// The APOD "day" is keyed off US Eastern time, so using this avoids fetching
/// "tomorrow" in other time zones.
fn get_today_est() -> (i32, u32, u32) {
    let est_now: DateTime<Tz> = Utc::now().with_timezone(&Eastern);
    (est_now.year(), est_now.month(), est_now.day())
}

/// Builds the CLI definition (commands/flags) for `nasa-wallpaper`.
fn cli() -> Command {
    Command::new("nasa-wallpaper")
        .version(VERSION)
        .author("David Población Criado")
        .about("Change desktop wallpaper with NASA images")
        .arg_required_else_help(true)
        .subcommand(
            Command::new("apod")
                .about("Get the APOD (Astronomical Picture of the Day)")
                .arg(
                    Arg::new("date")
                        .short('d')
                        .long("date")
                        .value_name("DATE")
                        .help("Date of the APOD. Format: YYYY-MM-DD (default: today)"),
                )
                // Deprecated: the new APOD API does not need a key. Kept (hidden)
                // so existing scripts keep working.
                .arg(
                    Arg::new("key")
                        .short('k')
                        .long("key")
                        .value_name("API_KEY")
                        .hide(true),
                )
                .arg(
                    Arg::new("low")
                        .short('l')
                        .long("low")
                        .action(clap::ArgAction::SetTrue)
                        .help("Use a lower resolution image"),
                ),
        )
        .subcommand(
            Command::new("nasa_image")
                .about("Get a random image from the NASA Image Library (https://images.nasa.gov)")
                .arg(
                    Arg::new("query")
                        .short('q')
                        .long("query")
                        .value_name("Q")
                        .action(clap::ArgAction::Set)
                        .help("Free text search terms to compare to all indexed metadata"),
                )
                .arg(
                    Arg::new("center")
                        .short('c')
                        .long("center")
                        .value_name("CENTER")
                        .action(clap::ArgAction::Set)
                        .help("NASA center which published the media"),
                )
                .arg(
                    Arg::new("location")
                        .short('o')
                        .long("location")
                        .value_name("LOCATION")
                        .action(clap::ArgAction::Set)
                        .help("Terms to search for in “Location” fields"),
                )
                .arg(
                    Arg::new("nasa_id")
                        .short('i')
                        .long("nasa_id")
                        .value_name("NASA_ID")
                        .action(clap::ArgAction::Set)
                        .help("The media asset’s NASA ID"),
                )
                .arg(
                    Arg::new("photographer")
                        .short('p')
                        .long("phtographer")
                        .value_name("PHOTOGRAPHER")
                        .action(clap::ArgAction::Set)
                        .help("The primary photographer’s name"),
                )
                .arg(
                    Arg::new("title")
                        .short('t')
                        .long("title")
                        .value_name("TITLE")
                        .action(clap::ArgAction::Set)
                        .help("Terms to search for in “Title” fields"),
                )
                .arg(
                    Arg::new("year_start")
                        .long("year_start")
                        .value_name("YEAR_START")
                        .action(clap::ArgAction::Set)
                        .help("The start year for results. Format: YYYY"),
                )
                .arg(
                    Arg::new("year_end")
                        .long("year_end")
                        .value_name("YEAR_END")
                        .action(clap::ArgAction::Set)
                        .help("The end year for results. Format: YYYY"),
                ),
        )
        .subcommand(Command::new("unsplash").about(
            "Get a random image from the NASA's account in Unsplash (https://unsplash.com/@nasa)",
        ))
        .subcommand(Command::new("license").about("Print the license of this program"))
}

/// Normalizes arguments for backwards-compatible shorthand flags.
///
/// Converts legacy forms like `nasa-wallpaper -a ...` into
/// `nasa-wallpaper apod ...` and similarly `-n` to `nasa_image`.
fn normalize_args(mut args: Vec<OsString>) -> Vec<OsString> {
    // Backwards-compatible shorthand flags:
    // `nasa-wallpaper -a ...` => `nasa-wallpaper apod ...`
    // `nasa-wallpaper -n ...` => `nasa-wallpaper nasa_image ...`
    if args.len() >= 2 {
        if args[1] == OsStr::new("-a") {
            args[1] = OsString::from("apod");
        } else if args[1] == OsStr::new("-n") {
            args[1] = OsString::from("nasa_image");
        }
    }
    args
}

/// Program entry point.
///
/// Parses CLI arguments and dispatches to the chosen subcommand.
fn main() {
    let args = normalize_args(std::env::args_os().collect());
    let matches = cli().get_matches_from(args);

    match matches.subcommand() {
        Some(("apod", sub_matches)) => {
            let date = match sub_matches.get_one::<String>("date") {
                Some(date) => match parse_apod_date(date) {
                    Ok(date) => date,
                    Err(err) => {
                        eprintln!(
                            "{}",
                            format!("Error: invalid date '{}' ({}). Use YYYY-MM-DD", date, err)
                                .red()
                        );
                        process::exit(1);
                    }
                },
                None => {
                    let (year, month, day) = get_today_est();
                    NaiveDate::from_ymd_opt(year, month, day).expect("valid date")
                }
            };
            if sub_matches.get_one::<String>("key").is_some() {
                println!(
                    "{}",
                    "Warning: the APOD API no longer requires an API key. The --key option is ignored."
                        .yellow()
                );
            }
            let low = sub_matches.get_flag("low");

            let apod = match get_apod(APOD_API_URL, date) {
                Ok(apod) => apod,
                Err(err) => {
                    eprintln!("{}", format!("Error: {}", err).red());
                    process::exit(1);
                }
            };
            println!("{}", apod);
            if apod.media_type != "image" {
                println!(
                    "{} {}",
                    "The date you have chosen for the APOD has no image. See the original content in:"
                        .yellow(),
                    apod.url.yellow()
                );
                return;
            }
            println!("{}", MSG_CHANGING.yellow());
            if let Err(err) = set_wallpaper(&apod, low) {
                eprintln!("{}", format!("Error: {}", err).red());
                process::exit(1);
            }
            println!("{}", MSG_DONE.green());
        }
        Some(("unsplash", _)) => {
            println!("{}", MSG_CHANGING.yellow());
            wallpaper::set_from_url(URL_UNSPLASH).unwrap();
            println!("{}", MSG_DONE.green());
        }
        Some(("nasa_image", sub_matches)) => {
            let q = sub_matches
                .get_one::<String>("query")
                .map(|s| s.as_str())
                .unwrap_or("");
            let center = sub_matches
                .get_one::<String>("center")
                .map(|s| s.as_str())
                .unwrap_or("");
            let location = sub_matches
                .get_one::<String>("location")
                .map(|s| s.as_str())
                .unwrap_or("");
            let nasa_id = sub_matches
                .get_one::<String>("nasa_id")
                .map(|s| s.as_str())
                .unwrap_or("");
            let photographer = sub_matches
                .get_one::<String>("photographer")
                .map(|s| s.as_str())
                .unwrap_or("");
            let title = sub_matches
                .get_one::<String>("title")
                .map(|s| s.as_str())
                .unwrap_or("");
            let year_start = sub_matches
                .get_one::<String>("year_start")
                .map(|s| s.as_str())
                .unwrap_or("1900");
            let (est_year, _, _) = get_today_est();
            let est_year_str = est_year.to_string();
            let year_end = sub_matches
                .get_one::<String>("year_end")
                .map(|s| s.as_str())
                .unwrap_or(&est_year_str);

            let nasa_image = get_nasa_image(
                q,
                center,
                location,
                nasa_id,
                photographer,
                title,
                year_start,
                year_end,
            );
            println!("{}", nasa_image);
            println!("{}", MSG_CHANGING.yellow());
            wallpaper::set_from_url(&nasa_image.url).unwrap();
            println!("{}", MSG_DONE.green());
        }
        Some(("license", _)) => {
            print_license();
        }
        _ => {}
    }
}

#[cfg(test)]
mod tests;
