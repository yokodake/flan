//! custom version of [https://docs.rs/codemap/](https://docs.rs/codemap/).

use std::io;
use std::ops::{Add, AddAssign, Sub, SubAssign};
use std::path::PathBuf;
use std::sync::atomic::AtomicU64;
use std::sync::{Arc, RwLock};
use std::{borrow::Cow, fs::read_to_string};

#[derive(Hash, Debug, Clone, PartialEq)]
/// Information about the Source
pub enum SourceInfo {
    Src(String),
    /// we do not need the source for binary files
    Binary,
}
/// File info + source
#[derive(Hash, Debug, Clone, PartialEq)]
pub struct File {
    /// file name without path
    pub name: String,
    pub path: PathBuf,
    pub destination: PathBuf,
    /// Source or its state
    pub src: SourceInfo,
    /// start positions of lines
    pub lines: Vec<Pos>,
    pub start: Pos,
    pub end: Pos,
}
impl File {
    /// panics if not a file name
    pub fn new(path: PathBuf) -> File {
        let name = path.file_name().unwrap().to_string_lossy().into();
        File {
            name,
            path: path,
            destination: PathBuf::from(""), // @TODO
            src: SourceInfo::Src(String::from("")),
            lines: Vec::new(),
            start: Pos(0),
            end: Pos(0),
        }
    }
    pub fn is_source(&self) -> bool {
        match self.src {
            SourceInfo::Src(_) => true,
            _ => false,
        }
    }
    pub fn is_binary(&self) -> bool {
        match self.src {
            SourceInfo::Binary => true,
            _ => false,
        }
    }
    pub fn lookup_line(&self, pos: Pos) -> Option<(usize, Cow<'_, str>)> {
        let i = self.get_line_num(pos)?;
        let s = self.get_loc(i)?;
        Some((i, s))
    }
    pub fn get_line_num(&self, pos: Pos) -> Option<usize> {
        if self.lines.is_empty() {
            return None;
        }
        let i = match self.lines.binary_search(&pos) {
            Ok(i) => i,
            Err(i) => i.saturating_sub(1),
        };
        assert!(i < self.lines.len());
        Some(i)
    }
    pub fn get_loc(&self, line_num: usize) -> Option<Cow<'_, str>> {
        let s = (*(self.lines.get(line_num)?) - self.start).as_usize();
        if let SourceInfo::Src(src) = &self.src {
            let lbeg = &src.as_str()[s..];
            let loc = match src.as_str()[s..].find('\n') {
                Some(e) => &lbeg[..e],
                // until EOF
                None => lbeg,
            };
            Some(Cow::from(loc))
        } else {
            None
        }
    }
    pub fn contains(&self, span: Span) -> bool {
        self.start <= span.lo && self.end >= span.hi
    }
}

/// type synonym for easier refactoring
pub type SrcFile = Arc<File>;

#[derive(Debug)]
/// A map of source files. @NOTE Maybe shouldn't be a new type.
pub struct SrcMap {
    pub sources: RwLock<Vec<SrcFile>>,
    start: AtomicU64,
}

