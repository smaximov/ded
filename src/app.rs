use std::collections::{HashSet};
use std::fs::{remove_dir_all, remove_file, rename, File};
use std::io;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Command};
use std::result;

use eventual::{Async, Future};

use entry::{Entry, EntryMap};
use error::{Error};
use formatter::{Formatter};
use parser::{Parser, Transform};
use util::{sha1, width};

#[derive(Debug)]
pub struct Config {
    dir: PathBuf,
    transforms_path: PathBuf,
    editor: String,
    hash_width: usize,
    show_hidden: bool
}

impl Config {
    pub fn new<P: AsRef<Path>>(dir: P, tmp_dir: P, editor: &str, show_hidden: bool) -> Self {
        let dir = dir.as_ref();
        Config {
            dir: dir.to_path_buf(),
            editor: String::from(editor),
            transforms_path: tmp_dir.as_ref().join(sha1(&dir.to_string_lossy())),
            hash_width: 8,
            show_hidden: show_hidden
        }
    }
}

#[derive(Debug)]
pub struct App {
    config: Config
}

pub type Result<T> = result::Result<T, Error>;

impl App {
    pub fn new(config: Config) -> Self {
        App {
            config: config
        }
    }

    pub fn run(&mut self) -> Result<()> {
        let mut entries = try!(self.list_entries());
        entries.sort();
        try!(self.write_transforms(&entries));

        let future = Future::spawn(move || EntryMap::from_vec(entries));

        try!(self.edit_transforms());

        let entries = try!(future.await());

        let transforms = try!(self.read_transforms());
        try!(self.apply_transforms(&entries, &transforms));

        try!(remove_file(&self.config.transforms_path));

        Ok(())
    }

    fn list_entries(&mut self) -> Result<Vec<Entry>> {
        let entries = try!(self.config.dir.read_dir());
        let mut result = Vec::new();

        let mut set: HashSet<String> = HashSet::new();

        for entry in entries {
            let entry = Entry::new(try!(entry).path());

            if !self.config.show_hidden && entry.is_hidden() {
                continue;
            }

            {
                let mut hash = entry.hash_short(self.config.hash_width);

                if set.contains(hash) {
                    self.config.hash_width += 1;
                    hash = entry.hash_short(self.config.hash_width);
                }

                set.insert(String::from(hash));
            }

            result.push(entry);
        }

        Ok(result)
    }

    fn write_transforms(&self, entries: &Vec<Entry>) -> Result<()> {
        let path = &self.config.transforms_path;
        let mut file = try!(File::create(path));

        try!(writeln!(file, "# Edit directory {}", self.config.dir.display()));

        for entry in entries {
            try!(writeln!(file, "{1: >0$} {2}",
                          self.config.hash_width,
                          entry.hash_short(self.config.hash_width),
                          Formatter::escape(&entry.basename())));
        }

        Ok(())
    }

    fn edit_transforms(&self) -> Result<()> {
        let edit_cmd = format!("{} {}", self.config.editor, self.config.transforms_path.display());
        let status = try!(Command::new("sh")
                          .arg("-c")
                          .arg(edit_cmd)
                          .status());

        if !status.success() {
            return Err(Error::CmdFailure(status));
        }

        Ok(())
    }

    fn read_transforms(&self) -> Result<Vec<Transform>> {
        let mut file = try!(File::open(&self.config.transforms_path));
        let mut input = String::new();
        try!(file.read_to_string(&mut input));

        let mut parser = Parser::new(&input);
        parser.parse().map_err(|e| e.into())
    }

    fn apply_transforms(&self, entries: &EntryMap, transforms: &Vec<Transform>) -> Result<()> {
        let mut fmt = Formatter::new();
        let width = width(transforms.len());
        fmt.width(width);

        for ref transform in transforms {
            let result = self.apply_transform(entries, transform, &mut fmt);
            if let Err(e) = result {
                let mut stderr = io::stderr();
                try!(writeln!(stderr, "error: {}", e));
            }
        }

        Ok(())
    }

    fn apply_transform(&self, entries: &EntryMap, transform: &Transform, fmt: &mut Formatter) -> Result<()> {
        let hash = transform.hash_fragment();
        let entry = try!(entries.get(hash));
        let old = entry.path();

        match *transform {
            Transform::Rename { ref pattern, .. } => {
                let path = try!(fmt.format(pattern));
                let new = self.config.dir.join(path);

                if old != new {
                    print!("rename `{}' -> `{}'... ", old.display(), new.display());
                    let result = rename(&old, &new);
                    let status = if result.is_err() { "failure" } else { "success" };
                    println!("{}", status);
                    try!(result);
                }
            },
            Transform::Remove { .. } => {
                print!("remove `{}'...", old.display());
                let result = if old.is_dir() {
                    remove_dir_all(&old)
                } else {
                    remove_file(&old)
                };
                let status = if result.is_err() { "failure" } else { "success" };
                println!("{}", status);
                try!(result);
            }
        }

        Ok(())
    }
}
