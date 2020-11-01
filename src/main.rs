#![feature(type_ascription)]
#![feature(option_result_contains)]
#![feature(result_flattening)]
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
        Err(e) => {
            // @FIXME error handling
            eprintln!("fatal error: failed to read config file.");
            eprintln!("{}", e);
            std::process::exit(FAILURE);
        }
    };
    let decisions = match opt.parse_decisions() {
        Ok(decisions) => decisions,
        Err(err) => {
            eprintln!("fatal error: arguments error: {}", err);
            std::process::exit(FAILURE)
        }
    };

    let (source_map, sources) = load_sources(config_file.paths());

    let mut hp = make_handler(&opt, &config_file, source_map.clone());
    let trees = parse_sources(sources, &mut hp);

    // @TODO handle errors
    let mut he = make_handler(&opt, &config_file, source_map.clone());
    let mut env = make_env(&config_file, decisions, &mut he).unwrap();

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
        he.abort();
    }

    hp.abort_if_err();
    if opt.query_dims || opt.dry_run {
        std::process::exit(SUCCESS)
    }

    let mut dests = vec![];
    // @FIXME binary files aren't copied
    // @TODO driver::write_files
    for (source, tree) in &trees {
        dests.push(source.destination.as_path());

        let r: Result<(), _> = std::panic::catch_unwind(|| {
            write(source.clone(), &tree, &env)
                .map_err(|_| Box::new(()) as Box<dyn std::any::Any + Send>)
        })
        .flatten();

        if r.is_err() {
            eprintln!("Failed to write. Cleanup.");
            cleanup(dests);
            std::process::exit(FAILURE);
        }
    }
}

const SUCCESS: i32 = 0;
const FAILURE: i32 = 0x100;
