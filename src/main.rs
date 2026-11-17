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

pub mod models;
pub mod sources;
pub mod utils;

pub use models::*;
pub use sources::apod::*;
pub use sources::nasa_image::*;
pub use sources::unsplash::*;
pub use utils::*;

use chrono::NaiveDate;
use clap::{Arg, Command};
use colored::*;
use std::ffi::{OsStr, OsString};
use std::process;

const VERSION: &str = "2.1.3";
const MSG_DONE: &str = "Done";
const MSG_CHANGING: &str = "Changing wallpaper...";

/// Builds the CLI specification for `nasa-wallpaper`.
fn cli() -> Command {
    Command::new("nasa-wallpaper")
        .version(VERSION)
        .author("David Población Criado <davidpblcrd@protonmail.com>")
        .about("Change your desktop wallpaper with a NASA image")
        .subcommand_required(true)
        .arg_required_else_help(true)
        .subcommand(
            Command::new("apod")
                .about("Download and set the Astronomy Picture of the Day (APOD)")
                .arg(
                    Arg::new("date")
                        .short('d')
                        .long("date")
                        .value_name("YYYY-MM-DD")
                        .help("Date of the APOD to fetch (defaults to today in US Eastern time)"),
                )
                .arg(
                    Arg::new("key")
                        .short('k')
                        .long("key")
                        .value_name("API_KEY")
                        .hide(true)
                        .help("Deprecated: the APOD API no longer requires an API key"),
                )
                .arg(
                    Arg::new("low")
                        .short('l')
                        .long("low")
                        .action(clap::ArgAction::SetTrue)
                        .help("Use a lower resolution version of the APOD image (1024 px wide)"),
                ),
        )
        .subcommand(
            Command::new("unsplash")
                .about("Download and set a NASA wallpaper from Unsplash"),
        )
        .subcommand(
            Command::new("nasa_image")
                .about("Download and set a random image from the NASA Image and Video Library")
                .arg(
                    Arg::new("query")
                        .short('q')
                        .long("query")
                        .value_name("SEARCH_QUERY")
                        .help("Free text search terms"),
                )
                .arg(
                    Arg::new("center")
                        .short('c')
                        .long("center")
                        .value_name("CENTER")
                        .help("NASA center that published the media"),
                )
                .arg(
                    Arg::new("location")
                        .short('L')
                        .long("location")
                        .value_name("LOCATION")
                        .help("Location terms to search for"),
                )
                .arg(
                    Arg::new("nasa_id")
                        .short('i')
                        .long("nasa_id")
                        .value_name("NASA_ID")
                        .help("Media asset NASA ID"),
                )
                .arg(
                    Arg::new("photographer")
                        .short('p')
                        .long("photographer")
                        .value_name("NAME")
                        .help("Primary photographer name"),
                )
                .arg(
                    Arg::new("title")
                        .short('t')
                        .long("title")
                        .value_name("TITLE")
                        .help("Terms to search for in title fields"),
                )
                .arg(
                    Arg::new("year_start")
                        .short('s')
                        .long("year_start")
                        .value_name("YYYY")
                        .help("Start year for results (defaults to 1900)"),
                )
                .arg(
                    Arg::new("year_end")
                        .short('e')
                        .long("year_end")
                        .value_name("YYYY")
                        .help("End year for results (defaults to the current year)"),
                ),
        )
        .subcommand(
            Command::new("license")
                .about("Display license information"),
        )
}

/// Normalizes legacy CLI flags for backwards compatibility.
///
/// Rewrites `-a` to `apod` and `-n` to `nasa_image`.
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
