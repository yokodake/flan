/// Errors for Parsing and Lexing.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub enum Error {
    UnexpectedToken,
    NonTerminatedToken,
    IllegalCharacter,
    UnclosedDelimiter,
    UnexpectedEOF,
    FatalError,
    LexerError,
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
