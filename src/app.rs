use std::collections::{HashSet};
use std::fs::{remove_dir_all, remove_file, rename, File};
use std::io::{self, Read, Write};
use std::process::{Command};
use std::result;

use eventual::{Async, Future};
use glob::{MatchOptions, Pattern};

use config::{Config, Only};
use entry::{Entry, EntryMap};
use error::{Error};
use formatter::{Formatter};
use parser::{Parser, Transform};
use util::{width};

#[derive(Debug)]
pub struct App {
    config: Config
}

pub type Result<T> = result::Result<T, Error>;

const MATCH_OPTIONS: MatchOptions = MatchOptions {
    case_sensitive: true,
    require_literal_separator: false,
    require_literal_leading_dot: false
};

impl App {
    pub fn new(config: Config) -> Self {
        App {
            config: config
        }
    }

    pub fn run(&mut self) -> Result<()> {
        let mut entries = try!(self.list_entries());

        // Abort if nothing to do
        if entries.is_empty() {
            let mut stderr = io::stderr();
            try!(writeln!(stderr, "nothing to do: directory listing is empty"));
            return Ok(());
        }

        entries.sort();
        try!(self.write_transforms(&entries));

        let future = Future::spawn(move || EntryMap::from(entries));

        try!(self.edit_transforms());

        let entries = try!(future.await());

        let transforms = try!(self.read_transforms());
        try!(self.apply_transforms(&entries, &transforms));

        try!(remove_file(&self.config.transforms_path));

        Ok(())
    }

