#![allow(dead_code)]
use std::io::Cursor;

use flan::driver::*;
use flan::env::Env;
use flan::error::{ErrorFlags, Handler};
use flan::output::write_terms;
use flan::sourcemap::{Spanned, SrcMap};
use flan::syntax::lexer::{Token, TokenK};
use flan::syntax::{Parsed, TermK, Terms};

pub type Kinds = Vec<Kind>;
#[derive(Clone, PartialEq, Debug)]
pub enum Kind {
    /// Text
    Txt,
    /// variable
    Var(String),
    /// dimension name
    Dim(String, Vec<Kinds>),
}
pub fn get_kinds(ts: Terms) -> Kinds {
    use Kind::*;
    let mut v = Vec::new();
    for Spanned { node, span: _ } in ts {
        match node {
            TermK::Text => v.push(Txt),
            TermK::Var(n) => v.push(Var(n)),
            TermK::Dimension { name, children } => {
                let mut cs = Vec::new();
                for c in children {
                    cs.push(get_kinds(c));
                }
                v.push(Dim(name, cs))
            }
        }
    }
    v
}
pub fn parse_str(src: &str) -> Parsed<Terms> {
    let mut h = Handler::new(ErrorFlags::default(), SrcMap::new());
    let p = string_to_parser(&mut h, src.into());
    assert!(p.is_some());
    p.unwrap().parse()
}
pub fn lex_str(src: &str) -> Vec<TokenK> {
    let mut h = Handler::new(ErrorFlags::default(), SrcMap::new());
    let s = source_to_stream(&mut h, src);
    assert!(s.is_some());
    s.unwrap().iter().map(|t| t.node).collect()
}
pub fn stream_str(src: &str) -> Vec<Token> {
    let mut h = Handler::new(ErrorFlags::default(), SrcMap::new());
    let s = source_to_stream(&mut h, src);
    assert!(s.is_some());
    let mut v = Vec::new();
    for &t in s.unwrap().iter() {
        v.push(t)
    }
    v
}

pub fn write_str<'a>(src: &'a str, env: &Env) -> String {
    let terms = {
        let t = parse_str(src);
        assert!(t.is_ok());
        t.unwrap()
    };
    let (mut from, mut to) = (Cursor::new(src.as_bytes()), Cursor::new(vec![]));
    assert!(write_terms(&terms, &mut from, &mut to, 0, env).is_ok());
    return std::str::from_utf8(to.get_ref()).unwrap().into();
}
