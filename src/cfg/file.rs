//! configuration file
use std::collections::HashMap;
use std::path::PathBuf;

use serde::Deserialize;
use toml::de;

/// contents of a configuration file.
#[derive(Deserialize, Debug)]
pub struct File {
    /// changing defaults. similar to cmd-line args
    pub options: Option<Options>,
    /// variable declarations
    pub variables: Option<HashMap<String, String>>,
    /// dimension declarations
    pub dimensions: Option<HashMap<String, Choices>>,
    /// source -> destination map
    pub paths: Option<HashMap<PathBuf, PathBuf>>,
}
impl File {
    pub fn from_str(s: &str) -> Result<Self, de::Error> {
        toml::from_str(s)
    }

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

impl Default for File {
    fn default() -> Self {
        File {
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
    /// ignore unset variables
    pub ignore_unset: Option<bool>,
    /// prefix for all the relative source files
    pub in_prefix: Option<PathBuf>,
    /// prefix for all the relative destination files
    pub out_prefix: Option<PathBuf>,
}
impl Options {
    pub fn force(&self) -> Option<bool> {
        self.force
    }
    pub fn verbosity(&self) -> Option<u8> {
        self.verbosity
    }
    pub fn ignore_unset(&self) -> Option<bool> {
        self.ignore_unset
    }
    pub fn in_prefix(&self) -> Option<&PathBuf> {
        self.in_prefix.as_ref()
    }
    pub fn out_prefix(&self) -> Option<&PathBuf> {
        self.out_prefix.as_ref()
    }
}

/// dimension Declarations.  
/// @FIXME check whether all Names are unique!!
#[derive(Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum Choices {
    Size(u8),
    Names(Vec<String>),
}
impl Choices {
    pub fn valid(&self) -> bool {
        fn has_dup(xs: &Vec<String>) -> bool {
            use std::collections::HashSet;
            let mut hs = HashSet::new();
            for x in xs {
                if hs.contains(x) {
                    return true;
                } else {
                    hs.insert(x);
                }
            }
            false
        }
        match self {
            Choices::Size(i) => *i <= i8::MAX as u8,
            Choices::Names(ns) => !has_dup(ns),
        }
    }
}
