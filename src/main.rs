use std::env;
use std::io;
use std::io::{Write};
use std::path::{PathBuf};
use std::process::{exit};

extern crate clap;
extern crate crypto;
extern crate eventual;
extern crate sequence_trie;

use clap::{Arg};

pub mod app;
pub mod entry;
pub mod error;
pub mod formatter;
pub mod parser;
pub mod util;

use app::{App, Config};
use util::{temp_dir};

const TMP_PREFIX: &'static str = "ded";

fn get_editor() -> String {
    env::var("VISUAL")
        .or(env::var("EDITOR"))
        .unwrap_or(String::from("vi"))
}

fn main() {
    let matches = clap::App::new(env!("CARGO_PKG_NAME"))
        .version(env!("CARGO_PKG_VERSION"))
        .author(env!("CARGO_PKG_AUTHORS"))
        .about(env!("CARGO_PKG_DESCRIPTION"))
        .arg(Arg::with_name("WORKING_DIR")
             .index(1)
             .help("working directory [defaults to $PWD]"))
        .get_matches();

    let editor = get_editor();

    let working_dir = matches.value_of("WORKING_DIR")
        .map(|dir| PathBuf::from(dir).canonicalize())
        .unwrap_or(env::current_dir());

    let working_dir = match working_dir {
        Ok(dir) => dir,
        Err(e) => {
            let mut stderr = io::stderr();
            writeln!(stderr, "Cannot resolve working directory: {}", e).unwrap();
            exit(1);
        }
    };

    let path = temp_dir(TMP_PREFIX)
        .expect("cannot create temporary directory");

    let config = Config::new(&working_dir, &path, &editor);
    let mut app = App::new(config);

    if let Err(e) = app.run() {
        let mut stderr = io::stderr();
        writeln!(stderr, "Directory edit error: {}", e).unwrap();
    }
}
