use crate::models::Apod;
use crate::utils::WallpaperResult;
use chrono::NaiveDate;

/// APOD Basic JSON endpoint
pub const APOD_API_URL: &str = "https://science.nasa.gov/wp-json/wp/v2/apod-basic";
/// Width (in pixels) requested from the image CDN when `--low` is used.
pub const LOW_RES_WIDTH: u32 = 1024;

/// Parses a user-supplied APOD date in `YYYY-MM-DD` format.
///
/// Month and day may be written without zero padding (e.g. `2026-9-3`).
pub fn parse_apod_date(date: &str) -> Result<NaiveDate, chrono::ParseError> {
    NaiveDate::parse_from_str(date.trim(), "%Y-%m-%d")
}

/// Returns the legacy APOD date code (`YYMMDD`) used by the APOD API routes.
pub fn apod_date_code(date: NaiveDate) -> String {
    date.format("%y%m%d").to_string()
}

/// Converts an HTML fragment from the APOD API into plain terminal text.
///
/// Removes a leading `Explanation:` label, turns `<br>` into line breaks,
/// strips every other tag and decodes common HTML entities.
pub fn html_to_text(html: &str) -> String {
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
pub fn decode_html_entities(text: &str) -> String {
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
pub fn low_res_url(url: &str) -> String {
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
pub fn get_apod(base_url: &str, date: NaiveDate) -> WallpaperResult<Apod> {
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
