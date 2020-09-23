//! Configuration module
//!
//! @TODO: more precise error handling
pub mod opt_parse;

use std::collections::HashMap;
use std::fmt;
use std::fs::File;
use std::io;
use std::path::{Path, PathBuf};

use serde::Deserialize;
use toml::de;

/// contents of a configuration file.
#[derive(Deserialize, Debug)]
pub struct Config {
    /// changing defaults. similar to cmd-line args
    pub options: Option<Options>,
    /// variable declarations
    pub variables: Option<HashMap<String, String>>,
    /// dimension declarations
    pub dimensions: Option<HashMap<String, DimDecl>>,
    /// source -> destination map
    pub paths: Option<HashMap<PathBuf, PathBuf>>,
}
/// default values for command-line optional arguments.
#[derive(Deserialize, Debug)]
pub struct Options {
    /// overwrite destination files if they already exist?
    pub force: Option<bool>,
    /// verbosity level. see [`error::ErrorFlags`]
    ///
    /// [`error::ErrorFlags`]: ../error/struct.ErrorFlags.html
    pub verbosity: Option<u8>,
}
/// Dimension Declarations.
#[derive(Deserialize, Debug)]
#[serde(untagged)]
pub enum DimDecl {
    Size(u8),
    Choices(Vec<String>),
}

impl Config {
    pub fn from_str(s: &str) -> Result<Config, de::Error> {
        toml::from_str(s)
    }
}

#[derive(Debug)]
pub enum Error {
    IO(io::Error),
    TOML(de::Error),
}
impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::IO(e) => e.fmt(f),
            Error::TOML(e) => e.fmt(f),
        }
    }
}

/// open config file and parse it.
pub fn path_to_cfg(path: &Path) -> Result<Config, Error> {
    file_to_cfg(&mut File::open(path).map_err(Error::IO)?)
}
/// parse config file.
pub fn file_to_cfg(file: &mut File) -> Result<Config, Error> {
    use std::io::Read;
    let mut buf = String::new();
    file.read_to_string(&mut buf).map_err(Error::IO)?;
    string_to_cfg(&buf).map_err(Error::TOML)
}
/// parse config string.
pub fn string_to_cfg(s: &String) -> Result<Config, de::Error> {
    Config::from_str(s.as_ref())
}