impl SrcMap {
    pub fn new() -> Arc<Self> {
        Arc::new(SrcMap {
            sources: RwLock::new(Vec::new()),
            start: AtomicU64::new(0),
        })
    }
    /// load a file and add it to the map
    pub fn load_file(&self, path: &PathBuf, dest: &PathBuf) -> io::Result<SrcFile> {
        let mut file = Self::path_to_file(path, dest)?;
        let start = self.bump_start(file.end.0);
        file.start = Pos(start);
        file.end += file.start;
        for p in file.lines.iter_mut() {
            *p += file.start;
        }
        let af = Arc::new(file);
        self.sources.write().unwrap().push(af.clone());
        Ok(af)
    }
    /// helper that builds a [`File`] from a path
    pub fn path_to_file(path: &PathBuf, dest: &PathBuf) -> io::Result<File> {
        use std::io::{Error, ErrorKind};
        // @TODO
        if !path.is_file() {
            Err(Error::new(
                ErrorKind::InvalidInput,
                format!("`{}` not a file.", path.to_string_lossy()).as_ref(),
            ))?;
        }
        let lines;
        let start = Pos(0);
        let name = path.file_name().unwrap().to_string_lossy().into();
        let (src, len) = match read_to_string(path.as_path()) {
            Err(e) => {
                if e.kind() == ErrorKind::InvalidData {
                    lines = vec![];
                    (SourceInfo::Binary, 1)
                } else {
                    return Err(e);
                }
            }
            Ok(s) => {
                let l = s.len();
                lines = Self::anal_src(s.as_ref(), start);
                (SourceInfo::Src(s), l)
            }
        };
        Ok(File {
            name,
            path: path.clone(),
            src: src,
            destination: dest.clone(), // @TODO absolute path?
            lines,
            start,
            end: Pos(len as u64),
        })
    }
    pub fn anal_src(src: &str, offset: Pos) -> Vec<Pos> {
        use super::source_analysis::*;
        let mut lines = vec![offset];
        if cfg!(not(any(target_arch = "x86", target_arch = "x86_64"))) {
            anal_src_slow(src, src.len(), offset, &mut lines);
        }
        if is_x86_feature_detected!("avx2") {
            unsafe {
                anal_src_avx2(src, offset, &mut lines);
            }
        } else if is_x86_feature_detected!("sse2") {
            unsafe {
                anal_src_sse2(src, offset, &mut lines);
            }
        }

        lines
    }
    pub fn exists(&self, span: Span) -> bool {
        // @SPEED treshold for linear search
        use std::cmp::Ordering;
        self.sources
            .read()
            .unwrap()
            .binary_search_by(|s| {
                if span.is_inbounds(s.start, s.end) {
                    return Ordering::Equal;
                } else if span.hi <= s.start {
                    return Ordering::Less;
                } else {
                    return Ordering::Greater;
                }
            })
            .is_ok()
    }
    pub fn lookup_source(&self, pos: Pos) -> Option<SrcFile> {
        // should we binary search instead? use a threshold?
        for it in self.sources.read().unwrap().iter() {
            if it.start <= pos {
                return Some(it.clone());
            }
        }
        None
    }
    fn bump_start(&self, size: u64) -> u64 {
        use std::sync::atomic::Ordering;
        self.start.fetch_add(size + 1, Ordering::Relaxed)
    }
}

pub type PosInner = u64;
/// A position inside a sourcemap.
#[derive(Copy, Clone, Hash, Eq, PartialEq, Ord, PartialOrd, Debug)]
#[repr(transparent)]
pub struct Pos(pub PosInner);

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
impl From<i32> for Pos {
    fn from(p: i32) -> Pos {
        Pos(p as u64)
    }
}
impl Pos {
    pub fn as_usize(&self) -> usize {
        self.0 as usize
    }
}
impl std::fmt::Display for Pos {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
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
    /// makes a subspan from inside (`offset = span.lo`)
    /// Panics if begin and end are invalid
    pub fn subspan(&self, begin: u64, end: u64) -> Span {
        assert!(end >= begin);
        assert!(self.lo + end <= self.hi);
        Span {
            lo: self.lo + begin,
            hi: self.lo + end,
        }
    }
    pub fn is_inbounds(&self, begin: Pos, end: Pos) -> bool {
        begin <= self.lo && end >= self.hi && self.lo <= end && self.hi >= begin
        // redundant?
    }
    pub fn contains(&self, p: Pos) -> bool {
        self.lo <= p && self.hi >= p
    }
    /// computes length of the span
    pub fn len(&self) -> usize {
        (self.hi.0 - self.lo.0) as usize
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
        write!(f, "{}:{}", self.lo, self.hi)
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
    pub fn new_lit(node: T, lo: impl Into<Pos>, hi: impl Into<Pos>) -> Self {
        Spanned {
            node: node,
            span: Span {
                lo: lo.into(),
                hi: hi.into(),
            },
        }
    }
}
