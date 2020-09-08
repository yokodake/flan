#![allow(dead_code)]
use std::io::BufRead;
use std::io::Read;

// use syntax::codemap::Spanned;

pub struct Parser<R>{reader: std::io::BufReader<R>, pos: u32}

impl<R: Read> Parser<R> {
    fn new(source : R) -> Parser<R> {
        Parser{reader: std::io::BufReader::new(source), pos: 0}
    }
}

impl<R: Read> Read for Parser<R> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        self.reader.read(buf).and_then(|n| {self.pos += 1; Ok(n)})
    }
}
impl<R: Read> BufRead for Parser<R> {
    fn fill_buf(&mut self) -> std::io::Result<&[u8]> {
        self.reader.fill_buf()
    }

    fn consume(&mut self, amt: usize) {
        self.reader.consume(amt)
    }
}

impl<R: Read> Iterator for Parser<R> {
    type Item = Term_;

    fn next<'a>(&'a mut self) -> Option<Self::Item> {
        let mut buf = Vec::new();
        match self.read_until(b'&', &mut buf) {
            Ok(n) => {
                if buf[n - 1] != NDELIM as u8 || buf[n - 2] == b'\\' {
                    let bx : Box<[u8]> = buf.into_boxed_slice();
                    // SAFETY
                    let s : &str = unsafe { std::str::from_utf8_unchecked(bx.as_ref()) };
                    let bs = Box::from(s);
                    Some(Term_::Text(Box::from(bs)))
                } else {
                    None
                }
            }
            _ => None,
        }
    }
}

// terminal tokens
pub const OPENS: &str = "&{";
pub const CLOSES: &str = "}&";
pub const SEPS: &str = "&|&";
pub const PREVAR: &str = "&&";
pub const NDELIM: char = '&';
// e.g. &&Key = &{ $HOME &|& foo_bar &|& }&

pub type Terms = Vec<Term>;
pub type Term = Spanned<Term_>;
pub type Alt = Spanned<Alt_>;
pub type Name = String;
#[derive(Clone, PartialEq, PartialOrd, Eq, Ord, Debug, Hash)]
pub enum Term_ {
    Text(Box<str>),
    Var(Name),
    Sum(Vec<Alt>),
}
pub use Term_::*;
#[derive(Clone, PartialEq, PartialOrd, Eq, Ord, Debug, Hash)]
pub struct Alt_ {
    pub name: Option<Name>,
    pub node: Terms,
}
impl Alt_ {
    pub fn has_name(&self) -> bool {
        self.name.is_some()
    }
}
#[derive(Clone, PartialEq, PartialOrd, Eq, Ord, Debug, Hash)]
pub struct Spanned<T> {
    pub node: T,
    pub span: Span,
}
#[derive(Clone, PartialEq, PartialOrd, Eq, Ord, Debug, Hash)]
pub struct Span {
    // pub lo: u64,
// pub hi: u64,
// pub filename: String
}
