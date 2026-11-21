//! nasa-wallpaper — cambio lo sfondo del desktop con immagini NASA.
//!
//! Copyright 2019-2026 David Población Criado
//! Licenziato sotto Apache License, Version 2.0.
//!
//! Tutti gli errori di rete, parsing e configurazione sono gestiti: nessuna
//! funzione usa `unwrap`/`expect` su dati esterni e nessun warning è soppresso.

pub mod text_renderer;

use chrono::prelude::*;
use chrono_tz::Tz;
use chrono_tz::US::Eastern;
use clap::{Arg, ArgAction, Command};
use colored::*;
use rand::Rng;
use reqwest::Url;
use std::error::Error;
use std::ffi::{OsStr, OsString};
use std::fmt;
use std::process::ExitCode;
use std::time::Duration;

const LICENSE_TEXT: &str = r#"
    Copyright 2019-2026 David Población Criado

    Licensed under the Apache License, Version 2.0 (the 'License');
    you may not use this file except in compliance with the License.
    You may obtain a copy of the License at

        https://www.apache.org/licenses/LICENSE-2.0

    Unless required by applicable law or agreed to in writing, software
    distributed under the License is distributed on an "AS IS" BASIS,
    WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
    See the License for the specific language governing permissions and
    limitations under the License.
"#;

const VERSION: &str = "2.1.3";
const MSG_DONE: &str = "Done";
const MSG_CHANGING: &str = "Changing wallpaper...";
const URL_UNSPLASH: &str = "https://source.unsplash.com/user/nasa";
/// APOD Basic JSON endpoint.
const APOD_API_URL: &str = "https://science.nasa.gov/wp-json/wp/v2/apod-basic";
/// Larghezza richiesta al CDN quando si usa `--low`.
const LOW_RES_WIDTH: u32 = 1024;
/// Timeout per richieste HTTP: evita consumi illimitati (OWASP A06/A10).
const HTTP_TIMEOUT_SECONDS: u64 = 20;
/// Host autorizzati come sorgente di immagini.
const HOSTS_IMMAGINE: &[&str] = &[
    "assets.science.nasa.gov",
    "science.nasa.gov",
    "images-assets.nasa.gov",
    "source.unsplash.com",
];

type WallpaperResult<T> = Result<T, Box<dyn Error>>;

/// Metadati APOD restituiti dall'endpoint JSON.
#[derive(Debug, serde::Deserialize)]
struct Apod {
    date: String,
    title: String,
    media_type: String,
    /// Può contenere markup HTML.
    #[serde(default)]
    explanation: String,
    /// Può contenere markup HTML.
    #[serde(default)]
    copyright: Option<String>,
    /// URL della pagina APOD (non di un'immagine).
    url: String,
    /// URL dell'immagine a piena risoluzione, quando disponibile.
    #[serde(default)]
    hdurl: Option<String>,
}

impl Apod {
    /// Restituisce l'URL dell'immagine da usare come sfondo, se presente.
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
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
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

/// Elemento restituito dalla ricerca NASA Image and Video Library.
#[derive(Debug, serde::Deserialize)]
struct NasaImage {
    nasa_id: String,
    title: String,
    center: String,
    description: String,
    date: String,
    url: String,
}

impl fmt::Display for NasaImage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Title: {}\nDate: {}\nExplanation: {}\nCenter: {}\nNASA id: {}",
            html_to_text(&self.title).bold().italic(),
            self.date.italic(),
            html_to_text(&self.description),
            html_to_text(&self.center),
            self.nasa_id
        )
    }
}

/// Parametri tipizzati di una ricerca nella libreria immagini NASA.
#[derive(Debug, Default, Clone)]
struct NasaImageQuery<'a> {
    q: &'a str,
    center: &'a str,
    location: &'a str,
    nasa_id: &'a str,
    photographer: &'a str,
    title: &'a str,
    year_start: &'a str,
    year_end: &'a str,
}

/// Analizza una data APOD nel formato `YYYY-MM-DD`.
fn parse_apod_date(date: &str) -> Result<NaiveDate, chrono::ParseError> {
    NaiveDate::parse_from_str(date.trim(), "%Y-%m-%d")
}

/// Restituisce il codice data legacy (`YYMMDD`) usato dalle route APOD.
fn apod_date_code(date: NaiveDate) -> String {
    date.format("%y%m%d").to_string()
}

/// Converte un frammento HTML dell'API APOD in testo semplice per terminale.
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

/// Decodifica le entità HTML più comuni e quelle numeriche.
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

