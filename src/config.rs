use yaml_rust::{Yaml, YamlLoader};
use yaml_rust::scanner::ScanError;

use std::error::Error;
use std::fs::File;
use std::path::Path;
use std::fmt;
use std::io;
use std::io::Read;

pub struct Config {
    data: Vec<Yaml>,
}

type ConfigResult<T> = Result<T, ConfigError>;

#[derive(Debug)]
pub enum ConfigError {
    IoError(io::Error),
    ParseError(ScanError),
}

impl From<io::Error> for ConfigError {
    fn from(err: io::Error) -> ConfigError {
        ConfigError::IoError(err)
    }
}

impl From<ScanError> for ConfigError {
    fn from(err: ScanError) -> ConfigError {
        ConfigError::ParseError(err)
    }
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            ConfigError::IoError(ref e) => e.fmt(f),
            ConfigError::ParseError(ref e) => e.fmt(f),
        }
    }
}

impl Config {
    pub fn new(path: &Path) -> ConfigResult<Config> {
        let mut file = try!(File::open(&path));
        let mut s = String::new();
        try!(file.read_to_string(&mut s));
        let data = try!(YamlLoader::load_from_str(&s));
        Ok(Config { data: data })
    }
}
