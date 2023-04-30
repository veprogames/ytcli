use std::{collections::VecDeque, process::Command};

use crate::youtube::{self, VideoData};

pub enum CommandState {
    Ok(String),
    Error(String),
    Exit,
}

struct CommandStructure<'a> {
    command: &'a str,
    params: VecDeque<&'a str>,
}

pub struct CommandParser {
    current_videos: Vec<VideoData>
}

impl CommandParser {
    pub fn new() -> CommandParser {
        CommandParser { current_videos: vec![] }
    }

    fn parse_command(cmd: &str) -> Result<CommandStructure, String> {
        let mut parts: VecDeque<&str> = cmd.split(' ').collect();
        let command = match parts.pop_front() {
            Some(value) => value,
            None => { return Err("No Command given".to_string()); }
        };
        let params = parts;
        Ok(CommandStructure { command, params })
    }
    
    pub fn handle_command(&mut self, cmd: &str) -> Result<CommandState, String>{
        let command: CommandStructure = CommandParser::parse_command(cmd)?;
        let params = command.params;
        match (command.command, params.len()) {
            ("exit", 0) => Ok(CommandState::Exit),
            ("exit", _) => Ok(CommandState::Error("Usage: exit".to_string())),
            ("q" | "query", 1) => {
                let query = params[0];
                let body = match youtube::get_document(query) {
                    Ok(body) => body,
                    Err(yt_err) => {
                        return Ok(CommandState::Error(yt_err.to_string()));
                    }
                };
                match youtube::get_videos(body) {
                    Ok(videos) => {
                        let formatted = youtube::print_videos(&videos);
                        self.current_videos = videos;
                        return Ok(CommandState::Ok(format!("{}", formatted)));
                    },
                    Err(yt_err) => Ok(CommandState::Error(yt_err.to_string()))
                }
            },
            ("q" | "query", _) => Ok(CommandState::Error("Usage: q [term]".to_string())),
            ("w" | "watch", 1) => {
                let id = match params[0].parse::<usize>() {
                    Ok(num) => num,
                    Err(err) => {
                        return Ok(CommandState::Error(format!("Not a Number! ({})", err.to_string())));
                    }
                };

                match Command::new("mpv")
                    .arg(self.current_videos[id].get_url())
                    .spawn() {
                        Ok(mut child) => {
                            if let Err(_) = child.wait() {
                                return Ok(CommandState::Error("mpv wasn't running".to_string()));
                            }
                            return Ok(CommandState::Ok(String::new()));
                        },
                        Err(err) => Ok(CommandState::Error(format!("Cannot start mpv. Is it installed? {}", err.to_string()))),
                    }
            },
            ("w" | "watch", _) => Ok(CommandState::Error("Usage: watch [index]".to_string())),
            (unknown_command, _) => Ok(CommandState::Error(format!("Unknown Command: {unknown_command}"))),
        }
    }
}