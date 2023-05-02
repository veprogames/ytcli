use std::{collections::VecDeque, process::Command};

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
    current_videos: Vec<youtube::Content>
}

impl CommandParser {
    pub fn new() -> CommandParser {
        CommandParser { current_videos: vec![] }
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
                self.current_videos = content;
                return CommandState::Ok(format!("{}", formatted));
            },
            Err(yt_err) => CommandState::Error(yt_err.to_string())
        }
    }

    fn handle_watch(&self, index_as_str: &str) -> CommandState{
        let index = match index_as_str.parse::<usize>() {
            Ok(num) => num,
            Err(err) => return CommandState::Error(format!("Not a Number! ({})", err.to_string()))
        };

        let content = match self.current_videos.get(index) {
            Some(content) => content,
            None => return CommandState::Error("Index out of bounds!".to_string())
        };

        let url = match content {
            youtube::Content::Video(video) => video.get_url(),
            youtube::Content::Playlist(..) | youtube::Content::Channel(..) => return CommandState::Error("Cannot watch Channel or Playlist directly".to_string()),
            youtube::Content::Navigation(..) => return CommandState::Error("Cannot watch navigational Content".to_string()),
            youtube::Content::Unknown => return CommandState::Error("Cannot watch unknown Content".to_string())
        };

        match Command::new("mpv")
            .arg(url)
            .spawn() {
                Ok(mut child) => {
                    if let Err(_) = child.wait() {
                        return CommandState::Error("mpv wasn't running".to_string());
                    }
                    CommandState::Ok(String::new())
                },
                Err(err) => CommandState::Error(format!("Cannot start mpv. Is it installed? {}", err.to_string())),
            }
    }

    fn get_help() -> String {
        "ytcli help:
Query Videos: q(uery) [term]
Watch Video: w(atch) [index]
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
            ("q" | "query", 1) => self.handle_query(&params[0]),
            ("q" | "query", _) => CommandState::Error("Usage: q [term]".to_string()),
            ("w" | "watch", 1) => self.handle_watch(&params[0]),
            ("w" | "watch", _) => CommandState::Error("Usage: watch [index]".to_string()),
            ("h" | "help", _) => CommandState::Ok(CommandParser::get_help()),
            (unknown_command, _) => CommandState::Error(format!("Unknown Command: {unknown_command}\n{}", CommandParser::get_help())),
        }
    }
}