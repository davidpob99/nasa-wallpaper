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
use clap::{App, Arg};
use colored::*;
use rand::Rng;
use std::fmt;

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

const API_KEY: &str = "Z1an39TefCytrrClcLSxNGJDGwv09QLHD6zo74R8";

const MSG_DONE: &str = "Done";
const MSG_CHANGING: &str = "Changing wallpaper...";

const URL_UNSPLASH: &str = "https://source.unsplash.com/user/nasa";

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
    match reqwest::get(&request_url)?.json() {
        Ok(json) => return Ok(json),
        Err(err) => {
            let text_error: &str = &reqwest::get(&request_url)?.text()?;
            let json_error = json::from(text_error);
            println!("{}", json_error);
            return Err(err);
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
        &reqwest::get(&request_url)
            .expect("Something went wrong")
            .text()
            .unwrap(),
    )
    .unwrap();
    let num_hits: &usize = &response["collection"]["metadata"]["total_hits"]
        .as_usize()
        .expect("Could not convert to usize");
    if *num_hits == 0 {
        panic!("No items were found :(")
    }
    let pages = {
        if (*num_hits / 100) - 1 <= 100 {
            (*num_hits / 100) - 1
        } else {
            100
        }
    };
    // Get a random page
    let index_page = rand::thread_rng().gen_range(0, pages);
    request_url.push_str(&format!("&page={page}", page = index_page));
    response = json::parse(
        &reqwest::get(&request_url)
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
            rand::thread_rng().gen_range(0, 100)
        }
    };
    let items = &response["collection"]["items"];
    let item = &items[index];
    let data = &item["data"][0];
    let url_collection = item["href"].as_str().unwrap();
    let response_collection =
        json::parse(&reqwest::get(url_collection).unwrap().text().unwrap()).unwrap();
    let mut date = data["date_created"].as_str().unwrap().to_owned();
    date.truncate(10);
    NasaImage {
        nasa_id: data["nasa_id"].as_str().unwrap().to_owned(),
        title: data["title"].as_str().unwrap().to_owned(),
        center: data["center"].as_str().unwrap().to_owned(),
        description: data["description"].as_str().unwrap().to_owned(),
        date: date,
        url: response_collection[0].as_str().unwrap().to_owned(),
    }
}

/// Sets an APOD element as desktop wallpaper
/// # Parameters
/// * apod
/// * hd: if true high definition is used
/// # Return
/// None
fn set_wallpaper(apod: &Apod, hd: bool) {
    println!("{}", MSG_CHANGING.yellow());
    if hd {
        wallpaper::set_from_url(&apod.hdurl).unwrap();
    } else {
        wallpaper::set_from_url(&apod.url).unwrap();
    }
    println!("{}", "Done".green());
}

/// Prints the program license in terminal
fn print_license() {
    println!("{}", LICENSE_TEXT);
}

