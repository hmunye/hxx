use std::env;
use std::fs::File;
use std::io::{BufReader, Read};

const BYTES_PER_LINE: usize = 16;
const _BYTE_GROUPING: usize = 2;

fn main() -> Result<(), String> {
    let mut args = env::args();
    let _program = args.next().unwrap_or_else(|| "unknown program".into());

    let file_path = args.next().ok_or("no file provided")?;

    let file = File::open(&file_path)
        .map_err(|e| format!("failed to open file '{}': {}", file_path, e))?;

    let mut reader = BufReader::new(file);
    let mut buf = [0u8; BYTES_PER_LINE];

    loop {
        let bytes_read = reader
            .read(&mut buf)
            .map_err(|e| format!("failed to read file '{}': {}", file_path, e))?;

        if bytes_read == 0 {
            break;
        }

        for byte in &buf[..bytes_read] {
            print!("{}", *byte as char);
        }
    }

    Ok(())
}
