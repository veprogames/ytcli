use std::{collections::VecDeque, process::Command};

use crate::youtube::{self, VideoData};

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
    current_videos: Vec<VideoData>
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
            None => { return Err("No Command given".to_string()); }
        };
        let params = parts;
        Ok(CommandStructure { command, params })
    }
    
    pub fn handle_command(&mut self, cmd: &str) -> CommandState{
        let command: CommandStructure = match CommandParser::parse_command(cmd) {
            Ok(structure) => structure,
            Err(_) => {
                return CommandState::Error("Invalid Syntax".to_string());
            }
        };
        let params = command.params;
        match (command.command.as_str(), params.len()) {
            ("exit", 0) => CommandState::Exit,
            ("exit", _) => CommandState::Error("Usage: exit".to_string()),
            ("q" | "query", 1) => {
                let query = params[0].as_str();
                let body = match youtube::get_document(query) {
                    Ok(body) => body,
                    Err(yt_err) => {
                        return CommandState::Error(yt_err.to_string());
                    }
                };
                match youtube::get_videos(body) {
                    Ok(videos) => {
                        let formatted = youtube::print_videos(&videos);
                        self.current_videos = videos;
                        return CommandState::Ok(format!("{}", formatted));
                    },
                    Err(yt_err) => CommandState::Error(yt_err.to_string())
                }
            },
            ("q" | "query", _) => CommandState::Error("Usage: q [term]".to_string()),
            ("w" | "watch", 1) => {
                let id = match params[0].parse::<usize>() {
                    Ok(num) => num,
                    Err(err) => {
                        return CommandState::Error(format!("Not a Number! ({})", err.to_string()));
                    }
                };

                match Command::new("mpv")
                    .arg(self.current_videos[id].get_url())
                    .spawn() {
                        Ok(mut child) => {
                            if let Err(_) = child.wait() {
                                return CommandState::Error("mpv wasn't running".to_string());
                            }
                            return CommandState::Ok(String::new());
                        },
                        Err(err) => CommandState::Error(format!("Cannot start mpv. Is it installed? {}", err.to_string())),
                    }
            },
            ("w" | "watch", _) => CommandState::Error("Usage: watch [index]".to_string()),
            (unknown_command, _) => CommandState::Error(format!("Unknown Command: {unknown_command}")),
        }
    }
}