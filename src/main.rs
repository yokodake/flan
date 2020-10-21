#![feature(type_ascription)]
#![feature(option_result_contains)]
use std::collections::{HashMap, HashSet};
use std::io;
use std::path::{Path, PathBuf};

use structopt::StructOpt;

use flan::cfg;
use flan::cfg::{path_to_cfg, Config, DEFAULT_VERBOSITY};
#[allow(unused_imports)]
use flan::debug;
use flan::error::{ErrorFlags, Handler};
use flan::infer;
use flan::opt_parse::{Index, OptDec};
use flan::sourcemap::SrcMap;

fn main() {
    use flan::driver::*;
    let opt = Opt::from_args();
    let config_file = match opt.config_file {
        Some(ref path) => path_to_cfg(path.clone()),
        None => {
            if Path::new(".flan").exists() {
                path_to_cfg(".flan")
            } else {
                Ok(Config::default())
            }
        }
    };
    let config_file = match config_file {
        Ok(f) => f,
        Err(_) => {
            // @FIXME error handling
            eprintln!("config failure.");
            std::process::abort();
        }
    };
    let eflags = override_flags(opt.error_flags(), config_file.options.as_ref());
    let source_map = SrcMap::new();
    let mut sources = vec![];
    for (src, dst) in config_file.paths() {
        if src.is_dir() {
            // @TODO traverse and collect files
        } else {
            match source_map.load_file(src, dst) {
                // @FIXME error handling
                Err(_) => eprintln!("couldn't load {}", src.to_string_lossy()),
                Ok(f) => sources.push(f.clone()),
            }
        }
    }
    let mut trees = vec![];
    for f in sources {
        let mut h = Handler::new(eflags, source_map.clone());
        // @FIXME error handling
        match file_to_parser(&mut h, f.clone()) {
            Ok(mut p) => match p.parse() {
                Ok(tree) => {
                    trees.push(tree);
                }
                Err(_) => {
                    h.print_all();
                }
            },
            Err(_) => {
                h.print_all();
                eprintln!("failed to parse {}", f.path.to_string_lossy());
                continue;
            }
        }
    }
    let mut h = Handler::new(eflags, source_map.clone());
    let variables = config_file.variables().collect();
    let decl_dims = config_file.dimensions().collect();
    // @TODO handle errors
    let decisions = opt.parse_decisions().unwrap();
    let mut env = make_env(variables, decl_dims, decisions, &mut h).unwrap();

    // @TODO handle collect
    for tree in trees.iter() {
        // @TODO handle errors
        infer::check(&tree, &mut env);
    }

    // @TODO driver::write_files
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
    #[structopt(long = "Werror")]
    /// make all warnings into errors (@TODO: handle this in handler)
    warn_error: bool,
    #[structopt(short = "q", long = "query-dimensions")]
    /// list all dimensions (@TODO: that require a decision).
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
    pub fn error_flags(&self) -> ErrorFlags {
        let mut report_level = DEFAULT_VERBOSITY;
        if self.verbose {
            report_level = 5;
        }
        if self.no_warn {
            report_level = 2;
        }
        if self.silence {
            report_level = 0;
        }
        let mut no_extra = false;
        if self.silence {
            no_extra = true;
        }
        ErrorFlags {
            report_level,
            no_extra,
            warn_as_error: self.warn_error,
            dry_run: self.dry_run,
        }
    }
}

pub fn override_flags(flags: ErrorFlags, config: Option<&cfg::Options>) -> ErrorFlags {
    let mut flags = flags;
    if config.is_none() {
        return flags;
    }
    let config = config.unwrap();

    flags.report_level = config.verbosity.unwrap_or(flags.report_level);
    flags
}
