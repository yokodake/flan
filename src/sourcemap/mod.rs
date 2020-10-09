//! custom version of [https://docs.rs/codemap/](https://docs.rs/codemap/).
pub mod loc;
pub mod source_analysis;
pub mod sourcemap;
pub mod span;

#[doc(inline)]
pub use loc::Loc;
#[doc(inline)]
pub use sourcemap::{File, SourceInfo, SrcFile, SrcMap};
#[doc(inline)]
pub use span::{span, Pos, PosInner, Span, Spanned};
