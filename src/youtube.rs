use std::io;
use scraper::{Html, Selector, ElementRef};
use ureq::Response;
use colored::{Colorize, ColoredString};

use crate::format;

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
    views: u64,
    length: String,
}

impl VideoData {
    pub fn get_url(&self) -> String {
        format!("{INSTANCE}{}", self.link)
    }
}

impl std::fmt::Display for VideoData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} by {} | {} Views [{}] [{}]", self.title, self.author, 
            format::format(self.views), self.length, self.link)
    }
}

pub struct ChannelData {
    link: String,
    name: String,
    subscribers: u64,
}

impl std::fmt::Display for ChannelData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} [{} Subscribers] [{}]", self.name, format::format(self.subscribers), self.link)
    }
}

pub struct PlaylistData {
    link: String,
    name: String,
    length: u16,
}

impl std::fmt::Display for PlaylistData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} [{} Videos] [{}]", self.name, self.length, self.link)
    }
}

pub struct NavigationData {
    link: String,
}

impl std::fmt::Display for NavigationData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.link)
    }
}

pub enum Content {
    Video(VideoData),
    Channel(ChannelData),
    Playlist(PlaylistData),
    Navigation(NavigationData),
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

fn get_inner_text(html_element: ElementRef, sel: &str, fallback: &str) -> Result<String, YoutubeError> {
    match html_element.select(
        &selector(sel)?
    ).next() {
        Some(element) => Ok(element.text().next().unwrap_or(fallback).to_string()),
        None => Ok(fallback.to_string()),
    }
}

fn get_content_video(html_element: ElementRef, link: &str) -> Result<Content, YoutubeError>{
    let title = get_inner_text(html_element, r#"a:first-child > p[dir="auto"]"#, "Unknown Title")?;
    let author = get_inner_text(html_element, "p.channel-name", "Unknown Author")?;
    let length = get_inner_text(html_element, "p.length", "0:00")?;
    let views = get_inner_text(html_element, "div.flex-right > p.video-data", "0 views")?;
    let views = views.split(' ').next().unwrap_or("0");
    let views = format::parse(views)?;
    Ok(Content::Video(VideoData { link: link.to_string(), title, author, length, views }))
}

fn get_content_channel(html_element: ElementRef, link: &str) -> Result<Content, YoutubeError> {
    let channel_name = get_inner_text(html_element, "a:first-child > p", "Unknown Channel")?;
    let subscribers = get_inner_text(html_element, "a~p", "0 subscribers")?;
    let subscribers = subscribers.split(' ').next().unwrap_or("0");
    let subscribers = format::parse(subscribers)?;
    Ok(Content::Channel(ChannelData { link: link.to_string(), name: channel_name, subscribers }))
}

fn get_content_playlist(html_element: ElementRef, link: &str) -> Result<Content, YoutubeError> {
    let playlist_name = get_inner_text(html_element, "a:first-child > p", "Unknown Playlist")?;
    let playlist_length = get_inner_text(html_element, "p.length", "0 videos")?;
    let length = match playlist_length.split(' ').next().unwrap_or("0")
        .parse::<u16>() {
            Ok(count) => count,
            Err(err) => return Err(YoutubeError::ParseError(err.to_string()))
    };

    Ok(Content::Playlist(PlaylistData { link: link.to_string(), name: playlist_name, length }))
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
            get_content_playlist(element, link)?
        }
        else if link.starts_with("/channel") {
            get_content_channel(element, link)?
        }
        else if link.starts_with("/") {
            Content::Navigation(NavigationData { link: link.to_string() })
        }
        else {
            Content::Unknown
        };

        content.push(next_content);
    }

    Ok(content)
}

fn get_line_styled(line: String, content: &Content) -> ColoredString {
    match content {
        Content::Channel(..) => line.green(),
        Content::Playlist(..) => line.yellow(),
        Content::Navigation(..) => line.underline(),
        _ => line.normal()
    }
}

pub fn print_content(videos: &Vec<Content>) -> String {
    let mut result = String::new();
    for (index, content) in videos.iter().enumerate() {
        let content_string = match content {
            Content::Video(video) => video.to_string(),
            Content::Channel(channel) => channel.to_string(),
            Content::Playlist(playlist) => playlist.to_string(),
            Content::Navigation(navigation) => navigation.to_string(),
            Content::Unknown => String::from("Unknown Content"),
        };
        // do not display a line break at the end, which was produced by the first element
        let line_break = if index == 0 { "" } else { "\n" };
        let line = format!("[{index}] {content_string}{line_break}");
        let line = get_line_styled(line, &content).to_string();
        result = line + &result;
    }
    result
}