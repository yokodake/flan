//! Command line parsing helpers
use std::io;

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
    fn parse_dim(k: &str, i: &str) -> io::Result<Self> {
        Self::validate_id(k)?;
        let idx = Self::parse_idx(i)?;
        Ok(Self::WithDim(k.into(), idx))
    }
    fn parse_name(n: &str) -> io::Result<Self> {
        Self::validate_id(n)?;
        Ok(Self::Name(n.into()))
    }
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
