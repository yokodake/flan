#![feature(type_ascription)]
#![feature(option_result_contains)]
use std::collections::{HashMap, HashSet};
use std::io;
use std::path::PathBuf;

use structopt::StructOpt;

use flan::cfg::Config;
use flan::debug;
use flan::opt_parse::{Index, OptDec};

fn main() {
    let opt = Opt::from_args();
    // println!("{:?}\n", opt);
    dummy(&opt);
}

fn dummy(opt: &Opt) {
    use flan::driver::{file_to_parser, make_env};
    use flan::error::{ErrorFlags, Handler};
    use flan::infer;
    use flan::sourcemap::SrcMap;

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
    let flags = ErrorFlags {
        no_extra: false,
        report_level: 5,
        warn_as_error: false,
        dry_run: false,
    };
    let map = SrcMap::new();
    let mut hp = Handler::new(flags, map.clone());
    let mut env: infer::Env = match make_env(declared_vars, declared_dims, (n, ni)) {
        Some(e) => e,
        None => {
            eprintln!("Could not make environment");
            hp.abort()
        }
    };

    match map.load_file(&opt.file_in, &"".into()) {
        Err(e) => {
            hp.print_all();
            eprintln!("{}", e);
            hp.abort();
        }
        Ok(f) => match file_to_parser(&mut hp, f.clone()) {
            Err(_) => {
                hp.abort();
            }
            Ok(mut p) => {
                match p.parse().map(|tree| infer::check(&tree, &mut env, &mut hp)) {
                    Err(_) => {
                        hp.abort();
                    }
                    Ok(None) => {
                        eprintln!("Type Checking failure.");
                        hp.abort();
                    }
                    Ok(Some(_)) => println!("success."),
                };
            }
        },
    };
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
    #[structopt(short = "q", long = "query-dimensions")]
    /// list all dimensions (TODO: that require a decision).
    query_dims: bool,
    #[structopt(name = "PATH", short = "c", long = "config")]
    /// use this config file instead
    config_file: Option<PathBuf>,
    #[structopt(name = "OUTPUT", short = "o", long = "output", parse(from_os_str))]
    /// destination file
    file_out: Option<PathBuf>,
    #[structopt(name = "INPUT")]
    /// source file
    file_in: PathBuf,
    #[structopt(name = "DECISIONS")]
    /// Can be Choice or Dimension_name=Index pairs. An Index is either a
    /// a choice name or a natural smaller than 128. Valid names contain `_` or alphanumeric chars but
    /// cannot start with a digit
    decisions: Vec<String>,
}
impl Opt {
    pub fn parse_decisions(&self) -> io::Result<(HashSet<String>, HashMap<String, Index>)> {
        let mut nc = HashSet::new();
        let mut dc = HashMap::new();
        for s in &self.decisions {
            match OptDec::parse_decision(s)? {
                OptDec::Name(s) => {
                    nc.insert(s);
                }
                OptDec::WithDim(dname, idx) => {
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
