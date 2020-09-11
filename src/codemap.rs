//! see https://docs.rs/codemap/ but with u64 instead

use std::io;
use std::ops::{Add, Sub};
use std::path::PathBuf;
use std::sync::Arc;

pub enum Source {
    NotLoaded,
    Processed,
    Src(String),
    /// we do not need the source for binary files
    Binary,
}
pub struct SrcFile {
    name: String,
    absolute_path: PathBuf,
    relative_path: PathBuf,
    src: Source,
}

pub type SrcFiles = Vec<Arc<SrcFile>>;

pub struct SrcFileMap(SrcFiles);
impl SrcFileMap {
    pub fn file_exists(&self, path: &PathBuf) -> bool {
        todo!()
    }

    pub fn load_file(&self, path: &PathBuf) -> io::Result<Arc<SrcFile>> {
        todo!()
    }
}

pub type PosInner = u64;
#[derive(Copy, Clone, Hash, Eq, PartialEq, Ord, PartialOrd, Debug)]
pub struct Pos(PosInner);
impl From<PosInner> for Pos {
    fn from(p: PosInner) -> Pos {
        Pos(p)
    }
}

impl Add<u64> for Pos {
    type Output = Pos;
    fn add(self, other: PosInner) -> Pos {
        Pos(self.0 + other)
    }
}
impl Sub<Pos> for Pos {
    type Output = PosInner;
    fn sub(self, other: Pos) -> PosInner {
        self.0 - other.0
    }
}

/// an offset inside the sourcemap
#[derive(Clone, PartialEq, PartialOrd, Eq, Ord, Debug, Hash)]
pub struct Span {
    /// first byte
    lo: Pos,
    /// *after* last byte
    hi: Pos,
}
/// span ctor from inner values
pub fn span(lo: PosInner, hi: PosInner) -> Span {
    Span {
        lo: Pos(lo),
        hi: Pos(hi),
    }
}
/// I'm not sure what the invariants of Add are supposed to be,
/// but since Pos is bounded (u64::MIN, u64::MAX) it is at least a Monoid
impl Add<Span> for Span {
    type Output = Span;
    fn add(self, other: Span) -> Span {
        use std::cmp;
        Span {
            lo: cmp::min(self.lo, other.lo),
            hi: cmp::max(self.hi, other.hi),
        }
    }
}
impl Span {
    /// Panics if begin and end are invalid
    pub fn subspan(&self, begin: u64, end: u64) -> Span {
        assert!(end >= begin);
        assert!(self.lo + end <= self.hi);
        Span {
            lo: self.lo + begin,
            hi: self.lo + end,
        }
    }
    /// computes length of the span
    pub fn len(&self) -> u64 {
        self.hi - self.lo
    }
    /// merges two spans, same as `+` operator
    pub fn merge(self, other: Span) -> Span {
        self + other
    }
    /// identity for Span merging/addition
    #[allow(dead_code)]
    pub const MEMPTY: Span = Span {
        lo: Pos(std::u64::MAX),
        hi: Pos(std::u64::MIN),
    };
}

#[derive(Clone, PartialEq, PartialOrd, Eq, Ord, Debug, Hash)]
pub struct Spanned<T> {
    pub node: T,
    pub span: Span,
}

impl<T> Spanned<T> {
    pub fn new(node: T, lo: Pos, hi: Pos) -> Spanned<T> {
        Spanned {
            node: node,
            span: Span { lo: lo, hi: hi },
        }
    }
}
