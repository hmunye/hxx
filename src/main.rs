use std::env;
use std::fs::File;
use std::io::{BufReader, Read};

const BYTES_PER_LINE: usize = 16;
const BYTE_GROUPING: usize = 2;

fn main() -> Result<(), String> {
    let mut args = env::args();
    let _program = args.next().unwrap_or_else(|| "unknown program".into());

    let file_path = args.next().ok_or("no file provided")?;

    let file = File::open(&file_path)
        .map_err(|e| format!("failed to open file '{}': {}", file_path, e))?;

    let mut reader = BufReader::new(file);
    let mut buf = [0u8; BYTES_PER_LINE];
    let mut offset = 0;

    loop {
        let bytes_read = reader
            .read(&mut buf)
            .map_err(|e| format!("failed to read file '{}': {}", file_path, e))?;

        if bytes_read == 0 {
            break;
        }

        print!("{:08x}: ", offset);

        for (i, byte) in buf[..bytes_read].iter().enumerate() {
            if i != 0 && i % BYTE_GROUPING == 0 {
                print!(" ");
            }

            print!("{:02x}", *byte);
        }

        if bytes_read < BYTES_PER_LINE {
            // padding = (remaining bytes) * 2 for hex-width, + SP between byte groups
            let padding =
                (BYTES_PER_LINE - bytes_read) * 2 + ((BYTES_PER_LINE - bytes_read) / BYTE_GROUPING);

            print!("{:>width$}", "", width = padding);
        }

        print!("  ");

        for byte in &buf[..bytes_read] {
            match *byte {
                0x21..0x7f => print!("{}", *byte as char),
                0x20 => print!(" "),
                _ => print!("."),
            }
        }

        println!();

        offset += bytes_read;
    }

    Ok(())
}
