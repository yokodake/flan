#![feature( bufreader_seek_relative
          , new_uninit
          , try_trait
          , type_ascription
          , int_error_matching
          , option_result_contains
          , format_args_nl
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
