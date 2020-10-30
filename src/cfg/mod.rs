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

pub use opt_parse::StructOpt;
pub use opt_parse::{Opt, DEFAULT_VERBOSITY};

/// contents of a configuration file.
#[derive(Deserialize, Debug)]
pub struct Config {
    /// changing defaults. similar to cmd-line args
    pub options: Option<Options>,
    /// variable declarations
    pub variables: Option<HashMap<String, String>>,
    /// dimension declarations
    pub dimensions: Option<HashMap<String, Choices>>,
    /// source -> destination map
    pub paths: Option<HashMap<PathBuf, PathBuf>>,
}
impl Config {
    pub fn dimensions(&self) -> impl Iterator<Item = (&String, &Choices)> + '_ {
        self.dimensions.iter().flatten()
    }
    pub fn variables(&self) -> impl Iterator<Item = (&String, &String)> + '_ {
        self.variables.iter().flatten()
    }
    pub fn paths(&self) -> impl Iterator<Item = (&PathBuf, &PathBuf)> + '_ {
        self.paths.iter().flatten()
    }
    pub fn variables_cloned(&self) -> impl Iterator<Item = (String, String)> + '_ {
        self.variables.clone().into_iter().flatten()
    }
    pub fn dimensions_cloned(&self) -> impl Iterator<Item = (String, Choices)> + '_ {
        self.dimensions.clone().into_iter().flatten()
    }
}
impl Default for Config {
    fn default() -> Self {
        Config {
            options: None,
            variables: None,
            dimensions: None,
            paths: None,
        }
    }
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
pub const DEFAULT_FORCE: bool = false;

/// Dimension Declarations.
/// @FIXME check whether all Names are unique!!
#[derive(Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum Choices {
    Size(u8),
    Names(Vec<String>),
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
pub fn path_to_cfg<P: AsRef<Path>>(path: P) -> Result<Config, Error> {
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