/// Restituisce una versione ridotta di un URL immagine APOD.
///
/// Solo gli URL serviti dal CDN di NASA (`/dynamicimage/`) possono essere
/// ridimensionati; gli altri vengono restituiti invariati.
fn low_res_url(url: &str) -> String {
    if !url.contains("/dynamicimage/") {
        return url.to_owned();
    }
    let base = url.split(['?', '#']).next().unwrap_or(url);
    format!("{base}?w={LOW_RES_WIDTH}&fit=clip")
}

/// Costruisce un client HTTP con timeout esplicito.
fn http_client() -> WallpaperResult<reqwest::blocking::Client> {
    Ok(reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(HTTP_TIMEOUT_SECONDS))
        .build()?)
}

/// Verifica che un URL sia HTTPS e appartenga a un host autorizzato.
///
/// Questa validazione impedisce che un campo controllato dall'API remota
/// venga usato per indicare al client di scaricare risorse da host arbitrari.
fn url_immagine_sicura(raw: &str) -> WallpaperResult<Url> {
    let url = Url::parse(raw)?;
    if url.scheme() != "https" {
        return Err("URL immagine non HTTPS".into());
    }
    let host = url.host_str().ok_or("URL immagine senza host")?;
    if !HOSTS_IMMAGINE.contains(&host) {
        return Err("Host immagine non autorizzato".into());
    }
    Ok(url)
}

/// Recupera i metadati APOD per una data.
///
/// # Errori
/// Restituisce errore se la richiesta fallisce, se lo stato non è 2xx
/// (includendo il messaggio API quando disponibile) o se il JSON non è valido.
fn get_apod(base_url: &str, date: NaiveDate) -> WallpaperResult<Apod> {
    let request_url = format!(
        "{}/{}",
        base_url.trim_end_matches('/'),
        apod_date_code(date)
    );
    let client = http_client()?;
    let response = client.get(&request_url).send()?;
    let status = response.status();
    if !status.is_success() {
        let body = response.text()?;
        let message = json::parse(&body)
            .ok()
            .and_then(|err| {
                err["message"]
                    .as_str()
                    .map(|m| format!("{} [{}]", m, err["code"]))
            })
            .map_or_else(
                || format!("APOD API error ({status})"),
                |detail| format!("APOD API error ({}): {}", status.as_u16(), detail),
            );
        return Err(message.into());
    }
    Ok(response.json::<Apod>()?)
}

/// Costruisce l'URL di ricerca codificando correttamente ogni parametro.
fn costruisci_url_ricerca(
    parametri: &NasaImageQuery<'_>,
    page: Option<u32>,
) -> WallpaperResult<Url> {
    let mut url = Url::parse("https://images-api.nasa.gov/search")?;
    {
        let mut pairs = url.query_pairs_mut();
        pairs.append_pair("media_type", "image");
        pairs.append_pair("q", parametri.q);
        pairs.append_pair("center", parametri.center);
        pairs.append_pair("location", parametri.location);
        pairs.append_pair("nasa_id", parametri.nasa_id);
        pairs.append_pair("photographer", parametri.photographer);
        pairs.append_pair("title", parametri.title);
        pairs.append_pair("year_start", parametri.year_start);
        pairs.append_pair("year_end", parametri.year_end);
        if let Some(page) = page {
            pairs.append_pair("page", &page.to_string());
        }
    }
    Ok(url)
}

/// Verifica che un anno sia nel formato `YYYY` e in intervallo plausibile.
fn valida_anno(anno: &str) -> WallpaperResult<u32> {
    let valore: u32 = anno.parse()?;
    if !(1800..=2200).contains(&valore) {
        return Err("Anno fuori intervallo".into());
    }
    Ok(valore)
}

