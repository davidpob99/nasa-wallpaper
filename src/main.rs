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
use clap::{Arg, ArgAction, ArgMatches, Command};
use colored::Colorize;
use rand::Rng;
use reqwest::Url;
use std::error::Error;
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

/// Scarica un URL e interpreta il corpo come JSON.
fn json_da_url(client: &reqwest::blocking::Client, url: Url) -> WallpaperResult<json::JsonValue> {
    let testo = client.get(url).send()?.text()?;
    Ok(json::parse(&testo)?)
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

/// Conta i risultati della ricerca nella libreria immagini NASA.
fn conta_risultati(
    client: &reqwest::blocking::Client,
    parametri: &NasaImageQuery<'_>,
) -> WallpaperResult<usize> {
    let url = costruisci_url_ricerca(parametri, None)?;
    let json = json_da_url(client, url)?;
    json["collection"]["metadata"]["total_hits"]
        .as_usize()
        .ok_or("Campo total_hits mancante o non valido".into())
}

/// Restituisce gli elementi di una pagina di risultati scelta a caso.
fn pagina_casuale(
    client: &reqwest::blocking::Client,
    parametri: &NasaImageQuery<'_>,
    total_hits: usize,
) -> WallpaperResult<Vec<json::JsonValue>> {
    // 100 risultati per pagina: page_count è almeno 1.
    let page_count = total_hits.div_ceil(100).min(101);
    let mut rng = rand::rng();
    let index_page = rng.random_range(0..page_count) as u32;
    let url = costruisci_url_ricerca(parametri, Some(index_page))?;
    let json = json_da_url(client, url)?;
    let items = &json["collection"]["items"];
    if items.is_empty() {
        return Err("Pagina di risultati vuota".into());
    }
    Ok(items.members().cloned().collect())
}

/// Risolve e valida l'URL dell'asset a partire dalla collezione.
fn url_asset_collezione(
    client: &reqwest::blocking::Client,
    url_collezione: &str,
) -> WallpaperResult<String> {
    let url_immagine = url_immagine_sicura(url_collezione)?;
    let json = json_da_url(client, url_immagine)?;
    let asset = json[0].as_str().ok_or("URL dell'asset mancante")?;
    url_immagine_sicura(asset)?;
    Ok(asset.to_owned())
}

/// Legge un campo obbligatorio dall'elemento selezionato.
fn campo_obbligatorio(data: &json::JsonValue, nome: &str) -> WallpaperResult<String> {
    data[nome]
        .as_str()
        .map(str::to_owned)
        .ok_or_else(|| format!("Campo {nome} mancante").into())
}

/// Recupera un'immagine casuale dalla NASA Image Library.
///
/// Tutti i passaggi possono fallire e restituiscono un errore: non ci sono
/// `unwrap`/`expect` né `process::exit` al suo interno.
fn get_nasa_image(parametri: &NasaImageQuery<'_>) -> WallpaperResult<NasaImage> {
    valida_anno(parametri.year_start)?;
    valida_anno(parametri.year_end)?;

    let client = http_client()?;
    let total_hits = conta_risultati(&client, parametri)?;
    if total_hits == 0 {
        return Err("Nessun risultato per i criteri richiesti".into());
    }

    let items = pagina_casuale(&client, parametri, total_hits)?;
    let mut rng = rand::rng();
    let item = &items[rng.random_range(0..items.len())];
    let data = &item["data"][0];
    let url_collezione = item["href"]
        .as_str()
        .ok_or("URL della collezione mancante")?;
    let asset = url_asset_collezione(&client, url_collezione)?;

    let mut date = campo_obbligatorio(data, "date_created")?;
    date.truncate(10);

    Ok(NasaImage {
        nasa_id: campo_obbligatorio(data, "nasa_id")?,
        title: campo_obbligatorio(data, "title")?,
        center: campo_obbligatorio(data, "center")?,
        description: campo_obbligatorio(data, "description")?,
        date,
        url: asset,
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

/// Costruisce più argomenti con forma breve e lunga.
fn args_brevi(valori: &'static [(&'static str, char, &'static str, &'static str)]) -> Vec<Arg> {
    valori
        .iter()
        .map(|(nome, short, value_name, help)| {
            Arg::new(*nome)
                .short(*short)
                .long(*nome)
                .value_name(*value_name)
                .action(ArgAction::Set)
                .help(*help)
        })
        .collect()
}

/// Costruisce argomenti con sola forma lunga.
fn args_lunghe(valori: &'static [(&'static str, &'static str, &'static str)]) -> Vec<Arg> {
    valori
        .iter()
        .map(|(nome, value_name, help)| {
            Arg::new(*nome)
                .long(*nome)
                .value_name(*value_name)
                .action(ArgAction::Set)
                .help(*help)
        })
        .collect()
}

