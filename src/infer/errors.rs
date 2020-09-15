#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub enum Error {
    DimensionMismatch,
    UnknownDimension,
    UnknownVariable,
    UnknownDecision,
}

use crate::error::Pattern;
impl Pattern<Error> for Error {
    fn found(&self, e: &Error) -> bool {
        self == e
    }
}
