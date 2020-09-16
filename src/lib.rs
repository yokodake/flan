#![feature(option_result_contains)]
#![feature(int_error_matching)]
#![feature(type_ascription)]
#![feature(try_trait)]

pub mod cfg;
pub use cfg::opt_parse;

pub mod driver;

pub mod error;

pub mod infer;
pub use infer::env;

pub mod syntax;

pub mod utils;
pub use utils::codemap;