/// Argomento del fotografo, con alias storico.
fn arg_fotografo() -> Arg {
    Arg::new("photographer")
        .short('p')
        .long("photographer")
        .visible_alias("phtographer")
        .value_name("PHOTOGRAPHER")
        .action(ArgAction::Set)
        .help("The primary photographer’s name")
}

/// Argomenti di ricerca del sottocomando `nasa_image`.
fn arghi_ricerca() -> Vec<Arg> {
    args_brevi(&[
        (
            "query",
            'q',
            "Q",
            "Free text search terms to compare to all indexed metadata",
        ),
        (
            "center",
            'c',
            "CENTER",
            "NASA center which published the media",
        ),
        (
            "location",
            'o',
            "LOCATION",
            "Terms to search for in “Location” fields",
        ),
        ("nasa_id", 'i', "NASA_ID", "The media asset’s NASA ID"),
        (
            "title",
            't',
            "TITLE",
            "Terms to search for in “Title” fields",
        ),
    ])
}

/// Argomenti temporali del sottocomando `nasa_image`.
fn arghi_anni() -> Vec<Arg> {
    args_lunghe(&[
        (
            "year_start",
            "YEAR_START",
            "The start year for results. Format: YYYY",
        ),
        (
            "year_end",
            "YEAR_END",
            "The end year for results. Format: YYYY",
        ),
    ])
}

/// Costruisce il sottocomando `nasa_image`.
fn comando_nasa_image() -> Command {
    let mut arghi = arghi_ricerca();
    arghi.push(arg_fotografo());
    arghi.extend(arghi_anni());
    Command::new("nasa_image")
        .alias("-n")
        .about("Get a random image from the NASA Image Library (https://images.nasa.gov)")
        .args(arghi)
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
                .alias("-a")
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
        .subcommand(comando_nasa_image())
        .subcommand(Command::new("unsplash").about(
            "Get a random image from the NASA's account in Unsplash (https://unsplash.com/@nasa)",
        ))
        .subcommand(Command::new("license").about("Print the license of this program"))
        .subcommand(Command::new("status").about("Run a local health check of the configuration"))
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
///
/// clap legge gli argomenti dall'ambiente: il codice dell'applicazione non
/// accede ad argv e non usa argv[0] per decisioni di sicurezza.
/// Risolve la data APOD richiesta, con fallback a oggi nella zona Eastern.
fn apod_date(sub_matches: &ArgMatches) -> Result<NaiveDate, ExitCode> {
    if let Some(date) = sub_matches.get_one::<String>("date") {
        return match parse_apod_date(date) {
            Ok(date) => Ok(date),
            Err(err) => {
                eprintln!(
                    "{}",
                    format!("Error: invalid date ({err}). Use YYYY-MM-DD").red()
                );
                Err(ExitCode::from(2))
            }
        };
    }
    let (year, month, day) = get_today_est();
    match NaiveDate::from_ymd_opt(year, month, day) {
        Some(date) => Ok(date),
        None => {
            eprintln!("{}", "Error: data Eastern non valida".red());
            Err(ExitCode::FAILURE)
        }
    }
}

/// Esegue il sottocomando apod.
fn run_apod(sub_matches: &ArgMatches) -> ExitCode {
    let date = match apod_date(sub_matches) {
        Ok(date) => date,
        Err(code) => return code,
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
    ExitCode::SUCCESS
}

/// Esegue il sottocomando unsplash.
fn run_unsplash() -> ExitCode {
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
    ExitCode::SUCCESS
}

/// Esegue il sottocomando nasa_image.
fn run_nasa_image(sub_matches: &ArgMatches) -> ExitCode {
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
    ExitCode::SUCCESS
}

/// Verifica lo stato della configurazione e delle dipendenze.
fn run_status() -> ExitCode {
    match health_check() {
        Ok(()) => {
            println!("Health check: OK (client HTTP, font e URL confermati)");
            ExitCode::SUCCESS
        }
        Err(err) => {
            mostra_errore(err.as_ref());
            ExitCode::FAILURE
        }
    }
}

/// Punto di ingresso del programma.
///
/// clap legge gli argomenti dall'ambiente: il codice dell'applicazione non
/// accede ad argv e non usa argv[0] per decisioni di sicurezza.
fn main() -> ExitCode {
    let matches = cli().get_matches();
    match matches.subcommand() {
        Some(("apod", sub_matches)) => run_apod(sub_matches),
        Some(("unsplash", _)) => run_unsplash(),
        Some(("nasa_image", sub_matches)) => run_nasa_image(sub_matches),
        Some(("license", _)) => {
            print_license();
            ExitCode::SUCCESS
        }
        Some(("status", _)) => run_status(),
        _ => ExitCode::from(2),
    }
}

#[cfg(test)]
mod tests;
