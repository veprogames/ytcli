use std::collections::VecDeque;

pub enum CommandState {
    Success,
    Error(String),
    Exit,
}

struct CommandStructure<'a> {
    command: &'a str,
    params: VecDeque<&'a str>,
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

pub fn handle_command(cmd: &str) -> Result<CommandState, String>{
    let command: CommandStructure = parse_command(cmd)?;
    match (command.command, command.params.len()) {
        ("exit", 0) => Ok(CommandState::Exit),
        ("exit", _) => Ok(CommandState::Error("Usage: exit".to_string())),
        (unknown_command, _) => Ok(CommandState::Error(format!("Unknown Command: {unknown_command}"))),
    }
}