//! helpers and TL functions
use std::collections::{HashMap, HashSet, VecDeque};
use std::iter::FromIterator;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::{fs, io};

use crate::env::{Dim, Env};
use crate::error::{ErrorBuilder, Handler};
use crate::output::write_terms;
use crate::sourcemap::{SrcFile, SrcMap};
use crate::syntax::*;
use crate::{cfg, infer};
use crate::{
    cfg::{Choices, Index},
    utils::RelativeSeek,
};

/* infer */

/// helper to make an env from config file (`variables` and `decl_dim`) and cmd line options
/// (`chs` and `idxs`)
pub fn make_env(config: &cfg::Config, handler: Handler) -> Result<Env, Handler> {
    let variables = config.variables.clone();
    let decl_dim = config.dimensions.clone();
    let names = &config.decisions_name;
    let pairs = &config.decisions_pair;
    let mut handler = handler;

    let mut dimensions = HashMap::new();
    let err_diff = handler.err_count;
    for (dn, chs) in decl_dim {
        let r = match chs {
            Choices::Names(chns) => handle_named(&dn, chns, names, pairs, &mut handler),
            Choices::Size(i) => handle_sized(&dn, i, pairs, &mut handler),
        };
        match r {
            Ok(dim) => {
                dimensions.insert(dn, dim);
            }
            Err(eb) => {
                if eb.is_error() {
                    eb.delay();
                } else {
                    eb.print();
                }
            }
        }
    }
    if handler.err_count == err_diff {
        // add idxs left to env
        let mut env = Env::new(HashMap::from_iter(variables), dimensions, handler);
        // @SPEEDUP don't clone
        fill_env(pairs.clone(), &mut env);
        return Ok(env);
    }
    handler.print_all();
    Err(handler)
}

/// handle named choices of declared dimension for [`make_env`]
fn handle_named<'a>(
    dn: &str,
    chns: Vec<String>,
    names: &HashSet<String>,        // standalone decision names
    pairs: &HashMap<String, Index>, // `dimension=decision` pairs
    handler: &'a mut Handler,
) -> Result<Dim, ErrorBuilder<'a>> {
    use std::fmt::Write;
    // we keep this binding for error reporting
    let idx = pairs.get(dn);
    let mut ni = maybe_idx(idx, &chns);
    // list of valid decisions for the current dimension
    let mut found = Vec::new();
    // conflict between `names` and `pairs => ni`
    let mut conflict = false;

    for (p, chn) in chns.iter().enumerate() {
        if names.contains(chn) {
            // if the decision we found in the `pairs` is different than the one we found in `names`
            // we have a conflict.
            if ni.map_or(false, |(n, _)| n != chn) {
                conflict = true
            }
            // if there is both a standalone and pair for the same decision, it's redundant
            if ni.map_or(false, |(n, _)| n == chn) {
                handler
                    .warn(
                        format!(
                            "decisions `{}` and `{}={}` are redundant.",
                            chn,
                            &dn,
                            idx.unwrap()
                        )
                        .as_ref(),
                    )
                    .print();
            }
            if ni.is_none() {
                ni = Some((chn, p as u8));
            }
        } else {
            if ni.map_or(true, |(n, _)| n != chn) {
                continue;
            }
        }
        found.push(chn);
    }
    // @SAFETY: write! does not fail on Strings
    #[allow(unused_must_use)]
    if conflict || found.len() > 1 {
        // if conflicting decisions
        let mut msg = String::from("the following choices are conflicting: ");
        let mut it = found.iter();
        if conflict { // @SAFETY unwrap(): conflict = true, implies that `ni.is_some` (which implies `idx.is_some`)
            write!(&mut msg, "{}={}", &dn, idx.unwrap());
        } else { // @SAFETY unwrap(): found.len() > 1
            write!(&mut msg, "{}", it.next().unwrap());  
        }
        for &i in it {
            write!(&mut msg, ", {}", i);
        }
        Err(handler.error(msg.as_ref()))
    } else if !conflict && found.len() == 0 {
        // if no decision for declared dimension
        // @NOTE should this be a warning instead?
        Err(handler.note(format!("no decision found for declared dimension `{}`.", dn).as_ref()))
    } else {
        // !conflict && found.len() == 1
        Ok(Dim {
            choices: chns.len() as i8,
            // @DOC: unwrap safety
            decision: ni.unwrap().1,
        })
    }
}

