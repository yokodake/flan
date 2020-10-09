use super::span::Span;
pub use std::borrow::Cow;

/// a Line of code from a source file
pub struct Loc<'a> {
    /// index in the `File::lines`. Not a line number
    pub index: usize,
    /// span of the line relative to source file, **not source map**
    pub span: Span,
    /// contents of the line
    pub line: Cow<'a, str>,
}
