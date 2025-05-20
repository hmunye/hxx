//! # hxx
//!
//! `hxx` is a minimal re-implementation of the `xxd` command-line utility.
//!
//! # Features
//! - Generate hex dumps from files or `stdin`, with output directed to a file or `stdout`.
//! - Customize hex dump formatting, including column width and byte grouping.
//! - Rebuild original binary data from hex dump input.

#![warn(missing_docs)]

mod config;
mod hex;

pub use config::{Config, print_usage, print_version};
pub use hex::{hex_dump, reverse_hex_dump, run};
