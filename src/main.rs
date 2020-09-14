#![feature(type_ascription)]
#![feature(option_result_contains)]
use std::collections::{HashMap, HashSet};
use std::io;
use std::iter::FromIterator;
use std::path::PathBuf;

use structopt::StructOpt;

use flan::env::{Dim, Env};
use flan::opt_parse::{Index, OptCh};

fn main() {
    let opt = Opt::from_args();
    // println!("{:?}\n", opt);
    dummy(&opt);
}

fn dummy(opt: &Opt) {
    let (n, ni);
    match opt.parse_decisions() {
        Ok((x, y)) => {
            n = x;
            ni = y;
        }
        Err(e) => return println!("{}", e.to_string()),
    }
    let declared_dims: Vec<(String, Vec<String>)> = vec![
        ("dim1".into(), vec!["opt11".into(), "opt12".into()]),
        (
            "dim2".into(),
            vec!["opt21".into(), "opt22".into(), "opt23".into()],
        ),
    ];
    let declared_vars: Vec<(String, String)> = vec![
        ("foo".into(), "foo_val".into()),
        ("bar/baz".into(), "bar/baz_val".into()),
    ];
    let env = make_env(declared_vars, declared_dims, (n, ni));
}
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
                // @TODO use handler for verbosity level.
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
        println!("{}", e);
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

#[derive(StructOpt, Clone, PartialEq, Eq, Debug)]
#[structopt(version = "0.1", rename_all = "kebab-case")]
struct Opt {
    #[structopt(long)]
    /// overwrite existing destination files
    force: bool,
    #[structopt(long)]
    /// run without substituting the files.
    dry_run: bool,
    #[structopt(long)]
    /// ignore all warnings
    no_warn: bool,
    #[structopt(short = "z", long)]
    /// silence all errors and warnings
    silence: bool,
    #[structopt(short, long)]
    /// explain what is being done
    verbose: bool,
    #[structopt(name = "PATH", short = "c", long = "config")]
    /// use this config file instead
    config_file: Option<PathBuf>,
    #[structopt(name = "OUTPUT", short = "o", long = "output", parse(from_os_str))]
    /// destination file
    file_out: Option<PathBuf>,
    #[structopt(name = "INPUT")]
    /// source file
    file_in: PathBuf,
    #[structopt(name = "CHOICES")]
    /// Can be choice_names or Dimension_name=Index pairs. An Index is either a
    /// a choice name or a natural smaller than 128. Valid names contain `_` or alphanumeric chars but
    /// cannot start with a digit
    choices: Vec<String>,
}
impl Opt {
    pub fn parse_decisions(&self) -> io::Result<(HashSet<String>, HashMap<String, Index>)> {
        let mut nc = HashSet::new();
        let mut dc = HashMap::new();
        for s in &self.choices {
            match OptCh::parse_decision(s)? {
                OptCh::Name(s) => {
                    nc.insert(s);
                }
                OptCh::KV(dname, idx) => {
                    dc.insert(dname, idx);
                }
            }
        }
        Ok((nc, dc))
    }
}

struct PrettyDim {
    name: String,
    choices: Option<Vec<String>>,
    size: u8,
}

impl PrettyDim {
    pub fn new(name: String, size: u8) -> Self {
        PrettyDim {
            name,
            choices: None,
            size,
        }
    }
    pub fn new_choices(name: String, size: u8, choices: Vec<String>) -> Self {
        PrettyDim {
            name,
            choices: Some(choices),
            size,
        }
    }
}

impl std::fmt::Display for PrettyDim {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "dim #{}{{", self.name)?;
        if self.choices.is_some() {
            let mut it = self.choices.as_ref().unwrap().iter();
            match it.next() {
                Some(i) => write!(f, " {} ", i)?,
                None => return write!(f, " "),
            }
            for i in it {
                write!(f, "## {} ", i)?;
            }
        } else {
            write!(f, " {} ", self.size)?;
        }
        write!(f, "}}#")
    }
}
