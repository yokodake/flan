#![feature(option_result_contains)]
#![feature(int_error_matching)]
#![feature(type_ascription)]
#![feature(try_trait)]

pub mod cfg;
#[doc(inline)]
pub use cfg::opt_parse;

pub mod driver;

pub mod error;

pub mod infer;
#[doc(inline)]
pub use infer::env;

pub mod syntax;

pub mod utils;
#[doc(inline)]
pub use utils::codemap;
