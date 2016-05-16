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
    pub globs: Option<Vec<String>>,
    pub only: Option<Only>,
}

arg_enum! {
    #[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
    pub enum Only {
        Dirs,
        Files
    }
}

impl Config {
    pub fn set_tmp_dir<P: AsRef<Path>>(&mut self, tmp_dir: P) -> &mut Self {
        self.transforms_path = tmp_dir.as_ref().join(sha1(&self.dir.to_string_lossy()));
        self
    }
}

const TMP_PREFIX: &'static str = "ded";

impl<'a> convert::From<ArgMatches<'a>> for Config {
    fn from(args: ArgMatches<'a>) -> Self {
        let working_dir = args.value_of("dir")
            .map_or_else(env::current_dir, |dir| PathBuf::from(dir).canonicalize());

        let editor = args.value_of("editor")
            .map_or_else(get_editor, String::from);

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
            .map_or_else(|| {
                match temp_dir(TMP_PREFIX) {
                    Ok(path) => path,
                    Err(e) => {
                        let mut stderr = io::stderr();
                        write!(stderr, "error: cannot create temporary directory: {}", e).unwrap();
                        exit(1);
                    }
                }
            }, PathBuf::from);

        let default_answer = if args.is_present("yes") {
            Some(true)
        } else if args.is_present("no") {
            Some(false)
        } else {
            None
        };

        let dry_run = args.is_present("dry-run");

        let globs = args.values_of_lossy("match");

        let only = if args.is_present("only") {
            Some(value_t!(args, "only", Only).unwrap_or_else(|e| e.exit()))
        } else {
            None
        };

        let mut transforms_file_name = sha1(&working_dir.to_string_lossy());
        transforms_file_name.push_str(".ded");

        Config {
            dir: working_dir,
            editor: editor,
            transforms_path: path.join(transforms_file_name),
            hash_width: 8,
            show_hidden: all,
            verbose: verbose,
            default_answer: default_answer,
            dry_run: dry_run,
            globs: globs,
            only: only,
        }
    }
}
