use std::collections::{HashMap, HashSet, VecDeque};
use std::io::Write;
use std::iter::FromIterator;
use std::{fs, io};

use crate::cfg::Choices;
use crate::env::{Dim, Env};
use crate::error::Handler;
use crate::infer;
use crate::opt_parse::Index;
use crate::sourcemap::{Pos, SrcFile};
use crate::syntax::*;

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
        // @SAFETY: write! does not fail on Strings
        #[allow(unused_must_use)]
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
                    choices: ons.len() as i8,
                    decision: ni.unwrap().1,
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

/// tries get the name and index pair from an [`Index`] and a list of choices
pub fn maybe_idx<'a>(i: Option<&'a Index>, choices: &'a Vec<String>) -> Option<(&'a String, u8)> {
    match i? {
        Index::Name(n) => {
            let i = choices.iter().position(|s| n == s)?;
            Some((n, i as u8))
        }
        Index::Num(i) => {
            let n = choices.get(*i as usize)?;
            Some((n, *i))
        }
    }
}

/// transform a source into a [`TokenStream`]
pub fn source_to_stream(h: &mut Handler, src: &str) -> Option<TokenStream> {
    use crate::sourcemap::Pos;
    // @REFACTOR
    let mut vd = VecDeque::new();
    let mut lexer = Lexer::new(h, src, Pos::from(0 as usize));
    loop {
        let t = lexer.next_token();
        vd.push_back(t);
        if lexer.failed() {
            return None;
        }
        if t.is_eof() {
            break;
        }
    }
    Some(vd)
}

pub fn string_to_parser<'a>(h: &'a mut Handler, str: String) -> Option<Parser<'a>> {
    use crate::sourcemap::Pos;
    source_to_stream(h, str.as_ref()).map(move |ts| Parser::new(h, str, ts, Pos::from(0 as usize)))
}

pub fn file_to_parser<'a>(h: &'a mut Handler, source: SrcFile) -> io::Result<Parser<'a>> {
    use crate::sourcemap::SourceInfo;
    use std::io::{Error, ErrorKind};
    match &source.src {
        SourceInfo::Binary => {
            return Err(Error::new(
                ErrorKind::InvalidInput,
                "binary data cannot be parsed",
            ))
        }
        SourceInfo::Src(s) => {
            return Ok(string_to_parser(h, s.clone())
                .ok_or_else(|| Error::new(ErrorKind::Other, "aborting due to previous errors"))?)
        }
    };
}

pub fn collect_dims<'a>(
    terms: &Terms,
    h: &mut Handler,
    declared_dims: &HashMap<Name, Vec<Name>>,
) -> Vec<(Name, Choices)> {
    let mut map = HashMap::new();
    infer::collect(terms, h, &mut map);
    map.into_iter()
        .map(|(k, v)| match declared_dims.get(&k) {
            Some(v) => (k, Choices::Names(v.clone())),
            None => (k, Choices::Size(v)),
        })
        .collect()
}

/// @FIXME pass flags (overriding) and handle escapes
/// @FIXME handle escaped values
/// @TODO we could benefit from [`Write::write_vectored`]
/// @TODO modify Terms with the decision during typechecking so we don't have to search in env?
pub fn write(terms: &Terms, file: SrcFile, env: &Env) -> io::Result<()> {
    let in_f = fs::File::open(&file.path)?;
    let mut reader = io::BufReader::new(in_f);
    let mut out_f = fs::File::create(&file.destination)?;
    write_terms(terms, &mut reader, &mut out_f, file.start.as_u64(), env)
}

fn write_terms(
    terms: &Terms,
    from: &mut io::BufReader<fs::File>,
    to: &mut impl Write,
    pos: u64,
    env: &Env,
) -> io::Result<()> {
    use std::io::{Seek, SeekFrom};
    for t in terms {
        let off = t.span.lo.as_u64() - pos;
        if off > i64::MAX as u64 {
            // we'll bigger than the buffer anyways so no need to use
            // seek_relative
            todo!();
        } else {
            from.seek_relative(off as i64)?;
        }
        write_term(t, from, to, pos, env)?;
    }
    Ok(())
}

fn write_term(
    term: &Term,
    from: &mut io::BufReader<fs::File>,
    to: &mut impl Write,
    pos: u64,
    env: &Env,
) -> io::Result<usize> {
    use std::io::Read;
    // @TODO use write_vectored?
    match &term.node {
        TermK::Text => {
            // safe alternative?
            let mut buf = unsafe { Box::<[u8]>::new_uninit_slice(term.span.len()).assume_init() };
            from.read(&mut buf)?;
            to.write(&buf)?;
        }
        TermK::Var(name) => match env.get_var(name) {
            Some(v) => {
                to.write(v.as_bytes())?;
            }
            None => panic!("@TODO: var `{}` not found", name),
        },
        TermK::Dimension { name, children } => match env.get_dimension(name) {
            Some(dim) => match children.get(dim.decision as usize) {
                Some(child) => write_terms(child, from, to, pos, env)?,
                None => panic!("@TODO: OOB decision for `{}`", name),
            },
            None => panic!("@TODO: dim `{}` not found", name),
        },
    }
    Ok(term.span.len())
}
