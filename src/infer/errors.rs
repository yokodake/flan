//! error types for inference/type checking

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub enum Error {
    DimensionMismatch,
    UnknownDimension,
    UnknownVariable,
    UnknownDecision,
}
