use std::io;
use scraper::{Html, Selector, ElementRef};
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
        write!(f, "{} by {} [{}]", self.title, self.author, self.link)
    }
}

pub enum Content {
    Video(VideoData),
    Channel,
    Playlist,
    Unknown,
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

fn get_document(query: &str) -> Result<String, YoutubeError> {
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

fn selector(selector: &str) -> Result<Selector, YoutubeError> {
    match Selector::parse(selector) {
        Ok(selector) => Ok(selector),
        Err(err) => return Err(YoutubeError::ParseError(err.to_string()))
    }
}

fn get_content_video(html_element: ElementRef, link: &str) -> Result<Content, YoutubeError>{
    let title = match html_element.select(
        &selector(r#"a:first-child > p[dir="auto"]"#)?
    ).next() {
        Some(element) => element.text().next().unwrap_or("Unknown Title"),
        None => "Unknown Title"
    };
    
    let author = match html_element.select(
        &selector("p.channel-name")?
    ).next() {
        Some(element) => element.text().next().unwrap_or("Unknown Author"),
        None => "Unknown Author"
    };

    Ok(Content::Video(VideoData { link: link.to_string(), title: title.to_string(), author: author.to_string() }))
}

pub fn get_content(query: &str) -> Result<Vec<Content>, YoutubeError> {
    let body = get_document(query)?;
    let fragment = Html::parse_fragment(&body);
    let mut content: Vec<Content> = vec![];

    let selector_content = selector("div.h-box")?;
    let selector_link = selector("a:first-child")?;

    for element in fragment.select(&selector_content) {
        let link = match element.select(&selector_link).next() {
            Some(a) => match a.value().attr("href") {
                Some(href) => href,
                None => continue,
            },
            None => continue,
        };

        let next_content = if link.starts_with("/watch") {
            get_content_video(element, link)?
        }
        else if link.starts_with("/playlist") {
            Content::Playlist
        }
        else if link.starts_with("/channel") {
            Content::Channel
        }
        else {
            Content::Unknown
        };

        content.push(next_content);
    }

    Ok(content)
}

pub fn print_content(videos: &Vec<Content>) -> String {
    let mut result = String::new();
    for (index, content) in videos.iter().enumerate() {
        let content_string = match content {
            Content::Video(video) => video.to_string(),
            Content::Channel => String::from("Channel"),
            Content::Playlist => String::from("Playlist"),
            Content::Unknown => String::from("Unknown Content")
        };
        result = format!("[{index}] {content_string}\n") + &result;
    }
    result
}