/// Recupera un'immagine casuale dalla NASA Image Library.
///
/// Tutti i passaggi possono fallire e restituiscono un errore: non ci sono
/// `unwrap`/`expect` né `process::exit` al suo interno.
fn get_nasa_image(parametri: &NasaImageQuery<'_>) -> WallpaperResult<NasaImage> {
    valida_anno(parametri.year_start)?;
    valida_anno(parametri.year_end)?;

    let client = http_client()?;
    let url_base = costruisci_url_ricerca(parametri, None)?;

    let testo = client.get(url_base.clone()).send()?.text()?;
    let json_risposta = json::parse(&testo)?;
    let num_hits = json_risposta["collection"]["metadata"]["total_hits"]
        .as_usize()
        .ok_or("Campo total_hits mancante o non valido")?;

    if num_hits == 0 {
        return Err("Nessun risultato per i criteri richiesti".into());
    }

    // 100 risultati per pagina: page_count è almeno 1.
    let page_count = num_hits.div_ceil(100).min(101);
    let mut rng = rand::rng();
    let index_page = rng.random_range(0..page_count) as u32;
    let url_pagina = costruisci_url_ricerca(parametri, Some(index_page))?;

    let testo_pagina = client.get(url_pagina).send()?.text()?;
    let json_pagina = json::parse(&testo_pagina)?;
    let items = &json_pagina["collection"]["items"];
    let num_items = items.len();
    if num_items == 0 {
        return Err("Pagina di risultati vuota".into());
    }
    let index = rng.random_range(0..num_items);
    let item = &items[index];
    let data = &item["data"][0];
    let url_collezione = item["href"]
        .as_str()
        .ok_or("URL della collezione mancante")?;

    let url_immagine = url_immagine_sicura(url_collezione)?;
    let testo_collezione = client.get(url_immagine).send()?.text()?;
    let json_collezione = json::parse(&testo_collezione)?;
    let url_asset = json_collezione[0]
        .as_str()
        .ok_or("URL dell'asset mancante")?;
    url_immagine_sicura(url_asset)?;

    let campo = |nome: &str| -> WallpaperResult<String> {
        data[nome]
            .as_str()
            .map(|s| s.to_owned())
            .ok_or_else(|| format!("Campo {nome} mancante").into())
    };

    let mut date = campo("date_created")?;
    date.truncate(10);

    Ok(NasaImage {
        nasa_id: campo("nasa_id")?,
        title: campo("title")?,
        center: campo("center")?,
        description: campo("description")?,
        date,
        url: url_asset.to_owned(),
    })
}

/// Imposta lo sfondo con l'immagine APOD, dopo averne validato l'URL.
fn set_wallpaper(apod: &Apod, low: bool) -> WallpaperResult<()> {
    let url = apod
        .image_url(low)
        .ok_or("The APOD for this date has no image URL")?;
    url_immagine_sicura(&url)?;
    wallpaper::set_from_url(&url)?;
    Ok(())
}

/// Stampa il testo di licenza.
fn print_license() {
    println!("{}", LICENSE_TEXT);
}

/// Restituisce la data odierna nel fuso US Eastern (EST/EDT).
fn get_today_est() -> (i32, u32, u32) {
    let est_now: DateTime<Tz> = Utc::now().with_timezone(&Eastern);
    (est_now.year(), est_now.month(), est_now.day())
}

/// Costruisce la definizione della CLI.
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
                // Deprecata: la nuova API APOD non richiede chiave. Mantenuta
                // nascosta per compatibilità degli script esistenti.
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
                        .action(ArgAction::SetTrue)
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
                        .action(ArgAction::Set)
                        .help("Free text search terms to compare to all indexed metadata"),
                )
                .arg(
                    Arg::new("center")
                        .short('c')
                        .long("center")
                        .value_name("CENTER")
                        .action(ArgAction::Set)
                        .help("NASA center which published the media"),
                )
                .arg(
                    Arg::new("location")
                        .short('o')
                        .long("location")
                        .value_name("LOCATION")
                        .action(ArgAction::Set)
                        .help("Terms to search for in “Location” fields"),
                )
                .arg(
                    Arg::new("nasa_id")
                        .short('i')
                        .long("nasa_id")
                        .value_name("NASA_ID")
                        .action(ArgAction::Set)
                        .help("The media asset’s NASA ID"),
                )
                .arg(
                    Arg::new("photographer")
                        .short('p')
                        .long("photographer")
                        .visible_alias("phtographer")
                        .value_name("PHOTOGRAPHER")
                        .action(ArgAction::Set)
                        .help("The primary photographer’s name"),
                )
                .arg(
                    Arg::new("title")
                        .short('t')
                        .long("title")
                        .value_name("TITLE")
                        .action(ArgAction::Set)
                        .help("Terms to search for in “Title” fields"),
                )
                .arg(
                    Arg::new("year_start")
                        .long("year_start")
                        .value_name("YEAR_START")
                        .action(ArgAction::Set)
                        .help("The start year for results. Format: YYYY"),
                )
                .arg(
                    Arg::new("year_end")
                        .long("year_end")
                        .value_name("YEAR_END")
                        .action(ArgAction::Set)
                        .help("The end year for results. Format: YYYY"),
                ),
        )
        .subcommand(Command::new("unsplash").about(
            "Get a random image from the NASA's account in Unsplash (https://unsplash.com/@nasa)",
        ))
        .subcommand(Command::new("license").about("Print the license of this program"))
        .subcommand(Command::new("status").about("Run a local health check of the configuration"))
}

