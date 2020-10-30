#![feature(type_ascription)]
#![feature(option_result_contains)]
use std::collections::HashMap;

use flan::cfg::{Opt, StructOpt};
#[allow(unused_imports)]
use flan::debug;
use flan::infer;

fn main() {
    use flan::driver::*;
    let opt = Opt::from_args();
    let config_file = match get_config(&opt.config_file) {
        Ok(f) => f,
        Err(_) => {
            // @FIXME error handling
            eprintln!("config failure.");
            std::process::abort();
        }
    };

    let (source_map, sources) = load_sources(config_file.paths());

    let mut h = make_handler(&opt, &config_file, source_map.clone());
    let trees = parse_sources(sources, &mut h);

    // @TODO handle errors
    let decisions = opt.parse_decisions().unwrap();
    let mut env = make_env(&config_file, decisions, &mut h).unwrap();

    if opt.query_dims {
        for (_, tree) in &trees {
            let mut h = make_handler(&opt, &config_file, source_map.clone());
            collect_dims(
                tree,
                &mut h,
                config_file.dimensions.as_ref().unwrap_or(&HashMap::new()),
            );
        }
    } else if trees.iter().fold(false, |acc, (_, tree)| {
        infer::check(tree, &mut env).is_none() || acc
    }) {
        h.abort();
    }

    if opt.query_dims || opt.dry_run {
        std::process::exit(SUCCESS)
    }

    // @TODO driver::write_files
    for (source, tree) in trees {
        if write(source, &tree, &env).is_err() {
            std::process::exit(FAILURE);
        }
    }
}

const SUCCESS: i32 = 0;
const FAILURE: i32 = -1;
