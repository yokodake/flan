use std::env;
use std::io;
use std::path::PathBuf;

use structopt::StructOpt;

// use flan::syntax;

fn main() {
    let opt = Opt::from_args();
    println!("{:?}", opt);
}

#[derive(StructOpt, Clone, PartialEq, Eq, Debug)]
#[structopt(version = "0.1", rename_all = "kebab-case")]
struct Opt {
    #[structopt(long)]
    /// overwrite existing destination files
    force: bool,
    #[structopt(long)]
    /// run without substituting the files.
    dry_run: bool,
    #[structopt(long)]
    /// ignore all warnings
    no_warn: bool,
    #[structopt(short = "z", long)]
    /// silence all errors and warnings
    silence: bool,
    #[structopt(short, long)]
    /// explain what is being done
    verbose: bool,
    #[structopt(name = "PATH", short = "c", long = "config")]
    /// use this config file instead
    config_file: Option<PathBuf>,
    #[structopt(name = "OUTPUT", short = "o", long = "output", parse(from_os_str))]
    /// destination file
    file_out: Option<PathBuf>,
    #[structopt(name = "INPUT")]
    /// source file
    file_in: PathBuf,
}
