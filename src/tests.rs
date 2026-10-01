use super::*;

/// Trimmed copy of the real API response for 2026-09-29.
const APOD_FIXTURE: &str = r#"{
    "date": "2026-09-29",
    "post_id": 1420794,
    "title": "Sh2-188: The Shrimp Nebula",
    "permalink": "https://science.nasa.gov/image-article/apod-2026-september-29-sh2-188-the-shrimp-nebula/",
    "media_type": "image",
    "explanation": "<strong>Explanation: </strong>What causes the swirl in the <a href=\"https://example.org\">Shrimp Nebula</a>? Stars &amp; gas.<br><strong>Tomorrow's picture: </strong>a harvest",
    "credit": "<a href=\"https://www.instagram.com/pawelpiechnik.astrophoto/\">Pawel Piechnik</a>",
    "copyright": "<a href=\"https://www.instagram.com/pawelpiechnik.astrophoto/\">Pawel Piechnik</a>",
    "url": "https://science.nasa.gov/image-article/apod-2026-september-29-sh2-188-the-shrimp-nebula/",
    "hdurl": "https://assets.science.nasa.gov/dynamicimage/assets/science/cds/apod/apod/2026/september/Shrimp_Pawel_2048.jpg?w=2048&h=2560&fit=clip&crop=faces%2Cfocalpoint",
    "basic_html": "<html></html>",
    "basic_html_url": "https://science.nasa.gov/wp-json/wp/v2/apod-basic/260929/html"
}"#;

const HDURL: &str = "https://assets.science.nasa.gov/dynamicimage/assets/science/cds/apod/apod/2026/september/Shrimp_Pawel_2048.jpg?w=2048&h=2560&fit=clip&crop=faces%2Cfocalpoint";

fn date(y: i32, m: u32, d: u32) -> NaiveDate {
    NaiveDate::from_ymd_opt(y, m, d).unwrap()
}

fn fixture() -> Apod {
    serde_json::from_str(APOD_FIXTURE).unwrap()
}

// --- parse_apod_date -------------------------------------------------

#[test]
fn parse_apod_date_accepts_padded_date() {
    assert_eq!(parse_apod_date("2026-09-30").unwrap(), date(2026, 9, 30));
}

#[test]
fn parse_apod_date_accepts_unpadded_date() {
    assert_eq!(parse_apod_date("2026-9-3").unwrap(), date(2026, 9, 3));
}

#[test]
fn parse_apod_date_trims_whitespace() {
    assert_eq!(parse_apod_date(" 1999-03-27 ").unwrap(), date(1999, 3, 27));
}

#[test]
fn parse_apod_date_rejects_wrong_format() {
    assert!(parse_apod_date("30/09/2026").is_err());
    assert!(parse_apod_date("260930").is_err());
    assert!(parse_apod_date("").is_err());
}

#[test]
fn parse_apod_date_rejects_impossible_date() {
    assert!(parse_apod_date("2026-02-30").is_err());
    assert!(parse_apod_date("2026-13-01").is_err());
}

// --- apod_date_code --------------------------------------------------

#[test]
fn apod_date_code_uses_yymmdd() {
    assert_eq!(apod_date_code(date(2026, 9, 30)), "260930");
    assert_eq!(apod_date_code(date(1995, 6, 16)), "950616");
    assert_eq!(apod_date_code(date(2000, 1, 1)), "000101");
}

// --- html_to_text ----------------------------------------------------

#[test]
fn html_to_text_removes_explanation_label() {
    assert_eq!(
        html_to_text("<strong>Explanation: </strong>Some text."),
        "Some text."
    );
}

