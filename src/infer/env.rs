//! Inference/type checking environment.  
//!
//! Possible improvements:
//! * Carry a hashset in the env of decisions left for delayed dimension declarations
//! * add spans to [`Env::dimensions`] and [`Env::variables`] for better error reporting.
//!   this might mean a span for every conflicting dimension call, as well as, a mechanism
//!   to refine delayed_errors.
use std::collections::{HashMap, VecDeque};
use std::fmt;

use crate::cfg::ErrorFlags;
use crate::error::Handler;

#[derive(Debug)]
/// typechecking/inference environment  
/// @TODO: use symbols?
pub struct Env {
    pub variables: HashMap<String, String>,
    pub dimensions: HashMap<String, Dim>,
    pub handler: Handler,
    pub ctx: Ctx,
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
            ctx: Ctx::new(),
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
    pub fn eflags(&self) -> ErrorFlags {
        self.handler.eflags
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
    /// tries to set the number of choices a dimension holds, returns false if:
    /// * it was already set before to a diferent value
    /// * `n` is also negative  
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

/// @SPEED this will incur extra string copies and comparisons... 
///        to fix copies we need a form of Arena, as the String will be owned by Term too
///        (Since the caller of `parse` could drop as soon as it returns the Term)
///        to fix comparisons a symbol table could be used
///        ...the symbol table could use the arena to fix both
pub struct Scope {
    pub dim  : String,
    pub child: u8,
}
#[derive(Default)]
pub struct Ctx(VecDeque<Scope>);
impl Ctx {
    pub fn new() -> Self {
        Ctx(VecDeque::new())
    }
    pub fn push(&mut self, scope: Scope) {
        self.0.push_front(scope);
    }
    pub fn pop(&mut self) -> Option<Scope> {
        self.0.pop_front()
    }
    /// enter a new scope
    pub fn enter(&mut self, dim: String) {
        self.push(Scope{dim, child: 0})
    }
    /// bump the child counter
    pub fn next_child(&mut self) -> bool {
        match self.0.front_mut() { 
            None => false,
            Some(Scope{child, ..}) => {
                *child += 1;
                true
            },
        }
    }
    /// exit the current scope
    pub fn exit(&mut self, name: &String) {
        let n = self.pop().expect("expected non-empty Ctx");
        assert!(*name == n.dim);
    }
    pub fn find(&self, name: &String) -> Option<&Scope> { 
        self.0.iter().find(|Scope{dim, ..}| dim == name)
    }
}

impl AsRef<VecDeque<Scope>> for Ctx {
    fn as_ref(&self) -> &VecDeque<Scope> {
        &self.0
    }
}
impl fmt::Debug for Ctx {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        f.write_str("...")
    }
}