fn main() {
    let matches = App::new("nasa-wallpaper")
        .usage("nasa-wallpaper <secondary options> [main options]
MAIN OPTIONS:\n\t-a (APOD), -n (NASA Image Library), -u (Unsplash)
SECONDARY OPTIONS:
\t* date, key and low flags are only available with APOD flag(-a)
\t* q, center, location, nasa_id, photographer, title, year_start and year_end are only available with NASA Image Library flag (-n)
\t* Unsplash has not got any secondary option")
        .version("2.0.0")
        .author("David Población (https://davidpob99.github.io)")
        .about("nasa-wallpaper is shell program which allows users to change the desktop wallpaper with NASA photographs. If there are more than one photo with the given options, it will choose a random one.")        .arg(
            Arg::with_name("apod")
                .short("a")
                .long("apod")
                .help("Get the APOD (Astronomical Picture of the Day)"),
        ).arg(
            Arg::with_name("date")
                .short("d")
                .long("date")
                .value_name("DATE")
                .takes_value(true)
                .help("Download the APOD from other date than today"),
        ).arg(
            Arg::with_name("key")
                .short("k")
                .long("key")
                .value_name("API_KEY")
                .takes_value(true)
                .help("Change the default API key. You can get one in https://api.nasa.gov/index.html#apply-for-an-api-key"),
        ).arg(
            Arg::with_name("low")
                .short("l")
                .long("low")
                .help("Use the low definition image. It is faster than the HD photo"),
        ).arg(
            Arg::with_name("nasa_image")
                .short("n")
                .long("nasa_image")
                .help("Get a random image from the NASA Image library (http://images.nasa.gov) with the parameters provided"),
        ).arg(
            Arg::with_name("query")
                .short("q")
                .long("query")
                .value_name("Q")
                .takes_value(true)
                .help("Free text search terms to compare to all indexed metadata"),
        ).arg(
            Arg::with_name("center")
                .short("c")
                .long("center")
                .value_name("CENTER")
                .takes_value(true)
                .help("NASA center which published the media"),
        ).arg(
            Arg::with_name("location")
                .short("o")
                .long("location")
                .value_name("LOCATION")
                .takes_value(true)
                .help("Terms to search for in “Location” fields"),
        ).arg(
            Arg::with_name("nasa_id")
                .short("i")
                .long("nasa_id")
                .value_name("NASA_ID")
                .takes_value(true)
                .help("The media asset’s NASA ID"),
        ).arg(
            Arg::with_name("photographer")
                .short("p")
                .long("phtographer")
                .value_name("PHOTOGRAPHER")
                .takes_value(true)
                .help("The primary photographer’s name"),
        ).arg(
            Arg::with_name("title")
                .short("t")
                .long("title")
                .value_name("TITLE")
                .takes_value(true)
                .help("Terms to search for in “Title” fields"),
        ).arg(
            Arg::with_name("year_start")
                .long("year_start")
                .value_name("YEAR_START")
                .takes_value(true)
                .help("The start year for results. Format: YYYY"),
        ).arg(
            Arg::with_name("year_end")
                .long("year_end")
                .value_name("YEAR_END")
                .takes_value(true)
                .help("The end year for results. Format: YYYY"),
        ).arg(
            Arg::with_name("unsplash")
                .short("u")
                .long("unsplash")
                .help("Get a random image from the NASA's account in Unsplash (https://unsplash.com/@nasa)"),
        ).arg(
            Arg::with_name("license")
                .long("license")
        ).get_matches();

    if matches.is_present("apod") {
        let date: String = if matches.is_present("date") {
            matches.value_of("date").unwrap().to_string()
        } else {
            Local::now().year().to_string()
                + "-"
                + &Local::now().month().to_string()
                + "-"
                + &Local::now().day().to_string()
        };

        let api_key = if matches.is_present("key") {
            matches.value_of("key").unwrap()
        } else {
            API_KEY
        };
        let hd: bool = if matches.is_present("low") {
            false
        } else {
            true
        };
        // println!("{}", hd);
        match get_apod(&date, &api_key) {
            Ok(apod) => {
                println!("{}", apod);
                if apod.media_type != "image" {
                    print!("{}", "The date you have chosen for the APOD has no image. If you want, you can see the content in: ".yellow());
                    println!("{}", apod.url.yellow());
                    return;
                }
                set_wallpaper(&apod, hd);
            }
            Err(_) => return,
        }
        return;
    } else if matches.is_present("unsplash") {
        println!("{}", MSG_CHANGING.yellow());
        wallpaper::set_from_url(URL_UNSPLASH).unwrap();
        println!("{}", MSG_DONE.green());
        return;
    } else if matches.is_present("nasa_image") {
        println!("{}", MSG_CHANGING.yellow());
        let q = if matches.is_present("query") {
            matches.value_of("query").unwrap()
        } else {
            ""
        };
        let center = if matches.is_present("center") {
            matches.value_of("center").unwrap()
        } else {
            ""
        };
        let location = if matches.is_present("location") {
            matches.value_of("location").unwrap()
        } else {
            ""
        };
        let nasa_id = if matches.is_present("nasa_id") {
            matches.value_of("nasa_id").unwrap()
        } else {
            ""
        };
        let photographer = if matches.is_present("photographer") {
            matches.value_of("photographer").unwrap()
        } else {
            ""
        };
        let title = if matches.is_present("title") {
            matches.value_of("title").unwrap()
        } else {
            ""
        };
        let year_start = if matches.is_present("year_start") {
            matches.value_of("year_start").unwrap()
        } else {
            "1900"
        };
        let tmp_year = Local::now().year().to_string();
        let year_end = if matches.is_present("year_end") {
            matches.value_of("year_end").unwrap()
        } else {
            &tmp_year
        };

        let nasa_image = get_nasa_image(
            q,
            center,
            location,
            nasa_id,
            photographer,
            title,
            year_start,
            &year_end,
        );
        wallpaper::set_from_url(&nasa_image.url).unwrap();
        println!("{}", nasa_image);
        println!("{}", MSG_DONE.green());
        return;
    } else if matches.is_present("license") {
        print_license()
    } else {
        println!("Execute the help with: nasa-wallpaper --help");
    }
    return;
}
