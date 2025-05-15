mod flag;

use crate::flag::*;

use std::env;
use std::fmt::Write as _;
use std::fs::File;
use std::io::{BufReader, BufWriter, Read, Write as IoWrite};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = env::args().peekable();
    let program = args.next().ok_or("unknown program")?;

    while let Some(arg) = args.peek() {
        if arg.starts_with("-") {
            // Safe to unwrap because peeked first
            let flag_name = args.next().unwrap();

            if let Some(flag) = FLAG_REGISTRY.iter().find(|flag| flag.name == flag_name) {
                match flag.name {
                    "-c" | "-g" => {
                        let value_str = match args.next() {
                            Some(str) => str,
                            None => {
                                // Exits program
                                print_usage(&program, None);
                                return Ok(());
                            }
                        };

                        let value = match value_str.parse::<usize>() {
                            Ok(val) if val > 0 => val,
                            _ => {
                                // Exits program
                                print_usage(&program, None);
                                return Ok(());
                            }
                        };

                        (flag.run)(&program, Some(value));
                    }
                    _ => (flag.run)(&program, None),
                }
            } else {
                // Exits program
                print_usage(&program, None);
                return Ok(());
            }
        } else {
            break;
        }
    }

    // Read from file if specified, otherwise read from stdin (e.g., piping)
    let input: Box<dyn Read> = if let Some(input_path) = args.next() {
        let file = File::open(input_path)?;
        Box::new(file)
    } else {
        let stdin = std::io::stdin().lock();
        Box::new(stdin)
    };

    // Write to file if specified, otherwise write to stdout
    let output: Box<dyn IoWrite> = if let Some(output_path) = args.next() {
        let file = File::options().append(true).open(output_path)?;
        Box::new(file)
    } else {
        let stdout = std::io::stdout().lock();
        Box::new(stdout)
    };

    // Optimizes I/O by buffering reads and writes, reducing syscalls
    let mut reader = BufReader::new(input);
    let mut writer = BufWriter::new(output);

    let bytes_per_line: usize = *BYTES_PER_LINE.lock().expect("failed to aquire lock");
    let byte_grouping: usize = *BYTE_GROUPING.lock().expect("failed to aquire lock");

    // Preallocate buffer for formatting a full line per read chunk
    let mut line = String::with_capacity(bytes_per_line << 3);

    let mut buf = vec![0u8; bytes_per_line];
    let mut offset = 0;

    while let Ok(bytes_read) = reader.read(&mut buf) {
        // EOF: no more bytes to read
        if bytes_read == 0 {
            break;
        }

        // Indicate the position in the data being processed
        write!(&mut line, "{:08x}: ", offset)?;

        for (i, byte) in buf[..bytes_read].iter().enumerate() {
            // Insert space after the first byte and if a byte group has been written
            if i != 0 && i % byte_grouping == 0 {
                line.push(' ');
            }

            // Write hexadecimal representation
            write!(&mut line, "{:02x}", *byte)?;
        }

        // Calculate padding to align the remaining ASCII representation
        if bytes_read < bytes_per_line {
            // padding = remaining bytes * 2 for hex-width + spaces between byte groups
            let padding =
                (bytes_per_line - bytes_read) * 2 + ((bytes_per_line - bytes_read) / byte_grouping);

            write!(&mut line, "{:>padding$}", "")?;
        }

        // To match `xxd` output
        line.push_str("  ");

        // Convert bytes to ASCII or placeholder characters
        line.extend(buf[..bytes_read].iter().map(|&b| match b {
            // Printable characters: SP (0x20) to ~ (0x7e)
            0x20..0x7f => b as char,
            _ => '.',
        }));

        writeln!(writer, "{line}")?;
        // Reset the line buffer for reuse
        line.clear();

        offset += bytes_read;
    }

    Ok(())
}
