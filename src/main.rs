/*
   Copyright 2019 David Población

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
use std::fmt;
use std::process;
use std::error::Error;

const LICENSE_TEXT: &str = r#"
   Copyright 2019 David Población

   Licensed under the Apache License, Version 2.0 (the 'License');
   you may not use this file except in compliance with the License.
   You may obtain a copy of the License at

       https://www.apache.org/licenses/LICENSE-2.0

   Unless required by applicable law or agreed to in writing, software
   distributed under the License is distributed on an "AS IS" BASIS,
   WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
"#;

const API_KEY: &str = "DEMO_KEY";
const VERSION: &str = "2.1.0";
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

/// Recupera APOD da NASA via HTTPS
fn get_apod(date: &str, api_key: &str) -> Result<Apod, reqwest::Error> {
    let request_url = format!(
        "https://api.nasa.gov/planetary/apod?api_key={api_key}&date={date}",
        api_key = api_key,
        date = date
    );

    let response = reqwest::blocking::get(&request_url)?.json::<Apod>()?;
    Ok(response)
}

/// Recupera immagine casuale dalla libreria NASA
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

    if num_hits == 0 {
        println!("Couldn't find the file you're looking for. Try another tag.");
        process::exit(0x0100);
    }

    let pages = if (num_hits / 100) - 1 <= 100 { (num_hits / 100) - 1 } else { 100 };
    let mut rng = rand::rng();
    let index_page = rng.random_range(0..=pages);
    request_url.push_str(&format!("&page={}", index_page));

    let response_text = reqwest::blocking::get(&request_url)
        .expect("Failed to fetch NASA image page")
        .text()
        .unwrap();
    response_json = json::parse(&response_text).unwrap();

    let index = if num_hits < 7 { num_hits - 1 } else { rng.random_range(0..100)};
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

fn set_wallpaper(apod: &Apod, hd: bool) -> WallpaperResult<()> {
    if hd {
        wallpaper::set_from_url(&apod.hdurl)?;
    } else {
        wallpaper::set_from_url(&apod.url)?;
    }
    Ok(())
}

fn print_license() {
    println!("{}", LICENSE_TEXT);
}

fn get_today_est() -> (i32, u32, u32) {
    let est_now: DateTime<Tz> = Utc::now().with_timezone(&Eastern);
    (est_now.year(), est_now.month(), est_now.day())
}

fn cli() -> Command {
    Command::new("nasa-wallpaper")
        .version(VERSION)
        .author("David Población Criado")
        .about("Change desktop wallpaper with NASA photographs")
        .arg_required_else_help(true)
        .subcommand(
            Command::new("apod")
                .about("Get the APOD (Astronomical Picture of the Day)")
                .arg(Arg::new("date").short('d').long("date").value_name("DATE"))
                .arg(Arg::new("key").short('k').long("key").value_name("API_KEY"))
                .arg(Arg::new("low").short('l').long("low").action(clap::ArgAction::SetTrue)),
        )
        .subcommand(Command::new("nasa_image").about("Get a random NASA image"))
        .subcommand(Command::new("unsplash").about("Get a random NASA image from Unsplash"))
        .subcommand(Command::new("license").long_flag("license"))
}

fn main() {
    let matches = cli().get_matches();

    match matches.subcommand() {
        Some(("apod", sub_matches)) => {
            let (year, month, day) = get_today_est();
            let today = format!("{}-{}-{}", year, month, day);
            let date = sub_matches.get_one::<String>("date").map(|s| s.as_str()).unwrap_or(&today);
            let api_key = sub_matches.get_one::<String>("key").map(|s| s.as_str()).unwrap_or(API_KEY);
            let hd = sub_matches.get_flag("low");

            if let Ok(apod) = get_apod(date, api_key) {
                println!("{}", apod);
                if apod.media_type != "image" {
                    println!("{}", format!("No image available. See: {}", apod.url).yellow());
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
        Some(("license", _)) => {
            print_license();
        }
        _ => {}
    }
}
