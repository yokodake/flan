//! Command line parsing helpers
use std::collections::{HashMap, HashSet};
use std::io;
use std::path::PathBuf;

pub use structopt::StructOpt;

use crate::error::ErrorFlags;

/// command line passed Decision
pub enum OptDec {
    /// by name
    Name(String),
    /// (dimension name, decision index or name) pair.
    WithDim(String, Index),
}
impl OptDec {
    /// parse one decision
    pub fn parse_decision(str: &String) -> io::Result<Self> {
        let mut it = str.splitn(2, '=');
        // splitn will give us at the very least "" as first elem
        let k = it.next().unwrap();
        let i = it.next();
        match i {
            Some(s) => Self::parse_dim(k, s),
            None => Self::parse_name(k),
        }
    }
    /// [`OptDec::WithDim`]
    fn parse_dim(k: &str, i: &str) -> io::Result<Self> {
        Self::validate_id(k)?;
        let idx = Self::parse_idx(i)?;
        Ok(Self::WithDim(k.into(), idx))
    }
    /// variable name
    fn parse_name(n: &str) -> io::Result<Self> {
        Self::validate_id(n)?;
        Ok(Self::Name(n.into()))
    }
    /// [`Index`]
    fn parse_idx(s: &str) -> io::Result<Index> {
        use std::io::{Error, ErrorKind};
        use std::num::IntErrorKind;
        return match s.parse() {
            Ok(i) if i < 128 => Ok(Index::Num(i)),
            Err(e) if  *e.kind() != IntErrorKind::Overflow => {
                if Self::validate_id(s).is_ok() {
                    Ok(Index::Name(s.into()))
                } else {
                    Err(Error::new(ErrorKind::InvalidInput, format!("`{}` is not a valid choice.\n note: consulte --help for a more detailed explanation.", s)))
                }
            }
            _ => Err(Error::new(ErrorKind::InvalidInput, format!("Numeric choice `{}` is out of range.\n note: consulte --help for a more detailed explanation.", s)))
        };
    }
    fn validate_id(s: &str) -> io::Result<()> {
        use std::io::{Error, ErrorKind};

        if s.len() > 0
            && (|c: char| c.is_alphabetic() || c == '_')(s.chars().next().unwrap())
            && !s.contains(|c: char| !c.is_alphanumeric() && c != '_')
        {
            Ok(())
        } else {
            Err(Error::new(ErrorKind::InvalidInput, format!("`{}` is not a valid identifier.\n note: consult --help for a more detailed explanation.", s)))
        }
    }
}
/// Decision for an explicitly named dimension
#[derive(Debug, Clone, PartialEq, Hash)]
pub enum Index {
    /// by name
    Name(String),
    /// by index
    Num(u8),
}

impl std::fmt::Display for Index {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Index::Name(s) => write!(f, "{}", s),
            Index::Num(n) => write!(f, "{}", n),
        }
    }
}

#[derive(StructOpt, Clone, PartialEq, Eq, Debug)]
#[structopt(version = "0.1", rename_all = "kebab-case")]
pub struct Opt {
    #[structopt(long)]
    /// overwrite existing destination files
    pub force: bool,
    #[structopt(long)]
    /// run without substituting the files.
    pub dry_run: bool,
    #[structopt(long)]
    /// ignore all warnings
    pub no_warn: bool,
    #[structopt(short = "z", long)]
    /// silence all errors and warnings
    pub silence: bool,
    #[structopt(short, long)]
    /// explain what is being done
    pub verbose: bool,
    #[structopt(long = "Werror")]
    /// make all warnings into errors (@TODO: handle this in handler)
    pub warn_error: bool,
    #[structopt(short = "q", long = "query-dimensions")]
    /// list all dimensions (@TODO: that require a decision).
    pub query_dims: bool,
    #[structopt(name = "PATH", short = "c", long = "config")]
    /// use this config file instead
    pub config_file: Option<PathBuf>,
    #[structopt(name = "OUTPUT", short = "o", long = "output", parse(from_os_str))]
    /// destination file
    pub file_out: Option<PathBuf>,
    #[structopt(name = "INPUT")]
    /// source file
    pub file_in: PathBuf,
    #[structopt(name = "DECISIONS")]
    /// Can be Choice or Dimension_name=Index pairs. An Index is either a
    /// a choice name or a natural smaller than 128. Valid names contain `_` or alphanumeric chars but
    /// cannot start with a digit
    pub decisions: Vec<String>,
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

pub const DEFAULT_VERBOSITY: u8 = 3;
