//! custom version of [https://docs.rs/codemap/](https://docs.rs/codemap/).

use std::io;
use std::ops::{Add, AddAssign, Sub, SubAssign};
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::RwLock;

#[derive(Hash, Debug, Clone, PartialEq)]
/// Information about the Source
pub enum SourceInfo {
    NotLoaded,
    Processed,
    Src(String),
    /// we do not need the source for binary files
    Binary,
}
/// File info + source
#[derive(Hash, Debug, Clone, PartialEq)]
pub struct File {
    /// file name without path
    pub name: String,
    pub absolute_path: PathBuf,
    /// relative path, error reporting and such
    pub relative_path: PathBuf,
    pub destination: PathBuf, 
    /// Source or its state
    pub src: SourceInfo,
    /// start positions of lines
    pub lines: Vec<Pos>,
    pub start: Pos, 
    pub end: Pos, 
}

/// type synonym for easier refactoring
pub type SrcFile = Arc<RwLock<File>>;

#[derive(Clone, Debug)]
/// A map of source files. @NOTE Maybe shouldn't be a new type.
pub struct SrcFileMap{pub cfg: String, pub sources: Vec<SrcFile>}

impl SrcFileMap {
    pub fn new() -> Self {
        SrcFileMap { cfg: String::from(""), sources: Vec::new() }
    }
    /// load a file and add it to the map
    pub fn load_file(&mut self, path: &PathBuf) -> io::Result<SrcFile> {
        let file = Arc::new(RwLock::new(Self::path_to_file(path)?));
        self.sources.push(file.clone());
        Ok(file)
    }
    /// helper that builds a [`File`] from a path
    pub fn path_to_file(path: &PathBuf) -> io::Result<File> {
        use std::env::current_dir;
        use std::io::{Error, ErrorKind};
        let absolute_path = path.canonicalize()?;
        let relative_path = PathBuf::from("relative/paths/not/implemented/yet");
        if !absolute_path.is_file() {
            Err(Error::new(
                ErrorKind::InvalidInput,
                format!("`{}` not a file.", path.to_string_lossy()).as_ref(),
            ))?;
        }
        let name = absolute_path.file_name().unwrap().to_string_lossy().into();
        Ok(File {
            name,
            absolute_path,
            relative_path,
            src: SourceInfo::NotLoaded,
            destination: "".into(),
            lines: Vec::new(),
            start: Pos(0),
            end: Pos(0),
        })
    }
}

pub type PosInner = u64;
/// A position inside a codemap.
#[derive(Copy, Clone, Hash, Eq, PartialEq, Ord, PartialOrd, Debug)]
#[repr(transparent)]
pub struct Pos(PosInner);

impl From<PosInner> for Pos {
    fn from(p: PosInner) -> Pos {
        Pos(p)
    }
}
impl From<usize> for Pos {
    fn from(p: usize) -> Pos {
        Pos(p as u64)
    }
}
impl std::fmt::Display for Pos {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{}", self.0)
    }
}

impl Add<Pos> for Pos {
    type Output = Pos;
    fn add(self, other: Pos) -> Pos {
        Pos(self.0 + other.0)
    }
}
impl Add<Pos> for PosInner {
    type Output = Pos;
    fn add(self, other: Pos) -> Pos {
        Pos(self + other.0)
    }
}
impl Add<PosInner> for Pos {
    type Output = Pos;
    fn add(self, other: PosInner) -> Pos {
        Pos(self.0 + other)
    }
}
impl Sub<Pos> for Pos {
    type Output = Pos;
    fn sub(self, other: Pos) -> Pos {
        Pos(self.0 - other.0)
    }
}
impl Sub<PosInner> for Pos {
    type Output = Pos;
    fn sub(self, other: PosInner) -> Pos {
        Pos(self.0 - other)
    }
}
impl AddAssign for Pos {
    fn add_assign(&mut self, other: Pos) {
        *self = Pos(self.0 + other.0);
    }
}
impl AddAssign<PosInner> for Pos {
    fn add_assign(&mut self, other: PosInner) {
        *self = Pos(self.0 + other);
    }
}
impl SubAssign for Pos {
    fn sub_assign(&mut self, other: Pos) {
        *self = Pos(self.0 - other.0)
    }
}
impl SubAssign<PosInner> for Pos {
    fn sub_assign(&mut self, other: PosInner) {
        *self = Pos(self.0 - other)
    }
}

/// an span inside the sourcemap
#[derive(Clone, Copy, PartialEq, PartialOrd, Eq, Ord, Debug, Hash)]
pub struct Span {
    /// first byte
    pub lo: Pos,
    /// *after* last byte
    pub hi: Pos,
}
/// span ctor from [`Pos`] values
pub fn span(lo: Pos, hi: Pos) -> Span {
    Span { lo: lo, hi: hi }
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
    /// Span ctor from inner values
    pub fn new(lo: PosInner, hi: PosInner) -> Span {
        Span {
            lo: Pos(lo),
            hi: Pos(hi),
        }
    }
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
        self.hi.0 - self.lo.0
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

    pub fn lo_as_usize(&self) -> usize {
        self.lo.0 as usize
    }
    pub fn hi_as_usize(&self) -> usize {
        self.hi.0 as usize
    }
}
impl std::fmt::Display for Span {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "({}:{})", self.lo, self.hi)
    }
}

/// helper for values that come with a span
#[derive(Clone, Copy, PartialEq, PartialOrd, Eq, Ord, Debug, Hash)]
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