/// handle Sized dimension declaration for [`make_env`]
fn handle_sized<'a>(
    dn: &str,
    size: u8,
    decisions: &HashMap<String, Index>,
    handler: &'a mut Handler,
) -> Result<Dim, ErrorBuilder<'a>> {
    match decisions.get(dn) {
        Some(Index::Num(i)) => {
            if *i < size {
                Ok(Dim {choices: size as i8, decision: *i})
            } else {
                // @TODO note: dimensions declared here: 
                Err(handler.error(format!("index greater than declared dimension size for decision `{}`=`{}`", dn, i).as_ref()))
            }
        }
        Some(Index::Name(n)) =>
            // @TODO note: dimensions declared here: 
            Err(handler.error(format!("dimension `{}` declared with size `{}`, but a decision name `{}` was given instead of an index.", dn, size, n).as_ref())),
        None =>
            Err(handler.note(format!("no decision found for dimension `{}`.", dn).as_ref())),
    }
}

/// tries to get the name and index pair from an [`Index`] and a list of choices
/// returns `None` if the named decision isn't in choices, or index is out of bounds of choices.
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

/// fill the env with the remaining decisions
pub fn fill_env(decisions: HashMap<String, Index>, env: &mut Env) {
    for (dn, idx) in decisions.into_iter() {
        match idx {
            Index::Num(i) => match env.get_dimension(&dn) {
                Some(Dim { .. }) => {}
                None => {
                    env.dimensions.insert(dn, Dim::new(i));
                }
            },
            Index::Name(_) => {}
        };
    }
}

/* syntax */

/// Does not fail, only report. Caller should check if all sources passed yielded Terms
pub fn parse_sources(
    sources: Vec<SrcFile>,
    h: &mut Handler,
) -> (Vec<(SrcFile, Terms)>, Vec<SrcFile>) {
    let mut bins = vec![];
    let mut trees = vec![];
    for f in sources {
        if f.is_binary() {
            bins.push(f);
            continue;
        }
        match file_to_parser(h, f.clone()) {
            Some(mut p) => match p.parse() {
                Ok(tree) => {
                    trees.push((f, tree));
                }
                Err(_) => {
                    h.print_all();
                }
            },
            None => {
                h.print_all();
                continue;
            }
        }
    }
    (trees, bins)
}

/// transform a source into a [`TokenStream`]
pub fn source_to_stream(h: &mut Handler, src: &str) -> Option<TokenStream> {
    use crate::sourcemap::BytePos;
    // @REFACTOR
    let mut vd = VecDeque::new();
    let mut lexer = Lexer::new(h, src, BytePos::from(0 as usize));
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
    use crate::sourcemap::BytePos;
    source_to_stream(h, str.as_ref()).map(move |ts| Parser::new(h, str, ts, BytePos::from(0 as usize)))
}

pub fn file_to_parser<'a>(h: &'a mut Handler, source: SrcFile) -> Option<Parser<'a>> {
    use crate::sourcemap::SourceInfo;
    match source.src {
        SourceInfo::Source(ref s) => string_to_parser(h, s.clone()),
        SourceInfo::Binary => None,
    }
}

/* collect */

/// wrapper around [`infer::collect`].
/// see [`cfg::opts::Opt::query_dims`]
pub fn collect_dims<'a, It: Iterator<Item = &'a Terms>>(
    trees: &mut It,
    env: &mut Env,
    declared_dims: &HashMap<Name, Choices>,
) -> Vec<(Name, Choices)> {
    let mut map = HashMap::new();
    for ref terms in trees {
        infer::check_collect(terms, &mut map, env);
    }
    // @NOTE is checking conflict between declared_dims here needed?
    map.into_iter()
        .map(|(k, v)| match declared_dims.get(&k) {
            Some(v) => (k, v.clone()),
            None => (k, Choices::Size(v)),
        })
        .collect()
}

pub fn pp_dim(dim: &Name, ch: &Choices) -> String {
    // @SAFETY write does not fail on `String`
    #![allow(unused_must_use)]
    use std::fmt::Write;
    let mut buf = format!("dim {} = ", dim);
    match ch {
        Choices::Size(n) => write!(buf, "size {}", n),
        Choices::Names(v) => write!(buf, "{:?}", v),
    };
    buf
}

/* output */

/// processes and writes to the destination file.  
/// @TODO we could benefit from [`Write::write_vectored`]  
/// @TODO modify Terms with the decision during typechecking so we don't have to search in env?  
pub fn write(flags: &cfg::Flags, file: SrcFile, terms: &Terms, env: &Env) -> io::Result<()> {
    use crate::sourcemap::SourceInfo;
    use std::io::{BufRead, Cursor};

    trait SrcReader: RelativeSeek + BufRead {}
    impl<T: BufRead + RelativeSeek> SrcReader for T {}

    let mut reader: Box<dyn SrcReader> = if file.is_stdin() {
        let src = match &file.src {
            SourceInfo::Source(s) => Cursor::new(s.as_bytes()),
            SourceInfo::Binary => panic!("cannot read form binary input in <stdin>"),
        };
        Box::new(io::BufReader::new(src))
    } else {
        Box::new(io::BufReader::new(fs::File::open(&file.path)?))
    };
    let dest = &file.destination;
    if !flags.force && file.destination.exists() {
        let msg = format!(
            "error: file `{}` already exists. [use --force to overwrite]",
            file.destination.display()
        );
        return Err(io::Error::new(io::ErrorKind::AlreadyExists, msg));
    }
    let mut out_f : Box<dyn io::Write> = if file.destination == PathBuf::from("<stdout>") {
        Box::new(io::stdout())
    } else {
        Box::new(fs::File::create(dest)?)
    };
    write_terms(terms, &mut reader, &mut out_f, file.start.as_usize(), env)?;
    Ok(())
}

