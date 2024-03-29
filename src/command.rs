use std::{collections::VecDeque, process::{Command, Child}};

use crate::youtube;

pub enum CommandState {
    Ok(String),
    Error(String),
    Exit,
}

struct CommandStructure {
    command: String,
    params: VecDeque<String>,
}

pub struct CommandParser {
    current_content: Vec<youtube::Content>
}

impl CommandParser {
    pub fn new() -> CommandParser {
        CommandParser { current_content: vec![] }
    }

    fn get_command_parts(cmd: &str) -> VecDeque<String> {
        let mut parts: VecDeque<String> = VecDeque::new();
        let mut current_part = String::new();
        let mut in_quotation = false;
        for char in cmd.chars(){
            match (char, in_quotation) {
                ('"', _) => {
                    in_quotation = !in_quotation;
                },
                (' ', false) => {
                    parts.push_back(current_part.clone());
                    current_part = String::new();
                },
                (_, _) => current_part += &char.to_string(),
            }
        }
        // add to the Vec what's left in the String
        parts.push_back(current_part.clone());
        parts
    }

    fn parse_command(cmd: &str) -> Result<CommandStructure, String> {
        let mut parts: VecDeque<String> = Self::get_command_parts(cmd);
        let command = match parts.pop_front() {
            Some(value) => value,
            None => return Err("No Command given".to_string())
        };
        let params = parts;
        Ok(CommandStructure { command, params })
    }

    fn handle_query(&mut self, query: &str) -> CommandState {
        match youtube::get_content(query) {
            Ok(content) => {
                let formatted = youtube::print_content(&content);
                self.current_content = content;
                return CommandState::Ok(format!("{}", formatted));
            },
            Err(yt_err) => CommandState::Error(yt_err.to_string())
        }
    }

    fn get_query_url(&self, param: &str) -> Result<String, CommandState> {
        match param.trim().parse::<usize>() {
            Ok(index) => {
                match self.current_content.get(index) {
                    Some(content) => match content {
                        youtube::Content::Video(..) => Err(
                            CommandState::Error(String::from("Cannot query a video directly. Use w(atch) instead"))
                        ),
                        _ => Ok(content.get_link())
                    },
                    None => Err(CommandState::Error(String::from("Index out of bounds!")))
                }
            },
            Err(..) => Ok(format!("/search?q={}", param))
        }
    }

    fn get_content_for_index(&self, index: &str) -> Result<&youtube::Content, CommandState> {
        let index = match index.parse::<usize>() {
            Ok(num) => num,
            Err(err) => return Err(CommandState::Error(format!("Not a Number! ({})", err.to_string())))
        };

        match self.current_content.get(index) {
            Some(content) => Ok(content),
            None => Err(CommandState::Error("Index out of bounds!".to_string()))
        }
    }

    fn handle_child_process(child: &mut Child) -> CommandState{
        if let Err(_) = child.wait() {
            return CommandState::Error("mpv wasn't running".to_string());
        }
        CommandState::Ok(String::new())
    }

    fn handle_watch(&self, index: &str) -> CommandState{
        let content = match self.get_content_for_index(index) {
            Ok(content) => content,
            Err(state) => return state,
        };

        let url = match content {
            youtube::Content::Video(..) => content.get_url(),
            youtube::Content::Playlist(..) | youtube::Content::Channel(..) => return CommandState::Error("Cannot watch Channel or Playlist directly".to_string()),
            youtube::Content::Navigation(..) => return CommandState::Error("Cannot watch navigational Content".to_string()),
            youtube::Content::Unknown => return CommandState::Error("Cannot watch unknown Content".to_string()),
        };

        if let Ok(mut child) = Command::new("mpv").arg(&url).spawn() {
            CommandParser::handle_child_process(&mut child)
        }
        else if let Ok(mut child) = Command::new("vlc").arg(&url).spawn() {
            CommandParser::handle_child_process(&mut child)
        }
        else {
            CommandState::Error(format!("Cannot start mpv or vlc. Is one of them installed?"))
        }
    }

    fn handle_download(&self, index: &str) -> CommandState{
        let content = match self.get_content_for_index(index) {
            Ok(content) => content,
            Err(state) => return state,
        };

        let url = match content {
            youtube::Content::Navigation(..) => return CommandState::Error("Cannot download navigational Content".to_string()),
            youtube::Content::Unknown => return CommandState::Error("Cannot download unknown Content".to_string()),
            _ => content.get_url()
        };

        match Command::new("yt-dlp")
            .arg(url)
            .args(["-o", "~/.ytcli/download/%(uploader)s/%(title)s.%(ext)s"])
            .spawn() {
                Ok(mut child) => {
                    if let Err(_) = child.wait() {
                        return CommandState::Error("yt-dlp wasn't running".to_string());
                    }
                    CommandState::Ok(String::new())
                },
                Err(err) => CommandState::Error(format!("Cannot start yt-dlp. Is it installed? {}", err.to_string())),
            }
    }

    fn get_help() -> String {
        "ytcli help:
Query Content: q(uery) [term | index]
Watch Content: w(atch) [index]
Download Content: d(ownload) [index]
Exit: exit".to_string()
    }
    
    pub fn handle_command(&mut self, cmd: &str) -> CommandState{
        let command: CommandStructure = match CommandParser::parse_command(cmd) {
            Ok(structure) => structure,
            Err(_) => return CommandState::Error("Invalid Syntax".to_string())
        };
        let params = command.params;
        match (command.command.as_str(), params.len()) {
            ("exit", 0) => CommandState::Exit,
            ("exit", _) => CommandState::Error("Usage: exit".to_string()),
            ("q" | "query", 1) => {
                let url = match self.get_query_url(&params[0]) {
                    Ok(url) => url,
                    Err(state) => return state
                };
                self.handle_query(&url)
            },
            ("q" | "query", _) => CommandState::Error("Usage: q(uery) [term]".to_string()),
            ("w" | "watch", 1) => self.handle_watch(&params[0]),
            ("w" | "watch", _) => CommandState::Error("Usage: w(atch) [index]".to_string()),
            ("d" | "download", 1) => self.handle_download(&params[0]),
            ("d" | "download", _) => CommandState::Error("Usage: d(ownload) [index]".to_string()),
            ("h" | "help", _) => CommandState::Ok(CommandParser::get_help()),
            (unknown_command, _) => CommandState::Error(format!("Unknown Command: {unknown_command}\n{}", CommandParser::get_help())),
        }
    }
}