use std::env;
use std::convert;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::exit;

use clap::{ArgMatches};

use util::{get_editor, sha1, temp_dir};

#[derive(Debug)]
pub struct Config {
    pub dir: PathBuf,
    pub transforms_path: PathBuf,
    pub editor: String,
    pub hash_width: usize,
    pub show_hidden: bool,
    pub verbose: bool,
    pub default_answer: Option<bool>,
    pub dry_run: bool,
    pub globs: Option<Vec<String>>
}

impl Config {
    pub fn new<P: AsRef<Path>>(dir: P, tmp_dir: P, editor: &str, show_hidden: bool,
                               verbose: bool, default_answer: Option<bool>, dry_run: bool,
                               globs: Option<Vec<String>>) -> Self {
        let dir = dir.as_ref();
        Config {
            dir: dir.to_path_buf(),
            editor: String::from(editor),
            transforms_path: tmp_dir.as_ref().join(sha1(&dir.to_string_lossy())),
            hash_width: 8,
            show_hidden: show_hidden,
            verbose: verbose,
            default_answer: default_answer,
            dry_run: dry_run,
            globs: globs
        }
    }
}

const TMP_PREFIX: &'static str = "ded";

impl<'a> convert::From<ArgMatches<'a>> for Config {
    fn from(args: ArgMatches<'a>) -> Self {
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
        let verbose = args.is_present("verbose");

        let path = args.value_of("tmp")
            .map(|path| PathBuf::from(path))
            .unwrap_or_else(|| {
                match temp_dir(TMP_PREFIX) {
                    Ok(path) => path,
                    Err(e) => {
                        let mut stderr = io::stderr();
                        write!(stderr, "error: cannot create temporary directory: {}", e).unwrap();
                        exit(1);
                    }
                }
            });

        let default_answer = if args.is_present("yes") {
            Some(true)
        } else if args.is_present("no") {
            Some(false)
        } else {
            None
        };

        let dry_run = args.is_present("dry-run");

        let globs = args.values_of_lossy("match");

        Config::new(&working_dir, &path, &editor, all,
                    verbose, default_answer, dry_run, globs)
    }
}
