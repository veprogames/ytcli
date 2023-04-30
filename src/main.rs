mod command;
use std::io;

use crate::command::handle_command;

fn main() -> Result<(), String> {
    loop {
        print!("(ytcli) ");
        if let Err(err) = io::Write::flush(&mut io::stdout()){
            return Err(err.to_string());
        }
        let mut input = String::new();
        if let Err(err) = io::stdin().read_line(&mut input) {
            return Err(err.to_string());
        }
        match handle_command(input.trim_end()) {
            Ok(state) => {
                match state {
                    command::CommandState::Success => {},
                    command::CommandState::Exit => { return Ok(()) },
                    command::CommandState::Error(reason) => println!("{reason}"),
                }
            },
            Err(err) => println!("Error: {err}"),
        }
    }
}
