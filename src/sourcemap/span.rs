//! Spans and Positions in source files (map).
use std::ops::{Add, AddAssign, Sub, SubAssign};

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
        (self.hi.0 - self.lo.0 + 1) as usize
    }
    /// merges two spans, same as `+` operator
    pub fn merge(self, other: Span) -> Span {
        self + other
    }
    /// removes the offset
    pub fn correct(&self, offset: Pos) -> Span {
        assert!(offset <= self.lo);
        assert!(offset <= self.hi);
        span(self.lo - offset, self.hi - offset)
    }
    pub fn lo_as_usize(&self) -> usize {
        self.lo.0 as usize
    }
    pub fn hi_as_usize(&self) -> usize {
        self.hi.0 as usize
    }
    /// identity for Span merging/addition
    #[allow(dead_code)]
    pub const MEMPTY: Span = Span {
        lo: Pos(std::u64::MAX),
        hi: Pos(std::u64::MIN),
    };
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
