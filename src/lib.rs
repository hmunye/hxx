//! # hxx
//!
//! `hxx` is a minimal re-implementation of the `xxd` command-line utility.
//!
//! # Features
//! - Generate hex dumps from files or standard input, outputting to a file or standard output
//! - Flexible hex dump formatting options for columns and byte grouping
//! - Convert hex dumps back to binary form

mod config;
mod hex;

pub use config::{Config, print_usage, print_version};
pub use hex::{hex_dump, reverse_hex_dump};
