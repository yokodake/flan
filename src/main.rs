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
            // @IMPROVEMENT error handling
            eprintln!("fatal error:");
            eprintln!("{}", e);
            std::process::exit(FAILURE);
        }
    };

    let (source_map, sources) = load_sources(&flags, config.paths.iter());

    let mut hp = Handler::new(flags.eflags, source_map.clone());
    let (trees, bins) = parse_sources(sources, &mut hp);

    // @TODO handle errors
    let he = Handler::new(flags.eflags, source_map.clone());
    let mut env = make_env(&config, he).unwrap();

    if flags.command == Command::Query {
        for (_, tree) in &trees {
            let mut h = Handler::new(flags.eflags, source_map.clone());
            collect_dims(tree, &mut h, &config.dimensions);
        }
    } else if trees.iter().fold(false, |acc, (_, tree)| {
        infer::check(tree, &mut env).is_none() || acc
    }) {
        env.handler.abort();
    }

    hp.abort_if_err();
    if flags.command == Command::Query || flags.command == Command::Query {
        std::process::exit(SUCCESS)
    }

    let flags_write = flags.clone();

    // the most important point about spawning these threads is to capture panics
    // without paying the cost of `catch_unwind`
    let write_t = std::thread::spawn(move ||
    // @TODO driver::write_files
    for (source, tree) in &trees {
        match write(&flags_write, source.clone(), &tree, &env) {
            Err(e) => panic!("io {}", e),
            Ok(_) => {}
        }
    });
    let bin_t = std::thread::spawn(move || {
        for bin in bins {
            match copy_bin(&flags, bin.clone()) {
                Err(e) => panic!("io {}", e),
                Ok(_) => {}
            }
        }
    });
    match write_t.join() {
        Err(_) => eprint!("@TODO: cleanup resources"),
        Ok(_) => {}
    }
    match bin_t.join() {
        Err(_) => eprintln!("@TODO: cleanup resources"),
        Ok(_) => {}
    }
}

const SUCCESS: i32 = 0;
const FAILURE: i32 = 0x100;
