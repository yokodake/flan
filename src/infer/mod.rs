//! Inference and Type Checking
//!
//! Since all decisions have to be made at the start of the program
//! without necessarily declaring all dimensions upfront we need a way
//! to check while evaluation if decisions aren't conflicting, and all
//! dimensions with the same name have the same number of choices.
pub mod env;
pub mod errors;

// re-exports
#[doc(inline)]
pub use env::{Dim, Env};
#[doc(inline)]
pub use errors::Error;

// imports
use std::collections::HashMap;

use crate::error::Handler;
use crate::sourcemap::{Span, Spanned};
use crate::syntax::{Name, TermK, Terms};

/// typecheck and infer (by mutating `env`) choices and dimensions.
pub fn check(terms: &Terms, env: &mut Env, handler: &mut Handler) -> Option<()> {
    let mut errors = false;
    for term in terms {
        match &term.node {
            TermK::Text => {}
            TermK::Var(name) => {
                if !env.variables.contains_key(name) {
                    handler
                        .error(format!("Undeclared variable `{}`.", name).as_ref())
                        .with_span(term.span)
                        .print();
                    errors = true;
                }
            }
            TermK::Dimension { name, children } => match env.dimensions.get_mut(name) {
                Some(d) => {
                    if !d.try_set_dim(children.len() as i8) {
                        error_size_conflict(handler, name, term.opend_span().unwrap());
                        errors = true;
                    }
                    for c in children {
                        errors = check(c, env, handler).is_none() || errors;
                    }
                }
                None => {
                    handler
                        .error(format!("Unknown dimension `{}`.", name).as_ref())
                        .with_span(term.opend_span().unwrap())
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

/// returns all the dimensions used and their size & report conflicts
/// @TODO merge with [`check`] ?
pub fn collect<'a>(
    terms: &Terms,
    handler: &mut Handler,
    dims: &'a mut HashMap<String, u8>,
) -> &'a mut HashMap<Name, u8> {
    for Spanned { node, span } in terms {
        match node {
            TermK::Text | TermK::Var(_) => {}
            TermK::Dimension { name, children } => {
                match dims.get(name) {
                    Some(&size) if size != children.len() as u8 => {
                        error_size_conflict(handler, name, *span);
                    }
                    None => {
                        dims.insert(name.clone(), children.len() as u8);
                    }
                    _ => {}
                }

                for c in children {
                    collect(c, handler, dims);
                }
            }
        }
    }
    dims
}

pub fn error_size_conflict(handler: &mut Handler, name: &String, span: Span) {
    // @TODO get span of declaration or previous use
    handler
        .error(format!("Conflicting number of choices for dimension `{}`.", name).as_ref())
        .with_span(span)
        .print();
}