#[doc(inline)]
pub use crate::output::copy_bin;

pub fn clean(paths: Vec<&Path>) {
    for path in paths {
        if path.exists() {
            #[allow(unused_must_use)]
            {
                std::fs::remove_file(path);
            }
        }
    }
}

/* source map */
fn mk_path(prefix: Option<&PathBuf>, path: PathBuf) -> PathBuf {
    if path == PathBuf::from("<stdin>") || path == PathBuf::from("<stdout>") {
        path
    } else if prefix.is_some() {
        prefix.unwrap().join(path)
    } else {
        path
    }
}

/// load all the sources in the source map and returns them in a `Vec`
pub fn load_sources<'a, It: Iterator<Item = (&'a PathBuf, &'a PathBuf)>>(
    flags: &cfg::Flags,
    paths: It,
) -> (Arc<SrcMap>, Vec<SrcFile>) {
    let source_map = SrcMap::new();
    let mut sources = vec![];
    let inp = flags.in_prefix.as_ref();
    let outp = flags.out_prefix.as_ref();

    if flags.stdin.is_some() {
        // @IMPROVEMENT error handling
        match source_map.load_file(
            "<stdin>".into(),
            mk_path(outp, flags.stdin.clone().unwrap()),
        ) {
            Err(e) => emit_error!("couldn't load `{}`:\n {}", "<stdin>", e),
            Ok(f) => sources.push(f.clone()),
        };
    }
    load_files(paths, inp, outp, &source_map, &mut sources);
    (source_map, sources)
}

fn load_files<'a, It: Iterator<Item = (&'a PathBuf, &'a PathBuf)>>(
    paths: It, 
    inp: Option<&PathBuf>, 
    outp: Option<&PathBuf>, 
    source_map: &Arc<SrcMap>, 
    sources: &mut Vec<SrcFile>
) {
    // @FIXME basically if we use a closure in the .map() we hit a recursion limit for instanciation of load_files
    //        another reason to rewrite the whole source loading API.
    fn ref_inner<T,U>(x : &(T, U)) -> (&T, &U) {
        match x {
            (t, u) => (t, u)
        }
    }
    for (src_, dst_) in paths {
        let src = mk_path(inp, src_.clone());
        let dst = mk_path(outp, dst_.clone());
        if src.is_dir() {
            // @IMPROVEMENT ignore sub-files/dirs
            // @FIXME rather ugly to go from It<&(x,y)> to It<(&x, &y)>.
            //        while the representations are obviously completely different
            //        this could probably benefit from some adjusting of the calling/caller types 
            match get_subpaths(src, src_, dst_) {
                Ok(paths) => {
                    let paths = paths.iter().map(ref_inner);
                    load_files(paths, inp, outp, source_map, sources)
                }
                Err(e) => 
                    emit_error!("couldn't load directory `{}`:\n  {}", src_.to_string_lossy(), e),
            }
        } else {
            match source_map.load_file(src, dst) {
                // @IMPROVEMENT error handling
                Err(e) => emit_error!("couldn't load `{}`:\n  {}", src_.to_string_lossy(), e),
                Ok(f) => sources.push(f.clone()),
            }
        }
    }
}

/// Get the path of the contents of a directory and appends the directory's (source and destination) relative path to each entry.
fn get_subpaths(dir: impl AsRef<Path>, src: &PathBuf, dst: &PathBuf) -> io::Result<Vec<(PathBuf, PathBuf)>> {
    dir.as_ref()
       .read_dir()
       .and_then(|rd| 
                   rd.map(|e| {
                            let f = e?.file_name();
                            Ok((src.join(&f), dst.join(f)))
                   }).collect())
}

/* cfg */

/// build a new Config and Flags, from arguments and config file
pub fn mk_cfgflags() -> Result<(cfg::Flags, cfg::Config), cfg::Error> {
    use cfg::StructOpt;
    let opt = cfg::Opt::from_args();
    let file = cfg::path_to_cfgfile(opt.config_file.as_ref())?;
    // @TODO finer grained error reporting. 
    let decisions = opt.parse_decisions()?;
    Ok((
        cfg::Flags::new(&opt, file.options.as_ref()),
        cfg::Config::new(decisions.0, decisions.1, file),
    ))
}