    fn matches(&self, entry: &Entry) -> Result<bool> {
        if let Some(ref globs) = self.config.globs {
            for ref glob in globs {
                let pattern = try!(Pattern::new(glob));

                if pattern.matches_with(entry.basename(), &MATCH_OPTIONS) {
                    return Ok(true);
                }
            }

            return Ok(false);
        }

        Ok(true)
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

            if let Some(ref only) = self.config.only {
                if entry.is_dir() && only == &Only::Files {
                    continue;
                }

                if entry.is_file() && only == &Only::Dirs {
                    continue
                }
            }

            if !try!(self.matches(&entry)) {
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

    fn write_transforms(&self, entries: &[Entry]) -> Result<()> {
        let path = &self.config.transforms_path;
        let mut file = try!(File::create(path));

        try!(writeln!(file, "# Edit directory {}\n", self.config.dir.display()));

        for entry in entries {
            if self.config.verbose {
                let kind = if entry.path().is_dir() {
                    "Directory"
                } else {
                    "File"
                };

                try!(writeln!(file, "# {} {}", kind, entry.path().display()));
            }

            try!(writeln!(file, "{1: >0$} {2}",
                          self.config.hash_width,
                          entry.hash_short(self.config.hash_width),
                          Formatter::escape(entry.basename())));
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

    fn apply_transforms(&self, entries: &EntryMap, transforms: &[Transform]) -> Result<()> {
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

    fn yes_or_no(&self, prompt: &str, default: bool) -> io::Result<bool> {
        if let Some(answer) = self.config.default_answer {
            return Ok(answer);
        }

        let mut stdout = io::stdout();
        let stdin = io::stdin();

        let suggest = if default {
            "Y/n"
        } else {
            "y/N"
        };

        loop {
            try!(write!(stdout, "{} ({}) ", prompt, suggest));
            try!(stdout.flush());

            let mut input = String::new();

            try!(stdin.read_line(&mut input));

            let answer = input.trim().to_lowercase();

            match &answer[..] {
                "y" => return Ok(true),
                "n" => return Ok(false),
                "" => return Ok(default),
                _ => {
                    try!(writeln!(stdout, "answer `y' or 'n'"));
                }
            }
        }
    }


    fn apply_transform(&self, entries: &EntryMap, transform: &Transform, fmt: &mut Formatter) -> Result<()> {
        let hash = transform.hash_fragment();
        let entry = try!(entries.get(hash));
        let old = entry.path();

        match *transform {
            Transform::Rename { ref pattern, .. } => {
                let path = try!(fmt.format(pattern));
                let new = self.config.dir.join(path);

                if old == new {
                    return Ok(());
                }

                println!("renaming `{}' -> `{}'... ", old.display(), new.display());

                if new.exists() {
                    let prompt = format!("target `{} exists, override?", new.display());

                    if !try!(self.yes_or_no(&prompt, false)) {
                        println!("skipped");
                        return Ok(());
                    }
                }

                if !self.config.dry_run {
                    try!(rename(&old, &new));
                }
            },
            Transform::Remove { .. } => {
                println!("remove `{}'...", old.display());

                if !self.config.dry_run {
                    try!(if old.is_dir() {
                        remove_dir_all(&old)
                    } else {
                        remove_file(&old)
                    });
                }
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::fs::{create_dir, File};
    use std::path::{Path, PathBuf};

    use tempdir::{TempDir};

    use super::*;

    struct Directory {
        name: TempDir,
    }

    impl Directory {
        fn new(name: &str, entries: Vec<DirEntry>) -> Result<Self> {
            let dir = try!(TempDir::new(name));

            for entry in &entries {
                try!(entry.create_inside(dir.path()));
            }

            Ok(Directory {
                name: dir,
            })
        }

        fn path(&self) -> &Path {
            self.name.path()
        }
    }

    #[derive(Debug)]
    pub enum DirEntry {
        Dir(String),
        File(String),
    }

    impl DirEntry {
        fn path(&self) -> &str {
            match *self {
                DirEntry::Dir(ref path) => path,
                DirEntry::File(ref path) => path
            }
        }

        fn create_inside<P: AsRef<Path>>(&self, dir: P) -> Result<()> {
            let mut path = PathBuf::from(dir.as_ref());
            path.push(self.path());

            match *self {
                DirEntry::Dir(..) => try!(create_dir(&path)),
                DirEntry::File(..) => {
                    let _ = try!(File::create(&path));
                }
            }

            Ok(())
        }
    }

    macro_rules! directory {
        ( $name:expr ) => { Directory::new($name, Vec::new()).unwrap() };
        ( $name:expr, [ $( $entry:expr ),+ ]) => {
            {
                let mut entries: Vec<DirEntry> = Vec::new();
                $(
                    let s = String::from($entry);
                    entries.push(if s.ends_with("/") {
                        DirEntry::Dir(s)
                    } else {
                        DirEntry::File(s)
                    });
                )+
                Directory::new($name, entries).unwrap()
            }
        };
    }

    macro_rules! app {
        ( $app:ident, [ $( $opt:expr ),* ], $dir:expr, $block:block ) => {
            {
                let tmp = directory!("ded");
                let args = $crate::cli::args_from(vec!["ded", $( $opt , )* ]);
                let mut config = $crate::config::Config::from(args);
                config.dir = $dir.path().to_path_buf();
                config.set_tmp_dir(tmp.path());

                let mut $app = $crate::app::App::new(config);

                let mut entries = $app.list_entries().unwrap();
                entries.sort();
                $app.write_transforms(&entries).unwrap();

                $block
            }
        };
    }

    #[test]
    #[ignore]
    fn hidden() {
        let dir = directory!("hidden", [
            "regular-file",
            "regular-directory/",
            ".hidden-file",
            ".hidden-directory/"
        ]);

        app!(app, [], dir, {
            let transforms = app.read_transforms().unwrap();
            assert_eq!(transforms.len(), 2);
        });

        app!(app, ["-a"], dir, {
            let transforms = app.read_transforms().unwrap();
            assert_eq!(transforms.len(), 4);
        });
    }

    #[test]
    #[ignore]
    fn only() {
        let dir = directory!("only", [
            "file",
            "dir1/",
            "dir2/",
            "dir3/"
        ]);

        app!(app, [], dir, {
            let transforms = app.read_transforms().unwrap();
            assert_eq!(transforms.len(), 4);
        });

        app!(app, ["--only", "dirs"], dir, {
            let transforms = app.read_transforms().unwrap();
            assert_eq!(transforms.len(), 3);
        });

        app!(app, ["--only", "files"], dir, {
            let transforms = app.read_transforms().unwrap();
            assert_eq!(transforms.len(), 1);
        });
    }

    #[test]
    #[ignore]
    fn match_glob() {
        let dir = directory!("match", [
            "foo",
            "foobar/",
            "baaz",
            "bar",
            "quuz"
        ]);

        app!(app, ["-m", "foo*"], dir, {
            let transforms = app.read_transforms().unwrap();
            assert_eq!(transforms.len(), 2);
        });

        app!(app, ["-m", "foo*", "-m", "ba*"], dir, {
            let transforms = app.read_transforms().unwrap();
            assert_eq!(transforms.len(), 4);
        });
    }
}
