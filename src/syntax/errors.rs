/// Errors for Parsing and Lexing.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub enum Error {
    UnexpectedToken,
    NonTerminatedToken,
    IllegalCharacter,
    UnclosedDelimiter,
    UnexpectedEOF,
    FatalError,
}

impl Error {
    /// whether parsing can safely continue or not.
    pub fn is_fatal(&self) -> bool {
        match self {
            // currently only happens in variable names.
            // if they're properly terminated, we can continue parsing
            Error::IllegalCharacter => false,
            _ => true,
        }
    }
}
use crate::error::Pattern;
impl Pattern<Error> for Error {
    fn found(&self, t: &Error) -> bool {
        self == t
    }
}
