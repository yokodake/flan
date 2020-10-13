//! Spans and Positions in source files (map).
use std::ops::{Add, AddAssign, Sub, SubAssign};

pub type PosInner = u64;
/// A position inside a sourcemap.
#[derive(Copy, Clone, Hash, Eq, PartialEq, Ord, PartialOrd, Debug)]
#[repr(transparent)]
pub struct Pos(pub PosInner);

/// Won't deal with size errors for now
macro_rules! pos_from {
    ( $($TY: ty )+ ) => {
        $(
        impl From<$TY> for Pos {
            fn from(p: $TY) -> Pos {
                Pos(p as PosInner)
            }
        }
        )+
    }
}
pos_from!( i32 u32 u64 i64 usize isize );

impl Pos {
    pub fn as_usize(&self) -> usize {
        self.0 as usize
    }
    pub fn as_u64(&self) -> u64 {
        self.0 as u64
    }
}
impl std::fmt::Display for Pos {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Add<Pos> for Pos {
    type Output = Pos;
    fn add(self, other: Pos) -> Self::Output {
        Pos(self.0 + other.0)
    }
}
impl Sub<Pos> for Pos {
    type Output = Pos;
    fn sub(self, other: Pos) -> Self::Output {
        Pos(self.0 - other.0)
    }
}
impl AddAssign<Pos> for Pos {
    fn add_assign(&mut self, other: Pos) {
        *self = Pos(self.0 + other.0);
    }
}
impl AddAssign<usize> for Pos {
    fn add_assign(&mut self, other: usize) {
        *self = Pos(self.0 + other as PosInner);
    }
}
impl SubAssign<Pos> for Pos {
    fn sub_assign(&mut self, other: Pos) {
        *self = Pos(self.0 - other.0);
    }
}
macro_rules! pos_arith {
    ($($TY:ty)+) => {
        $(
            impl Add<$TY> for Pos {
                type Output = Pos;
                fn add(self, other: $TY) -> Self::Output {
                    Pos(self.0 + other)
                }
            }
            impl Add<Pos> for $TY {
                type Output = Pos;
                fn add(self, other: Pos) -> Self::Output {
                    Pos(self + other.0)
                }
            }
            impl Sub<$TY> for Pos {
                type Output = Pos;
                fn sub(self, other: $TY) -> Self::Output {
                    Pos(self.0 - other)
                }
            }
            impl Sub<Pos> for $TY {
                type Output = Pos;
                fn sub(self, other: Pos) -> Self::Output {
                    Pos(self - other.0)
                }
            }
            impl AddAssign<$TY> for Pos {
                fn add_assign(&mut self, other: $TY) {
                    *self = *self + other;
                }
            }
            impl SubAssign<$TY> for Pos {
                fn sub_assign(&mut self, other: $TY) {
                    *self = *self - other;
                }
            }
        )+
    };
}
pos_arith!(PosInner);

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
    pub fn subspan(&self, begin: impl Into<Pos>, end: impl Into<Pos>) -> Span {
        let begin = begin.into();
        let end = end.into();
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
    /// Identity for Span merging/addition
    pub const MEMPTY: Span = Span {
        lo: Pos(PosInner::MAX),
        hi: Pos(PosInner::MIN),
    };
    /// Annihilator for Span merging/addition
    pub const NIL: Span = Span {
        lo: Pos(PosInner::MIN),
        hi: Pos(PosInner::MAX),
    };
    pub fn is_nil(&self) -> bool {
        *self == Self::NIL
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
