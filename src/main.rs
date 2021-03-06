use std::io::{self, Write};
use std::process::{exit};

#[macro_use]
extern crate clap;
extern crate crypto;
extern crate eventual;
extern crate glob;
extern crate sequence_trie;

#[cfg(test)]
extern crate tempdir;

pub mod app;
pub mod entry;
pub mod cli;
pub mod config;
pub mod error;
pub mod formatter;
pub mod parser;
pub mod util;

fn main() {
    let args = cli::args();

    let config = config::Config::from(args);
    let mut app = app::App::new(config);

    if let Err(e) = app.run() {
        let mut stderr = io::stderr();
        writeln!(stderr, "error: cannot edit directory: {}", e).unwrap();
        exit(1);
    }
}
