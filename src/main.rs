use std::collections::HashMap;
use std::env;
use std::io;
mod syntax;

fn main() {
    let args: Vec<String> = env::args().collect();
    parse_args(args);
}

fn parse_args(args: Vec<String>) -> DynFlags {
    let mut h = HashMap::new();
    if args.len() > 4 {
        h.insert(args[3].clone(), args[4].clone());
    }
    DynFlags {
        in_fn: args[1].clone(),
        out_fn: args[2].clone(),
        vars: h,
    }
}

#[allow(dead_code)]
struct DynFlags {
    in_fn: String,
    out_fn: String,
    vars: HashMap<syntax::Name, String>,
}

#[allow(dead_code)]
fn process_file(fl: DynFlags) -> io::Result<()> {
    use std::fs::File;
    use std::io::{BufRead, BufReader};
    let mut b = File::open(fl.in_fn).and_then(|f| Ok(BufReader::new(f)))?;
    let mut r: Vec<u8> = Vec::new();
    b.read_until(b'&', &mut r);
    Ok(())
}

fn parse_file(_buf: &std::io::BufReader<std::fs::File>) -> syntax::Terms {
    use syntax::Terms;

    Vec::new()
}
