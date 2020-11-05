//! command line options
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

use super::Error;
pub use structopt::StructOpt;

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
    /// make all warnings into errors
    pub warn_error: bool,
    #[structopt(long = "ignore-unset")]
    /// ignore unset variables: don't fail.
    pub ignore_unset: bool,
    #[structopt(short = "q", long = "query-dimensions")]
    /// list all dimensions
    pub query_dims: bool,
    #[structopt(name = "PATH", short = "c", long = "config")]
    /// use this config file instead
    pub config_file: Option<PathBuf>,
    #[structopt(name = "OUTPATH", short = "o", long = "out-prefix", parse(from_os_str))]
    /// destination path
    pub out_prefix: Option<PathBuf>,
    #[structopt(name = "INPATH", short = "i", long = "in-prefix", parse(from_os_str))]
    /// source path
    pub in_prefix: Option<PathBuf>,
    #[structopt(name = "DECISIONS")]
    /// Can be Choice or Dimension_name=Index pairs. An Index is either a
    /// a choice name or a natural smaller than 128. Valid names contain `_` or alphanumeric chars but
    /// cannot start with a digit
    pub decisions: Vec<String>,
}
impl Opt {
    pub fn parse_decisions(&self) -> Result<(HashSet<String>, HashMap<String, Index>), Error> {
        let mut nc = HashSet::new();
        let mut dc = HashMap::new();
        for s in &self.decisions {
            match Decision::from_str(s)? {
                Decision::Name(s) => {
                    nc.insert(s);
                }
                Decision::WithDim(dname, idx) => {
                    dc.insert(dname, idx);
                }
            }
        }
        Ok((nc, dc))
    }
    pub fn report_level(&self) -> Option<u8> {
        let mut report_level: Option<u8> = None;
        if self.verbose {
            report_level = Some(5);
        }
        if self.no_warn {
            report_level = Some(2);
        }
        if self.silence {
            report_level = Some(0);
        }
        report_level
    }
    pub fn no_extra(&self) -> bool {
        self.silence
    }
    pub fn dry_run(&self) -> bool {
        self.dry_run
    }
    pub fn warn_error(&self) -> bool {
        self.warn_error
    }
}

#[derive(Hash, Debug, PartialEq, Clone)]
/// command line passed Decision
pub enum Decision {
    /// by name
    Name(String),
    /// (dimension name, decision index or name) pair.
    WithDim(String, Index),
}
impl Decision {
    /// parse one decision
    pub fn from_str<Str: AsRef<str>>(str: &Str) -> Result<Self, Error> {
        let mut it = str.as_ref().splitn(2, '=');
        // splitn will give us at the very least "" as first elem
        let k = it.next().unwrap().trim();
        let i = it.next();
        match i {
            Some(s) => Self::parse_dim(k, s.trim()),
            None => Self::parse_name(k),
        }
    }
    /// [`Decision::WithDim`]
    fn parse_dim(k: &str, i: &str) -> Result<Self, Error> {
        Self::validate_id(k)?;
        let idx = Self::parse_idx(i)?;
        Ok(Self::WithDim(k.into(), idx))
    }
    /// [`Decision::Name`]
    fn parse_name(n: &str) -> Result<Self, Error> {
        Self::validate_id(n)?;
        Ok(Self::Name(n.into()))
    }
    /// [`Index`]
    fn parse_idx(s: &str) -> Result<Index, Error> {
        use std::num::IntErrorKind;
        return match s.parse() {
            Ok(i) if i < 128 => Ok(Index::Num(i)),
            Err(e) if *e.kind() == IntErrorKind::Overflow => Err(Error::out_of_range(s)),
            _ => {
                if Self::validate_id(s).is_ok() {
                    Ok(Index::Name(s.into()))
                } else {
                    Err(Error::invalid_choice(s))
                }
            }
        };
    }
    fn validate_id(s: &str) -> Result<(), Error> {
        if s.len() > 0
            && (|c: char| c.is_alphabetic() || c == '_')(s.chars().next().unwrap())
            && !s.contains(|c: char| !c.is_alphanumeric() && c != '_')
        {
            Ok(())
        } else {
            Err(Error::invalid_identifier(s))
        }
    }
}

/// decision for an explicitly named dimension
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
