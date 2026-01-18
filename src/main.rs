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

const API_KEY: &str = "DEMO_KEY";
const VERSION: &str = "2.1.1";
const MSG_DONE: &str = "Done";
const MSG_CHANGING: &str = "Changing wallpaper...";
const URL_UNSPLASH: &str = "https://source.unsplash.com/user/nasa";

type WallpaperResult<T> = Result<T, Box<dyn Error>>;

#[derive(Deserialize)]
struct Apod {
    #[serde(default)]
    copyright: String,
    date: String,
    explanation: String,
    #[serde(default)]
    hdurl: String,
    media_type: String,
    title: String,
    url: String,
}

impl fmt::Display for Apod {
    /// Formats an [`Apod`] instance for terminal output.
    ///
    /// This is the text shown by `println!("{apod}")`.
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

/// Fetches NASA's Astronomy Picture of the Day (APOD) metadata.
///
/// # Arguments
/// - `date`: Date string in `YYYY-MM-DD` format.
/// - `api_key`: NASA API key (e.g. `DEMO_KEY`).
///
/// # Returns
/// An [`Apod`] struct populated from the API response.
///
/// # Errors
/// Returns a [`reqwest::Error`] if the request fails or the response body
/// cannot be deserialized.
fn get_apod(date: &str, api_key: &str) -> Result<Apod, reqwest::Error> {
    let request_url = format!(
        "https://api.nasa.gov/planetary/apod?api_key={api_key}&date={date}",
        api_key = api_key,
        date = date
    );

    let response = reqwest::blocking::get(&request_url)?.json::<Apod>()?;
    Ok(response)
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
/// - `hd`: When `true`, use `apod.hdurl`; otherwise use `apod.url`.
///
/// # Errors
/// Returns an error if the underlying wallpaper backend fails to download or
/// set the image.
///
/// # Notes
/// If `apod.hdurl` is empty for a given day, using `hd = true` may fail.
fn set_wallpaper(apod: &Apod, hd: bool) -> WallpaperResult<()> {
    if hd {
        wallpaper::set_from_url(&apod.hdurl)?;
    } else {
        wallpaper::set_from_url(&apod.url)?;
    }
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
                .arg(Arg::new("date").short('d').long("date").value_name("DATE"))
                .arg(Arg::new("key").short('k').long("key").value_name("API_KEY"))
                .arg(
                    Arg::new("low")
                        .short('l')
                        .long("low")
                        .action(clap::ArgAction::SetTrue),
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
            let (year, month, day) = get_today_est();
            let today = format!("{}-{}-{}", year, month, day);
            let date = sub_matches
                .get_one::<String>("date")
                .map(|s| s.as_str())
                .unwrap_or(&today);
            let api_key = sub_matches
                .get_one::<String>("key")
                .map(|s| s.as_str())
                .unwrap_or(API_KEY);
            let hd = sub_matches.get_flag("low");

            if let Ok(apod) = get_apod(date, api_key) {
                println!("{}", apod);
                if apod.media_type != "image" {
                    print!("{}, {}", "The date you have chosen for the APOD has no image. See the original content in: {}".yellow(), apod.url.yellow());
                    return;
                }
                println!("{}", MSG_CHANGING.yellow());
                if let Err(err) = set_wallpaper(&apod, hd) {
                    println!("{}", format!("Error: {}", err).red());
                } else {
                    println!("{}", MSG_DONE.green());
                }
            }
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
