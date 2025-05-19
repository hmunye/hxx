use std::fs::File;
use std::io::{self, Read, Write};
use std::process;

/// Configuration for hex dumping and reverse hex dumping operations.
///
/// Defines the behavior of the hex processing functions, including formatting options,
/// direction of operation (dump or reverse), and I/O sources.
pub struct Config {
    /// Number of bytes to display per line in the hex dump.
    pub cols: usize,

    /// Number of bytes to group together in the hex output (for readability).
    pub byte_groups: usize,

    /// If `true`, performs a reverse hex dump (hex -> binary); otherwise, (binary -> hex).
    pub reverse: bool,

    /// Input source to read from (e.g., file or stdin).
    pub input: Box<dyn Read>,

    /// Output destination to write to (e.g., file or stdout).
    pub output: Box<dyn Write>,
}

impl Config {
    /// Constructs a `Config` from an iterator of command-line arguments.
    ///
    /// Parses arguments to determine formatting options, input/output streams, and mode (dump or reverse).
    ///
    /// `program` should be the name of the executable.
    ///
    /// # Examples
    ///
    /// Using a `Vec<String>`:
    /// ```
    /// let args = vec![
    ///     "-c".to_string(),
    ///     "40".to_string(),
    ///     "-g".to_string(),
    ///     "4".to_string(),
    /// ];
    ///
    /// let config = hxx::Config::build(args.into_iter(), "hxx").unwrap_or_else(|err| {
    ///     eprintln!("Error: {err}");
    ///     std::process::exit(1);
    /// });
    ///
    /// ```
    ///
    /// Using `env::args()` directly:
    /// ```
    /// let mut args = std::env::args();
    ///
    /// let program = args.next().unwrap_or_else(|| "hxx".to_string());
    ///
    /// let config = hxx::Config::build(args, &program).unwrap_or_else(|err| {
    ///     eprintln!("Error: {err}");
    ///     std::process::exit(1);
    /// });;
    /// ```
    pub fn build<T: Iterator<Item = String>>(args: T, program: &str) -> Result<Self, String> {
        let mut cols: usize = 16;
        let mut byte_groups: usize = 2;
        let mut reverse = false;

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
                        "-r" => {
                            reverse = true;
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
            let file = if let Ok(file) = File::options().append(true).open(&file_path) {
                file
            } else {
                // Create file if it doesn't exist
                File::create(&file_path).map_err(|err| format!("failed to create file: {err}"))?
            };

            Box::new(file)
        } else {
            Box::new(io::stdout().lock())
        };

        Ok(Self {
            cols,
            byte_groups,
            reverse,
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
    name: &'static str,
    description: &'static str,
    run: fn(&str),
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
        name: "-r",
        description: "          reverse operation: convert (or patch) hexdump into binary.",
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

/// Prints the usage information for the program and exits with a non-zero status.
///
/// Displays valid command-line syntax and available options.
/// Intended to be called when the user provides invalid input or provides the `-h` flag.
pub fn print_usage(program: &str) {
    println!("Usage:");
    println!("      {program} [options] [infile [outfile]]");
    println!("   or");
    println!("      {program} -r [infile [outfile]]");
    println!("Options:");

    for flag in FLAG_REGISTRY {
        println!("   {}  {}", flag.name, flag.description);
    }

    process::exit(1);
}

/// Prints the program name and version, then exits successfully.
///
/// Uses the version specified in the crate metadata (`CARGO_PKG_VERSION`).
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
