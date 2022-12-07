#![warn(clippy::all, clippy::pedantic)]
#![allow(dead_code)]

use std::process;

mod cli;
mod error;
mod config;
mod core;
mod proto;

use crate::config::Config;
use crate::core::application::Application;

fn main() {
    let config = match Config::load() {
        Ok(val) => val,
        Err(e) => {
            println!(
                "<simi>: can't read conf.toml\n
                <simi>: because: {}\n
                <simi>: falling back to defaults",e);
            Config::default()
        }
    };
    println!("<simi>: using following configuration: {:?}", config);
    let mut app = match Application::initialize(config) {
        Ok(val) => val,
        Err(e) => {
            println!("<simi> Fatal error: {}", e.descr);
            process::exit(1);
        }
    };
    if let Err(e) = app.run() {
        println!("<simi> Fatal error: {}", e.descr);
        process::exit(1);
    }
    println!("<simi>: exiting...");
}
