//! Inference and Type Checking
//!
//! Since all decisions have to be made at the start of the program
//! without necessarily declaring all dimensions upfront we need a way
//! to check while evaluation if decisions aren't conflicting, and all
//! dimensions with the same name have the same number of choices.

use crate::env::Env;
use crate::syntax::Terms;

pub fn check(terms: &Terms, env: &Env) -> Result<(), TError> {
    Ok(())
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub enum TError {
    DimensionMismatch,
    UnknownDimension,
}

use crate::error::Pattern;
impl Pattern<TError> for TError {
    fn found(&self, e: &TError) -> bool {
        self == e
    }
}
