//! error types for inference/type checking

/// @TODO
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub enum Error {
    DimensionMismatch,
    UnknownDimension,
    UnknownVariable,
    UnknownDecision,
}
