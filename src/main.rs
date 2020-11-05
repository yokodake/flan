#![feature(type_ascription)]
#![feature(option_result_contains)]
#![feature(result_flattening)]

use flan::cfg::Command;
#[allow(unused_imports)]
use flan::debug;
use flan::error::Handler;
use flan::infer;

fn main() {
    use flan::driver::*;
    let (flags, config) = match make_cfgflags() {
        Ok(f) => f,
        Err(e) => {
            // @FIXME error handling
            eprintln!("fatal error:");
            eprintln!("{}", e);
            std::process::exit(FAILURE);
        }
    };

    let (source_map, sources) = load_sources(config.paths.iter());

    let mut hp = Handler::new(flags.eflags, source_map.clone());
    let trees = parse_sources(sources, &mut hp);

    // @TODO handle errors
    let mut he = Handler::new(flags.eflags, source_map.clone());
    let mut env = make_env(&config, &mut he).unwrap();

    if flags.command == Command::Query {
        for (_, tree) in &trees {
            let mut h = Handler::new(flags.eflags, source_map.clone());
            collect_dims(tree, &mut h, &config.dimensions);
        }
    } else if trees.iter().fold(false, |acc, (_, tree)| {
        infer::check(tree, &mut env).is_none() || acc
    }) {
        he.abort();
    }

    hp.abort_if_err();
    if flags.command == Command::Query || flags.command == Command::Query {
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
