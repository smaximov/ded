use std::path::{Path, PathBuf};

use util::{sha1};

#[derive(Debug)]
pub struct Config {
    pub dir: PathBuf,
    pub transforms_path: PathBuf,
    pub editor: String,
    pub hash_width: usize,
    pub show_hidden: bool,
    pub verbose: bool
}

impl Config {
    pub fn new<P: AsRef<Path>>(dir: P, tmp_dir: P, editor: &str, show_hidden: bool, verbose: bool) -> Self {
        let dir = dir.as_ref();
        Config {
            dir: dir.to_path_buf(),
            editor: String::from(editor),
            transforms_path: tmp_dir.as_ref().join(sha1(&dir.to_string_lossy())),
            hash_width: 8,
            show_hidden: show_hidden,
            verbose: verbose
        }
    }
}
