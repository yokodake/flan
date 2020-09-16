//! Inference and Type Checking
//!
//! Since all decisions have to be made at the start of the program
//! without necessarily declaring all dimensions upfront we need a way
//! to check while evaluation if decisions aren't conflicting, and all
//! dimensions with the same name have the same number of choices.
pub mod env;
pub mod errors;

// re-exports
pub use env::{Dim, Env};
pub use errors::Error;

// imports
use crate::codemap::Spanned;
use crate::error::Handler;
use crate::syntax::{TermK, Terms};

pub fn check(terms: &Terms, env: &mut Env, handler: &mut Handler<Error>) -> Option<()> {
    let mut errors = false;
    for Spanned { node: t, span } in terms {
        match t {
            TermK::Text => {}
            TermK::Var(name) => {
                if !env.variables.contains_key(name) {
                    handler
                        .error(format!("Undeclared variable `{}`.", name).as_ref())
                        .with_span(*span)
                        .with_kind(Error::UnknownVariable)
                        .print();
                    errors = true;
                }
            }
            TermK::Dimension { name, children } => match env.dimensions.get_mut(name) {
                Some(d) => {
                    if !d.try_set_dim(children.len() as i8) {
                        handler
                            .error(
                                format!("Conflicting number of choices for dimension `{}`.", name)
                                    .as_ref(),
                            )
                            .with_span(*span)
                            .print();
                        errors = true;
                    }
                }
                None => {
                    handler
                        .error(format!("Unknown dimension `{}`.", name).as_ref())
                        .with_span(*span)
                        .with_kind(Error::UnknownDecision)
                        .note("Decision inference is not supported yet. This dimensions requires a decision given explicitly.")
                        .note("Postponed dimension declaration (in source files) is not supported yet.")
                        .print();
                    errors = true;
                }
            },
        }
    }
    if errors {
        None
    } else {
        Some(())
    }
}
