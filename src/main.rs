#![feature(type_ascription)]
#![feature(option_result_contains)]
#![feature(result_flattening)]
#![feature(duration_zero)]
#![feature(format_args_nl)]

use std::sync::Arc;

use flan::{emit_error};
use flan::cfg::Command;
#[allow(unused_imports)]
use flan::error::Handler;
use flan::infer;

fn main() {
    use flan::driver::*;
    let mut metrics = Metrics::new();

    let (flags, config) = match mk_cfgflags() {
        Ok(f) => f,
        Err(e) => {
            // @IMPROVEMENT error handling
            eprintln!("fatal error:");
            eprintln!("{}", e);
            std::process::exit(FAILURE);
        }
    };
    let flags = Arc::new(flags);

    let (source_map, sources) = load_sources(flags.as_ref(), config.paths.iter());
    metrics.total_files(sources.len() as isize);

    let start = Instant::now();
    let mut hp = Handler::new(flags.eflags, source_map.clone());
    if sources.len() == 0 {
        hp.warn("no paths given")
            .note("see `[paths]` section in the configuration file")
            .print();
        std::process::exit(SUCCESS);
    }
    let (trees, bins) = parse_sources(sources, &mut hp);
    metrics.front(start);

    let start = Instant::now();
    let he = Handler::new(flags.eflags, source_map.clone());
    let mut env = match make_env(&config, he) {
        Err(mut he) => he.abort(),
        Ok(e) => e,
    };

    if flags.command == Command::Query {
        let mut h = Handler::new(flags.eflags, source_map.clone());
        for (dim, ch) in collect_dims(&mut trees.iter().map(|t| &t.1), &mut h, &config.dimensions) {
            println!("{}", pp_dim(&dim, &ch));
        }
    } else if trees.iter().fold(false, |acc, (_, tree)| {
        infer::check(tree, &mut env).is_none() || acc
    }) {
        env.handler.abort();
    }
    metrics.infer(start);

    hp.abort_if_err();
    if flags.command == Command::Query || flags.command == Command::Query {
        metrics.report();
        std::process::exit(SUCCESS);
    }

    let start = Instant::now();
    // the most important point about spawning these threads is to capture panics
    // without paying the cost of `catch_unwind`
    // @TODO we need better error reporting inside, because panic! adds useless and
    //       ugly stuff to the error message.
    let flags_ = flags.clone();
    let write_th = std::thread::spawn(move || {
        let mut count = 0;
        // @TODO driver::write_files?
        for (source, tree) in &trees {
            match write(flags_.as_ref(), source.clone(), &tree, &env) {
                Err(e) => panic!("io {}", e),
                Ok(_) => count += 1,
            }
        }
        count
    });
    let flags_ = flags.clone();
    let bin_th = std::thread::spawn(move || {
        let mut count = 0;
        for bin in bins {
            match copy_bin(flags_.as_ref(), bin.clone()) {
                Err(e) => panic!("io {}", e),
                Ok(_) => count += 1,
            }
        }
        count
    });
    match write_th.join() {
        Err(_) => {
            emit_error!("@TODO: cleanup resources");
            metrics.processed(-1)
        }
        Ok(n) => metrics.processed(n),
    }
    match bin_th.join() {
        Err(_) => {
            emit_error!("@TODO: cleanup resources");
            metrics.copied(-1)
        }
        Ok(n) => metrics.copied(n),
    }
    metrics.end(start);
    if !flags.stdin.is_some() {
        metrics.report();
    }
}

use std::time::{Duration, Instant};
struct Metrics {
    /// processed file count
    pub proc_f: isize,
    /// copied file count
    pub copy_f: isize,
    /// total file count (in paths)
    pub total_f: isize,

    /// start time of the program
    start: Instant,
    /// frontend duration
    pub front: Duration,
    /// typechecking/inference duration
    pub infer: Duration,
    /// backend duration
    pub end: Duration,

    /// total time
    pub total: Duration,
}
impl Metrics {
    pub fn new() -> Self {
        Self {
            proc_f: -1,
            copy_f: -1,
            total_f: 0,
            start: Instant::now(),
            front: Duration::ZERO,
            infer: Duration::ZERO,
            end:   Duration::ZERO,
            total: Duration::ZERO,
        }
    }
    pub fn total_files(&mut self, total_files: isize) {
        self.total_f = total_files;
    }
    pub fn processed(&mut self, processed: isize) {
        self.proc_f = processed
    }
    pub fn copied(&mut self, copied: isize) {
        self.copy_f = copied;
    }
    pub fn front(&mut self, start: Instant) {
        self.front = start.elapsed();
    }
    pub fn infer(&mut self, start: Instant) {
        self.infer = start.elapsed();
    }
    pub fn end(&mut self, start: Instant) {
        self.end = start.elapsed();
    }
    pub fn report(&mut self) {
        self.total = self.start.elapsed();
        println!("\n");
        self.report_files();
        self.report_time();
    }
    pub fn report_files(&self) {
        let any = self.proc_f >= 0 || self.copy_f >= 0;
        if self.total_f >= 0 {
            print!("{}", self.total_f);
            if any {
                let p = isize::max(self.proc_f, 0);
                let b = isize::max(self.copy_f, 0);
                print!("[{}+{}]", p, b);
            }
            println!(" file{}.", if self.total_f > 1 { "s" } else { "" });
        }
    }
    pub fn report_time(&self) {
        println!("Total time: {}ms.", self.total.as_millis());
        if !self.front.is_zero() {
            println!(" ` front:  {}ms", self.front.as_millis());
        }
        if !self.infer.is_zero() {
            println!(" ` infer:  {}ms", self.infer.as_millis());
        }
        if !self.end.is_zero() {
            println!(" ` output: {}ms", self.end.as_millis());
        }
    }
}

const SUCCESS: i32 = 0;
const FAILURE: i32 = 0x100;
