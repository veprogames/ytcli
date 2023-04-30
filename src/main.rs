use std::io;

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
    }
}
