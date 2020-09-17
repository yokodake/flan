//! Configuration module
//!
pub mod opt_parse;

use serde::Deserialize;
#[derive(Deserialize)]
pub struct Config {
    pub options: Option<Options>,
}

#[derive(Deserialize)]
pub struct Options {
    /// overwrite destination files if they already exist?
    pub force: Option<bool>,
    /// verbosity level. see [`error::ErrorFlags`]
    ///
    /// [`error::ErrorFlags`]: ../error/struct.ErrorFlags.html
    pub verbosity: Option<u8>,
}
