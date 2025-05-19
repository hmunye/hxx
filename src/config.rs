use std::fs::File;
use std::io::{self, Read, Write};
use std::process;

pub struct Config {
    pub cols: usize,
    pub byte_groups: usize,
    pub input: Box<dyn Read>,
    pub output: Box<dyn Write>,
}

impl Config {
    pub fn build<T: Iterator<Item = String>>(args: T, program: &str) -> Result<Self, String> {
        let mut cols: usize = 16;
        let mut byte_groups: usize = 2;

        let mut args = args.peekable();

        // Peekable allows for flag parsing without consuming potential file/path arguments
        while let Some(arg) = args.peek() {
            if arg.starts_with("-") {
                // Next is guaranteed after peek; unwrap is safe
                let flag_name = args.next().unwrap();

                if let Some(flag) = FLAG_REGISTRY.iter().find(|flag| flag.name == flag_name) {
                    match flag.name {
                        // Flags expecting a proceeding value argument
                        "-c" => {
                            cols = Self::parse_value(args.next())?;
                        }
                        "-g" => {
                            byte_groups = Self::parse_value(args.next())?;
                        }
                        // No value argument expected
                        _ => (flag.run)(&program),
                    }
                } else {
                    return Err("unknown flag provided".into());
                }
            } else {
                // No remaining flags to process
                break;
            }
        }

        // Read from file if provided; fallback to stdin
        let input: Box<dyn Read> = if let Some(file_path) = args.next() {
            let file =
                File::open(file_path).map_err(|err| format!("failed to open file: {err}"))?;
            Box::new(file)
        } else {
            Box::new(io::stdin().lock())
        };

        // Write to file if provided; fallback to stdout
        let output: Box<dyn Write> = if let Some(file_path) = args.next() {
            let file = File::options()
                .append(true)
                .open(file_path)
                .map_err(|err| format!("failed to open file: {err}"))?;
            Box::new(file)
        } else {
            Box::new(io::stdout().lock())
        };

        Ok(Self {
            cols,
            byte_groups,
            input,
            output,
        })
    }

    fn parse_value(value: Option<String>) -> Result<usize, String> {
        match value.ok_or("missing value for flag")?.parse::<usize>() {
            Ok(value) if (0..=256).contains(&value) => Ok(value),
            _ => Err("invalid value for flag".into()),
        }
    }
}

struct Flag {
    pub name: &'static str,
    pub description: &'static str,
    pub run: fn(&str),
}

const FLAG_REGISTRY: &[Flag] = &[
    Flag {
        name: "-c",
        description: "cols      format <cols> octets per line (value must be > 0 and <= 256). Default 16.",
        run: noop,
    },
    Flag {
        name: "-g",
        description: "bytes     number of octets per group in normal output (value must be > 0 and <= 256). Default 2.",
        run: noop,
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

fn noop(_program: &str) {}

pub fn print_usage(program: &str) {
    println!("Usage:");
    println!("      {program} [options] [infile [outfile]]");
    println!("   or");
    println!("      {program} -r [-c cols] [infile [outfile]]");
    println!("Options:");

    for flag in FLAG_REGISTRY {
        println!("   {}  {}", flag.name, flag.description);
    }

    process::exit(1);
}

pub fn print_version(program: &str) {
    println!("{} - {}", program, env!("CARGO_PKG_VERSION"));
    process::exit(0);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_flags() {
        let flags = vec![
            String::from("-c"),
            String::from("10"),
            String::from("-g"),
            String::from("3"),
        ];

        let config = Config::build(flags.into_iter(), "test");

        assert!(config.is_ok());

        let config = config.unwrap();
        assert_eq!(config.cols, 10);
        assert_eq!(config.byte_groups, 3);
    }

    #[test]
    fn valid_without_flags() {
        let flags = vec![];
        let config = Config::build(flags.into_iter(), "test").unwrap();

        assert_eq!(config.cols, 16);
        assert_eq!(config.byte_groups, 2);
    }

    #[test]
    fn invalid_missing_value() {
        let flags = vec![String::from("-c")];
        let result = Config::build(flags.into_iter(), "test");
        assert!(result.is_err());
    }

    #[test]
    fn invalid_flag_value() {
        let flags = vec![String::from("-c"), String::from("300")];
        let result = Config::build(flags.into_iter(), "test");
        assert!(result.is_err());
    }

    #[test]
    fn invalid_unknown_flag() {
        let flags = vec![String::from("-z")];
        let result = Config::build(flags.into_iter(), "test");
        assert!(result.is_err());
    }
}
