#![warn(clippy::all, clippy::pedantic)]
#![allow(dead_code)]

use std::io::{BufRead, Write};

mod cli;
mod error;

use cli::{Command, interpret};

// This is a simple driver for CLI testing
fn main() {
    let stdin = std::io::stdin();
    let mut str_buf = String::new();
    print!("<simi>: Print yor commans for evaluation:\n[you]: ");
    std::io::stdout().flush().unwrap();
    loop {
        stdin.lock().read_line(&mut str_buf).unwrap();
        match interpret(str_buf.trim()) {
            Err(e) => {
                print!("<simi>: {}\n[you]: ", e.descr);
                std::io::stdout().flush().unwrap();
            }
            Ok(Command::Exit) => break,
            Ok(cmd) => {
                print!("<simi>: {:?}\n[you]: ", cmd);
                std::io::stdout().flush().unwrap();
            }
        }
        str_buf.clear();
    }
    println!("<simi>: exiting...");
}
