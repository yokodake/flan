#![allow(dead_code)]
use std::io::{Cursor, BufRead, Write, self};

use flan::driver::*;
use flan::env::Env;
use flan::error::{ErrorFlags, Handler};
use flan::output::{ReadCtx, WriteCtx};
use flan::output;
use flan::sourcemap::{Spanned, SrcMap};
use flan::syntax::lexer::{Token, TokenK};
use flan::syntax::{Parsed, TermK, Terms, Name};

pub type Kinds = Vec<Kind>;
#[derive(Clone, Debug)]
pub enum Kind {
    /// Text
    Txt,
    /// full text where contents are also checked
    Text(String),
    /// variable
    Var(String),
    /// dimension name
    Dim(String, Vec<Kinds>),
}
impl PartialEq for Kind {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Txt, Self::Text(_)) => true,
            (Self::Text(_), Self::Txt) => true,
            (Self::Text(l0), Self::Text(r0)) => l0 == r0,
            (Self::Var(l0), Self::Var(r0)) => l0 == r0,
            (Self::Dim(l0, l1), Self::Dim(r0, r1)) => l0 == r0 && l1 == r1,
            _ => core::mem::discriminant(self) == core::mem::discriminant(other),
        }
    }
}

pub fn ktxt() -> Kind { Kind::Txt }
pub fn ktext(txt : impl Into<String>) -> Kind { Kind::Text(txt.into()) }
pub fn kvar(name : impl Into<Name>) -> Kind { Kind::Var(name.into()) }
pub fn kdim(name : impl Into<Name>, children: Vec<Kinds>) -> Kind { 
    Kind::Dim(name.into(), children)
}
/// get kinds, but use [`Kind::Txt`] for text.
pub fn get_kinds(ts: Terms) -> Kinds {
    mk_kinds(ts, None)
}
/// get kinds, but use [`Kind::Text`] for text. i.e. actual text will be cloned and compared.
pub fn get_full_kinds(ts: Terms, src: &str) -> Kinds {
    mk_kinds(ts, Some(src))
}

/// if `[full]` then Kind::Text is used instead of Txt
fn mk_kinds(ts: Terms, src: Option<&str>) -> Kinds {
    use Kind::*;
    let mut v = Vec::new();
    for Spanned { node, span } in ts {
        match node {
            TermK::Text => match src {
                None => v.push(Txt),
                Some(src) => {
                    v.push(Text(src[span.as_range()].into()))
                }
            }
            TermK::Var(n) => v.push(Var(n)),
            TermK::Dimension { name, children } => {
                let mut cs = Vec::new();
                for c in children {
                    cs.push(mk_kinds(c, src));
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
    assert!(write_terms(&mut from, 0usize, &mut to, env, &terms).is_ok());
    return std::str::from_utf8(to.get_ref()).unwrap().into();
}

pub fn write_terms<R, W>(from: &mut R, start: impl Into<usize>, to: &mut W, env: &Env, terms: &Terms) -> io::Result<()> 
where R: BufRead, W : Write {
    output::write_terms(&mut ReadCtx::new(from, start), &mut WriteCtx::new(to), env, terms)
}