mod command;
mod youtube;
mod format;
use std::io;

use crate::command::CommandParser;
use colored::Colorize;

fn main() -> Result<(), String> {
    let mut parser = CommandParser::new();

    loop {
        print!("(ytcli) ");
        if let Err(err) = io::Write::flush(&mut io::stdout()){
            return Err(err.to_string());
        }
        let mut input = String::new();
        if let Err(err) = io::stdin().read_line(&mut input) {
            return Err(err.to_string());
        }
        match parser.handle_command(input.trim_end()) {
            command::CommandState::Ok(out) => println!("{out}"),
            command::CommandState::Exit => { return Ok(()) },
            command::CommandState::Error(reason) => println!("{}", reason.red()),
        }
    }
}