#[test]
fn html_to_text_strips_tags_and_keeps_link_text() {
    assert_eq!(
        html_to_text(r#"See <a href="https://x.org/a?b=1&amp;c=2">this <i>nebula</i></a>."#),
        "See this nebula."
    );
}

#[test]
fn html_to_text_converts_br_to_newlines() {
    assert_eq!(
        html_to_text("One<br>Two<br/>Three<BR />Four"),
        "One\nTwo\nThree\nFour"
    );
}

#[test]
fn html_to_text_decodes_entities() {
    assert_eq!(
        html_to_text("A &amp; B &lt;C&gt; &quot;D&quot; &apos;E&#39; F&nbsp;G &#8217; &#x2014;"),
        "A & B <C> \"D\" 'E' F G \u{2019} \u{2014}"
    );
}

#[test]
fn html_to_text_keeps_unknown_or_bare_ampersands() {
    assert_eq!(
        html_to_text("Tom & Jerry &unknown; &#xZZ;"),
        "Tom & Jerry &unknown; &#xZZ;"
    );
}

#[test]
fn html_to_text_keeps_unclosed_tag_text() {
    assert_eq!(html_to_text("a < b"), "a < b");
}

#[test]
fn html_to_text_passes_plain_text_through() {
    assert_eq!(html_to_text("  Plain text  "), "Plain text");
    assert_eq!(html_to_text(""), "");
}

// --- low_res_url -----------------------------------------------------

#[test]
fn low_res_url_replaces_query_on_dynamic_images() {
    assert_eq!(
        low_res_url(HDURL),
        "https://assets.science.nasa.gov/dynamicimage/assets/science/cds/apod/apod/2026/september/Shrimp_Pawel_2048.jpg?w=1024&fit=clip"
    );
}

#[test]
fn low_res_url_adds_query_on_dynamic_images_without_one() {
    assert_eq!(
        low_res_url("https://assets.science.nasa.gov/dynamicimage/assets/a.jpg"),
        "https://assets.science.nasa.gov/dynamicimage/assets/a.jpg?w=1024&fit=clip"
    );
}

#[test]
fn low_res_url_leaves_other_urls_unchanged() {
    let url = "https://assets.science.nasa.gov/content/dam/science/cds/apod/apod/2017/july/AS11-40-5872HR.jpg?w=2349&h=2361";
    assert_eq!(low_res_url(url), url);
}

// --- Apod::image_url -------------------------------------------------

#[test]
fn image_url_returns_hdurl_by_default() {
    assert_eq!(fixture().image_url(false).as_deref(), Some(HDURL));
}

#[test]
fn image_url_returns_low_res_url_when_requested() {
    assert_eq!(fixture().image_url(true), Some(low_res_url(HDURL)));
}

#[test]
fn image_url_is_none_without_hdurl() {
    let mut apod = fixture();
    apod.hdurl = None;
    assert_eq!(apod.image_url(false), None);
    apod.hdurl = Some(String::new());
    assert_eq!(apod.image_url(true), None);
}

// --- Deserialization -------------------------------------------------

#[test]
fn deserializes_real_api_response() {
    let apod = fixture();
    assert_eq!(apod.date, "2026-09-29");
    assert_eq!(apod.title, "Sh2-188: The Shrimp Nebula");
    assert_eq!(apod.media_type, "image");
    assert!(apod
        .url
        .starts_with("https://science.nasa.gov/image-article/"));
    assert_eq!(apod.hdurl.as_deref(), Some(HDURL));
    assert!(apod.copyright.unwrap().contains("Pawel Piechnik"));
}

#[test]
fn deserializes_response_without_optional_fields() {
    let apod: Apod = serde_json::from_str(
        r#"{"date":"2026-09-13","title":"T","media_type":"video","url":"https://science.nasa.gov/x/"}"#,
    )
    .unwrap();
    assert_eq!(apod.explanation, "");
    assert_eq!(apod.copyright, None);
    assert_eq!(apod.hdurl, None);
}

#[test]
fn deserializes_response_with_null_optional_fields() {
    let apod: Apod = serde_json::from_str(
        r#"{"date":"2026-09-13","title":"T","media_type":"image","url":"u","copyright":null,"hdurl":null}"#,
    )
    .unwrap();
    assert_eq!(apod.copyright, None);
    assert_eq!(apod.hdurl, None);
}

// --- Display ---------------------------------------------------------

#[test]
fn display_prints_plain_text() {
    colored::control::set_override(false);
    let out = fixture().to_string();
    assert!(out.contains("Title: Sh2-188: The Shrimp Nebula"));
    assert!(out.contains("Date: 2026-09-29"));
    assert!(out.contains("Explanation: What causes the swirl in the Shrimp Nebula? Stars & gas."));
    assert!(out.contains("Copyright: Pawel Piechnik"));
    assert!(out.contains(
        "Link: https://science.nasa.gov/image-article/apod-2026-september-29-sh2-188-the-shrimp-nebula/"
    ));
    assert!(!out.contains('<'));
    assert!(!out.contains("&amp;"));
}

// --- get_apod (mocked HTTP) ----------------------------------------

#[test]
fn get_apod_requests_date_code_and_parses_response() {
    let mut server = mockito::Server::new();
    let mock = server
        .mock("GET", "/wp-json/wp/v2/apod-basic/260929")
        .with_status(200)
        .with_header("content-type", "application/json; charset=UTF-8")
        .with_body(APOD_FIXTURE)
        .create();

    let base = format!("{}/wp-json/wp/v2/apod-basic/", server.url());
    let apod = get_apod(&base, date(2026, 9, 29)).unwrap();

    mock.assert();
    assert_eq!(apod.title, "Sh2-188: The Shrimp Nebula");
    assert_eq!(apod.hdurl.as_deref(), Some(HDURL));
}

#[test]
fn get_apod_reports_api_error_message() {
    let mut server = mockito::Server::new();
    server
        .mock("GET", "/300101")
        .with_status(404)
        .with_header("content-type", "application/json; charset=UTF-8")
        .with_body(
            r#"{"code":"apod_basic_not_found","message":"APOD not found.","data":{"status":404}}"#,
        )
        .create();

    let err = get_apod(&server.url(), date(2030, 1, 1))
        .unwrap_err()
        .to_string();

    assert_eq!(
        err,
        "APOD API error (404): APOD not found. [apod_basic_not_found]"
    );
}

#[test]
fn get_apod_reports_status_when_error_body_is_not_json() {
    let mut server = mockito::Server::new();
    server
        .mock("GET", "/260929")
        .with_status(503)
        .with_body("upstream connect error or disconnect/reset before headers")
        .create();

    let err = get_apod(&server.url(), date(2026, 9, 29))
        .unwrap_err()
        .to_string();

    assert!(err.starts_with("APOD API error (503"), "{err}");
}

#[test]
fn get_apod_fails_on_invalid_json() {
    let mut server = mockito::Server::new();
    server
        .mock("GET", "/260929")
        .with_status(200)
        .with_body("<html>not json</html>")
        .create();

    assert!(get_apod(&server.url(), date(2026, 9, 29)).is_err());
}

#[test]
fn get_apod_fails_when_server_is_unreachable() {
    assert!(get_apod("http://127.0.0.1:1", date(2026, 9, 29)).is_err());
}

// --- CLI -------------------------------------------------------------

#[test]
fn cli_parses_apod_date_and_low() {
    let matches = cli()
        .try_get_matches_from(["nasa-wallpaper", "apod", "-d", "2026-09-29", "-l"])
        .unwrap();
    let (name, sub) = matches.subcommand().unwrap();
    assert_eq!(name, "apod");
    assert_eq!(sub.get_one::<String>("date").unwrap(), "2026-09-29");
    assert!(sub.get_flag("low"));
}

#[test]
fn cli_low_defaults_to_false() {
    let matches = cli()
        .try_get_matches_from(["nasa-wallpaper", "apod"])
        .unwrap();
    assert!(!matches.subcommand_matches("apod").unwrap().get_flag("low"));
}

#[test]
fn cli_still_accepts_legacy_api_key() {
    let args = normalize_args(
        ["nasa-wallpaper", "-a", "-k", "DEMO_KEY"]
            .iter()
            .map(OsString::from)
            .collect(),
    );
    let matches = cli().try_get_matches_from(args).unwrap();
    let sub = matches.subcommand_matches("apod").unwrap();
    assert_eq!(sub.get_one::<String>("key").unwrap(), "DEMO_KEY");
}

#[test]
fn cli_hides_api_key_from_help() {
    let help = cli()
        .find_subcommand_mut("apod")
        .unwrap()
        .render_help()
        .to_string();
    assert!(!help.contains("--key"));
    assert!(help.contains("--date"));
    assert!(help.contains("--low"));
}
