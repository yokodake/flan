pub mod errors;
pub mod lexer;
pub mod parser;
// pub use lexer::{Lexer, Token, TokenK};

#[doc(inline)]
pub use errors::Error;
#[doc(inline)]
pub use lexer::Lexer;
#[doc(inline)]
pub use parser::{Name, Term, TermK, Terms};
#[doc(inline)]
pub use parser::{Parsed, Parser, TokenStream};

pub use crate::sourcemap::Spanned;
