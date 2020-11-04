//! Configuration module
//!
//! @TODO: more precise error handling

use std::collections::{HashMap, HashSet};
use std::fmt;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use toml::de;

use crate::error::ErrorFlags; // @TODO move here

pub mod file;
pub mod opts;

pub use file::{Choices, File};
pub use opts::{Decision, Index};
use opts::{Opt, StructOpt};

/// see [`ErrorFlags::report_level`]
pub const VERBOSITY_DEFAULT: u8 = 4;
pub const WARN_DEFAULT: bool = false;
pub const NO_EXTRA_DEFAULT: bool = false;
pub const FORCE_DEFAULT: bool = false;
pub const COMMAND_DEFAULT: Command = Command::Default;

#[derive(Debug, Clone)]
pub struct Config {
    pub variables: HashMap<String, String>,
    pub dimensions: HashMap<String, Choices>,
    pub paths: HashMap<PathBuf, PathBuf>,
    pub decisions_name: HashSet<String>,
    pub decisions_pair: HashMap<String, Index>,
}
impl Config {
    /// @FIXME decisions
    pub fn new(
        decisions_name: HashSet<String>,
        decisions_pair: HashMap<String, Index>,
        file: File,
    ) -> Self {
        let variables = file.variables.unwrap_or(HashMap::new());
        let dimensions = file.dimensions.unwrap_or(HashMap::new());
        let paths = file.paths.unwrap_or(HashMap::new());
        Config {
            variables,
            dimensions,
            paths,
            decisions_name,
            decisions_pair,
        }
    }
}
#[derive(Debug, Hash, PartialEq, Clone)]
pub struct Flags {
    /// see [`ErrorFlags`]
    pub eflags: ErrorFlags,
    /// `--in-prefix`
    pub in_prefix: Option<PathBuf>,
    /// `--out-prefix`
    pub out_prefix: Option<PathBuf>,
    /// `--force`
    pub force: bool,
    /// `--dry-run` or `--query-dimensions`
    pub command: Command,
}

impl Flags {
    /// cmd-line opts take precedence over config file. Otherwise use default values
    pub fn new(opt: &Opt, config: Option<&file::Options>) -> Self {
        let report_level = opt
            .report_level()
            .or(config.and_then(file::Options::verbosity))
            .unwrap_or(VERBOSITY_DEFAULT);

        let eflags = ErrorFlags {
            report_level,
            warn_as_error: opt.warn_error(),
            no_extra: opt.no_extra(),
        };

        let force = opt.force
            || config
                .and_then(file::Options::force)
                .unwrap_or(FORCE_DEFAULT);
        let command = Command::from_opt(&opt);
        Flags {
            eflags,
            in_prefix: opt.in_prefix.clone(),
            out_prefix: opt.out_prefix.clone(),
            force,
            command,
        }
    }
}

#[derive(Debug, Hash, PartialEq, Clone, Copy)]
pub enum Command {
    Default,
    /// `--dry-run`
    DryRun,
    /// `--query-dimensions`
    Query,
}
impl Command {
    pub fn from_opt(opt: &Opt) -> Self {
        if opt.query_dims {
            Command::Query
        } else if opt.dry_run {
            Command::DryRun
        } else {
            Command::Default
        }
    }
}

#[derive(Debug, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub enum ErrorKind {
    OutOfRange,
    InvalidChoice,
    InvalidIdentifier,
}
#[derive(Debug)]
pub enum Error {
    IO(io::Error),
    TOML(de::Error),
    Cfg { msg: String, kind: ErrorKind },
}
impl Error {
    pub fn out_of_range(lexeme: &str) -> Self {
        Error::Cfg {
            kind: ErrorKind::OutOfRange,
            msg: format!("Numeric choice `{}` is out of range.\n note: consulte --help for a more detailed explanation.", lexeme)
        }
    }
    pub fn invalid_choice(lexeme: &str) -> Self {
        Error::Cfg {
            kind: ErrorKind::InvalidChoice,
            msg: format!("`{}` is not a valid choice.\n note: consulte --help for a more detailed explanation.", lexeme),
        }
    }
    pub fn invalid_identifier(lexeme: &str) -> Self {
        Error::Cfg {
            kind: ErrorKind::InvalidIdentifier,
            msg: format!("`{}` is not a valid identifier.\n note: consult --help for a more detailed explanation.", lexeme),
        }
    }
}
impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::IO(e) => e.fmt(f),
            Error::TOML(e) => e.fmt(f),
            Error::Cfg { msg, .. } => write!(f, "{}", msg),
        }
    }
}

/// opens config file and parses it.
/// get the `.flan` file named in the current working directory if path is `None`;
/// or returns [`File::default()`] if `.flan` doesn't exist.
pub fn path_to_cfgfile<P: AsRef<Path>>(config_path: Option<P>) -> Result<File, Error> {
    let default = Path::new(".flan");
    let path = match config_path {
        Some(ref path) => Some(path.as_ref()),
        None => {
            if default.exists() {
                Some(default)
            } else {
                None
            }
        }
    };
    match path {
        Some(path) => file_to_cfgfile(&mut fs::File::open(path).map_err(Error::IO)?),
        None => Ok(File::default()),
    }
}
/// parse config file.
pub fn file_to_cfgfile(file: &mut fs::File) -> Result<File, Error> {
    use std::io::Read;
    let mut buf = String::new();
    file.read_to_string(&mut buf).map_err(Error::IO)?;
    string_to_cfgfile(&buf).map_err(Error::TOML)
}
/// parse config string.
pub fn string_to_cfgfile(s: &String) -> Result<File, de::Error> {
    File::from_str(s.as_ref())
}

pub fn new() -> Result<(Flags, Config), Error> {
    let opt = Opt::from_args();
    let file = path_to_cfgfile(opt.config_file.as_ref())?;
    // @TODO finer grained error reporting. (see previous commit in main.rs)
    let decisions = opt.parse_decisions()?;
    Ok((
        Flags::new(&opt, file.options.as_ref()),
        Config::new(decisions.0, decisions.1, file),
    ))
}
