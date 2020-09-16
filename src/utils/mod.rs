pub mod codemap;

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
    () => (println!("@DEBUG"));
    ($($arg:tt)*) => (println!("DEBUG: {}", format_args!($($arg)*)));
}
