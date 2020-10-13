pub mod path;

/// a strict version of haskell's [sequence](https://hackage.haskell.org/package/base-4.12.0.0/docs/src/Data.Traversable.html#sequence)
pub trait Sequenceable<T> {
    fn sequence<F: FnOnce(&T) -> ()>(self, f: F) -> Self;
}

impl<T> Sequenceable<T> for Option<T> {
    fn sequence<F: FnOnce(&T) -> ()>(self, f: F) -> Option<T> {
        self.map(|x| {
            f(&x);
            x
        })
    }
}

impl<T, E> Sequenceable<T> for Result<T, E> {
    fn sequence<F: FnOnce(&T) -> ()>(self, f: F) -> Result<T, E> {
        self.map(|x| {
            f(&x);
            x
        })
    }
}

#[macro_export]
macro_rules! debug {
    () => {#[cfg(debug_assertions)] println!("@DEBUG")};
    ($($arg:tt)*) => {#[cfg(debug_assertions)] println!("DEBUG: {}", format_args!($($arg)*))};
}

use std::io;
use std::io::{BufReader, Cursor, Seek, SeekFrom};
pub trait RelativeSeek {
    fn seek_relative(&mut self, offset: i64) -> io::Result<()>;
    fn seek(&mut self, offset: SeekFrom) -> io::Result<u64>;
}

impl<R: Seek> RelativeSeek for BufReader<R> {
    fn seek_relative(&mut self, offset: i64) -> io::Result<()> {
        self.seek_relative(offset)
    }
    fn seek(&mut self, offset: SeekFrom) -> io::Result<u64> {
        <Self as Seek>::seek(self, offset)
    }
}
impl<T> RelativeSeek for Cursor<T>
where
    T: AsRef<[u8]>,
{
    fn seek_relative(&mut self, offset: i64) -> io::Result<()> {
        <Self as Seek>::seek(self, io::SeekFrom::Current(offset))?;
        Ok(())
    }
    fn seek(&mut self, offset: SeekFrom) -> io::Result<u64> {
        <Self as Seek>::seek(self, offset)
    }
}
