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

pub struct VideoData {
    link: String,
    title: String,
    author: String,
}

impl VideoData {
    pub fn get_url(&self) -> String {
        format!("{INSTANCE}{}", self.link)
    }
}

impl std::fmt::Display for VideoData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{} by {} [{}]", self.title, self.author, self.link)
    }
}

/// ## Why Invidious?
/// * Easier to parse as Invidious is mostly independent of JS and
/// is less cluttered
/// * Better Privacy through Invidious
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
                ureq::Error::Status(code, _) => return Err(YoutubeError::RequestError(code)),
                ureq::Error::Transport(_) => return Err(YoutubeError::TransportError)
            }
        },
        Ok(response) => response
    };
    match response.into_string() {
        Ok(string) => Ok(string),
        Err(err) => Err(YoutubeError::IOError(err)),
    }
}

pub fn get_videos(html_body: String) -> Result<Vec<VideoData>, YoutubeError> {
    let mut videos: Vec<VideoData> = vec![];
    let fragment = Html::parse_fragment(&html_body);
    
    let selector_box = match Selector::parse("div.h-box") {
        Ok(selector) => selector,
        Err(err) => return Err(YoutubeError::ParseError(err.to_string()))
    };
    let selector_link = match Selector::parse("a:first-child") {
        Ok(selector) => selector,
        Err(err) => return Err(YoutubeError::ParseError(err.to_string()))
    };
    let selector_title = match Selector::parse(r#"a:first-child > p[dir="auto"]"#) {
        Ok(selector) => selector,
        Err(err) => return Err(YoutubeError::ParseError(err.to_string()))
    };
    let selector_author = match Selector::parse("p.channel-name") {
        Ok(selector) => selector,
        Err(err) => return Err(YoutubeError::ParseError(err.to_string()))
    };

    for el in fragment.select(&selector_box) {
        let title = match el.select(&selector_title).next(){
            Some(element) => element.inner_html(),
            None => continue
        };
        let author = match el.select(&selector_author).next(){
            Some(element) => element.text()
                .next().unwrap_or("Unknown Author").to_string(),
            None => "Unknown Author".to_string()
        };
        let link = match el.select(&selector_link).next(){
            Some(element) => match element.value().attr("href") {
                Some(href) => href.to_string(),
                None => "/".to_string()
            },
            None => "/".to_string()
        };
        videos.push(VideoData { link, title, author });
    }
    Ok(videos)
}

pub fn print_videos(videos: &Vec<VideoData>) -> String {
    let mut result = String::new();
    for (index, video) in videos.iter().enumerate() {
        result = format!("[{index}] {video}") + &result;
    }
    result
}