use std::collections::{HashMap, HashSet, VecDeque};
use std::io::Write;
use std::iter::FromIterator;
use std::{fs, io};

use crate::cfg::Choices;
use crate::env::{Dim, Env};
use crate::error::Handler;
use crate::infer;
use crate::opt_parse::Index;
use crate::sourcemap::SrcFile;
use crate::syntax::*;

pub enum EResult<T, E, N> {
    Ok(T),
    Err(E),
    None(N),
}

/// helper to make an env from config file (`variables` and `decl_dim`) and cmd line options
/// (`chs` and `idxs`)
pub fn make_env<'a>(
    variables: Vec<(String, String)>, // declared vars
    decl_dim: Vec<(String, Choices)>, // declared dimensions
    (cns, idxs): (HashSet<String>, HashMap<String, Index>), // decisions
    handler: &'a mut Handler,
) -> Option<Env<'a>> {
    let mut dimensions = HashMap::new();
    let mut errors = Vec::new();
    for (dn, chs) in decl_dim {
        let r = match chs {
            Choices::Names(ons) => handle_named(&dn, ons, &cns, &idxs),
            Choices::Size(i) => handle_sized(&dn, i, &idxs),
        };
        match r {
            EResult::Ok(dim) => {
                dimensions.insert(dn, dim);
            }
            // @TODO use error handler
            EResult::None(note) => {
                println!("{}", note);
            }
            EResult::Err(err) => {
                errors.push(err);
            }
        }
    }
    if errors.len() == 0 {
        // add idxs left
        return Some(Env::new(HashMap::from_iter(variables), dimensions, handler));
    }
    for e in errors {
        eprintln!("{}", e);
    }
    None
}

fn handle_named(
    dn: &str,
    ons: Vec<String>,
    chs: &HashSet<String>,
    idxs: &HashMap<String, Index>,
) -> EResult<Dim, String, String> {
    use std::fmt::Write;
    // we keep this binding (instead of only `ni`) for error repoorting
    let idx = idxs.get(dn);
    let mut ni = maybe_idx(idx, &ons);
    // list of valid decisions for the current dimension
    let mut found = Vec::new();
    // if there's a conflict between `idx` and a `chs`
    let mut conflict = false;

    for (p, on) in ons.iter().enumerate() {
        if !chs.contains(on) {
            continue;
        }
        if ni.map_or(false, |(n, _)| n != on) {
            conflict = true
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
        EResult::Err(msg)
    } else if !conflict && found.len() == 0 {
        // if no decision for declared dimension
        EResult::None(format!("no decision found for declared dimension `{}`.", dn))
    } else { // !conflict && found.len() == 1
        EResult::Ok(Dim {
            choices: ons.len() as i8,
            decision: ni.unwrap().1,
        })
    }
}

fn handle_sized(
    dn: &str,
    size: u8,
    decisions: &HashMap<String, Index>,
) -> EResult<Dim, String, String> {
    match decisions.get(dn) {
        Some(Index::Num(i)) => {
            if *i < size {
                EResult::Ok(Dim {choices: size as i8, decision: *i})
            } else {
                // @TODO note: dimensions declared here: 
                EResult::Err(format!("error: index greater than declared dimension size for decision `{}`=`{}`", dn, i))
            }
        }
        Some(Index::Name(n)) => 
            // @TODO note: dimensions declared here: 
            EResult::Err(format!("error: dimension `{}` declared with size `{}`, but a decision name `{}` was given instead of an index.", dn, size, n)),
        None => 
            EResult::None(format!("note: no decision found for dimension `{}`.", dn)),
    }
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

pub fn write_terms(
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
            // i64::MAX is bigger than the buffer anyways
            from.seek(SeekFrom::Current(i64::MAX))?;
            let rest = off - i64::MAX as u64;
            from.seek_relative(rest as i64)?;
        } else {
            from.seek_relative(off as i64)?;
        }
        write_term(t, from, to, pos, env)?;
    }
    Ok(())
}

pub fn write_term(
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
