//! custom version of [https://docs.rs/codemap/](https://docs.rs/codemap/).

use std::fs::read_to_string;
use std::io;
use std::ops::{Add, AddAssign, Sub, SubAssign};
use std::path::PathBuf;
use std::sync::atomic::AtomicU64;
use std::sync::Arc;
use std::sync::RwLock;

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
    /// @TODO
    pub fn new(name: String) -> File {
        File {
            name,
            path: PathBuf::from(""),
            destination: PathBuf::from(""),
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
}

/// type synonym for easier refactoring
pub type SrcFile = Arc<File>;

#[derive(Debug)]
/// A map of source files. @NOTE Maybe shouldn't be a new type.
pub struct SrcFileMap {
    sources: Vec<SrcFile>,
    start: AtomicU64,
}

impl SrcFileMap {
    pub fn new() -> Self {
        SrcFileMap {
            sources: Vec::new(),
            start: AtomicU64::new(0),
        }
    }
    /// load a file and add it to the map
    pub fn load_file(&mut self, path: &PathBuf, dest: &PathBuf) -> io::Result<SrcFile> {
        let mut file = Self::path_to_file(path, dest)?;
        let start = self.bump_start(file.end.0);
        file.start = Pos(start);
        file.end += file.start;
        for p in file.lines.iter_mut() {
            *p += file.start;
        }
        let af = Arc::new(file);
        self.sources.push(af.clone());
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
                lines = Self::line_pos(s.as_ref(), start);
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
    pub fn line_pos(src: &str, offset: Pos) -> Vec<Pos> {
        let mut lines = vec![offset];
        if cfg!(not(any(target_arch = "x86", target_arch = "x86_64"))) {
            line_pos_slow(src, src.len(), offset, &mut lines);
        }
        if is_x86_feature_detected!("avx2") {
            unsafe {
                line_pos_avx2(src, offset, &mut lines);
            }
        } else if is_x86_feature_detected!("sse2") {
            unsafe {
                line_pos_sse2(src, offset, &mut lines);
            }
        }

        lines
    }
    fn bump_start(&self, size: u64) -> u64 {
        use std::sync::atomic::Ordering;
        self.start.fetch_add(size + 1, Ordering::Relaxed)
    }
}

pub type PosInner = u64;
/// A position inside a codemap.
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
impl Pos {
    pub fn as_usize(&self) -> usize {
        self.0 as usize
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

pub unsafe fn line_pos_sse2(src: &str, offset: Pos, lines: &mut Vec<Pos>) {
    // see: https://doc.rust-lang.org/nightly/nightly-rustc/src/rustc_span/analyze_source_file.rs.html
    #[cfg(target_arch = "x86")]
    use std::arch::x86::*;
    #[cfg(target_arch = "x86_64")]
    use std::arch::x86_64::*;

    const CHUNK_SIZE: usize = 16;
    let src_bytes = src.as_bytes();
    let chunk_count = src.len() / CHUNK_SIZE;

    for chunk_index in 0..chunk_count {
        let ptr = src_bytes.as_ptr() as *const __m128i;
        // loadu because we don't know if aligned to 16bytes
        // @TODO align before?
        let chunk = _mm_loadu_si128(ptr.offset(chunk_index as isize));

        // check whether part of UTF-8 multibyte
        let mb_test = _mm_cmplt_epi8(chunk, _mm_set1_epi8(0));
        let mb_mask = _mm_movemask_epi8(mb_test);

        if mb_mask == 0 {
            // only ascii characters
            let lines_test = _mm_cmpeq_epi8(chunk, _mm_set1_epi8(b'\n' as i8));
            let lines_mask = _mm_movemask_epi8(lines_test);

            if lines_mask != 0 {
                // set the 16 irrelevant msb to '1'
                let mut lines_mask = 0xFFFF0000 | lines_mask as u32;
                // + 1 because we want the position of the newline start, not the '\n' before
                let offset = offset + Pos::from(chunk_index * CHUNK_SIZE + 1);

                loop {
                    let i = lines_mask.trailing_zeros();
                    if i >= CHUNK_SIZE as u32 {
                        // end of chunk
                        break;
                    }

                    lines.push(Pos(i as u64) + offset);
                    lines_mask &= (!1) << i;
                }
                // done with this chunk
                continue;
            } else {
                //  no newlines, nothing to do.
                continue;
            }
        }
        // ignore multibyte chars
    }
    // non aligned bytes on tail
    let tail_start = chunk_count * CHUNK_SIZE;
    if tail_start < src.len() {
        line_pos_slow(
            &src[tail_start..],
            src.len() - tail_start,
            Pos(tail_start as u64) + offset,
            lines,
        );
    }
}

pub unsafe fn line_pos_avx2(src: &str, offset: Pos, lines: &mut Vec<Pos>) {
    // see: https://doc.rust-lang.org/nightly/nightly-rustc/src/rustc_span/analyze_source_file.rs.html
    #[cfg(target_arch = "x86")]
    use std::arch::x86::*;
    #[cfg(target_arch = "x86_64")]
    use std::arch::x86_64::*;

    const CHUNK_SIZE: usize = 32;
    let src_bytes = src.as_bytes();
    let chunk_count = src.len() / CHUNK_SIZE;

    for chunk_index in 0..chunk_count {
        let ptr = src_bytes.as_ptr() as *const __m256i;
        // loadu because we don't know if aligned to 16bytes
        // @TODO align before?
        let chunk = _mm256_loadu_si256(ptr.offset(chunk_index as isize));

        // check whether part of UTF-8 multibyte
        let mb_test = _mm256_cmpgt_epi8(chunk, _mm256_set1_epi8(0));
        let mb_mask = _mm256_movemask_epi8(mb_test);

        if mb_mask == -1 {
            // only ascii characters
            let lines_test = _mm256_cmpeq_epi8(chunk, _mm256_set1_epi8(b'\n' as i8));
            let lines_mask = _mm256_movemask_epi8(lines_test);

            if lines_mask != 0 {
                // set the 16 irrelevant msb to '1'
                let mut lines_mask: u32 = std::mem::transmute(lines_mask);
                // + 1 because we want the position of the newline start, not the '\n' before
                let offset = offset + Pos::from(chunk_index * CHUNK_SIZE + 1);

                loop {
                    let i = lines_mask.trailing_zeros();
                    if i >= CHUNK_SIZE as u32 {
                        // end of chunk
                        break;
                    }

                    lines.push(Pos(i as u64) + offset);
                    lines_mask &= (!1) << i;
                }
                // done with this chunk
                continue;
            } else {
                //  no newlines, nothing to do.
                continue;
            }
        }
        // ignore multibyte chars
    }
    // non aligned bytes on tail
    let tail_start = chunk_count * CHUNK_SIZE;
    if tail_start < src.len() {
        line_pos_slow(
            &src[tail_start..],
            src.len() - tail_start,
            Pos(tail_start as u64) + offset,
            lines,
        );
    }
}

pub fn line_pos_slow(src: &str, len: usize, offset: Pos, lines: &mut Vec<Pos>) {
    let src_bytes = src.as_bytes();
    for i in 0..len {
        let b = unsafe { *src_bytes.get_unchecked(i) };
        if b == b'\n' {
            // + 1 because we want the position of the newline start, not the '\n' before
            lines.push(Pos::from(i) + offset + 1);
        }
    }
}
