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

use std::collections::HashMap;

use crate::error::{Handler, ErrorBuilder};
use crate::sourcemap::Span;
use crate::syntax::{Name, TermK, Terms, Term};

//// typecheck and infer (by mutating `env`) choices and dimensions.
pub fn check<'a>(terms: &Terms, env: &'a mut Env) -> (bool, &'a mut Env) {
    traverse(terms, (false, env), &check_pass)
}
fn check_pass<'a>(term: &Term, (mut err, env): (bool, &'a mut Env)) -> (bool, &'a mut Env) {
    match &term.node {
        TermK::Text => {},
        TermK::Var(name) => {
            if !env.eflags().ignore_unset && !env.variables.contains_key(name) {
                env.handler
                   .error(format!("Undeclared variable `{}`.", name).as_ref())
                   .with_span(term.span)
                   .print();
                err = true;
            } 
        },
        TermK::Dimension { name, children } => match env.dimensions.get_mut(name) {
                Some(d) => {
                    if !d.try_set_dim(children.len() as i8) {
                        error_size_conflict(&mut env.handler, name, term.span.subspan(0, name.len() - 1)).print();
                        err = true;
                    } 
                }
                None => {
                    env.handler
                        .error(format!("Unknown dimension `{}`.", name).as_ref())
                        .with_span(term.opend_span().unwrap())
                        .note("Decision inference is not supported yet. This dimension requires a decision given explicitly.")
                        .note("Postponed dimension declaration (in source files) is not supported yet.")
                        .print();
                    err = true;
                }
        }
    }
    (err, env)
}

pub type DMap = HashMap<Name, u8>;

/// returns all the dimensions used and their size & report conflicts
/// @REFACTOR merge with [`check`] ?
pub fn check_collect<'a>(terms: &Terms, dims: &'a mut DMap, env: &'a mut Env) -> (&'a mut DMap, bool, &'a mut Env) {
    traverse(terms, (dims, false, env), &check_collect_pass)
}
pub fn check_collect_pass<'a>(
    term: &Term,
    (dims, err, env) : (&'a mut DMap, bool, &'a mut Env),
) -> (&'a mut DMap, bool, &'a mut Env) {
    let (err, env) = check_pass(term, (err, env));
    if err { // do not collect if there are errors
        return (dims, err, env);
    }
    match &term.node {
        TermK::Text | TermK::Var(_) => {}
        TermK::Dimension { name, children } => {
            match dims.get(name) {
                None => {
                    dims.insert(name.clone(), children.len() as u8);
                }
                _ => {}
            }
        }
    }
    (dims, err, env)
}

/// helper for dimension size conflicts errors
fn error_size_conflict<'a>(handler: &'a mut Handler, name: &String, span: Span) -> ErrorBuilder<'a> {
    // @TODO get span of declaration or previous use
    handler
        .error(format!("Conflicting number of choices for dimension `{}`.", name).as_ref())
        .with_span(span)
}

pub fn traverse<F, T>(terms: &Terms, z: T, transform: &F) -> T
where F : Fn(&Term, T) -> T {
    let mut acc = z;
    for term in terms {
        acc = transform(term, acc);
        match &term.node {
            TermK::Dimension { children , .. } => {
                for child in children {
                    acc = traverse(child, acc,  transform);
                }
            } 
            _ => {}
        }
    }
    acc
}
pub fn traverse_mut<F, T>(terms: &mut Terms, z: T, transform: &F) -> T
where F : Fn(&mut Term, T) -> T {
    let mut acc = z;
    for term in terms {
        acc = transform(term, acc);
        match &mut term.node {
            TermK::Dimension { children , .. } => {
                for child in children {
                    acc = traverse_mut(child, acc,  transform);
                }
            } 
            _ => {}
        }
    }
    acc
}