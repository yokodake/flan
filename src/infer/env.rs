use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

use crate::codemap::SrcFileMap;
use crate::error::Handler;
use crate::infer::Error;

// @TODO use symbols
#[derive(Clone, Debug)]
pub struct Env {
    pub variables: HashMap<String, String>,
    pub dimensions: HashMap<String, Dim>,
    // pub choices: HashSet<String>,
    // pub file_map: HashMap<PathBuf, PathBuf>,
    // pub source_map: SrcFileMap
}

impl Env {
    pub fn new(variables: HashMap<String, String>, dimensions: HashMap<String, Dim>) -> Self {
        Env {
            variables,
            dimensions,
        }
    }
    pub fn get_var(&self, name: &String) -> Option<&String> {
        self.variables.get(name)
    }
    pub fn get_dimension(&self, name: &String) -> Option<&Dim> {
        self.dimensions.get(name)
    }
    pub fn get_dimension_mut(&mut self, name: &String) -> Option<&mut Dim> {
        self.dimensions.get_mut(name)
    }
    pub fn try_set_dimension(&mut self, name: &String, n: i8) -> Option<bool> {
        self.get_dimension_mut(name).map(|d| d.try_set_dim(n))
    }
}

#[derive(Clone, Copy, Debug, Hash, Ord, PartialOrd, Eq, PartialEq)]
pub struct Dim {
    /// the total number of options/alternatives this dimension can hold
    /// negative value = dimensions not yet inferred.
    pub dimensions: i8,
    /// the currently chosen option's. 0-indexed
    pub choice: u8,
}

impl Dim {
    pub fn new(choice: u8) -> Self {
        Dim {
            dimensions: -1,
            choice: choice,
        }
    }
    /// tries to set the number of options a dimension holds, returns false if it failed
    /// fails if it was already set before to a diferent value
    /// fails if `n` is negative too
    pub fn try_set_dim(&mut self, n: i8) -> bool {
        if self.dimensions != n && self.dimensions > 0 {
            false
        } else if n < 0 {
            false
        } else {
            self.dimensions = n;
            true
        }
    }
    pub fn has_been_inferred(&self) -> bool {
        self.dimensions >= 0
    }
}
