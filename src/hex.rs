#[cfg(test)]
use std::io::Cursor;

use std::fmt::Write as _;
use std::io::{BufRead, BufReader, BufWriter, Read, Write};

use crate::Config;

/// Performs the appropriate operation, depending on the provided `Config`.
///
/// Depending on the value of `reverse`, this function will perform either a
/// hex dump or reverse hex dump.
///
/// # Examples
///
/// ```
/// let config = hxx::Config {
///     cols: 16,
///     byte_groups: 2,
///     reverse: false,
///     input: Box::new(std::io::stdin()),
///     output: Box::new(std::io::stdout()),
/// };
///
/// // Performs a hex dump
/// if let Err(err) = hxx::run(config) {
///     eprintln!("Error: {err}");
///     std::process::exit(1);
/// }
/// ```
///
/// ```
/// let config = hxx::Config {
///     cols: 16,
///     byte_groups: 2,
///     reverse: true,
///     input: Box::new(std::io::stdin()),
///     output: Box::new(std::io::stdout()),
/// };
///
/// // Performs a reverse hex dump
/// if let Err(err) = hxx::run(config) {
///     eprintln!("Error: {err}");
///     std::process::exit(1);
/// }
/// ```
///
/// # Error
///
/// This function returns an error if the underlying `hex_dump` or `reverse_hex_dump`
/// function fails. The specific error conditions are documented in the respective
/// functions.
pub fn run(config: Config) -> Result<(), String> {
    match config.reverse {
        true => {
            reverse_hex_dump(config)?;
        }
        _ => hex_dump(config)?,
    }

    Ok(())
}

/// Processes input on a single thread and generates a hex dump using the provided `Config`.
///
/// Reads bytes from the configured input stream, formats each line with:
/// - an 8-digit hexadecimal offset,
/// - the hex representation of bytes grouped as specified,
/// - an ASCII representation of those bytes (`.` for non-printable characters),
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
///
/// if let Err(err) = hxx::hex_dump(config) {
///     eprintln!("Error: {err}");
///     std::process::exit(1);
/// }
/// ```
///
/// # Error
///
/// This function returns an error if:
/// - It fails to read from the input stream.
/// - It fails to write to the output stream.
/// - An internal formatting or I/O operation encounters a failure.
pub fn hex_dump(config: Config) -> Result<(), String> {
    // Buffer I/O to minimize syscall overhead
    let mut reader = BufReader::new(config.input);
    let mut writer = BufWriter::new(config.output);

    let cols = config.cols;
    let byte_groups = config.byte_groups;

    // Preallocate line buffer sized for a full read chunk
    let mut line = String::with_capacity(cols << 3);

    let mut buf = vec![0u8; cols];
    let mut offset: usize = 0;

    loop {
        let bytes_read = reader
            .read(&mut buf)
            .map_err(|err| format!("failed to read from input: {err}"))?;

        // Check for EOF
        if bytes_read == 0 {
            break;
        }

        format_hex_dump_line(&mut line, &buf[..bytes_read], offset, cols, byte_groups)?;

        writeln!(writer, "{line}").map_err(|err| format!("failed to write to output: {err}"))?;
        offset += bytes_read;

        // Reset buffer before reading again to avoid extra allocations
        line.clear();
    }

    Ok(())
}

fn format_hex_dump_line(
    line: &mut String,
    buffer: &[u8],
    offset: usize,
    cols: usize,
    byte_groups: usize,
) -> Result<(), String> {
    let bytes_read = buffer.len();

    // Position in the data being processed
    write!(line, "{:08x}: ", offset).map_err(|err| format!("failed to write to line: {err}"))?;

    for (i, byte) in buffer.iter().enumerate() {
        // Insert space after the first byte and if a byte group has been written
        if i != 0 && i % byte_groups == 0 {
            line.push(' ');
        }

        write!(line, "{:02x}", *byte).map_err(|err| format!("failed to write to line: {err}"))?;
    }

    if bytes_read < cols {
        // padding = remaining bytes * 2 for hex-width + spaces between byte groups
        let padding = (cols - bytes_read) * 2 + ((cols - bytes_read) / byte_groups);

        // Add padding to align the remaining ASCII representation
        write!(line, "{:>padding$}", "")
            .map_err(|err| format!("failed to write to line: {err}"))?;
    }

    // To match `xxd` formatting
    line.push_str("  ");

    // Convert bytes to ASCII or placeholder characters
    line.extend(buffer.iter().map(|&b| match b {
        // Printable characters: SP (0x20) to ~ (0x7e)
        0x20..=0x7e => b as char,
        _ => '.',
    }));

    Ok(())
}

/// Performs a reconstruction of binary data from a hex dump using the given `Config`.
///
/// Each input line is expected to be formatted similarly to `xxd` output:
/// - An 8-digit hex offset followed by a colon and a space,
/// - A hex byte section (grouping and column width do not affect parsing).
/// - Two spaces separating hex bytes from ASCII representation (which is ignored).
///
/// The function extracts only hex byte sections, converts them back to binary,
/// and writes them sequentially to the specified output stream.
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
///
/// if let Err(err) = hxx::reverse_hex_dump(config) {
///     eprintln!("Error: {err}");
///     std::process::exit(1);
/// }
/// ```
///
/// # Error
///
/// This function returns an error if:
/// - It fails to read from the input stream.
/// - It fails to write to the output stream.
/// - The input data is invalid or malformed in reverse mode.
pub fn reverse_hex_dump(config: Config) -> Result<(), String> {
    // Buffer I/O to minimize syscall overhead
    let mut reader = BufReader::new(config.input);
    let mut writer = BufWriter::new(config.output);

    let mut line = Vec::with_capacity(1024);
    let mut buf = String::with_capacity(1024);

    loop {
        let bytes_read = reader
            .read_line(&mut buf)
            .map_err(|err| format!("failed to read from input: {err}"))?;

        // Check for EOF
        if bytes_read == 0 {
            break;
        }

        format_reverse_hex_dump_line(&mut line, &buf[..bytes_read])?;

        writer
            .write_all(&line)
            .map_err(|err| format!("failed to write to output: {err}"))?;

        // Reset buffer before reading again to avoid extra allocations
        line.clear();

        // Reset buffer since `read_line()` preserves buffer contents
        buf.clear();
    }

    Ok(())
}

fn format_reverse_hex_dump_line(line: &mut Vec<u8>, buffer: &str) -> Result<(), String> {
    let colon_idx = buffer.find(':').ok_or("malformed line: missing ':'")?;

    // Skip colon and additional space
    let start = colon_idx + 2;

    let end = buffer[start..]
        .find("  ")
        .ok_or("malformed line: missing double space separator")?
        + start;

    if end > buffer.len() {
        return Err("malformed line: line too short".into());
    }

    let hex = &buffer[start..end];

    let mut chars = hex.chars().filter(|c| !c.is_whitespace());

    // Process one octet at a time
    loop {
        let high = match chars.next() {
            Some(c) => c,
            None => break,
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
        //
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

        line.push(byte);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

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
