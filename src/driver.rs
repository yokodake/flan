use std::collections::{HashMap, HashSet, VecDeque};
use std::io;
use std::iter::FromIterator;

use crate::codemap::SrcFile;
use crate::env::{Dim, Env};
use crate::error::Handler;
use crate::opt_parse::Index;
use crate::syntax;
use crate::syntax::{Lexer, Parser, TokenStream};

type PError = syntax::Error;

pub fn make_env(
    variables: Vec<(String, String)>,
    decl_dim: Vec<(String, Vec<String>)>,
    (chs, idxs): (HashSet<String>, HashMap<String, Index>),
) -> Option<Env> {
    use std::fmt::Write;
    let mut dimensions = HashMap::new();
    let mut errors = Vec::new();
    for (dn, ons) in decl_dim {
        // we keep this binding (instead of only `ni`) for error repoorting
        let idx = idxs.get(&dn);
        let mut ni = maybe_idx(idx, &ons);
        // list of valid decisions for the current dimension
        let mut found = Vec::new();
        // if there's a conflict between idx & and a `chs`
        let mut conflict = false;
        // if ni was not set by maybe_idx
        let mut set = false;

        for (p, on) in ons.iter().enumerate() {
            if !chs.contains(on) {
                continue;
            }
            if !set && ni.map_or(false, |(n, _)| n != on) {
                conflict = true;
            }
            if ni.map_or(false, |(n, _)| n == on) {
                // @TODO use error handler instead.
                println!(
                    "note: choices `{}` and `{}={}` are redundant.",
                    on,
                    &dn,
                    idx.unwrap()
                )
            }
            if ni.is_none() {
                ni = Some((on, p as u8));
                set = true;
            }
            found.push(on);
        }
        if conflict || found.len() > 1 {
            // if conflicting decisions
            // @TODO use error handler instead.
            let mut msg = String::from("The following choices are conflicting: ");
            let mut it = found.iter();
            if conflict {
                write!(&mut msg, "{}={}", &dn, idx.unwrap());
            } else {
                write!(&mut msg, "{}", it.next().unwrap());
            }
            for &i in it {
                write!(&mut msg, ", {}", i);
            }
            errors.push(msg);
        } else if !conflict && found.len() == 0 {
            // if no decision for declared dimension
            println!("note: no decision found for dimension `{}`.", dn)
        } else {
            dimensions.insert(
                dn,
                Dim {
                    dimensions: ons.len() as i8,
                    choice: ni.unwrap().1,
                },
            );
        }
    }

    if errors.len() == 0 {
        return Some(Env::new(HashMap::from_iter(variables), dimensions));
    }
    for e in errors {
        eprintln!("{}", e);
    }
    None
}

pub fn maybe_idx<'a>(i: Option<&'a Index>, options: &'a Vec<String>) -> Option<(&'a String, u8)> {
    match i? {
        Index::Name(n) => {
            let i = options.iter().position(|s| n == s)?;
            Some((n, i as u8))
        }
        Index::Num(i) => {
            let n = options.get(*i as usize)?;
            Some((n, *i))
        }
    }
}

pub fn source_to_stream(h: &mut Handler<PError>, src: &str) -> TokenStream {
    let mut vd = VecDeque::new();
    let mut lexer = Lexer::new(src, h);
    loop {
        let t = lexer.next_token();
        vd.push_back(t);
        if t.is_eof() {
            break;
        }
    }
    vd
}

pub fn string_to_parser<'a>(h: &'a mut Handler<PError>, str: String) -> io::Result<Parser<'a>> {
    let ts = source_to_stream(h, str.as_ref());
    if h.find(&PError::is_fatal).is_none() {
        Ok(Parser::new(str, h, ts))
    } else {
        // @TODO custom error instead of io::Error
        Err(io::Error::new(
            io::ErrorKind::Other,
            "Aborting due to previous errors.",
        ))
    }
}

pub fn file_to_parser<'a>(h: &'a mut Handler<PError>, source: SrcFile) -> io::Result<Parser<'a>> {
    use crate::codemap::Source;
    use std::io::{Error, ErrorKind};
    // @SPEED lots of stupid stuff in here
    let apath;
    let src;
    {
        let file = source.read().unwrap();
        src = file.src.clone();
        apath = file.absolute_path.clone();
    }
    match src {
        Source::Binary => {
            return Err(Error::new(
                ErrorKind::InvalidInput,
                "binary data cannot be parsed",
            ))
        }
        Source::Src(s) => return Ok(string_to_parser(h, s.clone())?),
        Source::NotLoaded => {
            let s = std::fs::read_to_string(&apath)?;
            let mut file = source.write().unwrap_or_else(|_| todo!("locks"));
            file.src = Source::Src(s.clone());
            return Ok(string_to_parser(h, s)?);
        }
        // process again?
        Source::Processed => {
            let s = std::fs::read_to_string(&apath)?;
            return Ok(string_to_parser(h, s)?);
        }
    }
}
