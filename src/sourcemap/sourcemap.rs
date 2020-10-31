//! Source file maps and Source files.
use std::borrow::Cow;
use std::fs::read_to_string;
use std::io;
use std::path::PathBuf;
use std::sync::atomic::AtomicU64;
use std::sync::{Arc, RwLock};

use super::loc::Loc;
use super::span::*;

#[derive(Hash, Debug, Clone, PartialEq)]
/// Information about the Source
pub enum SourceInfo {
    Source(String),
    /// we do not need the source for binary files
    Binary,
}
/// File info + source
#[derive(Debug)]
pub struct File {
    /// file name without path
    pub name: String,
    pub path: PathBuf,
    pub destination: PathBuf,
    /// Source or its state
    pub src: SourceInfo,
    /// start positions of lines, **relative to [`Self::start`]!**
    pub lines: Vec<Pos>,
    pub start: Pos,
    pub end: Pos,
}
impl File {
    /// panics if not a file name
    pub fn new(path: PathBuf, destination: PathBuf, src: SourceInfo) -> File {
        let name = path.file_name().unwrap().to_string_lossy().into();
        let end = match &src {
            SourceInfo::Source(s) => s.len() - 1,
            _ => 1,
        };
        File {
            name,
            path,
            destination,
            src: src,
            lines: Vec::new(),
            start: Pos(0),
            end: Pos::from(end),
        }
    }
    pub fn is_source(&self) -> bool {
        match self.src {
            SourceInfo::Source(_) => true,
            _ => false,
        }
    }
    pub fn is_binary(&self) -> bool {
        match self.src {
            SourceInfo::Binary => true,
            _ => false,
        }
    }
    pub fn lookup_line(&self, pos: Pos) -> Option<Loc<'_>> {
        use crate::sourcemap as sm;
        let index = self.get_line_num(pos)?;
        let line = self.get_loc(index)?;
        let start = unsafe { self.lines.get_unchecked(index) };
        let end: Pos = self
            .lines
            .get(index + 1)
            .map(|p| p.clone() - 1)
            .unwrap_or(self.end);
        let span = sm::span(*start, end);
        Some(Loc { index, span, line })
    }
    /// gets the index of the line containing `pos`.
    /// This is not a line number.
    pub fn get_line_num(&self, pos: Pos) -> Option<usize> {
        let pos = pos - self.start;
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
    /// gets the contents of the line of code from the source file.
    pub fn get_loc(&self, line_num: usize) -> Option<Cow<'_, str>> {
        let s = (*(self.lines.get(line_num)?) - self.start).as_usize();
        if let SourceInfo::Source(src) = &self.src {
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
    pub fn load_file(&self, path: PathBuf, dest: PathBuf) -> io::Result<SrcFile> {
        let mut file = Self::path_to_file(path, dest)?;
        let start = self.bump_start(file.end.0);
        file.start = Pos::from(start);
        file.end += file.start;
        let af = Arc::new(file);
        self.sources.write().unwrap().push(af.clone());
        Ok(af)
    }
    /// helper that builds a [`File`] from a path
    pub fn path_to_file(path: PathBuf, destination: PathBuf) -> io::Result<File> {
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
                    // @TODO double check if size `1` doesn't lead to bugs
                    (SourceInfo::Binary, 1)
                } else {
                    return Err(e);
                }
            }
            Ok(s) => {
                let l = s.len();
                lines = Self::anal_src(s.as_ref(), start);
                (SourceInfo::Source(s), l)
            }
        };
        Ok(File {
            name,
            path,
            src,
            destination, // @TODO absolute path?
            lines,
            start,
            end: Pos::from(len),
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
    fn bump_start(&self, size: PosInner) -> u64 {
        use std::sync::atomic::Ordering;
        self.start.fetch_add(size + 1, Ordering::Relaxed)
    }
}
