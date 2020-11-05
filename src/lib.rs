#![feature(
    bufreader_seek_relative,
    new_uninit,
    try_trait,
    type_ascription,
    int_error_matching,
    option_result_contains
)]

pub mod cfg;

pub mod driver;

pub mod error;

pub mod infer;
#[doc(inline)]
pub use infer::env;

pub mod sourcemap;

pub mod syntax;

pub mod utils;
