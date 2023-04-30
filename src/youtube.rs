use std::io;
use scraper::{Html, Selector};
use ureq::Response;

pub enum YoutubeError {
    RequestError(u16),
    TransportError,
    IOError(io::Error),
    ParseError(String),
}

impl std::fmt::Display for YoutubeError{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            YoutubeError::RequestError(code) => 
                writeln!(f, "Request failed with code {}", code),
            YoutubeError::IOError(err) => writeln!(f, "I/O Error: {}", err.to_string()),
            YoutubeError::TransportError => writeln!(f, "Transport Error"),
            YoutubeError::ParseError(reason) => writeln!(f, "Parse Error: {reason}"),
        }
    }
}

/// # Why Invidious?
/// * Requests to individual instances -> bettwr privacy
/// * Better Privacy through Invidious
/// * Easier to parse as Invidious is mostly independent of JS and
/// is less cluttered
const INSTANCE: &str = "https://yewtu.be";

fn get_response(query: &str) -> Result<Response, ureq::Error> {
    let request = ureq::get(
        format!("{INSTANCE}/search?q={query}").as_str()
    );
    request.call()
}

pub fn get_document(query: &str) -> Result<String, YoutubeError> {
    let response = match get_response(query) {
        Err(req_error) => {
            match req_error {
                ureq::Error::Status(code, _) => {
                    return Err(YoutubeError::RequestError(code));
                },
                ureq::Error::Transport(_) => {
                    return Err(YoutubeError::TransportError);
                }
            }
        },
        Ok(response) => response
    };
    match response.into_string() {
        Ok(string) => Ok(string),
        Err(err) => Err(YoutubeError::IOError(err)),
    }
}

pub fn get_videos(html_body: String) -> Result<Vec<String>, YoutubeError> {
    let mut videos: Vec<String> = vec![];
    let fragment = Html::parse_fragment(&html_body);
    let selector = match Selector::parse("div.h-box > a:first-child") {
        Ok(selector) => selector,
        Err(err) => { return Err(YoutubeError::ParseError(err.to_string())); }
    };
    for el in fragment.select(&selector) {
        if let Some(link) = el.value().attr("href") {
            videos.push(link.to_string());
        }
    }
    Ok(videos)
}