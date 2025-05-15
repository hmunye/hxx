use std::process;
use std::sync::Mutex;

pub static BYTES_PER_LINE: Mutex<usize> = Mutex::new(16);
pub static BYTE_GROUPING: Mutex<usize> = Mutex::new(2);

pub struct Flag {
    pub name: &'static str,
    pub description: &'static str,
    pub run: fn(&str, Option<usize>),
}

pub const FLAG_REGISTRY: &[Flag] = &[
    Flag {
        name: "-c",
        description: "cols      format <cols> octets per line (must be > 0). Default 16.",
        run: set_bytes_per_line,
    },
    Flag {
        name: "-g",
        description: "bytes     number of octets per group in normal output (must be > 0). Default 2.",
        run: set_byte_grouping,
    },
    Flag {
        name: "-h",
        description: "          print this summary.",
        run: print_usage,
    },
    Flag {
        name: "-v",
        description: "          show version.",
        run: print_version,
    },
];

pub fn print_usage(program: &str, _value: Option<usize>) {
    println!("Usage:");
    println!("      {program} [options] [infile [outfile]]");
    println!("  or");
    println!("      {program} -r [-c cols] [infile [outfile]]");
    println!("Options:");

    for flag in FLAG_REGISTRY {
        println!("  {}  {}", flag.name, flag.description);
    }

    process::exit(1);
}

pub fn print_version(program: &str, _value: Option<usize>) {
    println!("{} - {}", program, env!("CARGO_PKG_VERSION"));
    process::exit(0);
}

pub fn set_bytes_per_line(_program: &str, value: Option<usize>) {
    let mut bpl = BYTES_PER_LINE.lock().unwrap();

    // Safe to unwrap because parsed first
    *bpl = value.unwrap();
}

pub fn set_byte_grouping(_program: &str, value: Option<usize>) {
    let mut bg = BYTE_GROUPING.lock().unwrap();

    // Safe to unwrap because parsed first
    *bg = value.unwrap();
}
