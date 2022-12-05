#![warn(clippy::all, clippy::pedantic)]
#![allow(dead_code)]

mod cli;
mod error;
mod config;
mod core;
mod proto;

use crate::config::Config;
use crate::core::loops::idle_loop;

// This is a simple driver for CLI testing
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
    if let Err(e) = idle_loop(config) {
        println!("<simi> Fatal error: {}", e.descr);
    }
    println!("<simi>: exiting...");
}
