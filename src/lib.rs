#![feature( format_args_nl
          , new_uninit
          , option_result_contains
          , type_ascription
          )]

#[macro_use]
pub mod utils;

#[macro_use]
pub mod error;

pub mod cfg;

pub mod driver;

pub mod infer;
#[doc(inline)]
pub use infer::env;

pub mod output;

pub mod sourcemap;

pub mod syntax;
