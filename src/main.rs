mod api;
mod models;
mod utils;
mod text_renderer;

use anyhow::Result;
use clap::{Arg, Command};
use colored::*;
use crate::models::Apod;
use crate::utils::{get_today_est, print_license};

const API_KEY: &str = "DEMO_KEY";
const VERSION: &str = "2.1.2";
const MSG_DONE: &str = "Done";
const MSG_CHANGING: &str = "Changing wallpaper...";
const URL_UNSPLASH: &str = "https://source.unsplash.com/user/nasa";

fn set_wallpaper(apod: &Apod, low_res: bool, overlay: Option<(&str, &str)>) -> Result<()> {
    let url = if low_res { &apod.url } else { &apod.hdurl };
    crate::utils::set_from_url(url, overlay)
}

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
                        .action(clap::ArgAction::SetTrue)
                        .help("Use low resolution image"),
                )
                .arg(
                    Arg::new("explanation")
                        .short('e')
                        .long("explanation")
                        .action(clap::ArgAction::SetTrue)
                        .help("Add the explanation to the image"),
                ),
        )
        .subcommand(
            Command::new("nasa_image")
                .about("Get a random image from the NASA Image Library (https://images.nasa.gov)")
                .arg(
                    Arg::new("explanation")
                        .short('e')
                        .long("explanation")
                        .action(clap::ArgAction::SetTrue)
                        .help("Add the explanation to the image"),
                )
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
                        .long("photographer")
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

fn main() -> Result<()> {
    let matches = cli().get_matches();

    match matches.subcommand() {
        Some(("apod", sub_matches)) => {
            let (year, month, day) = get_today_est();
            let today = format!("{}-{:02}-{:02}", year, month, day);
            let date = sub_matches
                .get_one::<String>("date")
                .map(|s| s.as_str())
                .unwrap_or(&today);
            
            // Check for API key in this order: CLI arg > env var > DEMO_KEY
            let env_api_key = std::env::var("NASA_API_KEY").ok();
            let api_key = sub_matches
                .get_one::<String>("key")
                .map(|s| s.as_str())
                .or_else(|| env_api_key.as_deref())
                .unwrap_or(API_KEY);
            
            
            let low_res = sub_matches.get_flag("low");

            let apod = api::get_apod(date, api_key)?;
            println!("{}", apod);
            
            if apod.media_type != "image" {
                println!("{}", format!("The date you have chosen for the APOD has no image. See the original content in: {}", apod.url).yellow());
                return Ok(());
            }

            println!("{}", MSG_CHANGING.yellow());
            let overlay = if sub_matches.get_flag("explanation") {
                Some((apod.title.as_str(), apod.explanation.as_str()))
            } else {
                None
            };
            set_wallpaper(&apod, low_res, overlay)?;
            println!("{}", MSG_DONE.green());
        }
        Some(("unsplash", _)) => {
            println!("{}", MSG_CHANGING.yellow());
            // Unsplash command doesn't support explanation overlay yet as API struct is different or not fully used here
            crate::utils::set_from_url(URL_UNSPLASH, None)?;
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
            
            let (est_year, _, _) = get_today_est();
            let est_year_str = est_year.to_string();
            let year_end = sub_matches
                .get_one::<String>("year_end")
                .map(|s| s.as_str())
                .unwrap_or(&est_year_str);

            let nasa_image = api::get_nasa_image(
                q, center, location, nasa_id, photographer, title, year_start, year_end
            )?;
            
            println!("{}", nasa_image);
            println!("{}", MSG_CHANGING.yellow());
            
            let overlay = if sub_matches.get_flag("explanation") {
                Some((nasa_image.title.as_str(), nasa_image.description.as_str()))
            } else {
                None
            };
            
            crate::utils::set_from_url(&nasa_image.url, overlay)?;
            println!("{}", MSG_DONE.green());
        }
        Some(("license", _)) => {
            print_license();
        }
        _ => {}
    }

    Ok(())
}
