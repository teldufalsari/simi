use std::io::{stdout, Write};
use colored::Colorize;
use std::cell::RefCell;

pub mod application;

thread_local! { static DEBUG_PRINT_ENABLED: RefCell<bool> = RefCell::new(false); }

/// don't forget to make this fn private
pub fn prompt(str: &str) {
    print!("\r{}: {}\n{}: ", "<simi>".yellow(), str, "[you]".cyan());
    stdout().flush().unwrap();
}

pub fn empty_prompt() {
    print!("\r{}: ", "[you]".cyan());
    stdout().flush().unwrap();
}

pub fn named_prompt(name: &str, contents: &str) {
    print!("\r[{}]: {}\n{}: ", name.green(), contents, "[you]".cyan());
    stdout().flush().unwrap();
}

pub fn debug_prompt(str: &str) {
    DEBUG_PRINT_ENABLED.with(|enabled| {
        if *enabled.borrow() {
            print!("\r{}: {}\n{}:", "<simi>".magenta(), str.magenta(), "[you]".cyan());
            stdout().flush().unwrap();
        }
    });
}

pub fn toggle_debug() {
    DEBUG_PRINT_ENABLED.with(|enabled| {
        let new_state = !*enabled.borrow();
        *enabled.borrow_mut() = new_state;
        if new_state {
            prompt("debug info enabled");
        } else {
            prompt("debug info disabled");
        }
    })
}

pub fn secret_prompt(name: &str, contents: &str) {
    print!("\r[{}]: {}\n{}: ", name.red(), contents, "[you]".cyan());
    stdout().flush().unwrap();
}
