use anyhow::{anyhow, Context, Result};
use rand::Rng;
use serde_json::Value;
use crate::models::{Apod, NasaImage};

pub fn get_apod(date: &str, api_key: &str) -> Result<Apod> {
    let request_url = format!(
        "https://api.nasa.gov/planetary/apod?api_key={api_key}&date={date}",
        api_key = api_key,
        date = date
    );

    let response = reqwest::blocking::get(&request_url)
        .context("Failed to send request to NASA APOD API")?
        .json::<Apod>()
        .context("Failed to parse APOD response")?;
    
    Ok(response)
}

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
) -> Result<NasaImage> {
    let client = reqwest::blocking::Client::new();
    let base_url = "https://images-api.nasa.gov/search";

    let query_params = vec![
        ("media_type", "image"),
        ("q", q),
        ("center", center),
        ("location", location),
        ("nasa_id", nasa_id),
        ("photographer", photographer),
        ("title", title),
        ("year_start", year_start),
        ("year_end", year_end),
    ];

    // Initial request to get total hits
    let response = client
        .get(base_url)
        .query(&query_params)
        .send()
        .context("Failed to fetch NASA image metadata")?;

    let response_json: Value = response.json().context("Failed to parse search JSON response")?;

    let num_hits = response_json["collection"]["metadata"]["total_hits"]
        .as_u64()
        .unwrap_or(0);

    if num_hits == 0 {
        return Err(anyhow!("Couldn't find the file you're looking for. Try another tag."));
    }

    // NASA API returns up to 100 items per page
    let max_page = (num_hits / 100).min(100);
    let mut rng = rand::rng();
    let index_page = rng.random_range(1..=max_page + 1); // pages are 1-indexed

    let final_response_json = if index_page > 1 {
        let page_str = index_page.to_string();
        let mut paged_params = query_params.clone();
        paged_params.push(("page", &page_str));
        
        client
            .get(base_url)
            .query(&paged_params)
            .send()
            .context("Failed to fetch NASA image page")?
            .json::<Value>()
            .context("Failed to parse page JSON response")?
    } else {
        response_json
    };

    let items = final_response_json["collection"]["items"]
        .as_array()
        .ok_or_else(|| anyhow!("Malformed response: collection items not found"))?;

    if items.is_empty() {
        return Err(anyhow!("No items found in the collection page"));
    }

    let index = rng.random_range(0..items.len());
    let item = &items[index];
    let data = &item["data"][0];
    let url_collection = item["href"]
        .as_str()
        .ok_or_else(|| anyhow!("Malformed response: item href not found"))?;

    let response_collection: Value = client
        .get(url_collection)
        .send()
        .context("Failed to fetch image collection")?
        .json()
        .context("Failed to parse image collection JSON")?;

    let mut date = data["date_created"]
        .as_str()
        .ok_or_else(|| anyhow!("Malformed response: date_created not found"))?
        .to_owned();
    if date.len() > 10 {
        date.truncate(10);
    }

    let image_url = response_collection
        .as_array()
        .and_then(|arr| arr.get(0))
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow!("Failed to find image URL in collection"))?
        .to_owned();

    Ok(NasaImage {
        nasa_id: data["nasa_id"].as_str().unwrap_or("").to_owned(),
        title: data["title"].as_str().unwrap_or("").to_owned(),
        center: data["center"].as_str().unwrap_or("").to_owned(),
        description: data["description"].as_str().unwrap_or("").to_owned(),
        date,
        url: image_url,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_apod() {
        let json = r#"{
            "copyright": "NASA",
            "date": "2023-01-01",
            "explanation": "Test explanation",
            "hdurl": "https://example.com/hd.jpg",
            "media_type": "image",
            "title": "Test Title",
            "url": "https://example.com/sd.jpg"
        }"#;
        let apod: Apod = serde_json::from_str(json).unwrap();
        assert_eq!(apod.title, "Test Title");
        assert_eq!(apod.date, "2023-01-01");
    }
}
