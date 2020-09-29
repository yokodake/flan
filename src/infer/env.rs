use std::collections::HashMap;

// @TODO use symbols
#[derive(Clone, Debug)]
/// typechecking/inference environment
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
    pub fn new(choice: u8) -> Self {
        Dim {
            choices: -1,
            decision: choice,
        }
    }
    /// tries to set the number of choices a dimension holds, returns false if it failed:
    /// * if it was already set before to a diferent value
    /// * if `n` is also negative
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
    pub fn has_been_inferred(&self) -> bool {
        self.choices >= 0
    }
}
