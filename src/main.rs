use std::env;
use std::fmt::Write as _;
use std::fs::File;
use std::io::{BufReader, BufWriter, Read, Write as _};

static BYTES_PER_LINE: usize = 16;
static BYTE_GROUPING: usize = 2;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = env::args();
    // let program = args.next().ok_or("unknown program")?;

    let file_path = args.nth(1).ok_or("no file provided")?;

    let file = File::open(&file_path)?;

    let stdout = std::io::stdout().lock();

    // Optimizes I/O by buffering reads and writes, reducing syscalls
    let mut reader = BufReader::new(file);
    let mut writer = BufWriter::new(stdout);

    // Preallocate buffer for formatting a full line per read chunk
    let mut line = String::with_capacity(BYTES_PER_LINE << 3);

    let mut buf = [0u8; BYTES_PER_LINE];
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
            if i != 0 && i % BYTE_GROUPING == 0 {
                line.push(' ');
            }

            // Write hexadecimal representation
            write!(&mut line, "{:02x}", *byte)?;
        }

        // Calculate padding to align the remaining ASCII representation
        if bytes_read < BYTES_PER_LINE {
            // padding = remaining bytes * 2 for hex-width + spaces between byte groups
            let padding =
                (BYTES_PER_LINE - bytes_read) * 2 + ((BYTES_PER_LINE - bytes_read) / BYTE_GROUPING);

            write!(&mut line, "{:>padding$}", "")?;
        }

        // To match `xxd` output
        line.push_str("  ");

        // Convert bytes to ASCII or placeholder characters
        line.extend(buf[..bytes_read].iter().map(|&b| match b {
            // Printable characters: SP (0x20) to ~ (0x7e)
            0x20..0x7f => b as char,
            // Placeholder
            _ => '.',
        }));

        writeln!(writer, "{line}")?;
        // Reset the line buffer for reuse
        line.clear();

        offset += bytes_read;
    }

    Ok(())
}
