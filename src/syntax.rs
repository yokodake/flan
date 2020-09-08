use std::io::BufRead;
use std::io::Read;

type RawText = Vec<u8>;
// use syntax::codemap::Spanned;
pub struct SReader<T>(std::io::BufReader<T>);
impl<T> Read for SReader<T> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        self.read(buf)
    }
}
impl<T> BufRead for SReader<T> {
    fn fill_buf(&mut self) -> std::io::Result<&[u8]> {
        self.fill_buf()
    }

    fn consume(&mut self, amt: usize) {
        self.consume(amt);
    }
}

impl<T> Iterator for SReader<T> {
    type Item = u8;
    fn next(&mut self) -> Option<Self::Item> {
        let mut buf = Vec::new();
        match self.read_until(b'&', &mut buf) {
            Ok(n) => {
                if (buf[n - 1]) {
                    Some(Text())
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
    Text(),
    Var(Name),
    Sum(Vec<Alt>),
}
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
