/*
   Copyright 2019 David Población

   Licensed under the Apache License, Version 2.0 (the "License");
   you may not use this file except in compliance with the License.
   You may obtain a copy of the License at

       http://www.apache.org/licenses/LICENSE-2.0

   Unless required by applicable law or agreed to in writing, software
   distributed under the License is distributed on an "AS IS" BASIS,
   WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
   See the License for the specific language governing permissions and
   limitations under the License.
*/

extern crate chrono;
extern crate clap;
extern crate colored;
extern crate json;
extern crate rand;
extern crate reqwest;

#[macro_use]
extern crate serde_derive;

use chrono::prelude::*;
use clap::{Arg, Command};
use colored::*;
use rand::Rng;
use std::fmt;
use std::process;

const LICENSE_TEXT: &str = "
   Copyright 2019 David Población

   Licensed under the Apache License, Version 2.0 (the 'License');
   you may not use this file except in compliance with the License.
   You may obtain a copy of the License at

       http://www.apache.org/licenses/LICENSE-2.0

   Unless required by applicable law or agreed to in writing, software
   distributed under the License is distributed on an 'AS IS' BASIS,
   WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
   See the License for the specific language governing permissions and
   limitations under the License.";

const API_KEY: &str = "DEMO_KEY";
const VERSION: &str = "2.0.0+";
const MSG_DONE: &str = "Done";
const MSG_CHANGING: &str = "Changing wallpaper...";

const URL_UNSPLASH: &str = "https://source.unsplash.com/user/nasa";

use std::error::Error;
type WallpaperResult<T> = std::result::Result<T, Box<dyn Error>>;

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

/// Gets the APOD image from the NASA
/// # Parameters
/// * date
/// * api_key
/// # Return
/// Returns an APOD element if no errors ocurred.
/// Otherwise, it returns an error
fn get_apod(date: &str, api_key: &str) -> Result<Apod, reqwest::Error> {
    let request_url = format!(
        "https://api.nasa.gov/planetary/apod?api_key={api_key}&date={date}",
        api_key = api_key,
        date = date
    );
    match reqwest::blocking::get(&request_url)?.json() {
        Ok(json) => Ok(json),
        Err(err) => {
            let text_error = reqwest::blocking::get(&request_url)?.text()?;
            let json_error = json::from(text_error);
            println!("{}", json_error);
            Err(err)
        }
    }
}

/// Gets an image from the repository of the NASA
/// # Parameters
/// * q
/// * center
/// * location
/// * nasa_id
/// * photographer
/// * title
/// * year_start
/// * year_end
///
/// # Return
/// A NasaImage struct
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
    let mut response = json::parse(
        &reqwest::blocking::get(&request_url)
            .expect("Something went wrong")
            .text()
            .unwrap(),
    )
    .unwrap();
    let num_hits: &usize = &response["collection"]["metadata"]["total_hits"]
        .as_usize()
        .expect("Could not convert to usize");
    if *num_hits == 0 {
        println!("Couldn't find the file you're looking for. Try another tag.");
        process::exit(0x0100);
    }
    let pages = {
        if (*num_hits / 100) - 1 <= 100 {
            (*num_hits / 100) - 1
        } else {
            100
        }
    };
    // Get a random page
    let index_page = rand::thread_rng().gen_range(0..pages);
    request_url.push_str(&format!("&page={page}", page = index_page));
    response = json::parse(
        &reqwest::blocking::get(&request_url)
            .expect("Something went wrong")
            .text()
            .unwrap(),
    )
    .unwrap();
    // Get a random index from the page selected
    let index = {
        if *num_hits < 7 {
            *num_hits - 1
        } else {
            rand::thread_rng().gen_range(0..100)
        }
    };
    let items = &response["collection"]["items"];
    let item = &items[index];
    let data = &item["data"][0];
    let url_collection = item["href"].as_str().unwrap();
    let response_collection =
        json::parse(&reqwest::blocking::get(url_collection).unwrap().text().unwrap()).unwrap();
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

/// Sets an APOD element as desktop wallpaper
/// # Parameters
/// * apod
/// * hd: if true high definition is used
/// # Return
/// None
fn set_wallpaper(apod: &Apod, hd: bool) -> WallpaperResult<()> {
    if hd {
        wallpaper::set_from_url(&apod.hdurl)?;
    } else {
        wallpaper::set_from_url(&apod.url)?;
    }
    Ok(())
}

/// Prints the program license in terminal
fn print_license() {
    println!("{}", LICENSE_TEXT);
}

