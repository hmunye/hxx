use std::env;
use std::process;

use hxx::{Config, hex_dump, print_usage};

fn main() {
    let mut args = env::args();
    let program = args.next().unwrap_or_else(|| {
        eprintln!("\x1b[1;91mError: invalid or missing program\x1b[0m");
        process::exit(1);
    });

    let config = Config::build(args, &program).unwrap_or_else(|err| {
        eprintln!("\x1b[1;91mERROR: {err}\x1b[0m");
        // Will terminate program with exit code (1)
        print_usage(&program);
        unreachable!();
    });

    if let Err(err) = hex_dump(config) {
        eprintln!("\x1b[1;91mERROR: {err}\x1b[0m");
        process::exit(1);
    }
}
