#![warn(clippy::all, clippy::pedantic)]
#![allow(dead_code)]

use std::process;
use colored::Colorize;

mod cli;
mod error;
mod config;
mod core;
mod proto;

use crate::config::Config;
use crate::core::application::Application;
use crate::core::prompt;

fn main() {
    let config = match Config::load() {
        Ok(val) => val,
        Err(e) => {
            prompt("can't read conf.toml");
            prompt(&format!("because: {}", e.red()));
            prompt(&"falling back to defaults".yellow());
            Config::default()
        }
    };
    prompt("initializing RSA keys...");
    let mut app = match Application::initialize(config) {
        Ok(val) => val,
        Err(e) => {
            prompt(&format!("fatal error: {}", e.descr.red()));
            process::exit(1);
        }
    };
    if let Err(e) = app.run() {
        prompt(&format!("fatal error: {}", e.descr.red()));
        process::exit(1);
    }
    println!("{}: exiting...", "<simi>".yellow());
}
