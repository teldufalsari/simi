use std::str::FromStr;
use std::{collections::BTreeMap, path::PathBuf};
use std::fs;

use serde::{Serialize, Deserialize};
use toml;
use home::{self, home_dir};

const PATH_TO_CONFIG: &str = "~/.simi/conf.toml";

#[derive(Debug, Serialize, Deserialize)]
/// Representation of current running configuration.
///
/// This struct should be mutable to allow users
/// modify their contact list in runtime.
pub struct Config {
    /// String representation of a port number.
    /// 
    /// It should be converted to a numerical value
    /// before starting the main loop.
    pub port: u16,
    
    /// Path the directory with .png images
    /// 
    /// If `--secret` command is invoked without `--path` argument,
    /// Images are picked from here.
    /// 
    /// It should be converted to `Path` using
    /// `canonicalize_home` function
    /// before startig the main loop.
    pub assets: String,

    /// If true, imamges will be deleted from the directory
    /// after use.
    /// Images specified by `--path` are never deleted
    pub delete_images: bool,

    /// If true, images will be picked randomly from the directory
    /// If false, the first image in alphabetical order is picked
    /// False is recommended only with `delete_images=true`
    pub pick_randomly: bool,
    pub contacts: BTreeMap<String, String>,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            port: 1337,
            assets: "~/.simi/assets".to_owned(),
            delete_images: false,
            pick_randomly: true,
            contacts: BTreeMap::new(),
        }
    }
}

impl Config {
    /// Loads file specified by `PATH_TO_CONFIG` constant and deserializes it
    /// into a new `Config` struct
    /// 
    /// Returns `Err` if either I/O or parsing failed
    pub fn load() -> Result<Config, String> {
        let path = match canonicalize_home(PATH_TO_CONFIG) {
            Some(val) => val,
            None => return Err("cannot locate home directory. Why?...".to_owned())
        };
        let raw_config = match fs::read_to_string(path) {
            Ok(val) => val,
            Err(e) => return Err(e.to_string()),
        };
        toml::from_str(&raw_config).map_err(|e| e.to_string())
    }

    /// Converts `self` into TOML format and saves the contents to
    /// the file, specified by `PATH_TO_CONFIG` constant.
    /// 
    /// Returns `Err` if either I/O or parsing failed
    pub fn save(&self) -> Result<(), String> {
        let raw_config = match toml::to_string(&self) {
            Ok(val) => val,
            Err(e) => return Err(e.to_string())
        };
        let path = match canonicalize_home(PATH_TO_CONFIG) {
            Some(val) => val,
            None => return Err("Cannot locate home directory. Why?...".to_owned())
        };
        fs::write(path, &raw_config).map_err(|e| e.to_string())
    }
}

/// This function converts `String` that may start with `"~/"` symbol
/// (UNIX shell designation for current user home directory) into
/// a valid `PathBuf`
/// 
/// Very unlikely, but it can return `None` if `"~/"` cannot be resolved
/// in the current environment.
pub fn canonicalize_home(str: &str) -> Option<PathBuf> {
    if str.starts_with("~/") {
        let mut path = home_dir()?;
        path.push(str[2..].to_owned());
        Some(path)
    } else { 
        match PathBuf::from_str(str) {
            Ok(val) => Some(val),
            Err(_) => None
        }
    }
}
