use std::collections::HashMap;
use std::env;
use std::io;

fn main() {
    let args: Vec<String> = env::args().collect();
    parse_args(args);
}

fn parse_args(args: Vec<String>) -> DynFlags {
    let mut h = HashMap::new();
    if args.len() > 4 {
        h.insert(args[3].clone(), args[4].clone());
    }
    DynFlags {
        in_fn: args[1].clone(),
        out_fn: args[2].clone(),
        vars: h,
    }
}

#[allow(dead_code)]
struct DynFlags {
    in_fn: String,
    out_fn: String,
    vars: HashMap<syntax::Name, String>,
}

#[allow(dead_code)]
fn process_file(fl: DynFlags) -> io::Result<()> {
    use std::fs::File;
    use std::io::{BufRead, BufReader};
    let mut b = File::open(fl.in_fn).and_then(|f| Ok(BufReader::new(f)))?;
    let mut r: Vec<u8> = Vec::new();
    b.read_until(b'&', &mut r);
    Ok(())
}

fn parse_file(_buf: &std::io::BufReader<std::fs::File>) -> syntax::Terms {
    use syntax::Terms;
    Vec::new()
}

#[allow(dead_code)]
mod syntax {
    use std::io::BufRead;
    type RawText = Vec<u8>;
    // use syntax::codemap::Spanned;
    pub struct SReader<T> (
        std::io::BufReader<T>
    );
    impl<T> SReader<T> {fn get(&self) { }}
    impl<T> Iterator for SReader<T> {
        type Item = u8;
        fn next(&mut self) -> Option<Self::Item> {
            let mut b = [0; 1];
            match self.read(&mut b) {
                Ok(n) => if n < 1 { None } else { Some(b[0]) }
                _ => None
            }
        }
    }

    // terminal tokens
    pub static OPENS: &str = "&{";
    pub static CLOSES: &str = "}&";
    pub static SEPS: &str = "&|&";
    pub static PREVAR: &str = "&&";
    pub static NDELIM: char = '&';

    // &{ $HOME &|& foo_bar &|& }&

    pub type Terms = Vec<Term>;
    pub type Term = Spanned<Term_>;
    pub type Alt = Spanned<Alt_>;
    pub type Name = String;
    #[derive(Clone, PartialEq, PartialOrd, Eq, Ord, Debug, Hash)]
    pub enum Term_ {
        Text(),
        Var(Name),
        Sum(Vec<Alt>),
    }
    #[derive(Clone, PartialEq, PartialOrd, Eq, Ord, Debug, Hash)]
    pub struct Alt_ {
        pub name: Option<Name>,
        pub node: Terms,
    }
    impl Alt_ {
        pub fn has_name(&self) -> bool {
            self.name.is_some()
        }
    }
    #[derive(Clone, PartialEq, PartialOrd, Eq, Ord, Debug, Hash)]
    pub struct Spanned<T> {
        pub node: T,
        pub span: Span,
    }
    #[derive(Clone, PartialEq, PartialOrd, Eq, Ord, Debug, Hash)]
    pub struct Span {
        // pub lo: u64,
    // pub hi: u64,
    // pub filename: String
    }
}