fn cli() -> Command {
    Command::new("nasa-wallpaper")
    .version(VERSION)
    .author("David Población Criado")
    .about("nasa-wallpaper is a shell program which allows users to change the desktop wallpaper with NASA photographs. If there are more than one photo with the given options, it will choose a random one.")
    .arg_required_else_help(true)
    .subcommand(
        Command::new("apod")
        .short_flag('a')
        .long_flag("apod")
        .about("Get the APOD (Astronomical Picture of the Day)")
        .arg(
            Arg::new("date")
                .short('d')
                .long("date")
                .value_name("DATE")
                .help("Download the APOD from other date than today"))
        .arg(
            Arg::new("key")
                .short('k')
                .long("key")
                .value_name("API_KEY")
                .help("Change the demo API key. You can get one in https://api.nasa.gov/"))
        .arg(
            Arg::new("low")
                .short('l')
                .long("low")
                .help("Use the low definition image. It is faster than the HD photo")
                .action(clap::ArgAction::SetTrue)
                
        ))
    .subcommand(
        Command::new("nasa_image")
        .short_flag('n')
        .long_flag("nasa_image")
        .about("Get a random image from the NASA Image library (http://images.nasa.gov) with the parameters provided")
        .arg(
            Arg::new("query")
                .short('q')
                .long("query")
                .value_name("Q")
                .action(clap::ArgAction::Set)
                .help("Free text search terms to compare to all indexed metadata"))
        .arg(
            Arg::new("center")
                .short('c')
                .long("center")
                .value_name("CENTER")
                .action(clap::ArgAction::Set)
                .help("NASA center which published the media"))
        .arg(
            Arg::new("location")
                .short('o')
                .long("location")
                .value_name("LOCATION")
                .action(clap::ArgAction::Set)
                .help("Terms to search for in “Location” fields"))
        .arg(
            Arg::new("nasa_id")
                .short('i')
                .long("nasa_id")
                .value_name("NASA_ID")
                .action(clap::ArgAction::Set)
                .help("The media asset’s NASA ID"))
        .arg(
            Arg::new("photographer")
                .short('p')
                .long("phtographer")
                .value_name("PHOTOGRAPHER")
                .action(clap::ArgAction::Set)
                .help("The primary photographer’s name"))
        .arg(
            Arg::new("title")
                .short('t')
                .long("title")
                .value_name("TITLE")
                .action(clap::ArgAction::Set)
                .help("Terms to search for in “Title” fields"))
        .arg(
            Arg::new("year_start")
                .long("year_start")
                .value_name("YEAR_START")
                .action(clap::ArgAction::Set)
                .help("The start year for results. Format: YYYY"))
        .arg(
            Arg::new("year_end")
                .long("year_end")
                .value_name("YEAR_END")
                .action(clap::ArgAction::Set)
                .help("The end year for results. Format: YYYY")))
    .subcommand(
        Command::new("unsplash")
        .short_flag('u')
        .long_flag("unsplash")
        .about("Get a random image from the NASA's account in Unsplash (https://unsplash.com/@nasa)"))
    .subcommand(Command::new("license").long_flag("license"))
}

fn main() {
    let matches = cli().get_matches();

    match matches.subcommand() {
        Some(("apod", sub_matches)) => {
            let today = Local::now().year().to_string()+ "-" + &Local::now().month().to_string() + "-" + &Local::now().day().to_string();
            let date = sub_matches.get_one::<String>("date").map(|s| s.as_str()).unwrap_or(&today);
            let api_key = sub_matches.get_one::<String>("key").map(|s| s.as_str()).unwrap_or(API_KEY);
            let hd = sub_matches.get_flag("low");

            if let Ok(apod) = get_apod(date, api_key) {
                println!("{}", apod);
                if apod.media_type != "image" {
                    print!("{}", "The date you have chosen for the APOD has no image. If you want, you can see the content in: ".yellow());
                    println!("{}", apod.url.yellow());
                    return;
                }
                println!("{}", MSG_CHANGING.yellow());
                if let Err(err) = set_wallpaper(&apod, hd) {
                    println!("{}", format!("Error: {}", err).red());
                }
                else {
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
            let q = sub_matches.get_one::<String>("query").map(|s| s.as_str()).unwrap_or("");
            let center = sub_matches.get_one::<String>("center").map(|s| s.as_str()).unwrap_or("");
            let location = sub_matches.get_one::<String>("location").map(|s| s.as_str()).unwrap_or("");
            let nasa_id = sub_matches.get_one::<String>("nasa_id").map(|s| s.as_str()).unwrap_or("");
            let photographer = sub_matches.get_one::<String>("photographer").map(|s| s.as_str()).unwrap_or("");
            let title = sub_matches.get_one::<String>("title").map(|s| s.as_str()).unwrap_or("");
            let year_start = sub_matches.get_one::<String>("year_start").map(|s| s.as_str()).unwrap_or("1900");
            let tmp_year = Local::now().year().to_string();
            let year_end = sub_matches.get_one::<String>("year_end").map(|s| s.as_str()).unwrap_or(&tmp_year);

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
            println!("{}", MSG_CHANGING.yellow());
            wallpaper::set_from_url(&nasa_image.url).unwrap();
            println!("{}", nasa_image);
            println!("{}", MSG_DONE.green());
        }
        Some(("license", _)) => {
            print_license();
        }
        _ => {}
    }
}
