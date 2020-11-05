//! Inference/type checking environment.  
//!
//! Possible improvements:
//! * Carry a hashset in the env of decisions left for delayed dimension declarations
//! * add spans to [`Env::dimensions`] and [`Env::variables`] for better error reporting.
//!   this might mean a span for every conflicting dimension call, as well as, a mechanism
//!   to refine delayed_errors.
use std::collections::HashMap;

use crate::error::Handler;

#[derive(Debug)]
/// typechecking/inference environment  
/// @TODO: use symbols?
pub struct Env {
    pub variables: HashMap<String, String>,
    pub dimensions: HashMap<String, Dim>,
    pub handler: Handler,
}

impl Env {
    pub fn new(
        variables: HashMap<String, String>,
        dimensions: HashMap<String, Dim>,
        handler: Handler,
    ) -> Self {
        Env {
            variables,
            dimensions,
            handler,
        }
    }
}
impl Env {
    pub fn get_var(&self, name: &String) -> Option<&String> {
        self.variables.get(name)
    }
    pub fn get_dimension(&self, name: &String) -> Option<&Dim> {
        self.dimensions.get(name)
    }
    pub fn get_dimension_mut(&mut self, name: &String) -> Option<&mut Dim> {
        self.dimensions.get_mut(name)
    }
    /// see [`Dim::try_set_dim`]
    pub fn try_set_dimension(&mut self, name: &String, n: i8) -> Option<bool> {
        self.get_dimension_mut(name).map(|d| d.try_set_dim(n))
    }
}

#[derive(Clone, Copy, Debug, Hash, Ord, PartialOrd, Eq, PartialEq)]
/// Dimension
pub struct Dim {
    /// the total number of choices (alternatives) this dimension holds
    /// a negative value means it has not yet been inferred.
    pub choices: i8,
    /// the currently chosen choice. 0-indexed.
    pub decision: u8,
}

impl Dim {
    pub fn new(decision: u8) -> Self {
        Dim {
            choices: -1,
            decision,
        }
    }
    /// tries to set the number of choices a dimension holds, returns false if it failed:
    /// * if it was already set before to a diferent value
    /// * if `n` is also negative  
    /// @INCOMPLETE `self.decision > n`
    pub fn try_set_dim(&mut self, n: i8) -> bool {
        if self.choices != n && self.choices > 0 {
            false
        } else if n < 0 {
            false
        } else {
            self.choices = n;
            true
        }
    }
    /// Whether a dimension's size has already been inferred
    pub fn has_been_inferred(&self) -> bool {
        self.choices >= 0
    }
}
