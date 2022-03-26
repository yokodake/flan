//! Spans and BytePositions in source files (map).

pub use super::pos::{BytePos, BytePosInner};
use std::ops::{Add, Range, RangeInclusive};

/// an span inside the sourcemap
#[derive(Clone, Copy, PartialEq, PartialOrd, Eq, Ord, Debug, Hash)]
pub struct Span {
    /// first byte
    pub lo: BytePos,
    /// *after* last byte => not included
    pub hi: BytePos,
}
/// span ctor from [`BytePos`] values
pub fn span(lo: BytePos, hi: BytePos) -> Span {
    Span { lo: lo, hi: hi }
}
/// I'm not sure what the invariants of Add are supBytePosed to be,
/// but since BytePos is bounded (u64::MIN, u64::MAX) it is at least a Monoid
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
    pub fn new(lo: BytePosInner, hi: BytePosInner) -> Span {
        Span {
            lo: BytePos(lo),
            hi: BytePos(hi),
        }
    }
    /// makes a subspan from inside (`offset = span.lo`)
    /// Panics if begin and end are invalid
    pub fn subspan(&self, begin: impl Into<BytePos>, end: impl Into<BytePos>) -> Span {
        let begin = begin.into();
        let end = end.into();
        assert!(end >= begin);
        assert!(self.lo + end <= self.hi);
        Span {
            lo: self.lo + begin,
            hi: self.lo + end,
        }
    }
    pub fn is_inbounds(&self, begin: BytePos, end: BytePos) -> bool {
        begin <= self.lo && end >= self.hi && self.lo <= end && self.hi >= begin
        // redundant?
    }
    pub fn contains(&self, p: BytePos) -> bool {
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
    pub fn correct(&self, offset: BytePos) -> Span {
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
    /// Identity for Span merging/addition
    pub const MEMPTY: Span = Span {
        lo: BytePos(BytePosInner::MAX),
        hi: BytePos(BytePosInner::MIN),
    };
    /// Annihilator for Span merging/addition
    pub const NIL: Span = Span {
        lo: BytePos(BytePosInner::MIN),
        hi: BytePos(BytePosInner::MAX),
    };
    pub fn is_nil(&self) -> bool {
        *self == Self::NIL
    }
    /// lo .. hi
    pub fn as_range(&self) -> Range<usize> {
        self.lo_as_usize() .. self.hi_as_usize()
    }
    /// lo ..= hi - 1
    pub fn as_range_inc(&self) -> RangeInclusive<usize> {
        self.lo_as_usize() ..= self.hi_as_usize() - 1
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
    pub fn new(node: T, lo: impl Into<BytePos>, hi: impl Into<BytePos>) -> Spanned<T> {
        Spanned {
            node: node,
            span: Span {
                lo: lo.into(),
                hi: hi.into(),
            },
        }
    }
    pub fn new_lit(node: T, lo: impl Into<BytePos>, hi: impl Into<BytePos>) -> Self {
        Spanned {
            node: node,
            span: Span {
                lo: lo.into(),
                hi: hi.into(),
            },
        }
    }
}
impl<T> AsRef<T> for Spanned<T> {
    fn as_ref(&self) -> &T {
        &self.node
    }
}
