//! Position in Sourcefile

use std::ops::{Add, AddAssign, Sub, SubAssign};

pub type BytePosInner = u64;
/// A BytePosition inside a sourcemap.
#[derive(Copy, Clone, Hash, Eq, PartialEq, Ord, PartialOrd, Debug)]
#[repr(transparent)]
pub struct BytePos(pub BytePosInner);

/// Won't deal with size errors for now
macro_rules! BytePos_from {
    ( $($TY: ty )+ ) => {
        $(
        impl From<$TY> for BytePos {
            fn from(p: $TY) -> BytePos {
                BytePos(p as BytePosInner)
            }
        }
        )+
    }
}
BytePos_from!( i32 u32 u64 i64 usize isize );

impl BytePos {
    pub fn as_usize(&self) -> usize {
        self.0 as usize
    }
    pub fn as_u64(&self) -> u64 {
        self.0 as u64
    }
}
impl std::fmt::Display for BytePos {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Add<BytePos> for BytePos {
    type Output = BytePos;
    fn add(self, other: BytePos) -> Self::Output {
        BytePos(self.0 + other.0)
    }
}
impl Sub<BytePos> for BytePos {
    type Output = BytePos;
    fn sub(self, other: BytePos) -> Self::Output {
        BytePos(self.0 - other.0)
    }
}
impl AddAssign<BytePos> for BytePos {
    fn add_assign(&mut self, other: BytePos) {
        *self = BytePos(self.0 + other.0);
    }
}
impl AddAssign<usize> for BytePos {
    fn add_assign(&mut self, other: usize) {
        *self = BytePos(self.0 + other as BytePosInner);
    }
}
impl SubAssign<BytePos> for BytePos {
    fn sub_assign(&mut self, other: BytePos) {
        *self = BytePos(self.0 - other.0);
    }
}
macro_rules! BytePos_arith {
    ($($TY:ty)+) => {
        $(
            impl Add<$TY> for BytePos {
                type Output = BytePos;
                fn add(self, other: $TY) -> Self::Output {
                    BytePos(self.0 + other)
                }
            }
            impl Add<BytePos> for $TY {
                type Output = BytePos;
                fn add(self, other: BytePos) -> Self::Output {
                    BytePos(self + other.0)
                }
            }
            impl Sub<$TY> for BytePos {
                type Output = BytePos;
                fn sub(self, other: $TY) -> Self::Output {
                    BytePos(self.0 - other)
                }
            }
            impl Sub<BytePos> for $TY {
                type Output = BytePos;
                fn sub(self, other: BytePos) -> Self::Output {
                    BytePos(self - other.0)
                }
            }
            impl AddAssign<$TY> for BytePos {
                fn add_assign(&mut self, other: $TY) {
                    *self = *self + other;
                }
            }
            impl SubAssign<$TY> for BytePos {
                fn sub_assign(&mut self, other: $TY) {
                    *self = *self - other;
                }
            }
        )+
    };
}
BytePos_arith!(BytePosInner);

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