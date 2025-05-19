<div align="center">

# hxx

</div>

<br />

`hxx` is a minimal re-implementation of the `xxd` command-line utility.

## Usage

```bash
Usage:
      hxx [options] [infile [outfile]]
   or
      hxx -r [infile [outfile]]
Options:
   -c  cols      format <cols> octets per line (value must be > 0 and <= 256). Default 16.
   -g  bytes     number of octets per group in normal output (value must be > 0 and <= 256). Default 2.
   -r            reverse operation: convert (or patch) hexdump into binary.
   -h            print this summary.
   -v            show version.
```

## Installation

### From [crates.io](https://crates.io/crates/hxx)

```bash
cargo install hxx
```

### From Source

```bash
git clone https://github.com/hmunye/hxx.git

cd hxx

cargo build --release
```

## Example

```bash
# Hex dump a file to stdout
hxx myfile.bin

# Hex dump with 32 bytes per line and 4-byte groupings to stdout
hxx -c 32 -g 4 myfile.bin

# Read from stdin and hex dump to stdout
cat myfile.bin | hxx

# Read from stdin and hex dump to file
cat myfile.bin | hxx myfile.hex

# Reverse a hex dump back into a binary file
hxx -r myfile.hex myfile_out.bin
```
