use std::io::{stdout, Write};

use crate::cli::Command;
use crate::error::Error;

pub mod loops;

/// don't forget to make this fn private
pub fn prompt(str: &str) {
    print!("\r<simi>: {}\n[you]: ", str);
    stdout().flush().unwrap();
}

pub fn empty_prompt() {
    print!("[you]: ");
    stdout().flush().unwrap();
}

fn execute(cmd: Command) -> Result<(), Error> {
    match cmd {
        Command::List => prompt("\"List\" is not implemented"),
        Command::Add(alias, socket) => 
            prompt(&format!("\"Add\" {} = {} is not implemented", alias, socket)),
        Command::DialIp(socket) => 
            prompt(&format!("\"DialIP\" {} is not implemented", socket)),
        Command::DialAlias(alias) => 
            prompt(&format!("\"DialAlias\" {} is not implemented", alias)),
        Command::Remove(alias) =>
            prompt(&format!("\"Remove\" {} is not implemented", alias)),
        Command::Secret(path) =>
            prompt(&format!("\"Secret\" path={:?} is not implemented", path)),
        Command::SpeakPlain(str) =>
            prompt(&format!("You wanted to say \"{}\"", str)),
        Command::Exit => return Ok(())
    }
    Ok(())
}
