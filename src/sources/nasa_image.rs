use crate::models::NasaImage;
use rand::Rng;
use std::process;

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
pub fn get_nasa_image(
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
