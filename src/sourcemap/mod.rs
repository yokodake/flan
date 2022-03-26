//! custom version of [https://docs.rs/codemap/](https://docs.rs/codemap/).
pub mod loc;
pub mod source_analysis;
pub mod sourcemap;
pub mod span;
pub mod pos;

#[doc(inline)]
pub use loc::Loc;
#[doc(inline)]
pub use sourcemap::{File, SourceInfo, SrcFile, SrcMap};
#[doc(inline)]
pub use span::{span, BytePos, Span, Spanned};
