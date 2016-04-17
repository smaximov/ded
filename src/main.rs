use std::env;
use std::io;
use std::io::{Write};
use std::path::{PathBuf};
use std::process::{exit};

extern crate clap;
extern crate crypto;
extern crate eventual;
extern crate sequence_trie;

pub mod app;
pub mod entry;
pub mod cli;
pub mod error;
pub mod formatter;
pub mod parser;
pub mod util;

use app::{App, Config};
use util::{temp_dir, get_editor};

const TMP_PREFIX: &'static str = "ded";

fn main() {
    let args = cli::args();

    let working_dir = args.value_of("dir")
        .map(|dir| PathBuf::from(dir).canonicalize())
        .unwrap_or_else(|| env::current_dir());

    let editor = args.value_of("editor")
        .map(String::from)
        .unwrap_or_else(|| get_editor());

    let working_dir = match working_dir {
        Ok(dir) => dir,
        Err(e) => {
            let mut stderr = io::stderr();
            writeln!(stderr, "error: cannot resolve working directory: {}", e).unwrap();
            exit(1);
        }
    };

    let all = args.is_present("all");

    let path = temp_dir(TMP_PREFIX)
        .expect("error: cannot create temporary directory");

    let config = Config::new(&working_dir, &path, &editor, all);
    let mut app = App::new(config);

    if let Err(e) = app.run() {
        let mut stderr = io::stderr();
        writeln!(stderr, "error: cannot edit directory: {}", e).unwrap();
        exit(1);
    }
}
