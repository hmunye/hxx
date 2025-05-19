use std::fmt::Write as _;
use std::io::{BufRead, BufReader, BufWriter, Read, Write};

#[cfg(test)]
use std::io::Cursor;

use crate::Config;

/// Processes input and generates a hex dump using the provided `Config`.
///
/// Reads bytes from the configured input stream, formats each line with:
/// - an 8-digit hexadecimal offset,
/// - the hex representation of bytes grouped as specified,
/// - an ASCII representation of those bytes (`.` for non-printable),
/// matching the style of the `xxd`.
///
/// Lines are written to the configured output stream.
///
/// # Example
///
/// ```
/// let config = hxx::Config {
///     cols: 16,
///     byte_groups: 2,
///     reverse: false,
///     input: Box::new(std::io::stdin()),
///     output: Box::new(std::io::stdout()),
/// };
/// hxx::hex_dump(config).unwrap_or_else(|err| {
///     eprintln!("Error: {err}");
///     std::process::exit(1);
/// });
/// ```
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

/// Reconstructs the original binary data from a hex dump using the provided `Config`.
///
/// Each input line is expected to be formatted similarly to `xxd` output:
/// - An 8-digit hex offset followed by a colon and a space,
/// - Hex byte pairs grouped and separated by spaces,
/// - Two spaces separating hex bytes from ASCII representation (which is ignored).
///
/// The function extracts only the hex byte pairs, converts them back to binary,
/// and writes them sequentially.
///
/// # Example
///
/// ```
/// let config = hxx::Config {
///     cols: 16,
///     byte_groups: 2,
///     reverse: true,
///     input: Box::new(std::io::stdin()),
///     output: Box::new(std::io::stdout()),
/// };
/// hxx::reverse_hex_dump(config).unwrap_or_else(|err| {
///     eprintln!("Error: {err}");
///     std::process::exit(1);
/// });
/// ```
pub fn reverse_hex_dump(config: Config) -> Result<(), String> {
    let mut reader = BufReader::new(config.input);
    let mut writer = BufWriter::new(config.output);

    let mut buf = String::with_capacity(1024);

    loop {
        let bytes_read = reader
            .read_line(&mut buf)
            .map_err(|err| format!("failed to read from input: {err}"))?;

        // Check for EOF
        if bytes_read == 0 {
            break;
        }

        let colon_idx = buf.find(':').ok_or("malformed line: missing ':'")?;
        // Skip colon and additional space
        let start = colon_idx + 2;

        let end = buf[start..]
            .find("  ")
            .ok_or("malformed line: missing double space separator")?
            + start;

        if end > buf.len() {
            return Err("malformed line: line too short".into());
        }

        let hex = &buf[start..end];

        let mut chars = hex.chars().filter(|c| !c.is_whitespace());

        // Process one byte (octet) at a time from two hex characters
        loop {
            let high = match chars.next() {
                Some(c) => c,
                None => break, // End of input
            };

            let low = chars
                .next()
                .ok_or("malformed hex: odd number of hex digits")?;

            // Convert both hex characters to 4-bit numeric values
            let high_nibble = high
                .to_digit(16)
                .ok_or("malformed line: invalid hex char")? as u8;
            let low_nibble = low.to_digit(16).ok_or("malformed line: invalid hex char")? as u8;

            // Combine the two 4-bit nibbles into a full 8-bit byte
            // Shifts `high_nibble` into the upper 4 bits and merges it with `low_nibble`
            // Ex.
            //    0xA -> binary: 1010
            //    0xF -> binary: 1111
            //
            //    1010 << 4 = 10100000 (0xA0)
            //
            //         10100000
            //    |    00001111
            //    -------------
            //         10101111  -> 0xAF
            let byte: u8 = (high_nibble << 4) | low_nibble;

            writer
                .write_all(&[byte])
                .map_err(|err| format!("failed to write to output: {err}"))?;
        }

        // Reset buffer before reading again to avoid extra allocations
        buf.clear();
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    // Tests focusing on failure cases and malformed input

    #[test]
    fn test_missing_colon() {
        let input = Cursor::new("00000000  48 65 6c 6c 6f 20 77 6f  72 6c 64\n");
        let output = Cursor::new(Vec::new());

        let config = Config {
            cols: 16,
            byte_groups: 2,
            reverse: true,
            input: Box::new(input),
            output: Box::new(output),
        };

        let result = reverse_hex_dump(config);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("missing ':'"));
    }

    #[test]
    fn test_missing_double_space_separator() {
        let input = Cursor::new("00000000: 48 65 6c 6c 6f 20 776f726c64\n");
        let output = Cursor::new(Vec::new());

        let config = Config {
            cols: 16,
            byte_groups: 2,
            reverse: true,
            input: Box::new(input),
            output: Box::new(output),
        };

        let result = reverse_hex_dump(config);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .contains("missing double space separator")
        );
    }

    #[test]
    fn test_line_too_short() {
        let input = Cursor::new("00000000: 48\n");
        let output = Cursor::new(Vec::new());

        let config = Config {
            cols: 16,
            byte_groups: 2,
            reverse: true,
            input: Box::new(input),
            output: Box::new(output),
        };

        let result = reverse_hex_dump(config);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .contains("missing double space separator")
        );
    }

    #[test]
    fn test_odd_number_of_hex_digits() {
        let input = Cursor::new("00000000: 4 8 6 5 6 c 6 c 6 f 2 0 7 7 6 f 7 2 6 c 6  \n");
        let output = Cursor::new(Vec::new());

        let config = Config {
            cols: 16,
            byte_groups: 2,
            reverse: true,
            input: Box::new(input),
            output: Box::new(output),
        };

        let result = reverse_hex_dump(config);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("odd number of hex digits"));
    }

    #[test]
    fn test_invalid_hex_character() {
        let input = Cursor::new("00000000: 48 65 6c 6c 6f 2G 77 6f 72 6c 64  \n");
        let output = Cursor::new(Vec::new());

        let config = Config {
            cols: 16,
            byte_groups: 2,
            reverse: true,
            input: Box::new(input),
            output: Box::new(output),
        };

        let result = reverse_hex_dump(config);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("invalid hex char"));
    }
}
