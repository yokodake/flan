#[allow(dead_code)]
#[derive(Clone, Copy, PartialEq, PartialOrd, Eq, Ord, Debug, Hash)]
pub enum PError {
    UnexpectedToken,
    NonTerminatedToken,
    IllegalCharacter,
    UnclosedDelimiter,
    UnexpectedEOF,
    FatalError,
}