/// Normalizza gli argomenti per le shorthand legacy (`-a`, `-n`).
fn normalize_args(mut args: Vec<OsString>) -> Vec<OsString> {
    if args.len() >= 2 {
        if args[1] == OsStr::new("-a") {
            args[1] = OsString::from("apod");
        } else if args[1] == OsStr::new("-n") {
            args[1] = OsString::from("nasa_image");
        }
    }
    args
}

/// Esegue il controllo di integrità locale (rete, font e URL).
fn health_check() -> WallpaperResult<()> {
    let client = http_client()?;
    drop(client);
    let _font = text_renderer::find_system_font()?;
    url_immagine_sicura(APOD_API_URL)?;
    Ok(())
}

/// Converte un errore in messaggio utente senza dettagli interni.
fn mostra_errore(err: &dyn Error) {
    eprintln!("{}", format!("Error: {err}").red());
}

/// Punto di ingresso del programma.
fn main() -> ExitCode {
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
                            format!("Error: invalid date ({err}). Use YYYY-MM-DD").red()
                        );
                        return ExitCode::from(2);
                    }
                },
                None => {
                    let (year, month, day) = get_today_est();
                    match NaiveDate::from_ymd_opt(year, month, day) {
                        Some(date) => date,
                        None => {
                            eprintln!("{}", "Error: data Eastern non valida".red());
                            return ExitCode::FAILURE;
                        }
                    }
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
                    mostra_errore(err.as_ref());
                    return ExitCode::FAILURE;
                }
            };
            println!("{apod}");
            if apod.media_type != "image" {
                println!(
                    "{} {}",
                    "The date you have chosen for the APOD has no image. See the original content in:"
                        .yellow(),
                    apod.url.yellow()
                );
                return ExitCode::SUCCESS;
            }
            println!("{}", MSG_CHANGING.yellow());
            if let Err(err) = set_wallpaper(&apod, low) {
                mostra_errore(err.as_ref());
                return ExitCode::FAILURE;
            }
            println!("{}", MSG_DONE.green());
        }
        Some(("unsplash", _)) => {
            if let Err(err) = url_immagine_sicura(URL_UNSPLASH) {
                mostra_errore(err.as_ref());
                return ExitCode::FAILURE;
            }
            println!("{}", MSG_CHANGING.yellow());
            if let Err(err) = wallpaper::set_from_url(URL_UNSPLASH) {
                mostra_errore(&err);
                return ExitCode::FAILURE;
            }
            println!("{}", MSG_DONE.green());
        }
        Some(("nasa_image", sub_matches)) => {
            let valore = |nome: &str| {
                sub_matches
                    .get_one::<String>(nome)
                    .map(String::as_str)
                    .unwrap_or("")
            };
            let (est_year, _, _) = get_today_est();
            let anno_corrente = est_year.to_string();
            let parametri = NasaImageQuery {
                q: valore("query"),
                center: valore("center"),
                location: valore("location"),
                nasa_id: valore("nasa_id"),
                photographer: valore("photographer"),
                title: valore("title"),
                year_start: {
                    let v = valore("year_start");
                    if v.is_empty() {
                        "1900"
                    } else {
                        v
                    }
                },
                year_end: {
                    let v = valore("year_end");
                    if v.is_empty() {
                        &anno_corrente
                    } else {
                        v
                    }
                },
            };

            let nasa_image = match get_nasa_image(&parametri) {
                Ok(image) => image,
                Err(err) => {
                    mostra_errore(err.as_ref());
                    return ExitCode::FAILURE;
                }
            };
            println!("{nasa_image}");
            println!("{}", MSG_CHANGING.yellow());
            if let Err(err) = wallpaper::set_from_url(&nasa_image.url) {
                mostra_errore(&err);
                return ExitCode::FAILURE;
            }
            println!("{}", MSG_DONE.green());
        }
        Some(("license", _)) => print_license(),
        Some(("status", _)) => match health_check() {
            Ok(()) => println!("Health check: OK (client HTTP, font e URL confermati)"),
            Err(err) => {
                mostra_errore(err.as_ref());
                return ExitCode::FAILURE;
            }
        },
        _ => return ExitCode::from(2),
    }
    ExitCode::SUCCESS
}

#[cfg(test)]
mod tests;
