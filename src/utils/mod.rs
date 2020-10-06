pub mod path;
pub mod source_analysis;
pub mod sourcemap;

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
