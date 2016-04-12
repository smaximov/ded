use std::env;
use std::io;
use std::io::{Write};

extern crate clap;
extern crate crypto;
extern crate eventual;
extern crate sequence_trie;

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
    let current_dir = env::current_dir()
        .expect("cannot get current working directory");

    let editor = get_editor();

    let path = temp_dir(TMP_PREFIX)
        .expect("cannot create temporary directory");

    let config = Config::new(&current_dir, &path, &editor);
    let mut app = App::new(config);

    if let Err(e) = app.run() {
        let mut stderr = io::stderr();
        writeln!(stderr, "Directory edit error: {}", e).unwrap();
    }
}
