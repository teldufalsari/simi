use std::io::{stdout, Write};
use colored::Colorize;

pub mod application;

/// don't forget to make this fn private
pub fn prompt(str: &str) {
    print!("\r{}: {}\n{}: ", "<simi>".yellow(), str, "[you]".cyan());
    stdout().flush().unwrap();
}

pub fn empty_prompt() {
    print!("{}: ", "[you]".cyan());
    stdout().flush().unwrap();
}

pub fn named_prompt(name: &str, contents: &str) {
    print!("\r[{}]: {}\n{}: ", name.green(), contents, "[you]".cyan());
    stdout().flush().unwrap();
}

pub fn debug_prompt(str: &str) {
    print!("\r{}: {}\n{}:", "<simi>".magenta(), str.magenta(), "[you]".cyan());
    stdout().flush().unwrap();
}
