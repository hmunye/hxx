use std::fmt::Write as _;
use std::io::{BufReader, BufWriter, Read, Write};

use super::Config;

pub fn hex_dump(config: Config) -> Result<(), String> {
    // Buffer I/O to minimize syscall overhead
    let mut reader = BufReader::new(config.input);
    let mut writer = BufWriter::new(config.output);

    let cols = config.cols;
    let byte_groups = config.byte_groups;

    // Preallocate line buffer sized for a full read chunk
    let mut line = String::with_capacity(cols << 3);

    let mut buf = vec![0u8; cols];
    let mut offset = 0;

    loop {
        let bytes_read = reader
            .read(&mut buf)
            .map_err(|err| format!("failed to read from input: {err}"))?;

        // Check for EOF
        if bytes_read == 0 {
            break;
        }

        // Position in the data being processed
        write!(&mut line, "{:08x}: ", offset)
            .map_err(|err| format!("failed to write to line: {err}"))?;

        for (i, byte) in buf[..bytes_read].iter().enumerate() {
            // Insert space after the first byte and if a byte group has been written
            if i != 0 && i % byte_groups == 0 {
                line.push(' ');
            }

            write!(&mut line, "{:02x}", *byte)
                .map_err(|err| format!("failed to write to line: {err}"))?;
        }

        if bytes_read < cols {
            // padding = remaining bytes * 2 for hex-width + spaces between byte groups
            let padding = (cols - bytes_read) * 2 + ((cols - bytes_read) / byte_groups);

            // Add padding to align the remaining ASCII representation
            write!(&mut line, "{:>padding$}", "")
                .map_err(|err| format!("failed to write to line: {err}"))?;
        }

        // To match `xxd` formatting
        line.push_str("  ");

        // Convert bytes to ASCII or placeholder characters
        line.extend(buf[..bytes_read].iter().map(|&b| match b {
            // Printable characters: SP (0x20) to ~ (0x7e)
            0x20..=0x7e => b as char,
            _ => '.',
        }));

        writeln!(writer, "{line}").map_err(|err| format!("failed to write to output: {err}"))?;
        offset += bytes_read;

        // Reset buffer before reading again to avoid extra allocations
        line.clear();
    }

    Ok(())
}
